mod test_contract;

use anyhow::Result;
use rand::RngCore as _;
use test_contract::{Enum1, Struct1, Struct2};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let conn = aleph_client::Connection::new("ws://localhost:9944").await;
    let alice = aleph_client::keypair_from_string("//Alice");
    let conn = aleph_client::SignedConnection::from_connection(conn.clone(), alice);

    let mut salt = vec![0; 32];
    rand::thread_rng().fill_bytes(&mut salt);
    let contract = test_contract::Instance::default(&conn, salt).await?;

    println!("Connected");
    println!("{:?}", contract.get_u32(&conn).await?);
    println!("{:?}", contract.set_u32(&conn, 42).await?);
    println!("{:?}", contract.get_u32(&conn).await?);
    println!("{:?}", contract.get_struct2(&conn).await?);
    println!(
        "{:?}",
        contract
            .set_struct2(&conn, Struct2(Struct1 { a: 1, b: 2 }, Enum1::B(3)))
            .await?
    );
    println!("{:?}", contract.get_struct2(&conn).await?);

    Ok(())
}
