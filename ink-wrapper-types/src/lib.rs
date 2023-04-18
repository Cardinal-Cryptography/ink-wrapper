#[cfg(feature = "aleph_client")]
mod aleph_client;

use async_trait::async_trait;
use ink_primitives::AccountId;

/// Contracts will use this trait to invoke mutating operations - constructor and mutating methods.
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

/// Contracts will use this trait for reading data from the chain - non-mutating methods and fetching events.
#[async_trait]
pub trait Connection<TxInfo, E>: Sync {
    async fn read<T: scale::Decode>(&self, account_id: AccountId, data: Vec<u8>) -> Result<T, E>;

    async fn get_contract_events(&self, tx_info: TxInfo) -> Result<ContractEvents, E>;
}

/// Represents a raw event emitted by a contract.
pub struct ContractEvent {
    /// The account id of the contract that emitted the event.
    pub account_id: AccountId,
    /// The unparsed data of the event.
    pub data: Vec<u8>,
}

/// Represents a collection of events emitted by contracts in a single transaction.
pub struct ContractEvents {
    pub events: Vec<ContractEvent>,
}

/// A trait that allows to decode events emitted by a specific contract.
pub trait EventSource: Copy + Into<AccountId> {
    /// The type to decode the emitted events into.
    type Event: scale::Decode;
}

impl ContractEvents {
    /// Returns a events emitted by a specific contract.
    pub fn for_contract<C: EventSource>(&self, contract: C) -> Vec<C::Event> {
        use scale::Decode as _;

        self.events
            .iter()
            .filter(|e| e.account_id == contract.into())
            .filter_map(|e| C::Event::decode(&mut e.data.as_slice()).ok())
            .collect()
    }
}
