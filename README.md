# DDNS Server

A lightweight Dynamic DNS server built with Rust and Actix-web. This server allows routers to update their IP addresses dynamically and provides hostname-to-IP redirection.

## Features

- 🔐 **Simple Admin Authentication** - Single password login from .env
- 🏠 **Host Management** - Create multiple DDNS hosts with auto-generated credentials
- 🔄 **DynDNS Compatible** - Works with most routers supporting custom DDNS
- 🌐 **IP Redirection** - Access routers via hostname
- 📊 **Web Dashboard** - Retro 90s Windows 95 aesthetic interface
- 💾 **SQLite Database** - Lightweight, no external database required
- 🔒 **Nginx Support** - Production-ready reverse proxy with SSL
- 🔍 **Built-in DNS Server** - Real DNS resolution for dynamic hostnames (A/AAAA records)

## Quick Start

### 1. Prerequisites

- Rust (1.70 or later)
- Cargo

### 2. Installation

```bash
# Clone or navigate to the project directory
cd ddns-server

# Copy the example environment file
cp .env.example .env

# Edit .env and set your configuration
nano .env
```

### 3. Configuration

Edit `.env` file:

```env
# Admin login password
ADMIN_PASSWORD=your-secure-password-here

# Base domain for DDNS hosts
BASE_DOMAIN=yourdomain.com

# Database path
DATABASE_URL=sqlite:ddns.db

# Server binding
HOST=0.0.0.0
PORT=8080

# DNS Server (optional - for real DNS resolution)
DNS_PORT=5353  # Use 53 for production (requires root)

# Session secret (min 64 characters)
SESSION_SECRET=change-this-to-a-random-secret-key-at-least-64-chars-long
```

### 4. Run the Server

**Option A: Quick Start (Development)**
```bash
cargo run
```

**Option B: Production Build**
```bash
# Build release binary
./build.sh
# OR
make release

# Run directly
./target/release/ddns-server
```

**Option C: Install as Linux Service**
```bash
# Build and install as systemd service
./build.sh
sudo ./install.sh

# Start the service
sudo systemctl start ddns-server
sudo systemctl enable ddns-server  # Auto-start on boot

# Check status
sudo systemctl status ddns-server

# View logs
sudo journalctl -u ddns-server -f
```

The server will start on `http://0.0.0.0:8080`

## Linux Deployment

### Using Makefile

```bash
# Build optimized binary
make release

# Initialize database
make init-db

# Install as systemd service (requires sudo)
sudo make install

# Service management
sudo make start        # Start service
sudo make stop         # Stop service
sudo make restart      # Restart service
sudo make status       # View status
sudo make logs         # Follow logs

# Development
make run              # Run locally
make test             # Run tests
make clean            # Clean build artifacts

# Uninstall
sudo make uninstall
```

### Manual Installation

1. **Build the binary:**
   ```bash
   cargo build --release
   ```

2. **Create service user:**
   ```bash
   sudo useradd -r -s /bin/false -d /opt/ddns-server ddns
   ```

3. **Install files:**
   ```bash
   sudo mkdir -p /opt/ddns-server/templates
   sudo cp target/release/ddns-server /opt/ddns-server/
   sudo cp templates/* /opt/ddns-server/templates/
   sudo cp .env /opt/ddns-server/
   sudo chown -R ddns:ddns /opt/ddns-server
   ```

4. **Install systemd service:**
   ```bash
   sudo cp ddns-server.service /etc/systemd/system/
   sudo systemctl daemon-reload
   sudo systemctl start ddns-server
   sudo systemctl enable ddns-server
   ```

### Service Files

- `build.sh` - Build script for release binary
- `install.sh` - Automated installation script
- `ddns-server.service` - Systemd service unit file
- `Makefile` - Common build and deployment commands

## Nginx Reverse Proxy Setup

For production deployments with HTTPS and domain name:

```bash
# Quick setup (automated)
cd nginx
sudo ./setup-nginx.sh

# Manual setup
sudo cp nginx/ash-api.online.conf /etc/nginx/sites-available/ash-api.online
sudo ln -s /etc/nginx/sites-available/ash-api.online /etc/nginx/sites-enabled/
sudo nginx -t
sudo systemctl reload nginx

# Obtain SSL certificate
sudo certbot certonly --nginx -d ash-api.online -d "*.ash-api.online"
```

