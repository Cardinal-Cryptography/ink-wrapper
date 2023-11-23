mod client;

use ::drink::{errors::MessageResult, runtime::HashFor, session::error::SessionError, Weight};
pub use client::*;

use crate::{ContractEvent, InstantiateCall, ReadCall, UploadCall};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Drink error: {0}")]
    DrinkError(SessionError),
    #[error("Decoding error: {0}")]
    DecodingError(String),
    #[error("Code hash mismatch")]
    CodeHashMismatch,
}

impl From<SessionError> for Error {
    fn from(e: SessionError) -> Self {
        Self::DrinkError(e)
    }
}

pub trait Connection<R: frame_system::Config> {
    fn upload_code(&mut self, call: UploadCall) -> Result<HashFor<R>, Error>;

    fn instantiate<T: Send>(
        &mut self,
        call: InstantiateCall<T>,
    ) -> Result<ContractInstantiateResult<R::AccountId>, Error>;

    fn exec<T: scale::Decode + Send>(
        &mut self,
        call: ReadCall<T>,
    ) -> Result<ContractExecResult<MessageResult<T>>, Error>;

    // like `exec`, but does not commit changes
    fn read<T: scale::Decode + Send>(
        &mut self,
        call: ReadCall<T>,
    ) -> Result<ContractReadResult<MessageResult<T>>, Error>;
}

pub struct ContractResult<R> {
    pub gas_consumed: Weight,
    pub gas_required: Weight,
    pub result: R,
    pub events: Vec<ContractEvent>,
}

pub type ContractInstantiateResult<AccountId> = ContractResult<AccountId>;

pub type ContractExecResult<R> = ContractResult<R>;

pub type ContractReadResult<R> = ContractResult<R>;
