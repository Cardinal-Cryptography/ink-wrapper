use aleph_client::SignedConnection;
use anyhow::Result;
use assert2::assert;
use rand::RngCore as _;

use crate::{
    helpers::{connect_as_test_account, random_account},
    psp22_contract,
};

async fn connect_and_deploy() -> Result<(SignedConnection, psp22_contract::Instance)> {
    let conn = connect_as_test_account().await?;
    let mut salt = vec![0; 32];
    rand::thread_rng().fill_bytes(&mut salt);
    let contract = psp22_contract::Instance::new(&conn, salt, 1000).await?;

    Ok((conn, contract))
}

#[tokio::test]
async fn test_transfers() -> Result<()> {
    use psp22_contract::PSP22 as _;

    let (conn, contract) = connect_and_deploy().await?;
    let other_account = random_account();
    let other_account_id: [u8; 32] = *other_account.account_id().as_ref();

    contract
        .transfer(&conn, other_account_id.into(), 100, vec![])
        .await?;

    assert!(
        contract
            .balance_of(&conn, other_account_id.into())
            .await?
            .unwrap()
            == 100
    );

    Ok(())
}

#[tokio::test]
async fn test_burn() -> Result<()> {
    use aleph_client::SignedConnectionApi as _;
    use psp22_contract::{PSP22Burnable as _, PSP22 as _};

    let (conn, contract) = connect_and_deploy().await?;
    let supply_before = contract.total_supply(&conn).await?.unwrap();
    let account_id: [u8; 32] = *conn.account_id().as_ref();

    contract.burn(&conn, account_id.into(), 100).await?;

    assert!(contract.total_supply(&conn).await?.unwrap() == supply_before - 100);

    Ok(())
}
