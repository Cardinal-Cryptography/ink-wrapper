use anyhow::Result;
use assert2::assert;
use drink::{runtime::MinimalRuntime, session::Session, AccountId32};
use ink_primitives::AccountId;
use ink_wrapper_types::{util::ToAccountId, Connection};
use psp22_contract::{Instance, PSP22 as _};

use crate::psp22_contract;

const ALICE: drink::AccountId32 = AccountId32::new([0u8; 32]);
const BOB: drink::AccountId32 = AccountId32::new([1u8; 32]);

fn alice() -> ink_primitives::AccountId {
    AsRef::<[u8; 32]>::as_ref(&ALICE).clone().into()
}

fn bob() -> ink_primitives::AccountId {
    AsRef::<[u8; 32]>::as_ref(&BOB).clone().into()
}

fn balance_of(session: &mut Session<MinimalRuntime>, psp22: Instance, account: AccountId) -> u128 {
    session
        .query(psp22.balance_of(account))
        .unwrap()
        .result
        .unwrap()
}

#[test]
fn test_transfers() -> Result<()> {
    let mut session: Session<MinimalRuntime> = Session::new().expect("initi new Session");

    let _code_hash = session.upload_code(psp22_contract::upload()).unwrap();

    let _ = session.set_actor(BOB);

    let instance: Instance = session
        .instantiate(Instance::new(1000))
        .unwrap()
        .result
        .to_account_id()
        .into();

    let alice_balance = balance_of(&mut session, instance, alice().into());
    let bob_balance = balance_of(&mut session, instance, bob().into());

    println!("alice balance: {}", alice_balance);
    println!("bob balance: {}", bob_balance);

    let res = session.execute(instance.transfer(alice().into(), 100, vec![]));
    println!("res: {:?}", res);

    assert!(balance_of(&mut session, instance, alice().into()) == alice_balance + 100);

    Ok(())
}

// fn test_burn() -> Result<()> {
//     use psp22_contract::{PSP22Burnable as _, PSP22 as _};

//     let (conn, contract) = connect_and_deploy().await?;
//     let supply_before = conn.read(contract.total_supply()).await?.unwrap();
//     let account_id = conn.account_id().to_account_id();

//     conn.exec(contract.burn(account_id.into(), 100), TxStatus::Finalized)
//         .await?;

//     assert!(conn.read(contract.total_supply()).await?.unwrap() == supply_before - 100);

//     Ok(())
// }
