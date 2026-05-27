#!/usr/bin/env bash

# Nginx Quick Commands for ash-api.online

echo "🌐 DDNS Server - Nginx Quick Reference"
echo "========================================"
echo ""

# Check nginx status
echo "📊 Nginx Status:"
sudo systemctl status nginx --no-pager | grep -E "Active|Main PID" || echo "Nginx not running"
echo ""

# Check DDNS server status
echo "🖥️  DDNS Server Status:"
sudo systemctl status ddns-server --no-pager | grep -E "Active|Main PID" || echo "DDNS server not running"
echo ""

# Check listening ports
echo "🔌 Listening Ports:"
sudo netstat -tulpn 2>/dev/null | grep -E ":80|:443|:8181" || sudo ss -tulpn | grep -E ":80|:443|:8181"
echo ""

# Test nginx config
echo "✅ Nginx Config Test:"
sudo nginx -t 2>&1 | tail -n 2
echo ""

# Recent access logs
echo "📝 Recent Access (last 5):"
sudo tail -n 5 /var/log/nginx/ddns-access.log 2>/dev/null || echo "No access logs found"
echo ""

# Recent errors
echo "❌ Recent Errors (if any):"
sudo tail -n 3 /var/log/nginx/ddns-error.log 2>/dev/null | grep -v "^$" || echo "No recent errors"
echo ""

# SSL certificate info
echo "🔐 SSL Certificate:"
sudo certbot certificates 2>/dev/null | grep -A 5 "ash-api.online" || echo "No SSL certificate found"
echo ""

# Quick actions
echo "⚡ Quick Actions:"
echo "  Reload nginx:        sudo systemctl reload nginx"
echo "  Restart nginx:       sudo systemctl restart nginx"
echo "  Restart DDNS:        sudo systemctl restart ddns-server"
echo "  Follow access logs:  sudo tail -f /var/log/nginx/ddns-access.log"
echo "  Follow error logs:   sudo tail -f /var/log/nginx/ddns-error.log"
echo "  Follow DDNS logs:    sudo journalctl -u ddns-server -f"
echo ""
