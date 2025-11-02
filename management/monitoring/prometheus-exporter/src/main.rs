use http_body_util::Full;
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use prometheus::{Encoder, Gauge, Registry, TextEncoder};
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::{info, warn};

mod metrics;

use metrics::MetricsCollector;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("nnoe_prometheus_exporter=info")
        .init();

    info!("Starting NNOE Prometheus exporter");

    let addr = SocketAddr::from(([0, 0, 0, 0], 9090));
    let listener = TcpListener::bind(addr).await?;
    info!("Prometheus exporter listening on {}", addr);

    let mut metrics_collector = MetricsCollector::new()?;

    // Configure etcd connection if environment variables are set
    if let Ok(endpoints_str) = std::env::var("ETCD_ENDPOINTS") {
        let endpoints: Vec<String> = endpoints_str
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();
        let prefix = std::env::var("ETCD_PREFIX").unwrap_or_else(|_| "/nnoe".to_string());
        metrics_collector.set_etcd_config(endpoints, prefix);
        info!("Configured etcd metrics collection");
    } else {
        info!("No etcd configuration found - some metrics will be unavailable");
    }

    let metrics_collector = Arc::new(metrics_collector);

    // Start metrics collection task
    let collector_clone = Arc::clone(&metrics_collector);
    tokio::spawn(async move {
        collector_clone.collect_metrics_loop().await;
    });

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);

        let metrics_collector = Arc::clone(&metrics_collector);

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(
                    io,
                    service_fn(move |req| handle_request(req, Arc::clone(&metrics_collector))),
                )
                .await
            {
                warn!("Error serving connection: {}", err);
            }
        });
    }
}

async fn handle_request(
    req: Request<hyper::body::Incoming>,
    metrics_collector: Arc<MetricsCollector>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    match (req.method(), req.uri().path()) {
        (&hyper::Method::GET, "/metrics") => {
            let encoder = TextEncoder::new();
            let metric_families = metrics_collector.registry.gather();
            let mut buffer = Vec::new();

            if let Err(e) = encoder.encode(&metric_families, &mut buffer) {
                warn!("Error encoding metrics: {}", e);
                return Ok(Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(Full::new(Bytes::from("Error encoding metrics")))
                    .unwrap());
            }

            Ok(Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", encoder.format_type())
                .body(Full::new(Bytes::from(buffer)))
                .unwrap())
        }
        (&hyper::Method::GET, "/health") => Ok(Response::builder()
            .status(StatusCode::OK)
            .body(Full::new(Bytes::from("OK")))
            .unwrap()),
        _ => Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Full::new(Bytes::from("Not Found")))
            .unwrap()),
    }
}
