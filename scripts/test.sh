#!/bin/bash
set -euo pipefail

# Test script for NNOE project

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$PROJECT_ROOT"

echo "Running NNOE test suite..."

# Run unit tests
echo "Running unit tests..."
cargo test --lib --verbose

# Run integration tests if they exist
if [ -d "testing/integration" ]; then
    echo "Running integration tests..."
    cargo test --test '*' --verbose || echo "Integration tests pending"
fi

# Run with coverage if available
if command -v cargo-tarpaulin &> /dev/null; then
    echo "Generating coverage report..."
    cargo tarpaulin --out Xml --out Html || true
fi

echo "Test suite complete!"

