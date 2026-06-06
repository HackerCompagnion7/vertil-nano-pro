#!/bin/bash
# Vertil Nano Pro - Build Script
# Compiles the project for the current platform

set -e

PROJECT_NAME="vertil-nano-pro"
BINARY_NAME="nanopro"
AUTHOR="Ishmael Vertil"

echo "========================================"
echo "  Vertil Nano Pro - Build System"
echo "  Created by $AUTHOR"
echo "========================================"
echo ""

# Detect platform
OS="$(uname -s)"
ARCH="$(uname -m)"

echo "Platform: $OS $ARCH"
echo ""

# Check Rust installation
if ! command -v cargo &> /dev/null; then
    echo "Error: Rust/Cargo is not installed."
    echo "Install Rust: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

echo "Rust version: $(rustc --version)"
echo "Cargo version: $(cargo --version)"
echo ""

# Build type
BUILD_TYPE="${1:-release}"

case "$BUILD_TYPE" in
    release|--release)
        echo "Building release..."
        cargo build --release
        BINARY_PATH="target/release/$BINARY_NAME"
        ;;
    debug|--debug)
        echo "Building debug..."
        cargo build
        BINARY_PATH="target/debug/$BINARY_NAME"
        ;;
    *)
        echo "Unknown build type: $BUILD_TYPE"
        echo "Usage: $0 [release|debug]"
        exit 1
        ;;
esac

echo ""
if [ -f "$BINARY_PATH" ]; then
    BINARY_SIZE=$(du -h "$BINARY_PATH" | cut -f1)
    echo "Build successful!"
    echo "Binary: $BINARY_PATH"
    echo "Size: $BINARY_SIZE"
    echo ""
    echo "Install to /usr/local/bin:"
    echo "  sudo cp $BINARY_PATH /usr/local/bin/nanopro"
else
    echo "Build failed!"
    exit 1
fi
