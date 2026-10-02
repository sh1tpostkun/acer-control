#!/bin/bash
set -e

echo "Building AcerControl..."

# Build Rust Daemon and CLI
echo "Building Rust components..."
cargo build --release -p acercontrol-daemon -p acercontrol-cli

# Build Qt/QML GUI
echo "Building Qt GUI..."
mkdir -p build
cd build
cmake ..
make -j$(nproc)
cd ..

echo "Build complete."
