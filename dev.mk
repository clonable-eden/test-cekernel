PORT ?= 3000

.PHONY: help fmt fmt-check lint check ci test build build-release run stop clean

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-10s\033[0m %s\n", $$1, $$2}'

fmt: ## Format code
	cargo fmt

fmt-check: ## Check formatting (no auto-fix)
	cargo fmt --check

lint: ## Run clippy
	cargo clippy -- -D warnings

check: fmt lint test ## Full check (format → lint → test)

ci: fmt-check lint test build-release ## Reproduce CI locally (no auto-fix)

test: ## Run tests
	cargo test

build: ## Build the project
	cargo build

build-release: ## Build in release mode
	cargo build --release

run: ## Start server in background
	@cargo run &
	@echo "Server starting on http://0.0.0.0:$(PORT)/todos"

stop: ## Stop the running server
	@pkill -x test-cekernel && echo "Server stopped" || echo "Server not running"

clean: ## Remove build artifacts
	cargo clean
