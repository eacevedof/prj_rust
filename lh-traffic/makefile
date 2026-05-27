PS := powershell.exe -NoProfile -Command
TODAY := $(shell date +'%Y-%m-%d')
CURRENT_BRANCH := $(shell git rev-parse --abbrev-ref HEAD)

help: ## Show this help message
	@echo "usage: make [target]"
	@echo
	@echo "targets:"
	@egrep "^(.+)\:\ ##\ (.+)" ${MAKEFILE_LIST} | column -t -c 2 -s ":#"

build: ## Build the project
	$(PS) "cargo build"

build-release: ## Build release version
	$(PS) "cargo build --release"

check: ## Check code without building
	$(PS) "cargo check"

test: ## Run tests
	$(PS) "cargo test"

run-list: ## List all network connections
	$(PS) "cargo run --bin list-network-connections"

run-list-filter: ## List network connections with filter (use f=filter)
	$(PS) "cargo run --bin list-network-connections $(f)"

run-list-full: ## List connections with country and organization (hybrid whois + api)
	$(PS) "cargo run --bin list-enhanced-network-connections"

run-list-full-filter: ## List enhanced connections with filter (use f=filter)
	$(PS) "cargo run --bin list-enhanced-network-connections $(f)"

run-watch: ## Watch network connections (auto-refresh every 10s)
	$(PS) "cargo run --bin watch-network-connections"

run-watch-filter: ## Watch network connections with filter (use f=filter)
	$(PS) "cargo run --bin watch-network-connections $(f)"

run-enhanced: ## Show enhanced network info with geolocation and process details
	$(PS) "cargo run --bin enhanced-network-info"

run-whois: ## Show network info using whois (slower but no rate limit)
	$(PS) "cargo run --bin whois-network-info"

clean: ## Clean build artifacts
	$(PS) "cargo clean"

gitpush: ## git push m=any message
	clear;
	git add .; git commit -m "$(m)"; git push;

