.PHONY: init build test fmt clippy tree

# Initialize development environment
init:
	@echo "Checking Cargo version..."
	@cargo --version

# Build the project
build:
	@echo "Building project..."
	@cargo build --workspace --all-targets

# Run tests
test:
	@echo "Running tests..."
	@cargo test --workspace --all-targets -- --nocapture

# Format code
fmt:
	@echo "Formatting code..."
	@cargo fmt --all

# Run clippy
clippy:
	@echo "Running clippy..."
	@cargo clippy --workspace --all-targets -- -D warnings

# Install tree command if not exists
install-tree:
	@if ! command -v tree >/dev/null 2>&1; then \
		echo "Installing tree command..."; \
		if [ "$$(uname)" = "Darwin" ]; then \
			brew install tree; \
		else \
			sudo apt-get update && sudo apt-get install -y tree; \
		fi; \
	else \
		echo "tree is already installed"; \
	fi

# Show directory tree
tree: install-tree
	@echo "Directory tree:"
	@tree -L 3 -I 'target|.git|.idea|node_modules|dist|build' --dirsfirst
