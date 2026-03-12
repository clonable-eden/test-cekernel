PORT ?= 3000

.PHONY: help test build run stop clean

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-10s\033[0m %s\n", $$1, $$2}'

test: build ## Run tests
	cargo test

build: ## Build the project
	cargo build

run: ## Start server in background
	@cargo run &
	@echo "Server starting on http://0.0.0.0:$(PORT)/todos"

stop: ## Stop the running server
	@pkill -x test-cekernel && echo "Server stopped" || echo "Server not running"

clean: ## Remove build artifacts
	cargo clean
