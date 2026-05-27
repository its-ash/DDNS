#!/usr/bin/env bash
set -e

if [ "$EUID" -ne 0 ]; then 
    echo "❌ Please run as root (use sudo)"
    exit 1
fi

echo "🔧 Setting up Nginx for DDNS Server..."

# Check if nginx is installed
if ! command -v nginx &> /dev/null; then
    echo "📦 Nginx not found. Installing..."
    apt-get update
    apt-get install -y nginx
else
    echo "✅ Nginx is already installed"
fi

# Check if certbot is installed (for SSL)
if ! command -v certbot &> /dev/null; then
    echo "📦 Certbot not found. Installing..."
    apt-get install -y certbot python3-certbot-nginx
else
    echo "✅ Certbot is already installed"
fi

# Ask user which configuration to use
echo ""
echo "Choose nginx configuration:"
echo "  1) HTTPS with SSL (recommended for production)"
echo "  2) HTTP only (for testing/development)"
read -p "Enter choice [1-2]: " choice

case $choice in
    1)
        CONFIG_FILE="ash-api.online.conf"
        USE_SSL=true
        ;;
    2)
        CONFIG_FILE="ash-api.online-http-only.conf"
        USE_SSL=false
        ;;
    *)
        echo "❌ Invalid choice"
        exit 1
        ;;
esac

# Create directories
mkdir -p /var/www/certbot

# Copy nginx configuration
echo "📋 Installing nginx configuration..."
cp "nginx/$CONFIG_FILE" /etc/nginx/sites-available/ash-api.online
ln -sf /etc/nginx/sites-available/ash-api.online /etc/nginx/sites-enabled/ash-api.online

# Test nginx configuration
echo "🧪 Testing nginx configuration..."
nginx -t

if [ "$USE_SSL" = true ]; then
    echo ""
    echo "🔐 SSL Certificate Setup"
    echo "Before obtaining SSL certificate, ensure:"
    echo "  1. ash-api.online DNS points to this server"
    echo "  2. Port 80 and 443 are open in firewall"
    echo ""
    read -p "Proceed with SSL certificate? [y/N]: " proceed
    
    if [[ "$proceed" =~ ^[Yy]$ ]]; then
        # Reload nginx first (without SSL)
        systemctl reload nginx
        
        # Obtain certificate
        echo "📜 Obtaining SSL certificate..."
        certbot certonly --nginx \
            -d ash-api.online \
            -d "*.ash-api.online" \
            --agree-tos \
            --non-interactive \
            --email admin@ash-api.online || {
                echo "⚠️  Certificate generation failed. You can run it manually later:"
                echo "   sudo certbot certonly --nginx -d ash-api.online -d '*.ash-api.online'"
            }
        
        # Setup auto-renewal
        systemctl enable certbot.timer || true
    else
        echo "⚠️  Skipping SSL certificate. To obtain later, run:"
        echo "   sudo certbot certonly --nginx -d ash-api.online -d '*.ash-api.online'"
    fi
fi

# Reload nginx
echo "🔄 Reloading nginx..."
systemctl reload nginx

echo ""
echo "✅ Nginx setup complete!"
echo ""
echo "Configuration file: /etc/nginx/sites-available/ash-api.online"
echo ""
echo "Next steps:"
echo "  1. Ensure DDNS server is running: sudo systemctl status ddns-server"
echo "  2. Check nginx status: sudo systemctl status nginx"
echo "  3. View nginx logs: sudo tail -f /var/log/nginx/ddns-*.log"
echo ""
if [ "$USE_SSL" = true ]; then
    echo "Access your server at: https://ash-api.online"
else
    echo "Access your server at: http://ash-api.online"
fi
