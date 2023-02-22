#! /bin/bash

set -euo pipefail

pushd test_contract
cargo contract build --release
cargo contract instantiate --suri //Alice --url ws://localhost:9944 --constructor default --skip-confirm || true
popd

pushd ink-wrapper
cargo run -- -m ../test_contract/target/ink/metadata.json \
  | rustfmt --edition 2021 > ../test-project/src/test_contract.rs
popd

pushd test-project
cargo run
popd

