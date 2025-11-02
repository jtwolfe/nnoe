# Multi-stage build for NNOE Prometheus Exporter

FROM rust:1.70 as builder

WORKDIR /app

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY management/ ./management/

# Build dependencies first (caching layer)
RUN cargo build --release --package nnoe-prometheus-exporter || \
    (cargo build --package nnoe-prometheus-exporter --release)

# Final stage
FROM debian:bullseye-slim

LABEL maintainer="NNOE Contributors"
LABEL description="NNOE Prometheus Exporter - Metrics collection service"

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create nnoe user
RUN useradd -r -s /bin/false -d /var/lib/nnoe -m nnoe

# Copy binary
COPY --from=builder /app/target/release/nnoe-prometheus-exporter /usr/local/bin/nnoe-prometheus-exporter

# Switch to non-root user
USER nnoe

WORKDIR /var/lib/nnoe

# Expose metrics port
EXPOSE 9090

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:9090/health || exit 1

ENTRYPOINT ["/usr/local/bin/nnoe-prometheus-exporter"]

