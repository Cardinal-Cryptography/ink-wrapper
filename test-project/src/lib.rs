#[cfg(all(test, feature = "aleph_client"))]
mod aleph_client;

#[cfg(all(test, feature = "drink"))]
mod drink;

mod psp22_contract;
mod test_contract;
