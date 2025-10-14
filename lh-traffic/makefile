#!/bin/bash
TODAY := $(shell date +'%Y-%m-%d')
CURRENT_BRANCH := $(shell git rev-parse --abbrev-ref HEAD)

help: ## Show this help message
	@echo "usage: make [target]"
	@echo
	@echo "targets:"
	@egrep "^(.+)\:\ ##\ (.+)" ${MAKEFILE_LIST} | column -t -c 2 -s ":#"

build: ## Build the project
	cargo build --release

check: ## Check code without building
	cargo check

test: ## Run tests
	cargo test

run-list-connections: ## List all network connections
	cargo run --bin list-network-connections

run-list-connections-filter: ## List network connections with filter (use f=filter)
	cargo run --bin list-network-connections $(f)

run-watch-connections: ## Watch network connections (auto-refresh every 10s)
	cargo run --bin watch-network-connections

run-watch-connections-filter: ## Watch network connections with filter (use f=filter)
	cargo run --bin watch-network-connections $(f)

run-stresser-anti-phishing: ## Run stress test for anti-phishing API (use rps=10 dur=10)
	cargo run --bin stress-anti-phishing-domain $(rps) $(dur)

clean: ## Clean build artifacts
	cargo clean

gitpush: ## git push m=any message
	clear;
	git add .; git commit -m "$(m)"; git push;

