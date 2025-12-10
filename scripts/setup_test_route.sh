#!/bin/bash
ADMIN="http://localhost:3000"
# Login
LOGIN=$(curl -s -X POST $ADMIN/auth/login -H "Content-Type: application/json" -d '{"username":"admin","password":"admin123"}')
TOKEN=$(echo $LOGIN | jq -r .token)

# Get Tenant
SNAP=$(curl -s $ADMIN/snapshot -H "Authorization: Bearer $TOKEN")
TENANT=$(echo $SNAP | jq -r '.tenants[0].id')

# Create Service
echo "Creating Service..."
curl -s -X POST $ADMIN/services \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "tenant_id": "'$TENANT'",
    "name": "local-test",
    "host": "http://127.0.0.1:8080"
  }'

SNAP=$(curl -s $ADMIN/snapshot -H "Authorization: Bearer $TOKEN")
SID=$(echo $SNAP | jq -r '.services[] | select(.name=="local-test") | .id' | tail -n 1)

# Create Route
echo "Creating Route..."
curl -s -X POST $ADMIN/routes \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "service_id": "'$SID'",
    "name": "test-route",
    "path": "/test"
  }'

echo "Done. /test -> 127.0.0.1:8080"
