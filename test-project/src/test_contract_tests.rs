use aleph_client::SignedConnection;
use anyhow::Result;
use assert2::assert;
use ink_primitives::AccountId;
use ink_wrapper_types::{Connection as _, SignedConnection as _};
use rand::RngCore as _;
use test_contract::{Enum1, Struct1, Struct2};

use crate::{helpers::connect_as_test_account, test_contract};

async fn connect_and_deploy() -> Result<(SignedConnection, test_contract::Instance)> {
    let conn = connect_as_test_account().await?;
    let mut salt = vec![0; 32];
    rand::thread_rng().fill_bytes(&mut salt);
    let contract = conn
        .instantiate(test_contract::Instance::default().with_salt(salt))
        .await?;

    Ok((conn, contract))
}

#[tokio::test]
async fn test_simple_integer_messages() -> Result<()> {
    let (conn, contract) = connect_and_deploy().await?;

    let old_val = conn.read(contract.get_u32()).await??;
    let new_val = old_val + 42;
    contract.set_u32(&conn, new_val).await?;

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
    contract.set_struct2(&conn, val.clone()).await?;
    assert!(conn.read(contract.get_struct2()).await?? == val);

    Ok(())
}

#[tokio::test]
async fn test_array_messages() -> Result<()> {
    let (conn, contract) = connect_and_deploy().await?;

    contract.set_array(&conn, [1, 2, 3]).await?;
    contract.set_enum1(&conn, Enum1::A()).await?;
    assert!(conn.read(contract.get_array()).await?? == [(1, Enum1::A()), (1, Enum1::A())]);

    Ok(())
}

#[tokio::test]
async fn test_sequence_messages() -> Result<()> {
    let (conn, contract) = connect_and_deploy().await?;

    contract.set_sequence(&conn, vec![5, 2, 3]).await?;
    contract.set_enum1(&conn, Enum1::A()).await?;
    assert!(conn.read(contract.get_array()).await?? == [(5, Enum1::A()), (5, Enum1::A())]);

    Ok(())
}

#[tokio::test]
async fn test_compact_messages() -> Result<()> {
    let (conn, contract) = connect_and_deploy().await?;

    contract.set_compact(&conn, scale::Compact(42)).await?;
    assert!(conn.read(contract.get_compact()).await?? == scale::Compact(42));

    Ok(())
}

#[tokio::test]
async fn test_messages_with_clashing_argument_names() -> Result<()> {
    let (conn, contract) = connect_and_deploy().await?;

    contract.set_forbidden_names(&conn, 1, 2, 3, 4, 5).await?;
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
    contract.set_u32(&conn, 12345).await?;

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
    contract.set_u32(&conn, 123).await?;
    contract.set_struct2(&conn, struct2.clone()).await?;
    let struct1 = conn.read(contract.get_struct1()).await??;
    let tx_info = contract.generate_events(&conn).await?;
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

    assert!(test_contract::upload(&conn).await.is_ok());

    Ok(())
}
