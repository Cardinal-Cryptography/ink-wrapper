use aleph_client::{
    api::contracts::events::ContractEmitted,
    pallet_contracts::wasm::Determinism,
    pallets::contract::{ContractCallArgs, ContractRpc, ContractsUserApi},
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

#[async_trait]
impl<C: aleph_client::AsConnection + Send + Sync> crate::Connection<TxInfo, Error> for C {
    async fn read<T: scale::Decode>(&self, account_id: AccountId, data: Vec<u8>) -> Result<T> {
        let result = dry_run(&self.as_connection(), account_id, account_id, data)
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
            match event?.as_event::<ContractEmitted>()? {
                Some(event) => {
                    let account_id: [u8; 32] = event.contract.into();

                    result.push(crate::ContractEvent {
                        account_id: account_id.into(),
                        data: event.data,
                    })
                }
                None => (),
            }
        }

        Ok(crate::ContractEvents { events: result })
    }
}

#[async_trait]
impl crate::SignedConnection<TxInfo, anyhow::Error> for aleph_client::SignedConnection {
    async fn upload(&self, wasm: Vec<u8>, code_hash: Vec<u8>) -> Result<TxInfo> {
        let origin = self.account_id().clone().into();
        let determinism = Determinism::Deterministic;

        let args = CodeUploadRequest {
            origin,
            code: wasm.clone(),
            storage_deposit_limit: None,
            determinism,
        };

        let params = rpc_params!["ContractsApi_upload_code", Bytes(args.encode())];
        let dry_run_results: CodeUploadResult<CodeHash, Balance> =
            self.rpc_call("state_call".to_string(), params).await?;
        let actual_code_hash = dry_run_results
            .map_err(|e| anyhow!("Code upload failed {:?}", e))?
            .code_hash;

        if actual_code_hash.as_ref() != code_hash {
            return Err(anyhow!(
                "Code hash mismatch: expected {:?}, got {:?}",
                code_hash,
                actual_code_hash
            ));
        }

        let tx_info = self
            .upload_code(wasm, None, Determinism::Deterministic, TxStatus::Finalized)
            .await?;

        Ok(tx_info)
    }

    async fn instantiate(
        &self,
        code_hash: [u8; 32],
        salt: Vec<u8>,
        data: Vec<u8>,
    ) -> Result<AccountId> {
        let origin = self.account_id().clone().into();
        let value = 0;

        let args = InstantiateRequest {
            origin,
            value,
            gas_limit: None,
            storage_deposit_limit: None,
            code: Code::Existing(code_hash.into()),
            data: data.clone(),
            salt: salt.clone(),
        };

        let params = rpc_params!["ContractsApi_instantiate", Bytes(args.encode())];
        let dry_run_results: ContractInstantiateResult<AccountId, Balance> =
            self.rpc_call("state_call".to_string(), params).await?;
        let account_id = dry_run_results
            .result
            .map_err(|e| anyhow!("Contract instantiation failed {:?}", e))?
            .account_id;

        ContractsUserApi::instantiate(
            self,
            code_hash.into(),
            value,
            Weight {
                ref_time: dry_run_results.gas_required.ref_time(),
                proof_size: dry_run_results.gas_required.proof_size(),
            },
            None,
            data,
            salt,
            TxStatus::Finalized,
        )
        .await?;

        Ok(account_id.into())
    }

    async fn exec(&self, account_id: ink_primitives::AccountId, data: Vec<u8>) -> Result<TxInfo> {
        let result = dry_run(
            &self.as_connection(),
            account_id,
            self.account_id().clone(),
            data.clone(),
        )
        .await?;
        let account_id: [u8; 32] = *account_id.as_ref();

        self.call(
            account_id.into(),
            0,
            Weight {
                ref_time: result.gas_required.ref_time(),
                proof_size: result.gas_required.proof_size(),
            },
            None,
            data,
            TxStatus::Finalized,
        )
        .await
    }
}

async fn dry_run<A1: AsRef<[u8; 32]>, A2: AsRef<[u8; 32]>>(
    conn: &aleph_client::Connection,
    contract: A1,
    call_as: A2,
    data: Vec<u8>,
) -> Result<ContractExecResult<Balance>> {
    let args = ContractCallArgs {
        origin: call_as.as_ref().clone().into(),
        dest: contract.as_ref().clone().into(),
        value: 0,
        gas_limit: None,
        input_data: data,
        storage_deposit_limit: None,
    };

    conn.call_and_get(args)
        .await
        .context("RPC request error - there may be more info in node logs.")
}
