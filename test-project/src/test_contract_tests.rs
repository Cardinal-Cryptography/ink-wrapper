use aleph_client::SignedConnection;
use anyhow::Result;
use assert2::assert;
use ink_primitives::AccountId;
use ink_wrapper_types::{Connection as _, SignedConnection as _, TxStatus, UploadConnection as _};
use rand::RngCore as _;
use test_contract::{Enum1, Struct1, Struct2};

use crate::{helpers::connect_as_test_account, test_contract};

fn random_salt() -> Vec<u8> {
    let mut salt = vec![0; 32];
    rand::thread_rng().fill_bytes(&mut salt);
    salt
}

async fn connect_and_deploy() -> Result<(SignedConnection, test_contract::Instance)> {
    let conn = connect_as_test_account().await?;
    let contract = conn
        .instantiate(test_contract::Instance::default().with_salt(random_salt()))
        .await?;

    Ok((conn, contract))
}

#[tokio::test]
async fn test_simple_integer_messages() -> Result<()> {
    let (conn, contract) = connect_and_deploy().await?;

    let old_val = conn.read(contract.get_u32()).await??;
    let new_val = old_val + 42;
    conn.exec(contract.set_u32(new_val)).await?;

    assert!(conn.read(contract.get_u32()).await?? == new_val);

    Ok(())
}

#[tokio::test]
async fn test_struct_messages() -> Result<()> {
    let (conn, contract) = connect_and_deploy().await?;

    let val = Struct2(
        Struct1 {
            a: 1,
            b: 2,
            c: [2, 3, 4, 5],
        },
        Enum1::B(3),
    );
    conn.exec(contract.set_struct2(val.clone())).await?;
    assert!(conn.read(contract.get_struct2()).await?? == val);

    Ok(())
}

#[tokio::test]
async fn test_array_messages() -> Result<()> {
    let (conn, contract) = connect_and_deploy().await?;

    conn.exec(contract.set_array([1, 2, 3])).await?;
    conn.exec(contract.set_enum1(Enum1::A())).await?;
    assert!(conn.read(contract.get_array()).await?? == [(1, Enum1::A()), (1, Enum1::A())]);

    Ok(())
}

#[tokio::test]
async fn test_sequence_messages() -> Result<()> {
    let (conn, contract) = connect_and_deploy().await?;

    conn.exec(contract.set_sequence(vec![5, 2, 3])).await?;
    conn.exec(contract.set_enum1(Enum1::A())).await?;
    assert!(conn.read(contract.get_array()).await?? == [(5, Enum1::A()), (5, Enum1::A())]);

    Ok(())
}

#[tokio::test]
async fn test_compact_messages() -> Result<()> {
    let (conn, contract) = connect_and_deploy().await?;

    conn.exec(contract.set_compact(scale::Compact(42))).await?;
    assert!(conn.read(contract.get_compact()).await?? == scale::Compact(42));

    Ok(())
}

#[tokio::test]
async fn test_messages_with_clashing_argument_names() -> Result<()> {
    let (conn, contract) = connect_and_deploy().await?;

    conn.exec(contract.set_forbidden_names(1, 2, 3, 4, 5))
        .await?;
    assert!(conn.read(contract.get_u32()).await?? == 1 + 2 + 3 + 4 + 5);
    assert!(
        conn.read(contract.get_forbidden_names(1, 2, 3, 4, 5))
            .await??
            == 1 + 2 + 3 + 4 + 5
    );

    Ok(())
}

#[tokio::test]
async fn test_conversion_to_account_id() -> Result<()> {
    let (conn, contract) = connect_and_deploy().await?;
    conn.exec(contract.set_u32(12345)).await?;

    let account_id: AccountId = contract.into();
    let contract: test_contract::Instance = account_id.into();

    assert!(conn.read(contract.get_u32()).await?? == 12345);

    Ok(())
}

#[tokio::test]
async fn test_events() -> Result<()> {
    use test_contract::event::Event;

    let (conn, contract) = connect_and_deploy().await?;

    let struct2 = Struct2(
        Struct1 {
            a: 1,
            b: 2,
            c: [0; 4],
        },
        Enum1::B(3),
    );
    conn.exec(contract.set_u32(123)).await?;
    conn.exec(contract.set_struct2(struct2.clone())).await?;
    let struct1 = conn.read(contract.get_struct1()).await??;
    let tx_info = conn.exec(contract.generate_events()).await?;
    let events = conn.get_contract_events(tx_info).await?;
    let events = events.for_contract(contract);

    assert!(
        events[0]
            == Ok(Event::Event1 {
                a: 123,
                b: struct2.clone(),
                c: struct1.c,
                d: (struct1, struct2)
            })
    );
    assert!(events[1] == Ok(Event::Event2 {}));

    Ok(())
}

#[tokio::test]
async fn test_ink_lang_error() -> Result<()> {
    let (conn, contract) = connect_and_deploy().await?;

    assert!(
        conn.read(contract.generate_ink_lang_error())
            .await??
            .to_string()
            == "InkLangError(CouldNotReadInput)"
    );

    Ok(())
}

#[tokio::test]
async fn test_upload() -> Result<()> {
    let conn = connect_as_test_account().await?;

    assert!(conn.upload(test_contract::upload()).await.is_ok());

    Ok(())
}

#[tokio::test]
async fn test_receiving_value() -> Result<()> {
    let (conn, contract) = connect_and_deploy().await?;

    let tx_info = conn.exec(contract.receive_value().with_value(123)).await?;
    let events = conn.get_contract_events(tx_info).await?;
    let events = events.for_contract(contract);

    assert!(events[0] == Ok(test_contract::event::Event::Received { value: 123 }));

    Ok(())
}

#[tokio::test]
async fn test_receiving_value_in_constructor() -> Result<()> {
    let conn = connect_as_test_account().await?;

    let (contract, tx_info) = conn
        .instantiate_tx(
            test_contract::Instance::payable_constructor()
                .with_value(123)
                .with_salt(random_salt()),
        )
        .await?;
    let events = conn.get_contract_events(tx_info).await?;
    let events = events.for_contract(contract);

    assert!(events[0] == Ok(test_contract::event::Event::Received { value: 123 }));

    Ok(())
}

#[tokio::test]
async fn test_constructor_waiting_for_submitted() -> Result<()> {
    let conn = connect_as_test_account().await?;

    let (_contract, tx_info) = conn
        .instantiate_tx(
            test_contract::Instance::default()
                .with_salt(random_salt())
                .with_tx_status(TxStatus::Submitted),
        )
        .await?;

    assert!(tx_info.block_hash == [0; 32].into());

    Ok(())
}
