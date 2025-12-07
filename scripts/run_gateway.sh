#!/bin/bash
set -e

echo "🚀 Starting Drizzle Gatewayd..."
echo "Try: curl -v http://localhost:6188/get"
RUST_LOG=info cargo run -p gatewayd
