#!/bin/bash
# Vertil Nano Pro - Uninstall Script

set -e

BINARY_NAME="nanopro"
INSTALL_DIR="/usr/local/bin"
CONFIG_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/nanopro"

echo "========================================"
echo "  Vertil Nano Pro - Uninstaller"
echo "========================================"
echo ""

echo "This will remove:"
echo "  - $INSTALL_DIR/$BINARY_NAME"
echo "  - $CONFIG_DIR"
echo ""

read -p "Continue? (y/N) " -n 1 -r
echo ""

if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Uninstall cancelled."
    exit 0
fi

# Remove binary
if [ -f "$INSTALL_DIR/$BINARY_NAME" ]; then
    if [ "$EUID" -ne 0 ]; then
        sudo rm "$INSTALL_DIR/$BINARY_NAME"
    else
        rm "$INSTALL_DIR/$BINARY_NAME"
    fi
    echo "Removed: $INSTALL_DIR/$BINARY_NAME"
else
    echo "Binary not found at $INSTALL_DIR/$BINARY_NAME"
fi

# Remove config
if [ -d "$CONFIG_DIR" ]; then
    rm -rf "$CONFIG_DIR"
    echo "Removed: $CONFIG_DIR"
fi

echo ""
echo "Vertil Nano Pro has been uninstalled."
