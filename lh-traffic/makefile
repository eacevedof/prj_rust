SHELL := /bin/bash
CARGO := rustup run stable cargo
TODAY := $(shell date +'%Y-%m-%d')
CURRENT_BRANCH := $(shell git rev-parse --abbrev-ref HEAD)

help: ## Show this help message
	@echo "usage: make [target]"
	@echo
	@echo "targets:"
	@egrep "^(.+)\:\ ##\ (.+)" ${MAKEFILE_LIST} | column -t -c 2 -s ":#"

build: ## Build the project
	$(CARGO) build --release

check: ## Check code without building
	$(CARGO) check

test: ## Run tests
	$(CARGO) test

run-list-connections: ## List all network connections
	$(CARGO) run --bin list-network-connections

run-list-connections-filter: ## List network connections with filter (use f=filter)
	$(CARGO) run --bin list-network-connections $(f)

run-watch-connections: ## Watch network connections (auto-refresh every 10s)
	$(CARGO) run --bin watch-network-connections

run-watch-connections-filter: ## Watch network connections with filter (use f=filter)
	$(CARGO) run --bin watch-network-connections $(f)

clean: ## Clean build artifacts
	$(CARGO) clean

gitpush: ## git push m=any message
	clear;
	git add .; git commit -m "$(m)"; git push;

