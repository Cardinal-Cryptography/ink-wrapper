use std::sync::Mutex;

use async_trait::async_trait;
use drink::{
    chain_api::ChainApi, contract_api::ContractApi as _, runtime::Runtime, DispatchError, Sandbox,
    DEFAULT_GAS_LIMIT,
};
use ink_primitives::AccountId;
use thiserror::Error;

use crate::{
    Connection, ContractEvents, ExecCall, InstantiateCall, ReadCall, SignedConnection, UploadCall,
    UploadConnection,
};

pub struct DrinkConnection<R: Runtime> {
    sandbox: Mutex<Sandbox<R>>,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Dispatch error {:?}", .0)]
    DispatchError(DispatchError),
    #[error("Decode error {:?}", .0)]
    DecodeError(scale::Error),
    #[error("Drink connection mutex poisoned")]
    PoisonError,
    #[error("Code hash from upload does not match expected code hash")]
    CodeHashMismatch,
}

impl From<DispatchError> for Error {
    fn from(error: DispatchError) -> Self {
        Error::DispatchError(error)
    }
}

impl From<scale::Error> for Error {
    fn from(error: scale::Error) -> Self {
        Error::DecodeError(error)
    }
}

impl<T> From<std::sync::PoisonError<T>> for Error {
    fn from(error: std::sync::PoisonError<T>) -> Self {
        Error::PoisonError
    }
}

impl<R: Runtime> DrinkConnection<R> {
    pub fn new(sandbox: Sandbox<R>) -> Self {
        Self {
            sandbox: Mutex::new(sandbox),
        }
    }
}

type TxInfo = ();

#[async_trait]
impl<R: Runtime + Send> SignedConnection<(), Error> for DrinkConnection<R>
where
    R::AccountId: AsRef<[u8; 32]> + From<[u8; 32]>,
{
    async fn instantiate_tx<T: Send + From<AccountId>>(
        &self,
        call: InstantiateCall<T>,
    ) -> Result<(T, TxInfo), Error> {
        let mut sandbox = self.sandbox.lock().unwrap();
        let result = sandbox.instantiate_contract(
            call.code_hash.to_vec(),
            call.value,
            call.data,
            call.salt,
            R::default_actor(),
            DEFAULT_GAS_LIMIT,
            None,
        );

        let account_id: AccountId = (*result.result?.account_id.as_ref()).into();
        Ok((account_id.into(), ()))
    }

    async fn exec(&self, call: ExecCall) -> Result<(), Error> {
        let mut sandbox = self.sandbox.lock()?;
        let account_id: [u8; 32] = *call.account_id.as_ref();
        let result = sandbox.call_contract(
            account_id.into(),
            call.value,
            call.data,
            R::default_actor(),
            DEFAULT_GAS_LIMIT,
            None,
        );

        Ok(())
    }
}

#[async_trait]
impl<R: Runtime + Send> Connection<(), Error> for DrinkConnection<R>
where
    R::AccountId: AsRef<[u8; 32]> + From<[u8; 32]>,
{
    async fn read<T: scale::Decode + Send>(&self, call: ReadCall<T>) -> Result<T, Error> {
        let mut sandbox = self.sandbox.lock()?;
        let account_id: [u8; 32] = *call.account_id.as_ref();
        let result = sandbox.call_contract(
            account_id.into(),
            0,
            call.data,
            R::default_actor(),
            DEFAULT_GAS_LIMIT,
            None,
        );

        Ok(scale::Decode::decode(&mut result.result?.data.as_slice())?)
    }

    async fn get_contract_events(&self, tx_info: TxInfo) -> Result<ContractEvents, Error> {
        unimplemented!()
    }
}

#[async_trait]
impl<R: Runtime + Send> UploadConnection<(), Error> for DrinkConnection<R> {
    async fn upload(&self, call: UploadCall) -> Result<(), Error> {
        let mut sandbox = self.sandbox.lock()?;
        let res = sandbox.upload_contract(call.wasm, R::default_actor(), None)?;

        if res.code_hash.as_ref() != call.expected_code_hash {
            Err(Error::CodeHashMismatch)
        } else {
            Ok(())
        }
    }
}
