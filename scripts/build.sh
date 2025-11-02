#!/bin/bash
set -euo pipefail

# Build script for NNOE project

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$PROJECT_ROOT"

# Parse arguments
MODE="${1:-release}"
FEATURES="${2:-}"

echo "Building NNOE agent in $MODE mode..."

if [ "$MODE" = "release" ]; then
    CARGO_ARGS="--release"
else
    CARGO_ARGS=""
fi

if [ -n "$FEATURES" ]; then
    CARGO_ARGS="$CARGO_ARGS --features $FEATURES"
fi

# Check for protoc (required for building protobuf files)
if ! command -v protoc &> /dev/null; then
    echo "Error: protoc (Protocol Buffers compiler) is required but not found."
    echo "Install it with: sudo apt-get install protobuf-compiler libprotobuf-dev"
    exit 1
fi

# Build the agent
cargo build --package nnoe-agent $CARGO_ARGS

echo "Build complete!"
if [ "$MODE" = "release" ]; then
    echo "Binary location: $PROJECT_ROOT/target/release/nnoe-agent"
else
    echo "Binary location: $PROJECT_ROOT/target/debug/nnoe-agent"
fi

