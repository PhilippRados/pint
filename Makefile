help:  ## Display this help
	@awk 'BEGIN {FS = ":.*##"; printf "\nUsage:\n  make \033[36m<target>\033[0m\n"} /^[a-zA-Z_-]+:.*?##/ { printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2 } /^##@/ { printf "\n\033[1m%s\033[0m\n", substr($$0, 5) } ' $(MAKEFILE_LIST)

always:

target/debug/pint: always ## Compile in debug-mode
	cargo build

target/release/pint: always ## Compile in release-mode
	cargo build --release

bench: ## Bench tests
	cargo bench

flame: ## Generates flamegraph for passed input-file (root)
	sudo -E cargo flamegraph -- $(filter-out $@,$(MAKECMDGOALS))

integration_tests: ## Run integration_tests
	bash tests/integration_tests.sh

unit_test: ## Run unit_tests
	cargo t
