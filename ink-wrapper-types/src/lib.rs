#[cfg(feature = "aleph_client")]
mod aleph_client;
mod calls;

pub mod util;

use async_trait::async_trait;
pub use calls::*;
use ink_primitives::AccountId;

/// A connection with the ability to upload WASM code to the chain.
#[async_trait]
pub trait UploadConnection<TxInfo, E>: Sync {
    /// Upload the given WASM code to the chain.
    ///
    /// Implementation is optional, the default calls `unimplemented!()`.
    /// The implementor SHOULD verify that the code hash resulting from the upload is equal to the given `code_hash`.
    async fn upload(&self, _call: UploadCall) -> Result<TxInfo, E> {
        unimplemented!()
    }
}

/// A connection with the ability to invoke mutating operations - constructor and mutating methods.
#[async_trait]
pub trait SignedConnection<TxInfo, E>: Sync {
    /// Instantiate a contract according to the given `call`.
    async fn instantiate_tx<T: Send + From<AccountId>>(
        &self,
        call: InstantiateCall<T>,
    ) -> Result<(T, TxInfo), E>;

    /// A convenience method that unpacks the result of `instantiate_tx` if you're not interested in the `TxInfo`.
    async fn instantiate<T: Send + From<AccountId>>(
        &self,
        call: InstantiateCall<T>,
    ) -> Result<T, E> {
        let (contract, _) = self.instantiate_tx(call).await?;
        Ok(contract)
    }

    /// Perform the given mutating call.
    async fn exec(&self, call: ExecCall) -> Result<TxInfo, E>;
}

/// A read-only connection - can invoke non-mutating methods and fetch events.
#[async_trait]
pub trait Connection<TxInfo, E>: Sync {
    /// Perform the given read-only call.
    async fn read<T: scale::Decode + Send>(&self, call: ReadCall<T>) -> Result<T, E>;

    /// Fetch all events emitted by contracts in the transaction with the given `tx_info`.
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
    /// Returns the events emitted by a specific contract.
    ///
    /// Note that this method returns a `Vec<Result<_>>`. An error indicates that a particular event could not be
    /// decoded even though it was emitted byt the particular contract. This can happen if the metadata used to generate
    /// the contract wrapper is out of date. If you're sure that's not the case, then it might be a bug.
    pub fn for_contract<C: EventSource>(&self, contract: C) -> Vec<Result<C::Event, scale::Error>> {
        use scale::Decode as _;

        self.events
            .iter()
            .filter(|e| e.account_id == contract.into())
            .map(|e| C::Event::decode(&mut e.data.as_slice()))
            .collect()
    }
}

/// A wrapper around `ink_primitives::LangError` that implements `std::error::Error`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
pub struct InkLangError(ink_primitives::LangError);

impl From<ink_primitives::LangError> for InkLangError {
    fn from(e: ink_primitives::LangError) -> Self {
        Self(e)
    }
}

impl std::fmt::Display for InkLangError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "InkLangError({:?})", self.0)
    }
}

impl std::error::Error for InkLangError {}
