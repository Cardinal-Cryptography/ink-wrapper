use aleph_client::SignedConnection;
use anyhow::Result;
use assert2::assert;
use ink_wrapper_types::{util::ToAccountId, Connection as _, SignedConnection as _};
use rand::RngCore as _;

use crate::{connect_as_test_account, psp22_contract, random_account};

async fn connect_and_deploy() -> Result<(SignedConnection, psp22_contract::Instance)> {
    let conn = connect_as_test_account().await?;
    let mut salt = vec![0; 32];
    rand::thread_rng().fill_bytes(&mut salt);
    let contract = conn
        .instantiate(psp22_contract::Instance::new(1000).with_salt(salt))
        .await?;

    Ok((conn, contract))
}

#[tokio::test]
async fn test_transfers() -> Result<()> {
    use psp22_contract::PSP22 as _;

    let (conn, contract) = connect_and_deploy().await?;
    let other_account = random_account();
    let other_account_id = other_account.account_id().to_account_id();

    conn.exec(contract.transfer(other_account_id.into(), 100, vec![]))
        .await?;

    assert!(
        conn.read(contract.balance_of(other_account_id.into()))
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
    let supply_before = conn.read(contract.total_supply()).await?.unwrap();
    let account_id = conn.account_id().to_account_id();

    conn.exec(contract.burn(account_id.into(), 100)).await?;

    assert!(conn.read(contract.total_supply()).await?.unwrap() == supply_before - 100);

    Ok(())
}
