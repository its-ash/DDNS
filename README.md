# DDNS Server

A lightweight Dynamic DNS server built with Rust and Actix-web. This server allows routers to update their IP addresses dynamically and provides hostname-to-IP redirection.

## Features

- 🔐 **Simple Admin Authentication** - Single password login from .env
- 🏠 **Host Management** - Create multiple DDNS hosts with auto-generated credentials
- 🔄 **DynDNS Compatible** - Works with most routers supporting custom DDNS
- 🌐 **IP Redirection** - Access routers via hostname
- 📊 **Web Dashboard** - Simple HTML interface for managing hosts
- 💾 **SQLite Database** - Lightweight, no external database required

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

# Session secret (min 32 characters)
SESSION_SECRET=change-this-to-a-random-secret-key-at-least-32-chars-long
```

### 4. Run the Server

```bash
# Development mode
cargo run

# Production build
cargo build --release
./target/release/ddns-server
```

The server will start on `http://0.0.0.0:8080`

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
