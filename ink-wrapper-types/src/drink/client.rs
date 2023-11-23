use ::drink::{
    errors::MessageResult,
    pallet_contracts,
    runtime::{HashFor, MinimalRuntime},
    session::Session,
    Weight,
};
use scale::Decode;

use super::*;
use crate::util::ToAccountId;

// NOTE: This needs to be fixed at `MinimalRuntime` as `ink-wrapper` uses `u128` to represent
// token balances. `R::Balance` is a trait which does not provide conversion from `u128`.
// `MinimalRuntime` has its `Balance` type
impl Connection<MinimalRuntime> for Session<MinimalRuntime> {
    fn upload_code(&mut self, call: UploadCall) -> Result<HashFor<MinimalRuntime>, Error> {
        let code_hash = self.upload(call.wasm)?;
        if code_hash.as_ref() != call.expected_code_hash {
            return Err(Error::CodeHashMismatch);
        }
        Ok(code_hash)
    }

    fn instantiate<T: Send>(
        &mut self,
        call: InstantiateCall<T>,
    ) -> Result<ContractInstantiateResult<<MinimalRuntime as frame_system::Config>::AccountId>, Error>
    {
        let actor = self.get_actor();
        let gas_limit = self.get_gas_limit();

        let instantiate_contract_result = self.sandbox().instantiate_contract(
            call.code_hash.to_vec(),
            call.value,
            call.data,
            call.salt,
            actor,
            gas_limit,
            None,
        );

        let contract_address = match &instantiate_contract_result.result {
            Ok(exec_result) if exec_result.result.did_revert() => Err(Error::DeploymentReverted),
            Err(err) => Err(Error::DeploymentFailed(*err)),
            Ok(exec_result) => Ok(exec_result.account_id.clone()),
        }?;

        let events = extract_events(&instantiate_contract_result.events);

        Ok(ContractInstantiateResult {
            gas_consumed: instantiate_contract_result.gas_consumed,
            gas_required: instantiate_contract_result.gas_required,
            result: contract_address,
            events,
        })
    }

    fn exec<T: scale::Decode + Send>(
        &mut self,
        call: ExecCall<T>,
    ) -> Result<ContractExecResult<MessageResult<T>>, Error> {
        let actor = self.get_actor();
        let gas_limit = self.get_gas_limit();
        let contract_address = (*AsRef::<[u8; 32]>::as_ref(&call.account_id)).into();

        let result = call_contract(
            actor,
            gas_limit,
            self.sandbox(),
            contract_address,
            call.value,
            call.data,
        )?;

        Ok(result)
    }

    fn read<T: scale::Decode + Send>(
        &mut self,
        call: ReadCall<T>,
    ) -> Result<ContractReadResult<MessageResult<T>>, Error> {
        let actor = self.get_actor();
        let gas_limit = self.get_gas_limit();
        let contract_address = (*AsRef::<[u8; 32]>::as_ref(&call.account_id)).into();

        let result = self.sandbox().dry_run(|sandbox| {
            call_contract(
                actor,
                gas_limit,
                sandbox,
                contract_address,
                call.value,
                call.data,
            )
        })?;

        Ok(result)
    }
}

fn call_contract<T: scale::Decode + Send>(
    actor: <MinimalRuntime as frame_system::Config>::AccountId,
    gas_limit: Weight,
    sandbox: &mut drink::Sandbox<MinimalRuntime>,
    address: <MinimalRuntime as frame_system::Config>::AccountId,
    value: u128,
    data: Vec<u8>,
) -> Result<ContractResult<MessageResult<T>>, Error> {
    let result = sandbox.call_contract(
        address,
        value,
        data,
        actor,
        gas_limit,
        None,
        pallet_contracts::Determinism::Enforced,
    );

    let message_result: MessageResult<T> = match &result.result {
        Ok(exec_result) if exec_result.did_revert() => Err(Error::CallReverted),
        Ok(exec_result) => {
            let encoded = exec_result.data.clone();
            println!("encoded: {:?}", encoded);

            MessageResult::<T>::decode(&mut encoded.as_slice()).map_err(|err| {
                Error::DecodingError(format!(
                    "Failed to decode the result of calling a contract: {err:?}",
                ))
            })
        }
        Err(err) => Err(Error::CallFailed(*err)),
    }?;

    let events = extract_events(&result.events);

    Ok(ContractResult {
        gas_consumed: result.gas_consumed,
        gas_required: result.gas_required,
        result: message_result,
        events,
    })
}

fn extract_events(
    events: &Option<Vec<drink::EventRecordOf<MinimalRuntime>>>,
) -> Vec<ContractEvent> {
    events
        .clone()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|event| match event.event {
            // For now, just interested in pallet contract events.
            drink::runtime::minimal::RuntimeEvent::Contracts(events) => Some(events),
            _ => None,
        })
        .filter_map(|event| match event {
            pallet_contracts::pallet::Event::ContractEmitted { contract, data } => {
                Some(ContractEvent {
                    account_id: contract.to_account_id(),
                    data,
                })
            }
            _ => None,
        })
        .collect()
}
