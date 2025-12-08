#!/bin/bash
set -e

# Configuration
METRICS_PORT=9111
BASE_URL="http://localhost:${METRICS_PORT}"

echo "🚀 Testing Observability Endpoints at ${BASE_URL}..."

# 1. Test Health Probe
echo "----------------------------------------"
echo "📡 Checking /health..."
HEALTH_RESPONSE=$(curl -s "${BASE_URL}/health")
if [[ "$HEALTH_RESPONSE" == "OK" ]]; then
    echo "✅ /health is OK"
else
    echo "❌ /health failed: $HEALTH_RESPONSE"
    exit 1
fi

# 2. Test Readiness Probe
echo "----------------------------------------"
echo "📡 Checking /ready..."
READY_RESPONSE=$(curl -s "${BASE_URL}/ready")
if [[ "$READY_RESPONSE" == "OK" ]]; then
    echo "✅ /ready is OK"
else
    echo "❌ /ready failed: $READY_RESPONSE"
    exit 1
fi

# 3. Test Metrics Endpoint
echo "----------------------------------------"
echo "📊 Checking /metrics..."
METRICS_RESPONSE=$(curl -s "${BASE_URL}/metrics")

if echo "$METRICS_RESPONSE" | grep -q "drizzle_http_requests_total"; then
    echo "✅ /metrics contains 'drizzle_http_requests_total'"
else
    echo "❌ /metrics missing key metrics"
    echo "Response preview:"
    echo "$METRICS_RESPONSE" | head -n 5
    exit 1
fi

if echo "$METRICS_RESPONSE" | grep -q "drizzle_http_request_duration_seconds"; then
    echo "✅ /metrics contains 'drizzle_http_request_duration_seconds'"
else
    echo "❌ /metrics missing latency metrics"
    exit 1
fi

echo "----------------------------------------"
echo "🎉 All Observability tests passed!"
