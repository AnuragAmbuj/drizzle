#!/bin/bash
set -e

# Configuration
METRICS_PORT=9111
BASE_URL="http://localhost:${METRICS_PORT}"

echo "🚀 Testing Analytics Endpoints at ${BASE_URL}..."

# 1. Test Logs Endpoint
echo "----------------------------------------"
echo "📜 Checking /observability/logs..."
LOGS_RESPONSE=$(curl -s "${BASE_URL}/observability/logs")

# Check if response is a JSON array
if [[ "$LOGS_RESPONSE" =~ ^\[.*\]$ ]]; then
    echo "✅ /observability/logs returned a JSON array"
    # Optional: check if length > 0
    COUNT=$(echo "$LOGS_RESPONSE" | grep -o "timestamp" | wc -l)
    echo "   Found $COUNT logs"
else
    echo "❌ /observability/logs invalid response: $(echo "$LOGS_RESPONSE" | head -n 1)"
    exit 1
fi

# 2. Test Stats Endpoint
echo "----------------------------------------"
echo "📈 Checking /observability/stats..."
STATS_RESPONSE=$(curl -s "${BASE_URL}/observability/stats")

# Check if response is a JSON array
if [[ "$STATS_RESPONSE" =~ ^\[.*\]$ ]]; then
    echo "✅ /observability/stats returned a JSON array"
     COUNT=$(echo "$STATS_RESPONSE" | grep -o "time" | wc -l)
    echo "   Found $COUNT stats buckets"
else
    echo "❌ /observability/stats invalid response: $(echo "$STATS_RESPONSE" | head -n 1)"
    exit 1
fi

echo "----------------------------------------"
echo "🎉 All Analytics tests passed!"
