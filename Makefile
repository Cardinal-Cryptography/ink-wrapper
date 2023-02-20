SHELL=/bin/bash -o pipefail

test: test-project/src/test_contract.rs test-project/Cargo.toml test-project/src/main.rs ink-wrapper-types/src/lib.rs ink-wrapper-types/Cargo.toml
	cd test-project && cargo run

test-project/src/test_contract.rs: test_contract/target/ink/metadata.json Cargo.toml src/main.rs
	cargo run -- -m test_contract/target/ink/metadata.json | rustfmt --edition 2021 > test-project/src/test_contract.rs

test_contract/target/ink/metadata.json: test_contract/Cargo.toml test_contract/lib.rs
	cd test_contract && cargo contract build
