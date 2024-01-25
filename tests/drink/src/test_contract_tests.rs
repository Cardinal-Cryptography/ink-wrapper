use anyhow::Result;
use assert2::assert;
use drink::{runtime::MinimalRuntime, session::Session};
use ink_primitives::AccountId;
use ink_wrapper_types::{util::ToAccountId, Connection, ContractEvents};

use crate::{
    test_contract::{self, Enum1, Instance, Struct1, Struct2},
    *,
};

fn setup(caller: drink::AccountId32) -> (Session<MinimalRuntime>, Instance) {
    let mut session = Session::new().expect("Init new Session");
    let _code_hash = session.upload_code(test_contract::upload()).unwrap();

    let _ = session.set_actor(caller);

    let address = session
        .instantiate(Instance::default())
        .unwrap()
        .result
        .to_account_id()
        .into();

    (session, address)
}

#[test]
fn test_simple_integer_messages() -> Result<()> {
    let (mut session, instance) = setup(BOB);

    let old_val = session.query(instance.get_u32()).unwrap().result.unwrap();
    let new_val = old_val + 42;
    let _res = session
        .execute(instance.set_u32(new_val))
        .unwrap()
        .result
        .unwrap();

    let updated_val = session.query(instance.get_u32()).unwrap().result.unwrap();
    assert!(updated_val == updated_val);

    Ok(())
}

#[test]
fn test_struct_messages() -> Result<()> {
    let (mut session, instance) = setup(BOB);

    let val = Struct2(
        Struct1 {
            a: 1,
            b: 2,
            c: [2, 3, 4, 5],
        },
        Enum1::B(3),
    );
    let _r = session.execute(instance.set_struct2(val.clone())).unwrap();
    let get = session
        .query(instance.get_struct2())
        .unwrap()
        .result
        .unwrap();
    assert!(get == val);
    Ok(())
}

#[test]
fn test_array_messages() -> Result<()> {
    let (mut session, instance) = setup(BOB);

    let _r = session.execute(instance.set_array([1, 2, 3])).unwrap();
    let _r = session.execute(instance.set_enum1(Enum1::A())).unwrap();

    let got = session.query(instance.get_array()).unwrap().result.unwrap();
    assert!(got == [(1, Enum1::A()), (1, Enum1::A())]);
    Ok(())
}

#[test]
fn test_sequence_messages() -> Result<()> {
    let (mut session, instance) = setup(BOB);

    let _r = session
        .execute(instance.set_sequence(vec![5, 2, 3]))
        .unwrap();
    let _r = session.execute(instance.set_enum1(Enum1::A())).unwrap();
    let got = session.query(instance.get_array()).unwrap().result.unwrap();
    assert!(got == [(5, Enum1::A()), (5, Enum1::A())]);

    Ok(())
}

#[test]
fn test_compact_messages() -> Result<()> {
    let (mut session, instance) = setup(BOB);

    let _r = session
        .execute(instance.set_compact(scale::Compact(42)))
        .unwrap();
    let got = session
        .query(instance.get_compact())
        .unwrap()
        .result
        .unwrap();
    assert!(got == scale::Compact(42));

    Ok(())
}

#[test]
fn test_messages_with_clashing_argument_names() -> Result<()> {
    let (mut session, instance) = setup(BOB);

    let _r = session
        .execute(instance.set_forbidden_names(1, 2, 3, 4, 5))
        .unwrap();

    let read = session.query(instance.get_u32()).unwrap();
    assert!(read.result.unwrap() == 1 + 2 + 3 + 4 + 5);

    Ok(())
}

#[test]
fn test_conversion_to_account_id() -> Result<()> {
    let (mut session, instance) = setup(BOB);

    let _r = session.execute(instance.set_u32(12345)).unwrap();

    let account_id: AccountId = instance.into();
    let instance: Instance = account_id.into();

    assert!(session.query(instance.get_u32()).unwrap().result.unwrap() == 12345);

    Ok(())
}

#[test]
fn test_events() -> Result<()> {
    use test_contract::event::Event;

    let (mut session, instance) = setup(BOB);

    let struct2 = Struct2(
        Struct1 {
            a: 1,
            b: 2,
            c: [0; 4],
        },
        Enum1::B(3),
    );

    let _r = session
        .execute(instance.set_u32(123))
        .unwrap()
        .result
        .unwrap();

    let _r = session
        .execute(instance.set_struct2(struct2.clone()))
        .unwrap()
        .result
        .unwrap();

    let struct1 = session
        .query(instance.get_struct1())
        .unwrap()
        .result
        .unwrap();

    let txn = session.execute(instance.generate_events()).unwrap();

    let events = ContractEvents::from_iter(&txn.events, instance);

    assert!(
        events[0]
            == Ok(Event::Event1 {
                a: 123,
                b: struct2.clone(),
                c: struct1.c,
                d: (struct1.clone(), struct2),
                e: Some(struct1),
            })
    );
    assert!(events[1] == Ok(Event::Event2 {}));

    Ok(())
}

#[test]
fn test_ink_lang_error() -> Result<()> {
    let (mut session, instance) = setup(BOB);

    let r = session
        .query(instance.generate_ink_lang_error())
        .unwrap()
        .result;

    assert!(r.unwrap().to_string() == "InkLangError(CouldNotReadInput)");

    Ok(())
}

#[test]
fn test_upload() -> Result<()> {
    let mut session: Session<MinimalRuntime> = Session::new().expect("Init new Session");
    let code_hash = session.upload_code(test_contract::upload()).unwrap();
    assert!(code_hash.as_ref() == test_contract::CODE_HASH);
    Ok(())
}

#[test]
fn test_receiving_value() -> Result<()> {
    let (mut session, instance) = setup(BOB);

    let result = session
        .execute(instance.receive_value().with_value(123))
        .unwrap();

    let events = ContractEvents::from_iter(&result.events, instance);

    assert!(events[0] == Ok(test_contract::event::Event::Received { value: 123 }));

    Ok(())
}

#[test]
fn test_receiving_value_in_constructor() -> Result<()> {
    let mut session: Session<MinimalRuntime> = Session::new().expect("Init new Session");
    let _code_hash = session.upload_code(test_contract::upload()).unwrap();

    let _ = session.set_actor(BOB);

    let txn: ink_wrapper_types::ContractInstantiateResult<drink::AccountId32> = session
        .instantiate(Instance::payable_constructor().with_value(123))
        .unwrap();

    let instance: Instance = txn.result.to_account_id().into();

    let events = ContractEvents::from_iter(&txn.events, instance);

    assert!(events[0] == Ok(test_contract::event::Event::Received { value: 123 }));

    Ok(())
}
