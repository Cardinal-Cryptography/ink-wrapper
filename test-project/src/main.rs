mod test_contract;

use aleph_client::{
    pallets::contract::{ContractCallArgs, ContractRpc, ContractsUserApi},
    SignedConnectionApi, TxInfo, TxStatus,
};
use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use ink_primitives::AccountId;
use sp_core::crypto::Ss58Codec;
use test_contract::{Enum1, Struct1, Struct2};

struct Conn(aleph_client::Connection);

struct Signed(aleph_client::Connection, aleph_client::SignedConnection);

impl Conn {
    pub fn sign(&self, keypair: aleph_client::KeyPair) -> Signed {
        Signed(
            self.0.clone(),
            aleph_client::SignedConnection::from_connection(self.0.clone(), keypair),
        )
    }
}

#[async_trait]
impl ink_wrapper_types::SignedConnection<TxInfo, anyhow::Error> for Signed {
    async fn exec(&self, account_id: ink_primitives::AccountId, data: Vec<u8>) -> Result<TxInfo> {
        let result = dry_run(
            &self.0,
            account_id,
            self.1.account_id().clone(),
            data.clone(),
        )
        .await?;
        let account_id: [u8; 32] = *account_id.as_ref();

        self.1
            .call(
                account_id.into(),
                0,
                aleph_client::sp_weights::weight_v2::Weight {
                    ref_time: result.gas_required.ref_time(),
                    proof_size: result.gas_required.proof_size(),
                },
                None,
                data,
                TxStatus::Finalized,
            )
            .await
    }
}

#[async_trait]
impl ink_wrapper_types::Connection<anyhow::Error> for Conn {
    async fn read<T: scale::Decode>(
        &self,
        account_id: ink_primitives::AccountId,
        data: Vec<u8>,
    ) -> Result<T> {
        let result = dry_run(&self.0, account_id, account_id, data)
            .await?
            .result
            .map_err(|e| anyhow!("Contract exec failed {:?}", e))?;

        Ok(
            scale::Decode::decode(&mut result.data.as_slice())
                .context("Failed to decode result")?,
        )
    }
}

async fn dry_run<A1: AsRef<[u8; 32]>, A2: AsRef<[u8; 32]>>(
    conn: &aleph_client::Connection,
    contract: A1,
    call_as: A2,
    data: Vec<u8>,
) -> Result<pallet_contracts_primitives::ContractExecResult<aleph_client::Balance>> {
    let args = ContractCallArgs {
        origin: call_as.as_ref().clone().into(),
        dest: contract.as_ref().clone().into(),
        value: 0,
        gas_limit: None,
        input_data: data,
        storage_deposit_limit: None,
    };

    conn.call_and_get(args)
        .await
        .context("RPC request error - there may be more info in node logs.")
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let conn = Conn(aleph_client::Connection::new("ws://localhost:9944").await);
    let alice = aleph_client::keypair_from_string("//Alice");
    let signed = conn.sign(alice);
    let account_id: sp_core::sr25519::Public =
        Ss58Codec::from_string("5DcA89G6LjoGEqD3VHDoHXpDUoVtSMSJpXzMHysMommVJvYL")?;
    let account_id: [u8; 32] = account_id.into();
    let account_id = AccountId::from(account_id);

    let contract = test_contract::Instance::from(account_id);

    println!("Connected");
    println!("{:?}", contract.get_u32(&conn).await?);
    println!("{:?}", contract.set_u32(&signed, 42).await?);
    println!("{:?}", contract.get_u32(&conn).await?);
    println!("{:?}", contract.get_struct2(&conn).await?);
    println!(
        "{:?}",
        contract
            .set_struct2(
                &signed,
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
