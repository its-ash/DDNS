# Router DDNS Configuration Fix

## Issue
Routers using inadyn or similar DDNS clients fail with "301 Moved Permanently" error because nginx redirects all HTTP traffic to HTTPS, but most routers cannot follow redirects.

## Solution
The nginx configuration has been updated to:
1. Allow HTTP access to DDNS update endpoints (`/nic/update` and `/update`)
2. Allow HTTP access to `/redirect` endpoint
3. Redirect only the admin dashboard (`/`, `/login`, `/dashboard`) to HTTPS

## Updated Files
- `nginx/ash-api.online.conf` - Modified HTTP server block
- `src/main.rs` - Added `/update` route as alternative endpoint
- `src/handlers.rs` - Added update_ip_simple helper

## Router Configuration

Your router (D-Link DIR-3040) is using:
- **Hostname**: `home.ash-api.online`
- **Update URL**: `http://ash-api.online/update?hostname=home.ash-api.online`

This should now work correctly over HTTP without redirects.

### Supported Update Endpoints

Both endpoints are now available:

**Standard DynDNS endpoint:**
```
http://ash-api.online/nic/update?hostname=HOSTNAME&myip=IP
```

**Alternative endpoint (for routers that use it):**
```
http://ash-api.online/update?hostname=HOSTNAME&myip=IP
```

### Testing

Test the update manually:
```bash
# With explicit hostname and IP
curl -u username:password "http://ash-api.online/update?hostname=home.ash-api.online&myip=100.135.120.102"

# Auto-detect IP (from source address)
curl -u username:password "http://ash-api.online/update?hostname=home.ash-api.online"
```

Expected response:
```
good 100.135.120.102
```

## Security Note

The DDNS update endpoints are intentionally accessible via HTTP because:
1. Most routers don't support HTTPS for DDNS updates
2. Authentication is still required (Basic Auth)
3. Only IP update functionality is exposed, not the admin dashboard

If you want to restrict access by IP, uncomment the access control section in `nginx/ash-api.online.conf`:

```nginx
location ~ ^/(nic/update|update) {
    # Restrict to specific IPs
    allow 100.135.120.102;  # Your router's IP
    allow 1.2.3.4;          # Another allowed IP
    deny all;
    
    proxy_pass http://127.0.0.1:8181;
    # ... rest of config
}
```

## Deploying the Fix

### 1. Rebuild and restart the DDNS server:
```bash
cd /path/to/DDNS
./build.sh
sudo systemctl restart ddns-server
```

### 2. Update nginx configuration:
```bash
sudo cp nginx/ash-api.online.conf /etc/nginx/sites-available/ash-api.online
sudo nginx -t
sudo systemctl reload nginx
```

### 3. Test the endpoint:
```bash
# Replace with your actual credentials from the dashboard
curl -v -u i5nqisufpxxm:q1HAwwWkvWD0oxs7 "http://ash-api.online/update?hostname=home.ash-api.online&myip=100.135.120.102"
```

### 4. Check router logs:
Your router should now successfully update its IP without the 301 error.

## Troubleshooting

**Check if the endpoint is accessible:**
```bash
curl -I http://ash-api.online/update
# Should return 401 Unauthorized (auth required), not 301 redirect
```

**View nginx access logs:**
```bash
sudo tail -f /var/log/nginx/ddns-http-access.log
```

**View DDNS server logs:**
```bash
sudo journalctl -u ddns-server -f
```

**Verify router is sending correct request:**
The router should send:
- Method: GET
- URL: `/update?hostname=home.ash-api.online`
- Header: `Authorization: Basic <base64-encoded-credentials>`

## IPv6 Support

Your router detected IPv6 address `2401:4900:1c1a:8fff::36d:8b2`. The DDNS server will store whichever IP is provided in the update request.
