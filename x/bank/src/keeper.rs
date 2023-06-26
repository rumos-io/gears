use database::Database;

use gears::{error::AppError, types::context_v2::Context};
use params_module::ParamsSubspaceKey;
use proto_messages::cosmos::{bank::v1beta1::MsgSend, base::v1beta1::Coin};
use store::StoreKey;

use crate::BankParamsKeeper;

const SUPPLY_KEY: [u8; 1] = [0];
const ADDRESS_BALANCES_STORE_PREFIX: [u8; 1] = [2];

#[derive(Debug, Clone)]
pub struct Keeper<SK: StoreKey, PSK: ParamsSubspaceKey> {
    store_key: SK,
    bank_params_keeper: BankParamsKeeper<SK, PSK>,
}

impl<SK: StoreKey, PSK: ParamsSubspaceKey> gears::baseapp::ante_v2::BankKeeper for Keeper<SK, PSK> {}

impl<SK: StoreKey, PSK: ParamsSubspaceKey> Keeper<SK, PSK> {
    pub fn new(
        store_key: SK,
        params_keeper: params_module::Keeper<SK, PSK>,
        params_subspace_key: PSK,
    ) -> Self {
        let bank_params_keeper = BankParamsKeeper {
            params_keeper,
            params_subspace_key,
        };
        Keeper {
            store_key,
            bank_params_keeper,
        }
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
