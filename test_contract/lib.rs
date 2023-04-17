#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
mod test_contract {
    use ink::prelude::{vec, vec::Vec};
    #[cfg(feature = "std")]
    use ink::storage::traits::StorageLayout;
    use scale::Compact;

    #[ink(storage)]
    #[derive(Default)]
    pub struct TestContract {
        u32_val: u32,
        bool_val: bool,
        struct1_val: Struct1,
        enum1_val: Enum1,
        struct2_val: Struct2,
        enum2_val: Enum2,
        newtype1_val: NewType1,
    }

    /// Example docs for an event.
    /// They are multiline.
    #[ink(event)]
    pub struct Event1 {
        /// Example docs for an event field.
        /// They are multiline.
        #[ink(topic)]
        a: u32,
        b: Struct2,
    }

    #[ink(event)]
    pub struct Event2;

    #[derive(Debug, Clone, Copy, Default, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, StorageLayout))]
    pub struct Struct1 {
        a: u32,
        b: u64,
    }

    #[derive(Debug, Clone, Copy, Default, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, StorageLayout))]
    pub enum Enum1 {
        #[default]
        A,
        B(u32),
        C(u32, u64),
    }

    #[derive(Debug, Clone, Copy, Default, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, StorageLayout))]
    pub struct Struct2(Struct1, Enum1);

    #[derive(Debug, Clone, Copy, Default, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, StorageLayout))]
    pub enum Enum2 {
        #[default]
        A,
        B(Struct1),
        C {
            name1: Struct1,
            name2: (Enum1, Enum1),
        },
    }

    type NewType1 = u32;

    impl TestContract {
        /// Example docs for a constructor.
        /// They are multiline.
        #[ink(constructor)]
        pub fn new(an_u32: u32, a_bool: bool) -> Self {
            Self {
                u32_val: an_u32,
                bool_val: a_bool,
                ..Default::default()
            }
        }

        #[ink(constructor)]
        pub fn default() -> Self {
            Self {
                ..Default::default()
            }
        }

        #[ink(message)]
        pub fn get_account_id(&self, account_id: AccountId) -> AccountId {
            account_id
        }

        /// Example docs for a message.
        /// They are multiline.
        #[ink(message)]
        pub fn get_u32(&self) -> u32 {
            self.u32_val
        }

        #[ink(message)]
        pub fn get_struct1(&self) -> Struct1 {
            self.struct1_val
        }

        #[ink(message)]
        pub fn get_enum1(&self) -> Enum1 {
            self.enum1_val
        }

        #[ink(message)]
        pub fn get_struct2(&self) -> Struct2 {
            self.struct2_val
        }

        #[ink(message)]
        pub fn get_enum2(&self) -> Enum2 {
            self.enum2_val
        }

        #[ink(message)]
        pub fn get_newtype1(&self) -> NewType1 {
            self.newtype1_val
        }

        #[ink(message)]
        pub fn get_bool(&self) -> bool {
            self.bool_val
        }

        #[ink(message)]
        pub fn set_u32(&mut self, an_u32: u32) {
            self.u32_val = an_u32;
        }

        #[ink(message)]
        pub fn set_bool(&mut self, a_bool: bool) {
            self.bool_val = a_bool;
        }

        #[ink(message)]
        pub fn set_struct1(&mut self, a_struct1: Struct1) {
            self.struct1_val = a_struct1;
        }

        #[ink(message)]
        pub fn set_enum1(&mut self, an_enum1: Enum1) {
            self.enum1_val = an_enum1;
        }

        #[ink(message)]
        pub fn set_struct2(&mut self, a_struct2: Struct2) {
            self.struct2_val = a_struct2;
        }

        #[ink(message)]
        pub fn set_enum2(&mut self, an_enum2: Enum2) {
            self.enum2_val = an_enum2;
        }

        #[ink(message)]
        pub fn set_newtype1(&mut self, a_newtype1: NewType1) {
            self.newtype1_val = a_newtype1;
        }

        #[ink(message)]
        pub fn set_array(&mut self, an_array: [u32; 3]) {
            self.u32_val = an_array[0];
        }

        #[ink(message)]
        pub fn get_array(&self) -> [(u32, Enum1); 2] {
            [
                (self.u32_val, self.enum1_val),
                (self.u32_val, self.enum1_val),
            ]
        }

        #[ink(message)]
        pub fn set_sequence(&mut self, a_sequence: Vec<u32>) {
            self.u32_val = a_sequence[0];
        }

        #[ink(message)]
        pub fn get_sequence(&self) -> Vec<(u32, Enum1)> {
            vec![(self.u32_val, self.enum1_val); 2]
        }

        #[ink(message)]
        pub fn get_compact(&self) -> Compact<u32> {
            Compact(self.u32_val)
        }

        #[ink(message)]
        pub fn set_compact(&mut self, a_compact: Compact<u32>) {
            self.u32_val = a_compact.0;
        }

        #[ink(message)]
        pub fn get_forbidden_names(
            &self,
            conn: u32,
            code_hash: u32,
            data: u32,
            salt: u32,
            account_id: u32,
        ) -> u32 {
            conn + code_hash + data + salt + account_id
        }

        #[ink(message)]
        pub fn set_forbidden_names(
            &mut self,
            conn: u32,
            code_hash: u32,
            data: u32,
            salt: u32,
            account_id: u32,
        ) {
            self.u32_val = conn + code_hash + data + salt + account_id;
        }

        #[ink(message)]
        pub fn generate_events(&mut self) {
            Self::env().emit_event(Event1 {
                a: self.u32_val,
                b: self.struct2_val,
            });
            Self::env().emit_event(Event2 {});
        }
    }
}
