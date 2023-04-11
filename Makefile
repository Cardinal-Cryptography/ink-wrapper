.PHONY: test_contract test_contract.rs test-project test all check-ink-wrapper \
	check-test-project all-dockerized tooling build-builder build-node run-node

build-builder:
	docker build --tag ink-builder --file ci/Dockerfile.builder ci

build-node:
	docker build --tag aleph-onenode-chain --file ci/Dockerfile.aleph-node ci

run-node: build-node
	docker run --detach --rm --network host \
		--name ink-wrapper-node \
		aleph-onenode-chain

test_contract:
	cd test_contract && cargo contract build --release
	cd test_contract && cargo contract upload --suri //Alice --url ws://localhost:9944 || true

test_contract.rs: test_contract
	cd ink-wrapper && cargo run -- -m ../test_contract/target/ink/test_contract.json \
		| rustfmt --edition 2021 > ../test-project/src/test_contract.rs

test: test_contract.rs
	cd test-project && cargo test

check-ink-wrapper:
	cd ink-wrapper && cargo fmt --all --check
	cd ink-wrapper && cargo clippy --all-features -- --no-deps -D warnings

check-test-project:
	cd test-project && cargo fmt --all --check
	cd test-project && cargo clippy --all-features -- --no-deps -D warnings

all-dockerized: run-node build-builder
	docker run --rm --network host \
		--user "$(shell id -u):$(shell id -g)" \
		--volume "$(shell pwd)":/code \
		--workdir /code \
		--name ink-wrapper-builder \
		ink-builder \
		make all

tooling:
	rustup component add rustfmt clippy

all: tooling check-ink-wrapper check-test-project test

kill:
	docker kill ink-wrapper-builder ink-wrapper-node
