# DNS Server Setup Guide

## Overview
Your DDNS server now includes a built-in DNS server that responds to queries for your dynamic hostnames.

## How It Works

1. **Router updates IP** → HTTP POST to `/update` → Database stores IP
2. **DNS query arrives** → DNS server queries database → Returns current IP
3. **Result**: `ping home.ash-api.online` resolves to your router's current IP

## Configuration

### Environment Variables
```bash
DNS_PORT=5353  # Use 5353 for testing (53 requires root)
BASE_DOMAIN=ash-api.online
```

### DNS Server Details
- **Protocol**: UDP (standard DNS)
- **Port**: 5353 (or 53 for production)
- **Records Supported**: A (IPv4), AAAA (IPv6), SOA, NS
- **TTL**: 60 seconds (short for dynamic IPs)

## Production Setup

### 1. Update DNS Nameservers

Point your domain's nameservers to your VPS:

**At your domain registrar (e.g., Namecheap, GoDaddy):**
```
Nameserver 1: ns.ash-api.online → 178.128.26.139
```

Or use DigitalOcean's nameservers if domain is there.

### 2. Create NS and A Records

**In your DNS provider's control panel:**
```
Type  Name                Value               TTL
A     ns.ash-api.online   178.128.26.139     300
NS    ash-api.online      ns.ash-api.online  300
```

### 3. Run DNS Server on Port 53

DNS must run on port 53 (standard), which requires root/sudo:

```bash
# Update .env
DNS_PORT=53

# Update systemd service to run on port 53
sudo nano /etc/systemd/system/ddns-server.service
```

Add to `[Service]` section:
```ini
AmbientCapabilities=CAP_NET_BIND_SERVICE
```

Or run as root (not recommended):
```bash
sudo /opt/ddns-server/ddns-server
```

### 4. Open Firewall Port

```bash
# Allow UDP port 53 (DNS)
sudo ufw allow 53/udp

# Verify
sudo ufw status
```

### 5. Restart Service

```bash
sudo systemctl daemon-reload
sudo systemctl restart ddns-server
sudo systemctl status ddns-server
```

## Testing

### Local Testing (Port 5353)

```bash
# Test A record (IPv4)
dig @localhost -p 5353 home.ash-api.online A

# Test AAAA record (IPv6)
dig @localhost -p 5353 home.ash-api.online AAAA

# Test from another machine
dig @178.128.26.139 -p 5353 home.ash-api.online
```

### Production Testing (Port 53)

```bash
# Query your DNS server directly
dig @178.128.26.139 home.ash-api.online

# After NS records propagate (24-48 hours)
dig home.ash-api.online

# Test with ping
ping home.ash-api.online
ping6 home.ash-api.online
```

## Troubleshooting

### DNS Server Not Responding

```bash
# Check if DNS server is listening
sudo netstat -ulnp | grep 53

# Check logs
sudo journalctl -u ddns-server -f

# Test locally first
dig @127.0.0.1 -p 5353 home.ash-api.online
```

### "Connection refused"
- Check firewall: `sudo ufw status`
- Verify DNS_PORT in .env
- Ensure service is running: `systemctl status ddns-server`

### "NXDOMAIN" (domain not found)
- Verify hostname exists in database: `sqlite3 /opt/ddns-server/ddns.db "SELECT * FROM hosts"`
- Check hostname spelling matches exactly

### NS Records Not Propagating
- NS changes can take 24-48 hours
- Check with: `dig NS ash-api.online`
- Verify A record for ns.ash-api.online exists

## Architecture

```
Router/Client
    ↓ (HTTP POST /update)
Nginx (port 80/443)
    ↓
DDNS Server (port 8181)
    ↓
SQLite Database
    ↑
DNS Server (port 53/5353)
    ↑ (UDP queries)
Internet DNS Clients
```

## Security Notes

- DNS server only responds with IPs from your database
- Unknown hostnames return NXDOMAIN
- Short TTL (60s) means quick updates but more queries
- No DNSSEC support (implement if needed)

## Example Workflow

1. Router updates: `curl -u user:pass http://ash-api.online/update?hostname=home.ash-api.online&myip=2401:4900:1c1a:8fff::36d:8b2`
2. Database stores: `home.ash-api.online → 2401:4900:1c1a:8fff::36d:8b2`
3. DNS query: `dig home.ash-api.online AAAA`
4. DNS response: `home.ash-api.online. 60 IN AAAA 2401:4900:1c1a:8fff::36d:8b2`
5. Connection: Client connects to your router at that IP!
