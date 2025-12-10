#!/bin/bash
GATEWAY_URL="http://127.0.0.1:6188"
echo "Sending traffic to Drizzle Gateway at $GATEWAY_URL/ip..."

count=0
while true; do
  curl -s "$GATEWAY_URL/test" -o /dev/null -w "%{http_code}"
  echo " Request $((++count)) sent"
  sleep 0.1
done
echo "Done."
