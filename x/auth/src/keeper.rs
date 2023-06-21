use database::Database;

use gears::types::context_v2::Context;
use proto_messages::cosmos::base::v1beta1::Coin;
use store::StoreKey;

const SUPPLY_KEY: [u8; 1] = [0];
const ADDRESS_BALANCES_STORE_PREFIX: [u8; 1] = [2];

#[derive(Debug, Clone)]
pub struct AuthKeeper<SK: StoreKey> {
    store_key: SK,
}

impl<SK: StoreKey> gears::baseapp::ante_v2::AuthKeeper for AuthKeeper<SK> {
    fn get_auth_params<DB: Database>(
        &self,
        ctx: &gears::types::Context<DB>,
    ) -> gears::x::auth::Params {
        todo!()
    }
}

impl<SK: StoreKey> AuthKeeper<SK> {
    pub fn new(store_key: SK) -> Self {
        AuthKeeper { store_key }
    }
}
