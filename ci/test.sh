#! /bin/bash

set -euo pipefail

rustup component add rustfmt clippy

pushd test_contract
cargo contract build --release
cargo contract upload --suri //Alice --url ws://localhost:9944 || true
popd

pushd ink-wrapper
cargo run -- -m ../test_contract/target/ink/test_contract.json \
  | rustfmt --edition 2021 > ../test-project/src/test_contract.rs
popd

pushd test-project
cargo fmt --all --check
cargo clippy --all-features -- --no-deps -D warnings
cargo test
cargo run
popd
