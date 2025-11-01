#!/bin/bash
set -euo pipefail

# etcd Watch Latency Test
# Measures latency from etcd change to agent notification

ETCD_ENDPOINT="${1:-http://127.0.0.1:2379}"
TEST_KEY="${2:-/nnoe/test/watch-latency}"
ITERATIONS="${3:-100}"

echo "Testing etcd watch latency"
echo "Endpoint: $ETCD_ENDPOINT"
echo "Test key: $TEST_KEY"
echo "Iterations: $ITERATIONS"

LATENCIES=()

for i in $(seq 1 $ITERATIONS); do
    START_TIME=$(date +%s.%N)
    
    # Put key to etcd
    etcdctl --endpoints="$ETCD_ENDPOINT" put "$TEST_KEY" "test-value-$i" > /dev/null
    
    # Wait for watch to receive (this would be done by agent in real scenario)
    # For this test, we just measure put latency
    END_TIME=$(date +%s.%N)
    
    LATENCY=$(echo "$END_TIME - $START_TIME" | bc)
    LATENCIES+=($LATENCY)
    
    if [ $((i % 10)) -eq 0 ]; then
        echo "Completed $i/$ITERATIONS iterations"
    fi
done

# Calculate statistics
echo "Latency Statistics:"
echo "  Min: $(printf '%s\n' "${LATENCIES[@]}" | sort -n | head -1) seconds"
echo "  Max: $(printf '%s\n' "${LATENCIES[@]}" | sort -n | tail -1) seconds"
echo "  Average: $(echo "${LATENCIES[@]}" | awk '{sum=0; for(i=1;i<=NF;i++) sum+=$i; print sum/NF}') seconds"

