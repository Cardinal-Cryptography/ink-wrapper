mod test_contract;

use anyhow::Result;
use ink_primitives::AccountId;
use sp_core::crypto::Ss58Codec;
use test_contract::{Enum1, Struct1, Struct2};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let conn = aleph_client::Connection::new("ws://localhost:9944").await;
    let alice = aleph_client::keypair_from_string("//Alice");
    let conn = aleph_client::SignedConnection::from_connection(conn.clone(), alice);
    let account_id: sp_core::sr25519::Public =
        Ss58Codec::from_string("5DcA89G6LjoGEqD3VHDoHXpDUoVtSMSJpXzMHysMommVJvYL")?;
    let account_id: [u8; 32] = account_id.into();
    let account_id = AccountId::from(account_id);

    let contract = test_contract::Instance::from(account_id);

    println!("Connected");
    println!("{:?}", contract.get_u32(&conn).await?);
    println!("{:?}", contract.set_u32(&conn, 42).await?);
    println!("{:?}", contract.get_u32(&conn).await?);
    println!("{:?}", contract.get_struct2(&conn).await?);
    println!(
        "{:?}",
        contract
            .set_struct2(
                &conn,
                Struct2 {
                    a: Struct1 { a: 1, b: 2 },
                    b: Enum1::B(3)
                }
            )
            .await?
    );
    println!("{:?}", contract.get_struct2(&conn).await?);

    Ok(())
}
