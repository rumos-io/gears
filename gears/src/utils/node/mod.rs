mod helpers;
mod mock;
mod presets;

use address::AccAddress;
pub use helpers::*;
use keyring::key::pair::KeyPair;
pub use mock::*;
pub use presets::*;

use crate::{application::ApplicationInfo, crypto::keys::ReadAccAddress};

pub struct User {
    pub key_pair: KeyPair,
    pub account_number: u64,
}

impl User {
    pub fn address(&self) -> AccAddress {
        self.key_pair.get_address()
    }
}

#[derive(Debug, Clone)]
pub struct MockApplication;

impl ApplicationInfo for MockApplication {}
