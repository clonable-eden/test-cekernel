DEVCONTAINER = pnpm exec devcontainer
WORKSPACE = --workspace-folder .
DEV = $(DEVCONTAINER) exec $(WORKSPACE) make -f dev.mk

.PHONY: help setup up down rebuild fmt lint check ci test build run stop clean port shell exec

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

fmt: up ## Format code (delegates to dev.mk)
	$(DEV) fmt

lint: up ## Run clippy (delegates to dev.mk)
	$(DEV) lint

check: up ## Full check: format → lint → test (delegates to dev.mk)
	$(DEV) check

ci: up ## Reproduce CI locally (delegates to dev.mk)
	$(DEV) ci

test: up ## Run tests (delegates to dev.mk)
	$(DEV) test

build: up ## Build (delegates to dev.mk)
	$(DEV) build

run: up ## Start server in background (delegates to dev.mk)
	@CONTAINER=$$(docker ps -q --filter "label=devcontainer.local_folder=$$(pwd)"); \
	docker exec -d -w /workspaces/$$(basename $$(pwd)) $$CONTAINER make -f dev.mk run
	@PORT=$$(docker port $$(docker ps -q --filter "label=devcontainer.local_folder=$$(pwd)") 3000 2>/dev/null | head -1); \
	echo "Server starting on http://$$PORT/todos"

stop: up ## Stop the running server (delegates to dev.mk)
	$(DEV) stop

clean: up ## Remove build artifacts (delegates to dev.mk)
	$(DEV) clean

port: ## Show the host port mapped to the server
	@docker port $$(docker ps -q --filter "label=devcontainer.local_folder=$$(pwd)") 3000 2>/dev/null || echo "Container not running"

shell: up ## Open a shell inside devcontainer
	@CONTAINER=$$(docker ps -q --filter "label=devcontainer.local_folder=$$(pwd)"); \
	docker exec -it -w /workspaces/$$(basename $$(pwd)) $$CONTAINER bash

exec: up ## Run arbitrary command (usage: make exec CMD="cargo clippy")
	$(DEVCONTAINER) exec $(WORKSPACE) $(CMD)
