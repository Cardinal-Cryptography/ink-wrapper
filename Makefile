.PHONY: help
help: # Show help for each of the Makefile recipes.
	@grep -E '^[a-zA-Z0-9 -]+:.*#'  Makefile | sort | while read -r l; do printf "\033[1;32m$$(echo $$l | cut -f 1 -d':')\033[00m:$$(echo $$l | cut -f 2- -d'#')\n"; done

.PHONY: build-builder
build-builder:
	docker build --tag ink-builder --file ci/Dockerfile.builder \
		--build-arg UID=$(shell id -u) --build-arg GID=$(shell id -g) \
		ci

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
	cd tests/test_contract && cargo contract build --release

.PHONY: upload-test-contract
upload-test-contract: test_contract
	cd tests/test_contract && cargo contract upload --suri //Alice --url ws://localhost:9944 -x || true

.PHONY: psp22_contract
psp22_contract:
	cd tests/psp22_contract && cargo contract build --release

.PHONY: upload-psp22-contract
upload-psp22-contract: psp22_contract
	cd tests/psp22_contract && cargo contract upload --suri //Alice --url ws://localhost:9944 -x  || true

.PHONY: test_contract.rs
test_contract.rs: test_contract
	cd ink-wrapper && \
		cargo run -- -m ../tests/test_contract/target/ink/test_contract.json \
			--wasm-path ../../test_contract/target/ink/test_contract.wasm \
		| rustfmt --edition 2021 > ../tests/drink/src/test_contract.rs && \
		cp ../tests/drink/src/test_contract.rs ../tests/aleph_client/src/test_contract.rs

.PHONY: psp22_contract.rs
psp22_contract.rs: psp22_contract
	cd ink-wrapper && cargo run -- -m ../tests/psp22_contract/target/ink/psp22_contract.json \
			--wasm-path ../../psp22_contract/target/ink/psp22_contract.wasm \
		| rustfmt --edition 2021 > ../tests/drink/src/psp22_contract.rs && \
		cp ../tests/drink/src/psp22_contract.rs ../tests/aleph_client/src/psp22_contract.rs

.PHONY: generate-wrappers
generate-wrappers: test_contract.rs psp22_contract.rs # Generate wrappers for test contracts.

.PHONY: upload-contracts
upload-contracts: upload-test-contract upload-psp22-contract # Upload test contracts to the chain.

.PHONY: test
test: drink-tests aleph-client-tests # Run tests natively (needs tooling installed - see ci/Dockerfile.builder).
	cd tests/aleph_client && cargo test || echo "Failed to run tests in aleph_client"
	cd tests/drink && cargo test || echo "Failed to run tests in drink"

.PHONY: drink-tests
drink-tests: generate-wrappers ## Runs tests for drink
	cd tests/drink && cargo test || echo "Failed to run tests in drink"

.PHONY: alepg-client-tests
aleph-client-tests: generate-wrappers upload-contracts ## Runs tests for aleph_client
	cd tests/aleph_client && cargo test || echo "Failed to run tests in aleph_client"

.PHONY: check-ink-wrapper
check-ink-wrapper:
	cd ink-wrapper && cargo fmt --all --check
	cd ink-wrapper && cargo clippy --all-features -- --no-deps -D warnings

.PHONY: check-ink-wrapper-types
check-ink-wrapper-types:
	cd ink-wrapper-types && cargo fmt --all --check
	cd ink-wrapper-types && cargo clippy --features aleph_client  -- --no-deps -D warnings
	cd ink-wrapper-types && cargo clippy --features drink  -- --no-deps -D warnings

.PHONY: check-tests
check-tests: generate-wrappers
	cd tests/aleph_client && cargo fmt --all --check
	cd tests/drink && cargo fmt --all --check
	cd ink-wrapper-types && cargo clippy --features aleph_client  -- --no-deps -D warnings
	cd ink-wrapper-types && cargo clippy --features drink  -- --no-deps -D warnings

.PHONY: all-dockerized
all-dockerized: kill run-node build-builder # Run all checks in a dockerized environment.
	docker run --rm --network host \
		--user "$(shell id -u):$(shell id -g)" \
		--volume "$(shell pwd)":/code \
		--workdir /code \
		--name ink-wrapper-builder \
		ink-builder \
		make all

.PHONY: all
all: check-ink-wrapper check-ink-wrapper-types check-tests test # Run all checks natively (needs tooling installed - see ci/Dockerfile.builder).

.PHONY: kill
kill: # Remove dangling containers after a dockerized test run.
	docker kill ink-wrapper-builder ink-wrapper-node || true

.PHONY: clean
clean: kill # Remove dangling containers and built images.
	docker rmi -f ink-builder aleph-onenode-chain
