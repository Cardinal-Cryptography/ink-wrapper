use std::marker::PhantomData;

use ink_primitives::AccountId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TxStatus {
    #[default]
    Finalized,
    InBlock,
    Submitted,
}

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
    /// The tx_status to wait on.
    pub tx_status: TxStatus,
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
            tx_status: TxStatus::Finalized,
            _contract: Default::default(),
        }
    }

    /// Set the salt to use for the instantiation.
    pub fn with_salt(mut self, salt: Vec<u8>) -> Self {
        self.salt = salt;
        self
    }

    /// Set the tx_status to wait on.
    pub fn with_tx_status(mut self, tx_status: TxStatus) -> Self {
        self.tx_status = tx_status;
        self
    }
}

/// Represents a contract call to a payable constructor that still needs the value transferred to be specified.
/// Use the `with_value()` method to set the value.
#[derive(Debug, Clone)]
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
pub struct ExecCall<T: scale::Decode + Send> {
    /// The account id of the contract to call.
    pub account_id: AccountId,
    /// The encoded data of the call.
    pub data: Vec<u8>,
    /// The value to be sent with the call.
    pub value: u128,
    /// The tx_status to wait on.
    pub tx_status: TxStatus,
    /// A marker for the type to decode the result into.
    _return_type: PhantomData<T>,
}

impl<T: scale::Decode + Send> ExecCall<T> {
    /// Create a new exec call.
    pub fn new(account_id: AccountId, data: Vec<u8>) -> Self {
        Self {
            account_id,
            data,
            value: 0,
            tx_status: TxStatus::Finalized,
            _return_type: Default::default(),
        }
    }

    pub fn with_tx_status(mut self, tx_status: TxStatus) -> Self {
        self.tx_status = tx_status;
        self
    }
}

/// Reperesents a contract call to a payable method that still needs the value transferred to be specified.
/// Use the `with_value()` method to set the value.
#[derive(Debug, Clone)]
pub struct ExecCallNeedsValue<T: scale::Decode + Send> {
    /// The account id of the contract to call.
    pub account_id: AccountId,
    /// The encoded data of the call.
    pub data: Vec<u8>,
    /// A marker for the type to decode the result into.
    _return_type: PhantomData<T>,
}

impl<T: scale::Decode + Send> ExecCallNeedsValue<T> {
    /// Create a new needs value call.
    pub fn new(account_id: AccountId, data: Vec<u8>) -> Self {
        Self {
            account_id,
            data,
            _return_type: Default::default(),
        }
    }

    /// Set the value to be sent with the call.
    pub fn with_value(self, value: u128) -> ExecCall<T> {
        ExecCall {
            value,
            ..ExecCall::new(self.account_id, self.data)
        }
    }
}

#[derive(Debug, Clone)]
pub struct ReadCallArgs<T> {
    /// The account id of the contract to call.
    pub account_id: AccountId,
    /// The encoded data of the call.
    pub data: Vec<u8>,
    /// The value to be sent with the call.
    pub value: u128,
    /// A marker for the type to decode the result into.
    _return_type: PhantomData<T>,
}

impl<T: scale::Decode + Send> From<ReadCall<T>> for ReadCallArgs<T> {
    fn from(value: ReadCall<T>) -> ReadCallArgs<T> {
        ReadCallArgs {
            account_id: value.account_id,
            data: value.data.clone(),
            value: value.value,
            _return_type: Default::default(),
        }
    }
}

impl<T: scale::Decode + Send> From<ExecCall<T>> for ReadCallArgs<T> {
    fn from(value: ExecCall<T>) -> ReadCallArgs<T> {
        ReadCallArgs {
            account_id: value.account_id,
            data: value.data.clone(),
            value: value.value,
            _return_type: Default::default(),
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
    /// The value to be sent with the call.
    pub value: u128,
    /// A marker for the type to decode the result into.
    _return_type: PhantomData<T>,
}

pub struct ReadCallNeedsValue<T: scale::Decode + Send> {
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
            value: 0,
            _return_type: Default::default(),
        }
    }

    /// Set the value to be sent with the call.
    pub fn with_value(self, value: u128) -> Self {
        Self {
            value,
            ..Self::new(self.account_id, self.data)
        }
    }
}

impl<T: scale::Decode + Send> ReadCallNeedsValue<T> {
    /// Create a new needs value call.
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
    /// The tx_status to wait on.
    pub tx_status: TxStatus,
}

impl UploadCall {
    /// Create a new upload call.
    pub fn new(wasm: Vec<u8>, expected_code_hash: [u8; 32]) -> Self {
        Self {
            wasm,
            expected_code_hash,
            tx_status: TxStatus::Finalized,
        }
    }

    /// Set the tx_status to wait on.
    pub fn with_tx_status(mut self, tx_status: TxStatus) -> Self {
        self.tx_status = tx_status;
        self
    }
}
