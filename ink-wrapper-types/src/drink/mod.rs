mod client;

use ::drink::{
    runtime::{HashFor, RuntimeWithContracts},
    session::{Session, SessionError},
};
pub use client::*;

use crate::{ExecCall, InstantiateCall, ReadCall, UploadCall};

pub enum Error {
    SessionError(SessionError),
}

pub trait Connection {
    fn upload<R: frame_system::Config>(&self, call: UploadCall) -> Result<HashFor<R>, Error>;

    fn instantiate<T: Send>(
        &self,
        call: InstantiateCall<T>,
    ) -> Result<ink_primitives::AccountId, Error>;

    fn exec(&self, call: ExecCall) -> Result<ContractExecResult, Error>;

    fn read<T: scale::Decode + Send>(&self, call: ReadCall<T>) -> Result<T, Error>;
}

impl<R: RuntimeWithContracts> Connection for Session<R> {}
