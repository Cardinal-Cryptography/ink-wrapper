mod client;

use async_trait::async_trait;
pub use client::*;

use crate::*;

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
    async fn exec<T: scale::Decode + Send>(&self, call: ExecCall<T>) -> Result<TxInfo, E>;
}

/// A read-only connection - can invoke non-mutating methods and fetch events.
#[async_trait]
pub trait Connection<TxInfo, E>: Sync {
    /// Perform the given read-only call.
    async fn read<T: scale::Decode + Send>(&self, call: ReadCall<T>) -> Result<T, E>;

    /// Fetch all events emitted by contracts in the transaction with the given `tx_info`.
    async fn get_contract_events(&self, tx_info: TxInfo) -> Result<ContractEvents, E>;
}
