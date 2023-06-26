use std::collections::HashMap;

use cosmwasm_std::Uint256;
use database::Database;

use gears::{error::AppError, types::context_v2::Context};
use ibc_proto::protobuf::Protobuf;
use params_module::ParamsSubspaceKey;
use proto_messages::cosmos::{bank::v1beta1::MsgSend, base::v1beta1::Coin};
use proto_types::{AccAddress, Denom};
use store::StoreKey;

use crate::{BankParamsKeeper, GenesisState};

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

    pub fn init_genesis<DB: Database>(&self, ctx: &mut Context<DB, SK>, genesis: GenesisState) {
        // TODO:
        // 1. cosmos SDK sorts the balances first
        // 2. Need to confirm that the SDK does not validate list of coins in each balance (validates order, denom etc.)
        // 3. Need to set denom metadata
        self.bank_params_keeper.set(ctx, genesis.params);

        let bank_store = ctx.get_mutable_kv_store(&self.store_key);

        let mut total_supply: HashMap<Denom, Uint256> = HashMap::new();
        for balance in genesis.balances {
            let prefix = create_denom_balance_prefix(balance.address);
            let mut denom_balance_store = bank_store.get_mutable_prefix_store(prefix);

            for coin in balance.coins {
                denom_balance_store.set(coin.denom.to_string().into_bytes(), coin.encode_vec());
                let zero = Uint256::zero();
                let current_balance = total_supply.get(&coin.denom).unwrap_or(&zero);
                total_supply.insert(coin.denom, coin.amount + current_balance);
            }
        }

        // TODO: does the SDK sort these?
        for coin in total_supply {
            self.set_supply(
                ctx,
                Coin {
                    denom: coin.0,
                    amount: coin.1,
                },
            );
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

fn create_denom_balance_prefix(addr: AccAddress) -> Vec<u8> {
    let addr_len = addr.len();
    let mut addr: Vec<u8> = addr.into();
    let mut prefix = Vec::new();

    prefix.extend(ADDRESS_BALANCES_STORE_PREFIX);
    prefix.push(addr_len);
    prefix.append(&mut addr);

    return prefix;
}
