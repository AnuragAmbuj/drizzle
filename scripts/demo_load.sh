#!/bin/bash
set -e

ADMIN_URL="http://localhost:3000"
GATEWAY_URL="http://localhost:6188"
OPS_URL="http://localhost:9111"

echo "1. Logging in..."
LOGIN_RES=$(curl -s -X POST $ADMIN_URL/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username": "admin", "password": "admin123"}')

TOKEN=$(echo $LOGIN_RES | jq -r .token)
echo "Token: ${TOKEN:0:10}..."

echo -e "\n2. Creating Tenant 'demo-tenant'..."
curl -s -X POST $ADMIN_URL/tenants \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "slug": "demo-tenant",
    "display_name": "Demo Tenant"
  }'

# Get Tenant ID
TENANTS_RES=$(curl -s $ADMIN_URL/snapshot \
  -H "Authorization: Bearer $TOKEN")
TENANT_ID=$(echo $TENANTS_RES | jq -r '.tenants[] | select(.slug=="demo-tenant") | .id')
echo "Tenant ID: $TENANT_ID"

echo -e "\n3. Creating Service 'httpbin'..."
curl -s -X POST $ADMIN_URL/services \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d "{
    \"tenant_id\": \"$TENANT_ID\",
    \"name\": \"httpbin\",
    \"host\": \"https://httpbin.org\"
  }"

SERVICES_RES=$(curl -s $ADMIN_URL/snapshot \
  -H "Authorization: Bearer $TOKEN")
SERVICE_ID=$(echo $SERVICES_RES | jq -r ".services[] | select(.name==\"httpbin\" and .tenant_id==\"$TENANT_ID\") | .id" | tail -n 1)
echo "Service ID: $SERVICE_ID"

echo -e "\n4. Creating Route '/ip'..."
curl -s -X POST $ADMIN_URL/routes \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d "{
    \"service_id\": \"$SERVICE_ID\",
    \"name\": \"demo-route\",
    \"path\": \"/ip\"
  }"

echo ""
echo "4.5. Creating 'allow-all' Policy..."
curl -s -X POST $ADMIN_URL/policies \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "tenant_id": "'$TENANT_ID'",
    "name": "allow-all",
    "content": "permit(principal, action, resource);"
  }'
echo ""

echo -e "\n5. Waiting for 10s for Gateway to sync..."
sleep 11

echo -e "\n6. Sending Requests to Gateway..."
for i in {1..5}; do
    echo "Request $i:"
    curl -i "$GATEWAY_URL/ip"
    echo -e "\n"
    sleep 0.5
done

echo -e "\n7. Checking Observability Logs..."
curl -s "$OPS_URL/observability/logs" | jq '.'

echo -e "\nDone."
