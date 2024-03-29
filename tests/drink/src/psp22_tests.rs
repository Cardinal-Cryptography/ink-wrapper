use anyhow::Result;
use assert2::assert;
use drink::{runtime::MinimalRuntime, session::Session, AccountId32};
use ink_primitives::AccountId;
use ink_wrapper_types::{Connection, ToAccountId};
use psp22_contract::{Instance, PSP22 as _};

use crate::*;

pub fn balance_of(
    session: &mut Session<MinimalRuntime>,
    psp22: Instance,
    account: impl Into<AccountId>,
) -> u128 {
    session
        .query(psp22.balance_of(account.into()))
        .unwrap()
        .result
        .unwrap()
}

pub fn setup(caller: AccountId32) -> (Session<MinimalRuntime>, Instance) {
    let mut session = Session::new().expect("Init new Session");
    let _code_hash = session.upload_code(psp22_contract::upload()).unwrap();

    let _ = session.set_actor(caller);

    let address = session
        .instantiate(Instance::new(1000))
        .unwrap()
        .result
        .to_account_id()
        .into();

    (session, address)
}

#[drink::test]
fn test_transfers() -> Result<()> {
    let mut session = Session::<MinimalRuntime>::new().unwrap();
    session.upload_code(psp22_contract::upload()).unwrap();

    let _ = session.set_actor(BOB);

    let instance = session
        .instantiate(Instance::new(1000))
        .unwrap()
        .result
        .to_account_id()
        .into();

    let transfer_amount = 100;

    let alice_balance = balance_of(&mut session, instance, alice());
    let bob_balance = balance_of(&mut session, instance, bob());

    let _res = session
        .execute(instance.transfer(alice().into(), transfer_amount, vec![]))
        .unwrap();

    assert!(balance_of(&mut session, instance, bob()) == bob_balance - transfer_amount);
    assert!(balance_of(&mut session, instance, alice()) == alice_balance + transfer_amount);
    Ok(())
}

#[test]
fn test_burn() -> Result<()> {
    let (mut session, instance) = setup(BOB);
    let supply_before = session
        .query(instance.total_supply())
        .unwrap()
        .result
        .unwrap();

    let to_burn = 100;

    // Verify that we can pass `ExecCall` to `query`
    // and match on the error result.
    session.set_actor(ALICE);
    let err = session
        .query(instance.burn(to_burn))
        .unwrap()
        .result
        .unwrap();

    assert!(err == Err(psp22_contract::PSP22Error::InsufficientBalance()));

    session.set_actor(BOB);

    let _res = session.execute(instance.burn(to_burn)).unwrap();

    let supply_after = session
        .query(instance.total_supply())
        .unwrap()
        .result
        .unwrap();

    assert!(supply_after == supply_before - to_burn);

    Ok(())
}
