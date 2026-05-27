#!/usr/bin/env bash

echo "🧪 Testing DDNS Update Endpoints"
echo "================================"
echo ""

DOMAIN="ash-api.online"
HOSTNAME="home.ash-api.online"

# Test 1: Check if /update returns 401 (not 301)
echo "Test 1: HTTP /update endpoint (should return 401 Unauthorized, not 301)"
RESPONSE=$(curl -s -o /dev/null -w "%{http_code}" "http://${DOMAIN}/update")
if [ "$RESPONSE" = "401" ]; then
    echo "✅ PASS - Endpoint returns 401 (authentication required)"
elif [ "$RESPONSE" = "301" ]; then
    echo "❌ FAIL - Endpoint returns 301 (redirect to HTTPS)"
    echo "   The router fix is not deployed yet"
else
    echo "⚠️  UNEXPECTED - Endpoint returns $RESPONSE"
fi
echo ""

# Test 2: Check if /nic/update returns 401
echo "Test 2: HTTP /nic/update endpoint (should return 401 Unauthorized)"
RESPONSE=$(curl -s -o /dev/null -w "%{http_code}" "http://${DOMAIN}/nic/update")
if [ "$RESPONSE" = "401" ]; then
    echo "✅ PASS - Endpoint returns 401 (authentication required)"
elif [ "$RESPONSE" = "301" ]; then
    echo "❌ FAIL - Endpoint returns 301 (redirect to HTTPS)"
else
    echo "⚠️  UNEXPECTED - Endpoint returns $RESPONSE"
fi
echo ""

# Test 3: Check if / redirects to HTTPS
echo "Test 3: HTTP / (dashboard) should redirect to HTTPS"
RESPONSE=$(curl -s -o /dev/null -w "%{http_code}" "http://${DOMAIN}/")
if [ "$RESPONSE" = "301" ] || [ "$RESPONSE" = "302" ]; then
    echo "✅ PASS - Dashboard redirects to HTTPS"
else
    echo "⚠️  Dashboard returns $RESPONSE (expected 301/302)"
fi
echo ""

# Test 4: Full update test with credentials (if provided)
if [ -n "$1" ] && [ -n "$2" ]; then
    USERNAME=$1
    PASSWORD=$2
    echo "Test 4: Full update with credentials"
    RESPONSE=$(curl -s -u "${USERNAME}:${PASSWORD}" "http://${DOMAIN}/update?hostname=${HOSTNAME}&myip=1.2.3.4")
    if echo "$RESPONSE" | grep -q "good"; then
        echo "✅ PASS - Update successful: $RESPONSE"
    elif echo "$RESPONSE" | grep -q "badauth"; then
        echo "⚠️  Authentication failed - check credentials"
    elif echo "$RESPONSE" | grep -q "nohost"; then
        echo "⚠️  Host not found - create ${HOSTNAME} in dashboard"
    else
        echo "Response: $RESPONSE"
    fi
else
    echo "Test 4: Skipped (no credentials provided)"
    echo "   Run with credentials: ./test-ddns-fix.sh USERNAME PASSWORD"
fi
echo ""

echo "================================"
echo "Summary:"
echo "  Domain: $DOMAIN"
echo "  Update endpoints: /update and /nic/update"
echo "  Expected: HTTP allowed (401), dashboard redirects to HTTPS (301)"
