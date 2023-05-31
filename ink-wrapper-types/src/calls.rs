use std::marker::PhantomData;

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
/// Use the `with_value()` method to set the value.
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
