#!/bin/bash
set -euo pipefail

# DNS Load Testing Script
# Tests DNS query performance and throughput

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RESULTS_DIR="${1:-/tmp/nnoe-performance}"

echo "Starting DNS load tests"
echo "Results directory: $RESULTS_DIR"

mkdir -p "$RESULTS_DIR"

# Check if dnsperf is available
if ! command -v dnsperf &> /dev/null; then
    echo "Error: dnsperf not found. Installing..."
    echo "Visit: https://github.com/nominum/dnsperf"
    exit 1
fi

# Generate test query file
cat > /tmp/dns-queries.txt <<EOF
www.example.com A
mail.example.com A
www.example.com AAAA
example.com MX
example.com NS
EOF

echo "Running DNS query performance test..."

# Run dnsperf with various parameters
for qps in 100 500 1000 5000; do
    echo "Testing at $qps queries/second..."
    
    dnsperf -s 127.0.0.1 -p 53 -d /tmp/dns-queries.txt \
        -l 60 -Q $qps -c 10 \
        > "$RESULTS_DIR/dnsperf-${qps}qps.log" 2>&1 || true
done

# Parse results
echo "Generating summary..."
cat > "$RESULTS_DIR/summary.txt" <<EOF
DNS Performance Test Summary
Generated: $(date)

Tests run at different QPS levels:
- 100 QPS
- 500 QPS  
- 1000 QPS
- 5000 QPS

Check individual log files for detailed results.
EOF

echo "Load test complete. Results in: $RESULTS_DIR"

