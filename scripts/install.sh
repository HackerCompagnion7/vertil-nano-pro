#!/bin/bash
# Vertil Nano Pro - Install Script
# Installs the editor system-wide

set -e

BINARY_NAME="nanopro"
INSTALL_DIR="/usr/local/bin"
AUTHOR="Ishmael Vertil"

echo "========================================"
echo "  Vertil Nano Pro - Installer"
echo "  Created by $AUTHOR"
echo "========================================"
echo ""

# Find the binary
if [ -f "target/release/$BINARY_NAME" ]; then
    BINARY="target/release/$BINARY_NAME"
elif [ -f "target/debug/$BINARY_NAME" ]; then
    BINARY="target/debug/$BINARY_NAME"
else
    echo "Binary not found. Building release..."
    cargo build --release
    BINARY="target/release/$BINARY_NAME"
fi

echo "Binary: $BINARY"
echo "Install to: $INSTALL_DIR/$BINARY_NAME"
echo ""

# Check for root/sudo
if [ "$EUID" -ne 0 ]; then
    echo "Requesting sudo for installation..."
    sudo cp "$BINARY" "$INSTALL_DIR/$BINARY_NAME"
    sudo chmod +x "$INSTALL_DIR/$BINARY_NAME"
else
    cp "$BINARY" "$INSTALL_DIR/$BINARY_NAME"
    chmod +x "$INSTALL_DIR/$BINARY_NAME"
fi

echo ""

# Verify installation
if command -v nanopro &> /dev/null; then
    echo "Installation successful!"
    echo ""
    nanopro --version
    echo ""
    echo "Usage: nanopro <file>"
    echo "Help:  nanopro --about"
else
    echo "Warning: nanopro not found in PATH"
    echo "Make sure $INSTALL_DIR is in your PATH"
fi

# Create config directory
CONFIG_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/nanopro"
mkdir -p "$CONFIG_DIR"
echo ""
echo "Config directory: $CONFIG_DIR"

# Copy themes
THEMES_SRC="themes"
if [ -d "$THEMES_SRC" ]; then
    mkdir -p "$CONFIG_DIR/themes"
    cp -r "$THEMES_SRC"/* "$CONFIG_DIR/themes/" 2>/dev/null || true
    echo "Themes installed to: $CONFIG_DIR/themes/"
fi

echo ""
echo "Done! Start editing: nanopro myfile.py"
