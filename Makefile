.PHONY: help
help: # Show help for each of the Makefile recipes.
	@grep -E '^[a-zA-Z0-9 -]+:.*#'  Makefile | sort | while read -r l; do printf "\033[1;32m$$(echo $$l | cut -f 1 -d':')\033[00m:$$(echo $$l | cut -f 2- -d'#')\n"; done

.PHONY: build-builder
build-builder:
	docker build --tag ink-builder --file ci/Dockerfile.builder ci

.PHONY: build-node
build-node:
	docker build --tag aleph-onenode-chain --file ci/Dockerfile.aleph-node ci

.PHONY: run-node
run-node: build-node # Run a one-node chain in docker.
	docker run --detach --rm --network host \
		--name ink-wrapper-node \
		aleph-onenode-chain

.PHONY: test_contract
test_contract:
	cd test_contract && cargo contract build --release
	cd test_contract && cargo contract upload --suri //Alice --url ws://localhost:9944 || true

.PHONY: psp22_contract
psp22_contract:
	cd psp22_contract && cargo contract build --release
	cd psp22_contract && cargo contract upload --suri //Alice --url ws://localhost:9944 || true

.PHONY: test_contract.rs
test_contract.rs: test_contract
	cd ink-wrapper && cargo run -- -m ../test_contract/target/ink/test_contract.json \
		| rustfmt --edition 2021 > ../test-project/src/test_contract.rs

.PHONY: psp22_contract.rs
psp22_contract.rs: psp22_contract
	cd ink-wrapper && cargo run -- -m ../psp22_contract/target/ink/psp22_contract.json \
		| rustfmt --edition 2021 > ../test-project/src/psp22_contract.rs

.PHONY: generate-wrappers
generate-wrappers: test_contract.rs psp22_contract.rs # Generate wrappers for test contracts.

.PHONY: test
test: generate-wrappers # Run tests natively (needs tooling installed - see ci/Dockerfile.builder).
	cd test-project && cargo test

.PHONY: check-ink-wrapper
check-ink-wrapper:
	cd ink-wrapper && cargo fmt --all --check
	cd ink-wrapper && cargo clippy --all-features -- --no-deps -D warnings

.PHONY: check-test-project
check-test-project: generate-wrappers
	cd test-project && cargo fmt --all --check
	cd test-project && cargo clippy --all-features -- --no-deps -D warnings

.PHONY: all-dockerized
all-dockerized: run-node build-builder # Run all checks in a dockerized environment.
	docker run --rm --network host \
		--user "$(shell id -u):$(shell id -g)" \
		--volume "$(shell pwd)":/code \
		--workdir /code \
		--name ink-wrapper-builder \
		ink-builder \
		make all

.PHONY: tooling
tooling:
	rustup component add rustfmt clippy

.PHONY: all
all: tooling check-ink-wrapper check-test-project test # Run all checks natively (needs tooling installed - see ci/Dockerfile.builder).

.PHONY: kill
kill: # Remove dangling containers after a dockerized test run.
	docker kill ink-wrapper-builder ink-wrapper-node || true

.PHONY: clean
clean: kill # Remove dangling containers and built images.
	docker rmi -f ink-builder aleph-onenode-chain
