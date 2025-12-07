#!/bin/bash
set -e

echo "👑 Starting Drizzle Admin API..."
echo "Try: curl -X POST -H 'Content-Type: application/json' -d '{\"slug\":\"test\",\"display_name\":\"Test\"}' http://localhost:3000/tenants"
echo "Try: curl http://localhost:3000/snapshot"

cargo run -p admin-api
