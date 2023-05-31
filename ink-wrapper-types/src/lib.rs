#[cfg(feature = "aleph_client")]
mod aleph_client;
pub mod util;

use std::marker::PhantomData;

use async_trait::async_trait;
use ink_primitives::AccountId;

/// Represents a call to a contract constructor.
#[derive(Debug, Clone)]
pub struct InstantiateCall<T: Send> {
    /// The code hash of the contract to instantiate.
    pub code_hash: [u8; 32],
    /// The encoded data of the call.
    pub data: Vec<u8>,
    /// The salt to use for the contract.
    pub salt: Vec<u8>,
    /// The value to be sent with the call.
    pub value: u128,
    /// A marker for the type of contract to instantiate.
    _contract: PhantomData<T>,
}

impl<T: Send> InstantiateCall<T> {
    /// Create a new instantiate call.
    pub fn new(code_hash: [u8; 32], data: Vec<u8>) -> Self {
        Self {
            code_hash,
            data,
            salt: vec![],
            value: 0,
            _contract: Default::default(),
        }
    }

    /// Set the salt to use for the instantiation.
    pub fn with_salt(mut self, salt: Vec<u8>) -> Self {
        self.salt = salt;
        self
    }
}

pub struct InstantiateCallNeedsValue<T: Send> {
    /// The code hash of the contract to instantiate.
    pub code_hash: [u8; 32],
    /// The encoded data of the call.
    pub data: Vec<u8>,
    /// A marker for the type of contract to instantiate.
    _contract: PhantomData<T>,
}

impl<T: Send> InstantiateCallNeedsValue<T> {
    pub fn new(code_hash: [u8; 32], data: Vec<u8>) -> Self {
        Self {
            code_hash,
            data,
            _contract: Default::default(),
        }
    }

    pub fn with_value(self, value: u128) -> InstantiateCall<T> {
        InstantiateCall {
            value,
            ..InstantiateCall::new(self.code_hash, self.data)
        }
    }
}

/// Represents a mutating contract call to be made.
#[derive(Debug, Clone)]
pub struct ExecCall {
    /// The account id of the contract to call.
    pub account_id: AccountId,
    /// The encoded data of the call.
    pub data: Vec<u8>,
    /// The value to be sent with the call.
    pub value: u128,
}

impl ExecCall {
    /// Create a new exec call.
    pub fn new(account_id: AccountId, data: Vec<u8>) -> Self {
        Self {
            account_id,
            data,
            value: 0,
        }
    }
}

/// Reperesents a contract call to a payable method that still needs the value transferred to be specified.
#[derive(Debug, Clone)]
pub struct ExecCallNeedsValue {
    /// The account id of the contract to call.
    pub account_id: AccountId,
    /// The encoded data of the call.
    pub data: Vec<u8>,
}

impl ExecCallNeedsValue {
    /// Create a new needs value call.
    pub fn new(account_id: AccountId, data: Vec<u8>) -> Self {
        Self { account_id, data }
    }

    /// Set the value to be sent with the call.
    pub fn with_value(self, value: u128) -> ExecCall {
        ExecCall {
            value,
            ..ExecCall::new(self.account_id, self.data)
        }
    }
}

/// Represents a read-only contract call to be made.
#[derive(Debug, Clone)]
pub struct ReadCall<T: scale::Decode + Send> {
    /// The account id of the contract to call.
    pub account_id: AccountId,
    /// The encoded data of the call.
    pub data: Vec<u8>,
    /// A marker for the type to decode the result into.
    _return_type: PhantomData<T>,
}

impl<T: scale::Decode + Send> ReadCall<T> {
    /// Create a new read call.
    pub fn new(account_id: AccountId, data: Vec<u8>) -> Self {
        Self {
            account_id,
            data,
            _return_type: Default::default(),
        }
    }
}

/// Represents a call to upload a contract.
pub struct UploadCall {
    /// The WASM code to upload.
    pub wasm: Vec<u8>,
    /// The expected code hash of the uploaded code.
    pub expected_code_hash: [u8; 32],
}

impl UploadCall {
    /// Create a new upload call.
    pub fn new(wasm: Vec<u8>, expected_code_hash: [u8; 32]) -> Self {
        Self {
            wasm,
            expected_code_hash,
        }
    }
}

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
