#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]
#![no_main]

#[openbrush::contract]
pub mod my_psp22 {
    use openbrush::{contracts::psp22::extensions::burnable::*, traits::Storage};

    #[ink(storage)]
    #[derive(Storage)]
    pub struct Contract {
        #[storage_field]
        psp22: psp22::Data,
    }

    impl PSP22 for Contract {}
    impl PSP22Burnable for Contract {}

    impl Contract {
        #[ink(constructor)]
        pub fn new(total_supply: Balance) -> Self {
            let mut instance = Self {
                psp22: Default::default(),
            };

            instance
                ._mint_to(Self::env().caller(), total_supply)
                .expect("Should mint");

            instance
        }
    }
}