See [nginx/README.md](nginx/README.md) for detailed nginx configuration guide.

**Quick status check:**
```bash
cd nginx
./check-status.sh
```

## DNS Server Setup

The built-in DNS server allows real DNS resolution of your dynamic hostnames.

### Quick Test (Port 5353)

```bash
# Start the server (DNS runs automatically alongside HTTP server)
cargo run

# Test DNS resolution
./test-dns.sh
# OR manually:
dig @localhost -p 5353 home.ash-api.online AAAA
```

### Production Setup (Port 53)

See [DNS-SETUP.md](DNS-SETUP.md) for complete DNS server configuration guide.

**Quick overview:**
1. Set `DNS_PORT=53` in `.env`
2. Configure firewall: `sudo ufw allow 53/udp`
3. Update domain nameservers to point to your server
4. Create NS and A records for your nameserver

**Test production DNS:**
```bash
# Query your server directly
dig @your-server-ip home.ash-api.online

# After NS propagation
ping home.ash-api.online
```

## Usage

### Admin Dashboard

1. Open `http://your-server-ip:8080` in your browser
2. Login with the password from `.env`
3. Create a new host by entering a subdomain
4. Copy the generated username and password

### Router Configuration

Configure your router's DDNS settings:

**Option 1: Standard DynDNS Protocol**
- Service: Custom or DynDNS
- Server: `your-server-ip:8080`
- Update URL: `http://your-server-ip:8080/nic/update`
- Username: (from dashboard)
- Password: (from dashboard)
- Hostname: `subdomain.yourdomain.com`

**Option 2: Custom Update URL**
```
http://your-server-ip:8080/nic/update?hostname=%HOSTNAME%&myip=%IP%
```

Replace `%HOSTNAME%` and `%IP%` with your router's variables.

### Manual IP Update

Test DDNS update via command line:

```bash
curl -u username:password "http://your-server-ip:8080/nic/update?myip=1.2.3.4"
```

Response codes:
- `good` - Update successful
- `noauth` - Missing authentication
- `badauth` - Invalid credentials
- `noip` - No IP address provided
- `dnserr` - Server error

### Accessing via Hostname

To redirect to your router when accessing the hostname:

```bash
curl -H "Host: subdomain.yourdomain.com" http://your-server-ip:8080/redirect
```

For production use, configure DNS to point your domain to the DDNS server.

## API Endpoints

| Endpoint | Method | Auth | Description |
|----------|--------|------|-------------|
| `/` | GET | None | Login page |
| `/login` | POST | None | Admin login |
| `/dashboard` | GET | Session | Admin dashboard |
| `/host/create` | POST | Session | Create new host |
| `/host/delete/:id` | POST | Session | Delete host |
| `/nic/update` | GET | Basic | Update IP (router endpoint) |
| `/redirect` | GET | None | Redirect to host IP |

## DNS Configuration

For production deployment:

1. Point your domain's A record to your server's IP
2. Create a wildcard A record: `*.yourdomain.com` → your server IP
3. Routers will update their IPs via the DDNS endpoint
4. Users can access `subdomain.yourdomain.com` and get redirected to the router's current IP

## Development

### Project Structure

```
ddns-server/
├── src/
│   ├── main.rs          # Application entry point
│   ├── models.rs        # Data models
│   ├── db.rs            # Database operations
│   └── handlers.rs      # HTTP request handlers
├── templates/
│   ├── login.html       # Login page
│   └── dashboard.html   # Admin dashboard
├── Cargo.toml           # Rust dependencies
├── .env.example         # Example configuration
└── README.md            # This file
```

### Database Schema

```sql
CREATE TABLE hosts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    hostname TEXT NOT NULL UNIQUE,
    username TEXT NOT NULL UNIQUE,
    password TEXT NOT NULL,
    current_ip TEXT,
    last_update DATETIME,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

## Security Notes

- Change `ADMIN_PASSWORD` to a strong password
- Use HTTPS in production (reverse proxy with nginx/caddy)
- Keep `SESSION_SECRET` secret and random
- Consider implementing rate limiting for production
- Store `.env` securely and never commit it to version control

## License

MIT

## Support

For issues and questions, please open an issue on the repository.
