use anyhow::Result;
use assert2::assert;
use drink::{runtime::MinimalRuntime, session::Session};
use ink_primitives::AccountId;
use ink_wrapper_types::{util::ToAccountId, Connection};
use psp22_contract::{Instance, PSP22 as _};

use crate::*;

pub fn balance_of(
    session: &mut Session<MinimalRuntime>,
    psp22: Instance,
    account: AccountId,
) -> u128 {
    session
        .query(psp22.balance_of(account))
        .unwrap()
        .result
        .unwrap()
}

pub fn setup(session: &mut Session<MinimalRuntime>, caller: drink::AccountId32) -> Instance {
    let _code_hash = session.upload_code(psp22_contract::upload()).unwrap();

    let _ = session.set_actor(caller);

    session
        .instantiate(Instance::new(1000))
        .unwrap()
        .result
        .to_account_id()
        .into()
}

#[test]
fn test_transfers() -> Result<()> {
    let mut session: Session<MinimalRuntime> = Session::new().expect("Init new Session");
    let instance = setup(&mut session, BOB);

    let transfer_amount = 100;

    let alice_balance = balance_of(&mut session, instance, alice().into());
    let bob_balance = balance_of(&mut session, instance, bob().into());

    let _res = session
        .execute(instance.transfer(alice().into(), transfer_amount, vec![]))
        .unwrap();

    assert!(balance_of(&mut session, instance, bob().into()) == bob_balance - transfer_amount);
    assert!(balance_of(&mut session, instance, alice().into()) == alice_balance + transfer_amount);
    Ok(())
}

#[test]
fn test_burn() -> Result<()> {
    let mut session: Session<MinimalRuntime> = Session::new().expect("Init new Session");
    let instance = setup(&mut session, BOB);
    let supply_before = session
        .query(instance.total_supply())
        .unwrap()
        .result
        .unwrap();

    let to_burn = 100;

    let _res = session.execute(instance.burn(to_burn)).unwrap();

    let supply_after = session
        .query(instance.total_supply())
        .unwrap()
        .result
        .unwrap();

    assert!(supply_after == supply_before - to_burn);

    Ok(())
}
