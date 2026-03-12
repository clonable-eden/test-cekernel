DEVCONTAINER = pnpm exec devcontainer
WORKSPACE = --workspace-folder .

.PHONY: help setup up down rebuild test build run stop exec

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-10s\033[0m %s\n", $$1, $$2}'

node_modules: package.json
	pnpm install
	@touch $@

setup: node_modules ## Install host dependencies (devcontainer CLI)

up: node_modules ## Start devcontainer (skip if already running)
	@if ! docker ps -q --filter "label=devcontainer.local_folder=$$(pwd)" | grep -q .; then \
		$(DEVCONTAINER) up $(WORKSPACE); \
	fi

down: ## Stop and remove devcontainer
	@CONTAINER=$$(docker ps -aq --filter "label=devcontainer.local_folder=$$(pwd)"); \
	if [ -n "$$CONTAINER" ]; then docker rm -f $$CONTAINER; else echo "No container found"; fi

rebuild: down ## Rebuild and start devcontainer from scratch
	$(DEVCONTAINER) up $(WORKSPACE) --build-no-cache

test: up ## Run tests inside devcontainer
	$(DEVCONTAINER) exec $(WORKSPACE) cargo test

build: up ## Build inside devcontainer
	$(DEVCONTAINER) exec $(WORKSPACE) cargo build

run: up ## Start server in background
	@CONTAINER=$$(docker ps -q --filter "label=devcontainer.local_folder=$$(pwd)"); \
	docker exec -d -w /workspaces/$$(basename $$(pwd)) $$CONTAINER cargo run
	@echo "Server starting on http://localhost:3000"

stop: ## Stop the running server
	@CONTAINER=$$(docker ps -q --filter "label=devcontainer.local_folder=$$(pwd)"); \
	if [ -n "$$CONTAINER" ]; then docker exec $$CONTAINER pkill -f test-cekernel && echo "Server stopped" || echo "Server not running"; \
	else echo "No container found"; fi

exec: up ## Run arbitrary command (usage: make exec CMD="cargo clippy")
	$(DEVCONTAINER) exec $(WORKSPACE) $(CMD)
