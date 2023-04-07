# ink-wrapper

`ink-wrapper` is a tool that generates type-safe code for calling a substrate smart contract based on the metadata
(`.json`) file for that contract.

## Installation

Install the tool from [crates.io](https://crates.io):

```bash
cargo install ink-wrapper
```

## Usage

Given some metadata file like `metadata.json` run the tool and save the output to a file in your project:

```bash
ink-wrapper -m metadata.json > src/my_contract.rs
```

We only take minimal steps to format the output of the tool, so we recommend that you run it through a formatter when
(re)generating:

```bash
ink-wrapper -m metadata.json | rustfmt --edition 2021 > src/my_contract.rs
```

The output should compile with no warnings, please create an issue if any warnings pop up in your project in the
generated code.

Make sure the file you generated is included in your module structure:

```rust
mod test_contract;
```

You will need the following dependencies for the wrapper to work:

```toml
ink-wrapper-types = "0.1.0"
scale = { package = "parity-scale-codec", version = "3", default-features = false, features = ["derive"] }
ink_primitives = "4.0.1"

# This one is optional, but you most likely need it as well if you're using the default `aleph_client` implementation
# for actually making calls. Otherwise, you will need to implement `ink_wrapper_types::Connection` and
# `ink_wrapper_types::SignedConnection` yourself.

aleph_client = { git = "https://github.com/Cardinal-Cryptography/aleph-node.git", rev = "r-10.0" }
```

With that, you're ready to use the wrappers in your code. The generated module will have an `Instance` struct that
represents an instance of your contract. You can either talk to an existing instance by converting an `account_id` to
an `Instance`:

```rust
let account_id: ink_primitives::AccountId = ...;
let instance: my_contract::Instance = account_id.into();
```

Or (assuming the contract code has already been uploaded) create an instance using one of the generated constructors:

```rust
let instance = my_contract::Instance::some_constructor(&conn, arg1, arg2).await?;
```

And then call methods on your contract:

```rust
let result = instance.some_getter(&conn, arg1, arg2).await?;
let tx_info = instance.some_mutator(&conn, arg1, arg2).await?;
```

In the examples above, `conn` is anything that implements `ink_wrapper_types::Connection` (and
`ink_wrapper_types::SignedConnection` if you want to use constructors or mutators). Default implementations are provided
for the connection in `aleph_client`.

## Development

Use the commands provided in the `Makefile` to replicate the build process run on CI. The most hassle-free is to just
run everything in docker:

```bash
make all-dockerized
```

If you have the tooling installed on your host and start a node yourself, you can also run the build on your host:

```bash
make all
```

In case there are any runaway containers from `all-dockerized` you can kill them:

```bash
make kill
```
