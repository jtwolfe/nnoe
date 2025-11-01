#!/bin/bash
set -euo pipefail

# Build script for Kea etcd hook

echo "Building Kea etcd hook..."

# Create build directory
mkdir -p build
cd build

# Run CMake
cmake .. \
    -DCMAKE_BUILD_TYPE=Release \
    -DKEA_INCLUDE_DIRS=/usr/include/kea

# Build
make -j$(nproc)

echo "Build complete. Library: build/libdhcp_etcd.so"
echo "Install with: sudo cp build/libdhcp_etcd.so /usr/lib/kea/hooks/"

