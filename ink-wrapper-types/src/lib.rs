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
pub trait Connection<TxInfo, E>: Sync {
    async fn read<T: scale::Decode>(&self, account_id: AccountId, data: Vec<u8>) -> Result<T, E>;

    async fn get_contract_events(&self, tx_info: TxInfo) -> Result<ContractEvents, E>;
}

pub struct ContractEvent {
    pub account_id: AccountId,
    pub data: Vec<u8>,
}

pub struct ContractEvents {
    pub events: Vec<ContractEvent>,
}

pub trait EventSource: Copy + Into<AccountId> {
    type Event: scale::Decode;
}

impl ContractEvents {
    pub fn for_contract<C: EventSource>(&self, contract: C) -> Vec<C::Event> {
        use scale::Decode as _;

        self.events
            .iter()
            .filter(|e| e.account_id == contract.into())
            .filter_map(|e| C::Event::decode(&mut e.data.as_slice()).ok())
            .collect()
    }
}
