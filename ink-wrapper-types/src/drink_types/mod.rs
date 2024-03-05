mod client;

use crate::{ContractEvent, ExecCall, InstantiateCall, QueryArgs, UploadCall};
pub use client::*;

use drink::{frame_system, runtime::HashFor, DispatchError, Weight};
use pallet_contracts_primitives::StorageDeposit;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Decoding error: {0}")]
    DecodingError(String),
    #[error("Upload failed")]
    UploadFailed,
    #[error("Code hash mismatch")]
    CodeHashMismatch,
    #[error("Deployment reverted")]
    DeploymentReverted,
    #[error("Deployment failed: {0:?}")]
    DeploymentFailed(DispatchError),
    #[error("Contract call failed: {0:?}")]
    CallFailed(DispatchError),
}

pub trait Connection<R: frame_system::Config> {
    fn upload_code(&mut self, call: UploadCall) -> Result<HashFor<R>, Error>;

    fn instantiate<T: Send>(
        &mut self,
        call: InstantiateCall<T>,
    ) -> Result<ContractInstantiateResult<R::AccountId>, Error>;

    fn execute<T: scale::Decode + Send + std::fmt::Debug>(
        &mut self,
        call: ExecCall<T>,
    ) -> Result<ContractExecResult<T>, Error>;

    /// Like `exec`, but does not commit changes
    fn query<T: scale::Decode + Send + std::fmt::Debug>(
        &mut self,
        call: impl Into<QueryArgs<T>>,
    ) -> Result<ContractReadResult<T>, Error>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContractResult<R> {
    pub gas_consumed: Weight,
    pub gas_required: Weight,
    pub result: R,
    pub events: Vec<ContractEvent>,
    pub reverted: bool,
    pub debug_message: Vec<u8>,
    pub storage_deposit: StorageDeposit<u128>,
}

pub type ContractInstantiateResult<AccountId> = ContractResult<AccountId>;

pub type ContractExecResult<R> = ContractResult<R>;

pub type ContractReadResult<R> = ContractResult<R>;
