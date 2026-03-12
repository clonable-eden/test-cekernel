DEVCONTAINER = pnpm exec devcontainer
WORKSPACE = --workspace-folder .

.PHONY: help setup up test build run exec

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-10s\033[0m %s\n", $$1, $$2}'

node_modules: package.json
	pnpm install
	@touch $@

setup: node_modules ## Install host dependencies (devcontainer CLI)

up: node_modules ## Start devcontainer
	$(DEVCONTAINER) up $(WORKSPACE)

test: ## Run tests inside devcontainer
	$(DEVCONTAINER) exec $(WORKSPACE) cargo test

build: ## Build inside devcontainer
	$(DEVCONTAINER) exec $(WORKSPACE) cargo build

run: ## Run server inside devcontainer
	$(DEVCONTAINER) exec $(WORKSPACE) cargo run

exec: ## Run arbitrary command (usage: make exec CMD="cargo clippy")
	$(DEVCONTAINER) exec $(WORKSPACE) $(CMD)
