.PHONY: test_contract test_contract.rs test-project test all check-ink-wrapper \
	check-test-project all-dockerized tooling build-builder build-node run-node \
	help

help: # Show help for each of the Makefile recipes.
	@grep -E '^[a-zA-Z0-9 -]+:.*#'  Makefile | sort | while read -r l; do printf "\033[1;32m$$(echo $$l | cut -f 1 -d':')\033[00m:$$(echo $$l | cut -f 2- -d'#')\n"; done

build-builder:
	docker build --tag ink-builder --file ci/Dockerfile.builder ci

build-node:
	docker build --tag aleph-onenode-chain --file ci/Dockerfile.aleph-node ci

run-node: build-node # Run a one-node chain in docker.
	docker run --detach --rm --network host \
		--name ink-wrapper-node \
		aleph-onenode-chain

test_contract:
	cd test_contract && cargo contract build --release
	cd test_contract && cargo contract upload --suri //Alice --url ws://localhost:9944 || true

test_contract.rs: test_contract
	cd ink-wrapper && cargo run -- -m ../test_contract/target/ink/test_contract.json \
		| rustfmt --edition 2021 > ../test-project/src/test_contract.rs

test: test_contract.rs # Run tests natively (needs tooling installed - see ci/Dockerfile.builder).
	cd test-project && cargo test

check-ink-wrapper:
	cd ink-wrapper && cargo fmt --all --check
	cd ink-wrapper && cargo clippy --all-features -- --no-deps -D warnings

check-test-project:
	cd test-project && cargo fmt --all --check
	cd test-project && cargo clippy --all-features -- --no-deps -D warnings

all-dockerized: run-node build-builder # Run all checks in a dockerized environment.
	docker run --rm --network host \
		--user "$(shell id -u):$(shell id -g)" \
		--volume "$(shell pwd)":/code \
		--workdir /code \
		--name ink-wrapper-builder \
		ink-builder \
		make all

tooling:
	rustup component add rustfmt clippy

all: tooling check-ink-wrapper check-test-project test # Run all checks natively (needs tooling installed - see ci/Dockerfile.builder).

kill: # Remove dangling containers after a dockerized test run.
	docker kill ink-wrapper-builder ink-wrapper-node
