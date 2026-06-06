#!/bin/bash
# Vertil Nano Pro - Termux Build Script
# Builds and packages for Termux/Android

set -e

TERMUX_PKG_NAME="nanopro"
TERMUX_PKG_VERSION="1.0.0"
TERMUX_PKG_MAINTAINER="Ishmael Vertil <ishmaelvertil@github.com>"
TERMUX_PKG_DESCRIPTION="Modern terminal code editor — simple like Nano, powerful for 2026"
TERMUX_PKG_HOMEPAGE="https://github.com/ishmaelvertil/vertil-nano-pro"
TERMUX_PKG_LICENSE="MIT"

echo "========================================"
echo "  Vertil Nano Pro - Termux Build"
echo "  Created by Ishmael Vertil"
echo "========================================"
echo ""

# Check if running in Termux
if [ ! -d "/data/data/com.termux" ] && [ -z "$TERMUX_PREFIX" ]; then
    echo "Warning: Not running in Termux environment."
    echo "This script is designed for Termux/Android builds."
    echo ""
fi

# Set Termux prefix
TERMUX_PREFIX="${TERMUX_PREFIX:-/data/data/com.termux/files/usr}"
TERMUX_BIN_DIR="${TERMUX_PREFIX}/bin"

# Install build dependencies
echo "Installing build dependencies..."

# Try pkg (Termux) or apt
if command -v pkg &> /dev/null; then
    pkg install -y rust cmake git
elif command -v apt &> /dev/null; then
    apt update
    apt install -y rustc cargo cmake git build-essential
fi

# Ensure cargo is available
if ! command -v cargo &> /dev/null; then
    echo "Error: Cargo not found. Please install Rust first."
    exit 1
fi

echo ""
echo "Building Vertil Nano Pro for Termux..."

# Build release
cargo build --release

BINARY_PATH="target/release/nanopro"

if [ ! -f "$BINARY_PATH" ]; then
    echo "Build failed!"
    exit 1
fi

echo ""
echo "Build successful!"

# Install to Termux
echo "Installing to $TERMUX_BIN_DIR..."
mkdir -p "$TERMUX_BIN_DIR"
cp "$BINARY_PATH" "$TERMUX_BIN_DIR/nanopro"
chmod +x "$TERMUX_BIN_DIR/nanopro"

# Create config directory
CONFIG_DIR="${TERMUX_PREFIX}/etc/nanopro"
mkdir -p "$CONFIG_DIR/themes"

# Copy themes
if [ -d "themes" ]; then
    cp themes/*.toml "$CONFIG_DIR/themes/"
fi

echo ""
echo "Installation complete!"
echo ""
echo "Binary: $TERMUX_BIN_DIR/nanopro"
echo "Config: $CONFIG_DIR"
echo ""
echo "Usage: nanopro <file>"

# Create a Termux package if possible
TERMUX_PKG_DIR="/tmp/nanopro-package"
if command -v tar &> /dev/null; then
    echo ""
    echo "Creating portable package..."
    mkdir -p "$TERMUX_PKG_DIR/nanopro/bin"
    mkdir -p "$TERMUX_PKG_DIR/nanopro/themes"
    cp "$BINARY_PATH" "$TERMUX_PKG_DIR/nanopro/bin/nanopro"
    chmod +x "$TERMUX_PKG_DIR/nanopro/bin/nanopro"
    cp themes/*.toml "$TERMUX_PKG_DIR/nanopro/themes/" 2>/dev/null || true
    cp README.md "$TERMUX_PKG_DIR/nanopro/" 2>/dev/null || true
    cp LICENSE "$TERMUX_PKG_DIR/nanopro/" 2>/dev/null || true

    # Create install script
    cat > "$TERMUX_PKG_DIR/nanopro/install.sh" << 'INSTALL_EOF'
#!/bin/bash
PREFIX="${TERMUX_PREFIX:-/data/data/com.termux/files/usr}"
cp bin/nanopro "$PREFIX/bin/nanopro"
chmod +x "$PREFIX/bin/nanopro"
mkdir -p "$PREFIX/etc/nanopro/themes"
cp themes/*.toml "$PREFIX/etc/nanopro/themes/" 2>/dev/null || true
echo "Vertil Nano Pro installed! Run: nanopro <file>"
INSTALL_EOF
    chmod +x "$TERMUX_PKG_DIR/nanopro/install.sh"

    tar -czf "nanopro-termux-$(uname -m).tar.gz" -C "$TERMUX_PKG_DIR" nanopro
    rm -rf "$TERMUX_PKG_DIR"

    echo "Package created: nanopro-termux-$(uname -m).tar.gz"
    echo "Install with: tar xzf nanopro-termux-*.tar.gz && cd nanopro && ./install.sh"
fi
