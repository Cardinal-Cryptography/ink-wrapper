use scale::Encode;

pub struct Instance {
    account_id: ink_primitives::AccountId,
}

impl From<ink_primitives::AccountId> for Instance {
    fn from(account_id: ink_primitives::AccountId) -> Self {
        Self { account_id }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, scale::Encode, scale::Decode)]
pub struct Struct2 {
    pub a: Struct1,
    pub b: Enum1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, scale::Encode, scale::Decode)]
pub struct Struct1 {
    pub a: u32,
    pub b: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, scale::Encode, scale::Decode)]
pub enum Enum1 {
    A,
    B(u32),
    C(u32, u64),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, scale::Encode, scale::Decode)]
pub enum Enum2 {
    A,
    B(Struct1),
    C {
        name1: Struct1,
        name2: (Enum1, Enum1),
    },
}

impl Instance {
    pub async fn new<E, TxInfo, C: ink_wrapper_types::SignedConnection<TxInfo, E>>(
        conn: &C,
        an_u32: u32,
        a_bool: bool,
    ) -> Result<(), ink_primitives::LangError> {
        Ok(())
    }

    pub async fn default<E, TxInfo, C: ink_wrapper_types::SignedConnection<TxInfo, E>>(
        conn: &C,
    ) -> Result<(), ink_primitives::LangError> {
        Ok(())
    }

    pub async fn get_u32<E, C: ink_wrapper_types::Connection<E>>(
        &self,
        conn: &C,
    ) -> Result<Result<u32, ink_primitives::LangError>, E> {
        let args = vec![217, 45, 11, 204];
        conn.read(self.account_id, args).await
    }

    pub async fn get_struct1<E, C: ink_wrapper_types::Connection<E>>(
        &self,
        conn: &C,
    ) -> Result<Result<Struct1, ink_primitives::LangError>, E> {
        let args = vec![67, 225, 36, 205];
        conn.read(self.account_id, args).await
    }

    pub async fn get_enum1<E, C: ink_wrapper_types::Connection<E>>(
        &self,
        conn: &C,
    ) -> Result<Result<Enum1, ink_primitives::LangError>, E> {
        let args = vec![14, 243, 164, 76];
        conn.read(self.account_id, args).await
    }

    pub async fn get_struct2<E, C: ink_wrapper_types::Connection<E>>(
        &self,
        conn: &C,
    ) -> Result<Result<Struct2, ink_primitives::LangError>, E> {
        let args = vec![164, 200, 63, 19];
        conn.read(self.account_id, args).await
    }

    pub async fn get_enum2<E, C: ink_wrapper_types::Connection<E>>(
        &self,
        conn: &C,
    ) -> Result<Result<Enum2, ink_primitives::LangError>, E> {
        let args = vec![231, 221, 248, 25];
        conn.read(self.account_id, args).await
    }

    pub async fn get_newtype1<E, C: ink_wrapper_types::Connection<E>>(
        &self,
        conn: &C,
    ) -> Result<Result<u32, ink_primitives::LangError>, E> {
        let args = vec![8, 68, 100, 9];
        conn.read(self.account_id, args).await
    }

    pub async fn get_bool<E, C: ink_wrapper_types::Connection<E>>(
        &self,
        conn: &C,
    ) -> Result<Result<bool, ink_primitives::LangError>, E> {
        let args = vec![38, 2, 201, 24];
        conn.read(self.account_id, args).await
    }

    pub async fn set_u32<TxInfo, E, C: ink_wrapper_types::SignedConnection<TxInfo, E>>(
        &self,
        conn: &C,
        an_u32: u32,
    ) -> Result<TxInfo, E> {
        let mut args = vec![246, 7, 184, 246];
        an_u32.encode_to(&mut args);
        conn.exec(self.account_id, args).await
    }

    pub async fn set_bool<TxInfo, E, C: ink_wrapper_types::SignedConnection<TxInfo, E>>(
        &self,
        conn: &C,
        a_bool: bool,
    ) -> Result<TxInfo, E> {
        let mut args = vec![33, 77, 141, 9];
        a_bool.encode_to(&mut args);
        conn.exec(self.account_id, args).await
    }

    pub async fn set_struct1<TxInfo, E, C: ink_wrapper_types::SignedConnection<TxInfo, E>>(
        &self,
        conn: &C,
        a_struct1: Struct1,
    ) -> Result<TxInfo, E> {
        let mut args = vec![148, 223, 7, 132];
        a_struct1.encode_to(&mut args);
        conn.exec(self.account_id, args).await
    }

    pub async fn set_enum1<TxInfo, E, C: ink_wrapper_types::SignedConnection<TxInfo, E>>(
        &self,
        conn: &C,
        an_enum1: Enum1,
    ) -> Result<TxInfo, E> {
        let mut args = vec![143, 146, 36, 76];
        an_enum1.encode_to(&mut args);
        conn.exec(self.account_id, args).await
    }

    pub async fn set_struct2<TxInfo, E, C: ink_wrapper_types::SignedConnection<TxInfo, E>>(
        &self,
        conn: &C,
        a_struct2: Struct2,
    ) -> Result<TxInfo, E> {
        let mut args = vec![147, 42, 93, 250];
        a_struct2.encode_to(&mut args);
        conn.exec(self.account_id, args).await
    }

    pub async fn set_enum2<TxInfo, E, C: ink_wrapper_types::SignedConnection<TxInfo, E>>(
        &self,
        conn: &C,
        an_enum2: Enum2,
    ) -> Result<TxInfo, E> {
        let mut args = vec![254, 6, 195, 111];
        an_enum2.encode_to(&mut args);
        conn.exec(self.account_id, args).await
    }

    pub async fn set_newtype1<TxInfo, E, C: ink_wrapper_types::SignedConnection<TxInfo, E>>(
        &self,
        conn: &C,
        a_newtype1: u32,
    ) -> Result<TxInfo, E> {
        let mut args = vec![157, 123, 31, 26];
        a_newtype1.encode_to(&mut args);
        conn.exec(self.account_id, args).await
    }
}
