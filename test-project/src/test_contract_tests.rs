use aleph_client::SignedConnection;
use anyhow::Result;
use assert2::assert;
use ink_primitives::AccountId;
use rand::RngCore as _;
use test_contract::{Enum1, Struct1, Struct2};

use crate::{helpers::connect_as_test_account, test_contract};

async fn connect_and_deploy() -> Result<(SignedConnection, test_contract::Instance)> {
    let conn = connect_as_test_account().await?;
    let mut salt = vec![0; 32];
    rand::thread_rng().fill_bytes(&mut salt);
    let contract = test_contract::Instance::default(&conn, salt).await?;

    Ok((conn, contract))
}

#[tokio::test]
async fn test_simple_integer_messages() -> Result<()> {
    let (conn, contract) = connect_and_deploy().await?;

    let old_val = contract.get_u32(&conn).await?.unwrap();
    let new_val = old_val + 42;
    contract.set_u32(&conn, new_val).await?;

    assert!(contract.get_u32(&conn).await?.unwrap() == new_val);

    Ok(())
}

#[tokio::test]
async fn test_struct_messages() -> Result<()> {
    let (conn, contract) = connect_and_deploy().await?;

    let val = Struct2(Struct1 { a: 1, b: 2 }, Enum1::B(3));
    contract.set_struct2(&conn, val.clone()).await?;
    assert!(contract.get_struct2(&conn).await?.unwrap() == val);

    Ok(())
}

#[tokio::test]
async fn test_array_messages() -> Result<()> {
    let (conn, contract) = connect_and_deploy().await?;

    contract.set_array(&conn, [1, 2, 3]).await?;
    contract.set_enum1(&conn, Enum1::A()).await?;
    assert!(contract.get_array(&conn).await?.unwrap() == [(1, Enum1::A()), (1, Enum1::A())]);

    Ok(())
}

#[tokio::test]
async fn test_sequence_messages() -> Result<()> {
    let (conn, contract) = connect_and_deploy().await?;

    contract.set_sequence(&conn, vec![5, 2, 3]).await?;
    contract.set_enum1(&conn, Enum1::A()).await?;
    assert!(contract.get_array(&conn).await?.unwrap() == [(5, Enum1::A()), (5, Enum1::A())]);

    Ok(())
}

#[tokio::test]
async fn test_compact_messages() -> Result<()> {
    let (conn, contract) = connect_and_deploy().await?;

    contract.set_compact(&conn, scale::Compact(42)).await?;
    assert!(contract.get_compact(&conn).await?.unwrap() == scale::Compact(42));

    Ok(())
}

#[tokio::test]
async fn test_messages_with_clashing_argument_names() -> Result<()> {
    let (conn, contract) = connect_and_deploy().await?;

    contract.set_forbidden_names(&conn, 1, 2, 3, 4, 5).await?;
    assert!(contract.get_u32(&conn).await?.unwrap() == 1 + 2 + 3 + 4 + 5);
    assert!(
        contract
            .get_forbidden_names(&conn, 1, 2, 3, 4, 5)
            .await?
            .unwrap()
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

    assert!(contract.get_u32(&conn).await?.unwrap() == 12345);

    Ok(())
}

#[tokio::test]
async fn test_events() -> Result<()> {
    use ink_wrapper_types::Connection;
    use test_contract::event::Event;

    let (conn, contract) = connect_and_deploy().await?;

    let data = Struct2(Struct1 { a: 1, b: 2 }, Enum1::B(3));
    contract.set_u32(&conn, 123).await?;
    contract.set_struct2(&conn, data.clone()).await?;
    let tx_info = contract.generate_events(&conn).await?;
    let events = conn.get_contract_events(tx_info).await?;
    let events = events.for_contract(contract);

    assert!(events[0] == Event::Event1 { a: 123, b: data });
    assert!(events[1] == Event::Event2 {});

    Ok(())
}
