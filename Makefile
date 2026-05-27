.PHONY: build release clean install uninstall deploy start stop status logs test

# Build targets
build:
	@echo "🔨 Building debug version..."
	cargo build

release:
	@echo "🔨 Building release version..."
	cargo build --release
	@echo "✅ Binary: target/release/ddns-server"

# Run locally
run:
	cargo run

# Clean build artifacts
clean:
	@echo "🧹 Cleaning build artifacts..."
	cargo clean

# Install as systemd service (requires root)
install:
	@./install.sh

# Uninstall systemd service (requires root)
uninstall:
	@if [ "$$(id -u)" -ne 0 ]; then \
		echo "❌ Please run as root (use sudo)"; \
		exit 1; \
	fi
	@echo "🗑️  Uninstalling DDNS Server..."
	@systemctl kill ddns-server 2>/dev/null || true
	@sleep 2
	@systemctl stop ddns-server 2>/dev/null || true
	@systemctl disable ddns-server 2>/dev/null || true
	@rm -f /etc/systemd/system/ddns-server.service
	@systemctl daemon-reload 2>/dev/null || true
	@echo "⚠️  Service files removed. User data kept in /opt/ddns-server"
	@echo "   To completely remove: sudo rm -rf /opt/ddns-server"

# Deploy: build, uninstall, and install (requires root)
deploy: release
	@if [ "$$(id -u)" -ne 0 ]; then \
		echo "❌ Please run as root (use sudo make deploy)"; \
		exit 1; \
	fi
	@echo ""
	@echo "🚀 Deploying DDNS Server..."
	@echo ""
	@$(MAKE) uninstall 2>/dev/null || true
	@echo ""
	@./install.sh
	@echo ""
	@echo "🔄 Starting service..."
	@systemctl reset-failed ddns-server 2>/dev/null || true
	@systemctl start ddns-server
	@sleep 2
	@echo ""
	@echo "✅ Deployment complete!"
	@systemctl status ddns-server --no-pager || true

# Service management (requires root)
start:
	@sudo systemctl start ddns-server
	@echo "✅ Service started"

stop:
	@sudo systemctl stop ddns-server
	@echo "⏹️  Service stopped"

restart:
	@sudo systemctl restart ddns-server
	@echo "🔄 Service restarted"

status:
	@sudo systemctl status ddns-server

logs:
	@sudo journalctl -u ddns-server -f

# Development
test:
	cargo test

check:
	cargo check

fmt:
	cargo fmt

# Database
init-db:
	@if [ -f "ddns.db" ]; then \
		echo "⚠️  Database already exists"; \
	else \
		sqlite3 ddns.db < schema.sql; \
		echo "✅ Database initialized"; \
	fi

# Help
help:
	@echo "DDNS Server - Makefile Commands"
	@echo ""
	@echo "Build commands:"
	@echo "  make build         - Build debug version"
	@echo "  make release       - Build release version (optimized)"
	@echo "  make clean         - Remove build artifacts"
	@echo ""
	@echo "Development:"
	@echo "  make run           - Run server locally"
	@echo "  make test          - Run tests"
	@echo "  make check         - Check code without building"
	@echo "  make fmt           - Format code"
	@echo "  make init-db       - Initialize SQLite database"
	@echo ""
	@echo "Installation (requires sudo):"
	@echo "  make install       - Install as systemd service"
	@echo "  make uninstall     - Remove systemd service"
	@echo "  make deploy        - Build, uninstall, and install (full deployment)"
	@echo ""
	@echo "Service management (requires sudo):"
	@echo "  make start         - Start service"
	@echo "  make stop          - Stop service"
	@echo "  make restart       - Restart service"
	@echo "  make status        - Show service status"
	@echo "  make logs          - Follow service logs"
