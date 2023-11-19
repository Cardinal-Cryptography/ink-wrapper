mod codegen;
mod extensions;

use std::{fs, io::Write};

use anyhow::Result;
use clap::Parser;
use codegen::generate;
use ink_metadata::InkProject;
use serde::{Deserialize, Serialize};

#[derive(Parser)]
#[command(version, about)]
struct Args {
    #[arg(
        short,
        long,
        help = "Path to the metadata file to generate a wrapper for."
    )]
    metadata: String,

    #[arg(
        long,
        help = "Path to the WASM of the contract relative to the output file. If provided, the WASM will be embedded \
            in the output file. Making it possible to upload the contract to a chain."
    )]
    wasm_path: Option<String>,
}

/// Struct for deserializing metadata.json that contains the fields not present in an InkProject.
#[derive(Debug, Serialize, Deserialize)]
struct Metadata {
    source: Source,
}

#[derive(Debug, Serialize, Deserialize)]
struct Source {
    hash: String,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let jsonized = fs::read_to_string(args.metadata)?;
    let metadata: Metadata = serde_json::from_str(&jsonized)?;
    let code_hash = metadata.source.hash;
    let metadata: InkProject = serde_json::from_str(&jsonized)?;

    let tokens: proc_macro2::TokenStream = generate(&metadata, code_hash, args.wasm_path);

    println!("{}", tokens.to_string());

    let stdout = std::io::stdout();

    stdout.lock().write_all(tokens.to_string().as_bytes())?;

    Ok(())
}
