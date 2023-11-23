mod psp22_contract;
mod test_contract;

#[cfg(test)]
mod psp22_tests;
#[cfg(test)]
mod test_contract_tests;

use drink::AccountId32;

pub const ALICE: drink::AccountId32 = AccountId32::new([2u8; 32]);
pub const BOB: drink::AccountId32 = AccountId32::new([1u8; 32]);

pub fn alice() -> ink_primitives::AccountId {
    AsRef::<[u8; 32]>::as_ref(&ALICE).clone().into()
}

pub fn bob() -> ink_primitives::AccountId {
    AsRef::<[u8; 32]>::as_ref(&BOB).clone().into()
}
