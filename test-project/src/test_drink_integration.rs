use anyhow::Result;
use assert2::assert;
use drink::{
    contract_api::ContractApi,
    runtime::{MinimalRuntime, Runtime as _},
    Sandbox,
};
use ink_wrapper_types::{
    drink::DrinkConnection, Connection as _, SignedConnection, TxStatus, UploadConnection as _,
};

use crate::test_contract;

#[tokio::test]
async fn test_basic_interaction() -> Result<()> {
    let mut sandbox = Sandbox::<MinimalRuntime>::new().unwrap();
    let contract_bytes = std::fs::read("test_contract/target/ink/test_contract.wasm").unwrap();
    sandbox
        .upload_contract(contract_bytes, MinimalRuntime::default_actor(), None)
        .unwrap();

    let conn = DrinkConnection::new(sandbox);

    let contract = conn.instantiate(test_contract::Instance::default()).await?;
    conn.exec(contract.set_u32(123)).await?;
    let result = conn.read(contract.get_u32()).await??;

    assert!(result == 123);

    Ok(())
}

#[tokio::test]
async fn test_upload() -> Result<()> {
    let mut sandbox = Sandbox::<MinimalRuntime>::new().unwrap();
    let conn = DrinkConnection::new(sandbox);

    conn.upload(test_contract::upload()).await?;
    let contract = conn.instantiate(test_contract::Instance::default()).await?;
    conn.exec(contract.set_u32(123)).await?;
    let result = conn.read(contract.get_u32()).await??;

    assert!(result == 123);

    Ok(())
}

#[tokio::test]
async fn test_multiple_accounts() -> Result<()> {
    unimplemented!()
}

#[tokio::test]
async fn test_events() -> Result<()> {
    unimplemented!()
}
