mod client;

use ::drink::{frame_system, runtime::HashFor, DispatchError, Weight};
pub use client::*;

use crate::{ContractEvent, ExecCall, InstantiateCall, ReadCall, UploadCall};

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
    // TODO return deposit as well.
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
        call: ReadCall<T>,
    ) -> Result<ContractReadResult<T>, Error>;
}

#[derive(Debug)]
pub struct ContractResult<R> {
    pub gas_consumed: Weight,
    pub gas_required: Weight,
    pub result: R,
    pub events: Vec<ContractEvent>,
    pub reverted: bool,
}

impl<R: Clone> Clone for ContractResult<R> {
    fn clone(&self) -> Self {
        Self {
            gas_consumed: self.gas_consumed,
            gas_required: self.gas_required,
            result: self.result.clone(),
            events: self.events.clone(),
            reverted: self.reverted,
        }
    }
}

pub type ContractInstantiateResult<AccountId> = ContractResult<AccountId>;

pub type ContractExecResult<R> = ContractResult<R>;

pub type ContractReadResult<R> = ContractResult<R>;
