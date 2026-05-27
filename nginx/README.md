# Nginx Configuration for DDNS Server

This directory contains nginx reverse proxy configurations for the DDNS server.

## Files

- `ash-api.online.conf` - Full HTTPS configuration with SSL
- `ash-api.online-http-only.conf` - HTTP-only configuration (for testing)
- `setup-nginx.sh` - Automated setup script

## Quick Setup

### Automated Installation

```bash
cd nginx
sudo ./setup-nginx.sh
```

This script will:
1. Install nginx and certbot (if needed)
2. Configure nginx reverse proxy
3. Optionally obtain SSL certificate via Let's Encrypt
4. Enable and reload nginx

### Manual Installation

#### Option 1: HTTPS with SSL (Recommended)

```bash
# Install nginx and certbot
sudo apt-get update
sudo apt-get install nginx certbot python3-certbot-nginx

# Copy configuration
sudo cp nginx/ash-api.online.conf /etc/nginx/sites-available/ash-api.online
sudo ln -s /etc/nginx/sites-available/ash-api.online /etc/nginx/sites-enabled/

# Create directory for certbot
sudo mkdir -p /var/www/certbot

# Test configuration
sudo nginx -t

# Obtain SSL certificate
sudo certbot certonly --nginx -d ash-api.online -d "*.ash-api.online"

# Reload nginx
sudo systemctl reload nginx
```

#### Option 2: HTTP Only (Testing)

```bash
# Install nginx
sudo apt-get update
sudo apt-get install nginx

# Copy configuration
sudo cp nginx/ash-api.online-http-only.conf /etc/nginx/sites-available/ash-api.online
sudo ln -s /etc/nginx/sites-available/ash-api.online /etc/nginx/sites-enabled/

# Test and reload
sudo nginx -t
sudo systemctl reload nginx
```

## Configuration Details

### Proxy Settings

The configuration proxies all requests to the DDNS server running on `127.0.0.1:8181`.

Key features:
- HTTP/2 support
- Proper header forwarding (Host, X-Real-IP, X-Forwarded-For)
- Static file caching
- Security headers
- WebSocket support (for future use)

### DNS Configuration

Ensure your DNS records point to your server:

```
# A record for main domain
ash-api.online.        A    YOUR_SERVER_IP

# Wildcard A record for subdomains
*.ash-api.online.      A    YOUR_SERVER_IP
```

### Firewall Configuration

Open required ports:

```bash
# For HTTP
sudo ufw allow 80/tcp

# For HTTPS
sudo ufw allow 443/tcp

# Verify
sudo ufw status
```

## SSL Certificate Management

### Initial Certificate

```bash
sudo certbot certonly --nginx -d ash-api.online -d "*.ash-api.online"
```

Note: For wildcard certificates, you may need DNS validation:

```bash
sudo certbot certonly --manual --preferred-challenges dns -d ash-api.online -d "*.ash-api.online"
```

### Auto-Renewal

Certbot automatically sets up renewal. Verify:

```bash
# Check timer
sudo systemctl status certbot.timer

# Test renewal (dry run)
sudo certbot renew --dry-run
```

### Manual Renewal

```bash
sudo certbot renew
sudo systemctl reload nginx
```

## Troubleshooting

### Check Nginx Status

```bash
sudo systemctl status nginx
```

### View Logs

```bash
# Access logs
sudo tail -f /var/log/nginx/ddns-access.log

# Error logs
sudo tail -f /var/log/nginx/ddns-error.log

# All nginx logs
sudo tail -f /var/log/nginx/*.log
```

### Test Configuration

```bash
sudo nginx -t
```

### Verify Proxy is Working

```bash
# From the server
curl -I http://localhost:8181

# Through nginx
curl -I http://ash-api.online
curl -I https://ash-api.online
```

### Check if DDNS Server is Running

```bash
sudo systemctl status ddns-server
sudo journalctl -u ddns-server -f
```

### Port Already in Use

```bash
# Check what's using port 80/443
sudo netstat -tulpn | grep :80
sudo netstat -tulpn | grep :443

# Stop default nginx site if needed
sudo rm /etc/nginx/sites-enabled/default
sudo systemctl reload nginx
```

### SSL Certificate Issues

```bash
# Check certificate validity
sudo certbot certificates

# Renew specific certificate
sudo certbot renew --cert-name ash-api.online

# Delete and recreate
sudo certbot delete --cert-name ash-api.online
sudo certbot certonly --nginx -d ash-api.online -d "*.ash-api.online"
```

## Performance Tuning

For high-traffic scenarios, edit `/etc/nginx/nginx.conf`:

```nginx
http {
    # Connection settings
    keepalive_timeout 65;
    keepalive_requests 100;
    
    # Buffer sizes
    client_body_buffer_size 128k;
    client_max_body_size 10M;
    
    # Gzip compression
    gzip on;
    gzip_vary on;
    gzip_types text/plain text/css application/json application/javascript text/xml application/xml;
    
    # Rate limiting (optional)
    limit_req_zone $binary_remote_addr zone=ddns_limit:10m rate=10r/s;
}
```

Then in your server block:

```nginx
location / {
    limit_req zone=ddns_limit burst=20 nodelay;
    # ... rest of config
}
```

## Uninstallation

```bash
# Remove nginx configuration
sudo rm /etc/nginx/sites-enabled/ash-api.online
sudo rm /etc/nginx/sites-available/ash-api.online
sudo systemctl reload nginx

# Remove SSL certificate (optional)
sudo certbot delete --cert-name ash-api.online
```

## Security Recommendations

1. **Keep software updated:**
   ```bash
   sudo apt-get update && sudo apt-get upgrade
   ```

2. **Enable fail2ban for nginx:**
   ```bash
   sudo apt-get install fail2ban
   sudo systemctl enable fail2ban
   ```

3. **Use strong SSL configuration:**
   - The provided config uses TLS 1.2+ only
   - Modern cipher suites
   - HSTS enabled

4. **Monitor logs regularly:**
   ```bash
   sudo tail -f /var/log/nginx/ddns-access.log
   ```

5. **Consider rate limiting** for the `/nic/update` endpoint to prevent abuse
