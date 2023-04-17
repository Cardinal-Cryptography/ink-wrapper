#[cfg(feature = "aleph_client")]
mod aleph_client;

use async_trait::async_trait;
use ink_primitives::AccountId;

#[async_trait]
pub trait SignedConnection<TxInfo, E>: Sync {
    async fn instantiate(
        &self,
        code_hash: [u8; 32],
        salt: Vec<u8>,
        data: Vec<u8>,
    ) -> Result<AccountId, E>;

    async fn exec(&self, account_id: AccountId, data: Vec<u8>) -> Result<TxInfo, E>;
}

#[async_trait]
pub trait Connection<E>: Sync {
    async fn read<T: scale::Decode>(&self, account_id: AccountId, data: Vec<u8>) -> Result<T, E>;
}
