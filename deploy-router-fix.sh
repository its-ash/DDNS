#!/usr/bin/env bash
set -e

echo "🔧 Quick Fix Deployment for Router DDNS"
echo "========================================"
echo ""

# Check if running on the server
if [ ! -f "/etc/nginx/sites-available/ash-api.online" ]; then
    echo "⚠️  This script should be run on your production server"
    echo ""
    echo "Manual steps:"
    echo "  1. Copy nginx/ash-api.online.conf to server"
    echo "  2. Rebuild DDNS server: ./build.sh"
    echo "  3. Restart services"
    exit 0
fi

# Update nginx config
echo "📋 Updating nginx configuration..."
if [ -f "nginx/ash-api.online.conf" ]; then
    sudo cp nginx/ash-api.online.conf /etc/nginx/sites-available/ash-api.online
    echo "✅ Nginx config updated"
else
    echo "❌ nginx/ash-api.online.conf not found"
    exit 1
fi

# Test nginx config
echo "🧪 Testing nginx configuration..."
if sudo nginx -t 2>&1 | grep -q "successful"; then
    echo "✅ Nginx config is valid"
else
    echo "❌ Nginx config has errors"
    sudo nginx -t
    exit 1
fi

# Reload nginx
echo "🔄 Reloading nginx..."
sudo systemctl reload nginx
echo "✅ Nginx reloaded"

# Check if DDNS binary exists
if [ ! -f "target/release/ddns-server" ]; then
    echo ""
    echo "🔨 Building DDNS server..."
    ./build.sh || cargo build --release
fi

# Restart DDNS server
echo "🔄 Restarting DDNS server..."
if systemctl is-active --quiet ddns-server; then
    sudo systemctl restart ddns-server
    echo "✅ DDNS server restarted"
else
    echo "⚠️  DDNS server is not running as a service"
    echo "   You may need to start it manually"
fi

echo ""
echo "✅ Deployment complete!"
echo ""
echo "Testing the fix:"
echo "  curl -I http://ash-api.online/update"
echo "  # Should return 401 Unauthorized (not 301 redirect)"
echo ""
echo "Full test with credentials:"
echo "  curl -u USERNAME:PASSWORD 'http://ash-api.online/update?hostname=home.ash-api.online&myip=100.135.120.102'"
echo ""
