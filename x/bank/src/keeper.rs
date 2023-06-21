use database::Database;

use gears::{error::AppError, types::context_v2::Context};
use proto_messages::cosmos::{bank::v1beta1::MsgSend, base::v1beta1::Coin};
use store::StoreKey;

const SUPPLY_KEY: [u8; 1] = [0];
const ADDRESS_BALANCES_STORE_PREFIX: [u8; 1] = [2];

#[derive(Debug, Clone)]
pub struct Keeper<SK: StoreKey> {
    store_key: SK,
}

impl<SK: StoreKey> gears::baseapp::ante_v2::BankKeeper for Keeper<SK> {}

impl<SK: StoreKey> Keeper<SK> {
    pub fn new(store_key: SK) -> Self {
        Keeper { store_key }
    }

    pub fn set_supply<DB: Database>(&self, ctx: &mut Context<DB, SK>, coin: Coin) {
        // TODO: need to delete coins with zero balance

        let bank_store = ctx.get_mutable_kv_store(&self.store_key);
        let mut supply_store = bank_store.get_mutable_prefix_store(SUPPLY_KEY.into());

        supply_store.set(
            coin.denom.to_string().into(),
            coin.amount.to_string().into(),
        );
    }

    pub fn send_coins<DB: Database>(
        &self,
        ctx: &mut Context<DB, SK>,
        msg: &MsgSend,
    ) -> Result<(), AppError> {
        Ok(())
    }
}
