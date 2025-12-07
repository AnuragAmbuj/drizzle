#!/bin/bash
set -e

ADMIN_URL="http://localhost:3000"
GATEWAY_URL="http://localhost:6188"

echo "🧪 Starting E2E Verification..."

# 1. Check Admin API
echo "Checking Admin API..."
if curl -s "$ADMIN_URL/snapshot" > /dev/null; then
    echo "✅ Admin API is up"
else
    echo "❌ Admin API is down (expected at $ADMIN_URL)"
    exit 1
fi

# 2. Check Gateway
echo "Checking Gateway..."
# Access a dummy endpoint (gateway currently proxies everything to httpbin, but we just check connectivity)
if curl -s -o /dev/null -w "%{http_code}" "$GATEWAY_URL" > /dev/null; then
    echo "✅ Gateway is up"
else
    echo "❌ Gateway is down (expected at $GATEWAY_URL)"
    exit 1
fi

# 3. Create Tenant
SLUG="e2e-test-$(date +%s)"
echo "Creating tenant $SLUG..."
curl -s -X POST -H 'Content-Type: application/json' \
    -d "{\"slug\":\"$SLUG\",\"display_name\":\"E2E Test Tenant\"}" \
    "$ADMIN_URL/tenants" > /dev/null

echo "✅ Tenant created"

# 4. Verify Gateway Snapshot
# Since we can't query the gateway for its version directly in this MVP (no inspection API), 
# we rely on the fact that if it's polling, it should be logging updates.
# For this script, we just check that the admin API returns the new tenant in snapshot.

SNAPSHOT=$(curl -s "$ADMIN_URL/snapshot")
if echo "$SNAPSHOT" | grep -q "$SLUG"; then
    echo "✅ Tenant found in Admin Snapshot"
else
    echo "❌ Tenant NOT found in Admin Snapshot"
    exit 1
fi

echo "✅ E2E Verification Complete (Client Side)"
echo "⚠️  Check Gateway logs to confirm polling update."
