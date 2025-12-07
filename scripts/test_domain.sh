#!/bin/bash
set -e

echo "🧪 Running tests for control-plane/crates/domain..."
cargo test -p domain --lib

echo "✅ Domain tests passed!"
