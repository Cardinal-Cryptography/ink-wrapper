use ink_primitives::AccountId;

/// A convenience trait for converting different wrappers around an `[u8; 32]` into an
/// `ink_primitives::AccountId`. Implemented for any `AsRef<[u8; 32]>` by default.
pub trait ToAccountId {
    fn to_account_id(&self) -> AccountId;
}

impl<T: AsRef<[u8; 32]>> ToAccountId for T {
    fn to_account_id(&self) -> AccountId {
        (*self.as_ref()).into()
    }
}
