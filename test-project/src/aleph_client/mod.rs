#[cfg(test)]
mod psp22_tests;
#[cfg(test)]
mod test_contract_tests;

use std::sync::Mutex;

use aleph_client::{
    keypair_from_string, pallets::balances::BalanceUserApi, Connection, KeyPair, SignedConnection,
    TxStatus,
};
use anyhow::Result;
use once_cell::sync::Lazy;
use rand::RngCore as _;

static AUTHORITY_MUTEX: Lazy<Mutex<KeyPair>> =
    Lazy::new(|| Mutex::new(keypair_from_string("//Alice")));

/// Generates a random test account.
pub fn random_account() -> KeyPair {
    keypair_from_string(&format!("//TestAccount/{}", rand::thread_rng().next_u64()))
}

/// Connects to the local node and transfers some funds to it. Returns a connection signed by that account.
pub async fn connect_as_test_account() -> Result<SignedConnection> {
    let authority = AUTHORITY_MUTEX.lock().unwrap();
    let conn = Connection::new("ws://localhost:9944").await;
    let test_account = random_account();

    SignedConnection::from_connection(conn.clone(), authority.clone())
        .transfer(
            test_account.account_id().clone(),
            alephs(100),
            TxStatus::InBlock,
        )
        .await?;

    Ok(SignedConnection::from_connection(conn, test_account))
}

fn alephs(n: u128) -> aleph_client::Balance {
    n * 1_000_000_000_000
}
