#!/bin/bash
set -euo pipefail

# Nebula CA Initialization Script
# Creates a new Nebula Certificate Authority

CA_NAME="${1:-NNOE CA}"
CA_DURATION="${2:-87600h}"  # 10 years
CA_DIR="${3:-/etc/nebula/ca}"
OUTPUT_DIR="${CA_DIR}"

echo "Initializing Nebula CA: $CA_NAME"
echo "Duration: $CA_DURATION"
echo "Output directory: $OUTPUT_DIR"

mkdir -p "$OUTPUT_DIR"

# Check if nebula-cert exists
if ! command -v nebula-cert &> /dev/null; then
    echo "Error: nebula-cert not found. Please install Nebula."
    echo "Visit: https://github.com/slackhq/nebula"
    exit 1
fi

# Generate CA
nebula-cert ca \
    -name "$CA_NAME" \
    -duration "$CA_DURATION" \
    -out-crt "$OUTPUT_DIR/ca.crt" \
    -out-key "$OUTPUT_DIR/ca.key"

if [ $? -eq 0 ]; then
    echo "CA generated successfully:"
    echo "  Certificate: $OUTPUT_DIR/ca.crt"
    echo "  Key: $OUTPUT_DIR/ca.key"
    echo ""
    echo "IMPORTANT: Keep ca.key secure! Do not distribute."
else
    echo "CA generation failed!"
    exit 1
fi

