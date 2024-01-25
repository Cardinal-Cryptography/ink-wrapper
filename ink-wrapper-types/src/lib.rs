#[cfg(feature = "drink")]
pub use crate::drink::*;
#[cfg(feature = "drink")]
mod drink;

mod calls;

pub mod util;

pub use calls::*;
use ink_primitives::AccountId;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// Represents a raw event emitted by a contract.
pub struct ContractEvent {
    /// The account id of the contract that emitted the event.
    pub account_id: AccountId,
    /// The unparsed data of the event.
    pub data: Vec<u8>,
}

/// Represents a collection of events emitted by contracts in a single transaction.
pub struct ContractEvents {
    pub events: Vec<ContractEvent>,
}

/// A trait that allows to decode events emitted by a specific contract.
pub trait EventSource: Copy + Into<AccountId> {
    /// The type to decode the emitted events into.
    type Event: scale::Decode;
}

impl ContractEvents {
    /// Returns the events emitted by a specific contract.
    ///
    /// Note that this method returns a `Vec<Result<_>>`. An error indicates that a particular event could not be
    /// decoded even though it was emitted byt the particular contract. This can happen if the metadata used to generate
    /// the contract wrapper is out of date. If you're sure that's not the case, then it might be a bug.
    pub fn for_contract<C: EventSource>(&self, contract: C) -> Vec<Result<C::Event, scale::Error>> {
        ContractEvents::from_iter(&self.events, contract)
    }

    pub fn from_iter<'a, I: IntoIterator<Item = &'a ContractEvent>, C: EventSource>(
        events: I,
        contract: C,
    ) -> Vec<Result<C::Event, scale::Error>> {
        use scale::Decode as _;

        events
            .into_iter()
            .filter(|e| e.account_id == contract.into())
            .map(|e| C::Event::decode(&mut e.data.as_slice()))
            .collect()
    }
}

/// A wrapper around `ink_primitives::LangError` that implements `std::error::Error`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
pub struct InkLangError(ink_primitives::LangError);

impl From<ink_primitives::LangError> for InkLangError {
    fn from(e: ink_primitives::LangError) -> Self {
        Self(e)
    }
}

impl std::fmt::Display for InkLangError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "InkLangError({:?})", self.0)
    }
}

impl std::error::Error for InkLangError {}
