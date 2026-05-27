#!/usr/bin/env bash
set -e

echo "🔨 Building DDNS Server for Linux..."

# Build in release mode
cargo build --release

echo "✅ Build complete!"
echo ""
echo "Binary location: target/release/ddns-server"
echo "Binary size: $(du -h target/release/ddns-server | cut -f1)"
echo ""
echo "To run the server:"
echo "  ./target/release/ddns-server"
echo ""
echo "To install as systemd service:"
echo "  sudo ./install.sh"
