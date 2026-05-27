#!/usr/bin/env bash
set -e

if [ "$EUID" -ne 0 ]; then 
    echo "❌ Please run as root (use sudo)"
    exit 1
fi

echo "📦 Installing DDNS Server..."

# Check if binary exists
if [ ! -f "target/release/ddns-server" ]; then
    echo "❌ Binary not found. Run './build.sh' first"
    exit 1
fi

# Create user if doesn't exist
if ! id -u ddns &>/dev/null; then
    echo "👤 Creating ddns user..."
    useradd -r -s /bin/false -d /opt/ddns-server ddns
fi

# Create installation directory
echo "📁 Creating installation directory..."
mkdir -p /opt/ddns-server/templates

# Copy files
echo "📋 Copying files..."
cp target/release/ddns-server /opt/ddns-server/
cp templates/*.html /opt/ddns-server/templates/
cp templates/*.css /opt/ddns-server/templates/

# Copy schema.sql if it exists
if [ -f "schema.sql" ]; then
    cp schema.sql /opt/ddns-server/
    echo "✅ Copied schema.sql"
else
    echo "⚠️  schema.sql not found, creating it..."
    cat > /opt/ddns-server/schema.sql << 'EOF'
-- Create hosts table
CREATE TABLE IF NOT EXISTS hosts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    hostname TEXT NOT NULL UNIQUE,
    username TEXT NOT NULL UNIQUE,
    password TEXT NOT NULL,
    current_ip TEXT,
    last_update DATETIME,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_hostname ON hosts(hostname);
CREATE INDEX idx_username ON hosts(username);
EOF
    echo "✅ Created schema.sql"
fi

# Copy .env if it exists
if [ -f ".env" ]; then
    cp .env /opt/ddns-server/
    echo "✅ Copied .env file"
else
    echo "⚠️  No .env file found. You'll need to create one at /opt/ddns-server/.env"
    cat > /opt/ddns-server/.env.example << 'EOF'
ADMIN_PASSWORD=changeme
BASE_DOMAIN=example.com
DATABASE_URL=sqlite:ddns.db
HOST=0.0.0.0
PORT=8080
SESSION_SECRET=your-64-character-secret-key-here-change-this-to-something-random
EOF
    echo "   Created /opt/ddns-server/.env.example as template"
fi

# Set permissions
echo "🔒 Setting permissions..."
chown -R ddns:ddns /opt/ddns-server
chmod 755 /opt/ddns-server
chmod 755 /opt/ddns-server/ddns-server
chmod 640 /opt/ddns-server/.env 2>/dev/null || true

# Install systemd service
echo "🔧 Installing systemd service..."
cp ddns-server.service /etc/systemd/system/
systemctl daemon-reload

echo ""
echo "✅ Installation complete!"
echo ""
echo "Next steps:"
echo "  1. Edit configuration: sudo nano /opt/ddns-server/.env"
echo "  2. Initialize database: sudo -u ddns sqlite3 /opt/ddns-server/ddns.db < /opt/ddns-server/schema.sql"
echo "  3. Start service: sudo systemctl start ddns-server"
echo "  4. Enable on boot: sudo systemctl enable ddns-server"
echo "  5. Check status: sudo systemctl status ddns-server"
echo "  6. View logs: sudo journalctl -u ddns-server -f"
