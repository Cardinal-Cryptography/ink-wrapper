use async_trait::async_trait;
use ink_primitives::AccountId;

#[async_trait]
pub trait SignedConnection<TxInfo, E> {
    async fn exec(&self, account_id: AccountId, data: Vec<u8>) -> Result<TxInfo, E>;
}

#[async_trait]
pub trait Connection<E> {
    async fn read<T: scale::Decode>(&self, account_id: AccountId, data: Vec<u8>) -> Result<T, E>;
}
