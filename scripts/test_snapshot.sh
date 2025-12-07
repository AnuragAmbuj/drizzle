#!/bin/bash
set -e

echo "📸 Running tests for gateway/crates/snapshot..."
cargo test -p snapshot --lib

echo "✅ Snapshot tests passed!"
