.PHONY: init build test fmt clippy tree
init: ; cargo --version
build: ; cargo build --workspace --all-targets
test: ; cargo test --workspace --all-targets -- --nocapture
fmt: ; cargo fmt --all
clippy: ; cargo clippy --workspace --all-targets -- -D warnings
# Portable dir listing (macOS/Linux)
tree:
\tpython - <<'PY'
import os
max_depth = 4
for root, dirs, files in os.walk('.', topdown=True):
    depth = root.count(os.sep)
    if depth > max_depth:
        dirs[:] = []
        continue
    print(root)
PY
