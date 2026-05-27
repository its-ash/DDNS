#!/bin/bash

# DNS Server Test Script
# Usage: ./test-dns.sh [port] [hostname]

DNS_PORT="${1:-5353}"
HOSTNAME="${2:-home.ash-api.online}"
SERVER="127.0.0.1"

echo "Testing DNS server at $SERVER:$DNS_PORT"
echo "=========================================="
echo ""

# Check if dig is installed
if ! command -v dig &> /dev/null; then
    echo "❌ 'dig' command not found. Install with: brew install bind"
    exit 1
fi

# Test A record (IPv4)
echo "1. Testing A record (IPv4) for $HOSTNAME"
dig @$SERVER -p $DNS_PORT $HOSTNAME A +short
echo ""

# Test AAAA record (IPv6)
echo "2. Testing AAAA record (IPv6) for $HOSTNAME"
dig @$SERVER -p $DNS_PORT $HOSTNAME AAAA +short
echo ""

# Test SOA record
echo "3. Testing SOA record"
dig @$SERVER -p $DNS_PORT ash-api.online SOA +short
echo ""

# Test NS record
echo "4. Testing NS record"
dig @$SERVER -p $DNS_PORT ash-api.online NS +short
echo ""

# Full query details
echo "5. Full query details for $HOSTNAME"
dig @$SERVER -p $DNS_PORT $HOSTNAME ANY
echo ""

# Test non-existent hostname
echo "6. Testing non-existent hostname (should return NXDOMAIN)"
dig @$SERVER -p $DNS_PORT nonexistent.ash-api.online A
echo ""

echo "=========================================="
echo "DNS test complete!"
