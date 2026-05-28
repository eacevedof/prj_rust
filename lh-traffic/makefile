PS := powershell.exe -NoProfile -Command
TODAY := $(shell date +'%Y-%m-%d')
CURRENT_BRANCH := $(shell git rev-parse --abbrev-ref HEAD)

help: ## Show this help message
	@echo "usage: make [target]"
	@echo
	@echo "targets:"
	@egrep "^(.+)\:\ ##\ (.+)" ${MAKEFILE_LIST} | column -t -c 2 -s ":#"

gitpush: ## git push m=any message
	clear;
	git add .; git commit -m "lh-trafic:$(m)"; git push;

clean: ## Clean build artifacts
	$(PS) "cargo clean"

build: ## Build the project
	$(PS) "cargo build"

rebuild:
	make clean
	make build

build-release: ## Build release version
	$(PS) "cargo build --release"

check: ## Check code without building
	$(PS) "cargo check"

test: ## Run tests
	$(PS) "cargo test"

run-list: ## List all network connections with country and organization
	$(PS) "cargo run --bin list-network-connections"

run-list-filter: ## List network connections with filter (use f=filter)
	$(PS) "cargo run --bin list-network-connections $(f)"

run-list-estab: ## list established
	$(PS) "cargo run --bin list-network-connections estab,remote"

run-watch: ## Watch network connections (auto-refresh every 10s)
	$(PS) "cargo run --bin watch-network-connections"

run-watch-filter: ## Watch network connections with filter (use f=filter)
	$(PS) "cargo run --bin watch-network-connections $(f)"

