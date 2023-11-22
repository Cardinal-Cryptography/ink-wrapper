use aleph_client::{
    api::contracts::events::ContractEmitted,
    pallet_contracts::wasm::Determinism,
    pallets::contract::{ContractCallArgs, ContractRpc, ContractsUserApi, EventRecord},
    sp_core::H256,
    sp_weights::weight_v2::Weight,
    utility::BlocksApi,
    AsConnection, Balance, CodeHash, ConnectionApi, SignedConnectionApi, TxInfo, TxStatus,
};
use anyhow::{anyhow, Context, Error, Result};
use async_trait::async_trait;
use ink_primitives::AccountId;
use pallet_contracts_primitives::{
    CodeUploadResult, ContractExecResult, ContractInstantiateResult,
};
use scale::Encode;
use subxt::{ext::sp_core::Bytes, rpc_params};

use crate::{ExecCall, InstantiateCall, ReadCall, UploadCall};

/// This matches the expected API of an instantiate request in the pallet_contracts, do not change unless that changes.
#[derive(Encode)]
struct InstantiateRequest {
    origin: [u8; 32],
    value: Balance,
    gas_limit: Option<Weight>,
    storage_deposit_limit: Option<Balance>,
    code: Code,
    data: Vec<u8>,
    salt: Vec<u8>,
}

#[derive(Encode)]
enum Code {
    /// The Wasm blob to be instantiated.
    #[allow(dead_code)]
    Code(Vec<u8>),
    /// The code hash of an on-chain Wasm blob.
    Existing(CodeHash),
}

/// This matches the expected API of a code upload request in the pallet_contracts, do not change unless that changes.
#[derive(Encode)]
pub struct CodeUploadRequest {
    origin: [u8; 32],
    code: Vec<u8>,
    storage_deposit_limit: Option<Balance>,
    determinism: Determinism,
}

impl From<crate::TxStatus> for TxStatus {
    fn from(status: crate::TxStatus) -> Self {
        match status {
            crate::TxStatus::Submitted => TxStatus::Submitted,
            crate::TxStatus::Finalized => TxStatus::Finalized,
            crate::TxStatus::InBlock => TxStatus::InBlock,
        }
    }
}

#[async_trait]
impl<C: aleph_client::AsConnection + Send + Sync> crate::Connection<TxInfo, Error> for C {
    async fn read<T: scale::Decode + Send>(&self, call: ReadCall<T>) -> Result<T> {
        let result = dry_run(
            self.as_connection(),
            call.account_id,
            call.account_id,
            0,
            call.data,
        )
        .await?
        .result
        .map_err(|e| anyhow!("Contract exec failed {:?}", e))?;

        Ok(scale::Decode::decode(&mut result.data.as_slice())
            .context("Failed to decode contract call result")?)
    }

    async fn get_contract_events(&self, tx_info: TxInfo) -> Result<crate::ContractEvents> {
        let events = self.as_connection().get_tx_events(tx_info).await?;
        let mut result = vec![];

        for event in events.iter() {
            if let Some(event) = event?.as_event::<ContractEmitted>()? {
                let account_id: [u8; 32] = event.contract.0.into();

                result.push(crate::ContractEvent {
                    account_id: account_id.into(),
                    data: event.data,
                })
            }
        }

        Ok(crate::ContractEvents { events: result })
    }
}

#[async_trait]
impl crate::UploadConnection<TxInfo, anyhow::Error> for aleph_client::SignedConnection {
    async fn upload(&self, call: UploadCall, tx_status: crate::TxStatus) -> Result<TxInfo> {
        let origin = self.account_id().clone().into();
        let determinism = Determinism::Enforced;

        let args = CodeUploadRequest {
            origin,
            code: call.wasm.clone(),
            storage_deposit_limit: None,
            determinism,
        };

        let params = rpc_params!["ContractsApi_upload_code", Bytes(args.encode())];
        let dry_run_results: CodeUploadResult<CodeHash, Balance> =
            self.rpc_call("state_call".to_string(), params).await?;
        let actual_code_hash = dry_run_results
            .map_err(|e| anyhow!("Code upload failed {:?}", e))?
            .code_hash;

        let expected_code_hash = H256(call.expected_code_hash);
        if actual_code_hash != expected_code_hash {
            return Err(anyhow!(
                "Code hash mismatch: expected {:?}, got {:?}",
                expected_code_hash,
                actual_code_hash
            ));
        }

        let tx_info = self
            .upload_code(call.wasm, None, Determinism::Enforced, tx_status.into())
            .await?;

        Ok(tx_info)
    }
}

#[async_trait]
impl crate::SignedConnection<TxInfo, anyhow::Error> for aleph_client::SignedConnection {
    async fn instantiate_tx<T: Send + From<AccountId>>(
        &self,
        call: InstantiateCall<T>,
        tx_status: crate::TxStatus,
    ) -> Result<(T, TxInfo)> {
        let origin = self.account_id().clone().into();
        let value = call.value;

        let args = InstantiateRequest {
            origin,
            value,
            gas_limit: None,
            storage_deposit_limit: None,
            code: Code::Existing(call.code_hash.into()),
            data: call.data.clone(),
            salt: call.salt.clone(),
        };

        let params = rpc_params!["ContractsApi_instantiate", Bytes(args.encode())];
        let dry_run_results: ContractInstantiateResult<AccountId, Balance, EventRecord> =
            self.rpc_call("state_call".to_string(), params).await?;
        let account_id = dry_run_results
            .result
            .map_err(|e| anyhow!("Contract instantiation failed {:?}", e))?
            .account_id;

        let tx_info = ContractsUserApi::instantiate(
            self,
            call.code_hash.into(),
            value,
            Weight {
                ref_time: dry_run_results.gas_required.ref_time(),
                proof_size: dry_run_results.gas_required.proof_size(),
            },
            None,
            call.data,
            call.salt,
            tx_status.into(),
        )
        .await?;

        Ok((account_id.into(), tx_info))
    }

    async fn exec(&self, call: ExecCall, tx_status: crate::TxStatus) -> Result<TxInfo> {
        let result = dry_run(
            self.as_connection(),
            call.account_id,
            self.account_id().clone(),
            call.value,
            call.data.clone(),
        )
        .await?;
        let account_id: [u8; 32] = *call.account_id.as_ref();

        self.call(
            account_id.into(),
            call.value,
            Weight {
                ref_time: result.gas_required.ref_time(),
                proof_size: result.gas_required.proof_size(),
            },
            None,
            call.data,
            tx_status.into(),
        )
        .await
    }
}

async fn dry_run<A1: AsRef<[u8; 32]>, A2: AsRef<[u8; 32]>>(
    conn: &aleph_client::Connection,
    contract: A1,
    call_as: A2,
    value: Balance,
    data: Vec<u8>,
) -> Result<ContractExecResult<Balance, EventRecord>> {
    let args = ContractCallArgs {
        origin: (*call_as.as_ref()).into(),
        dest: (*contract.as_ref()).into(),
        value,
        gas_limit: None,
        input_data: data,
        storage_deposit_limit: None,
    };

    conn.call_and_get(args)
        .await
        .context("RPC request error - there may be more info in node logs.")
}
