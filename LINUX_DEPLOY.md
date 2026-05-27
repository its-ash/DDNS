# Linux Build & Deployment Quick Reference

## Build Commands

```bash
# Simple build script
./build.sh

# Or use Makefile
make release        # Build optimized binary
make build          # Build debug version
make clean          # Remove build artifacts

# Or use cargo directly
cargo build --release
```

## Installation

```bash
# Automated installation
./build.sh
sudo ./install.sh

# Or with Makefile
make release
sudo make install
```

## Service Management

```bash
# Start/Stop/Restart
sudo systemctl start ddns-server
sudo systemctl stop ddns-server
sudo systemctl restart ddns-server

# Enable/Disable auto-start
sudo systemctl enable ddns-server
sudo systemctl disable ddns-server

# Check status
sudo systemctl status ddns-server

# View logs
sudo journalctl -u ddns-server -f
sudo journalctl -u ddns-server --since "1 hour ago"
```

## Makefile Commands

```bash
make help           # Show all commands
make release        # Build release binary
make install        # Install as systemd service (sudo)
make start          # Start service (sudo)
make stop           # Stop service (sudo)
make restart        # Restart service (sudo)
make status         # Check service status (sudo)
make logs           # Follow service logs (sudo)
make uninstall      # Remove service (sudo)
make init-db        # Initialize database
make run            # Run locally for development
make test           # Run tests
```

## File Locations (After Installation)

```
/opt/ddns-server/               # Installation directory
├── ddns-server                 # Binary
├── .env                        # Configuration
├── ddns.db                     # SQLite database
├── schema.sql                  # Database schema
└── templates/                  # HTML/CSS files
    ├── dashboard.html
    ├── login.html
    └── retro.css

/etc/systemd/system/
└── ddns-server.service        # Systemd service file
```

## Troubleshooting

```bash
# Check if service is running
sudo systemctl is-active ddns-server

# Check if service is enabled
sudo systemctl is-enabled ddns-server

# View full service status
sudo systemctl status ddns-server -l

# Check for errors in logs
sudo journalctl -u ddns-server -p err

# Restart after configuration change
sudo systemctl daemon-reload
sudo systemctl restart ddns-server

# Check port binding
sudo netstat -tulpn | grep 8080
# or
sudo ss -tulpn | grep 8080

# Test locally
curl http://localhost:8080

# Check file permissions
ls -la /opt/ddns-server/
```

## Configuration

Edit `/opt/ddns-server/.env`:

```bash
sudo nano /opt/ddns-server/.env
```

After editing, restart the service:

```bash
sudo systemctl restart ddns-server
```

## Uninstallation

```bash
# Stop and disable service
sudo systemctl stop ddns-server
sudo systemctl disable ddns-server

# Remove service file
sudo rm /etc/systemd/system/ddns-server.service
sudo systemctl daemon-reload

# Remove installation directory (optional)
sudo rm -rf /opt/ddns-server

# Remove user (optional)
sudo userdel ddns
```

Or use Makefile:

```bash
sudo make uninstall
# Then manually: sudo rm -rf /opt/ddns-server
```
