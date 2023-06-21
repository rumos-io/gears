use std::{collections::HashMap, str::FromStr};

use bytes::Bytes;
use cosmwasm_std::Uint256;
use database::Database;
use ibc_proto::protobuf::Protobuf;
use proto_messages::cosmos::{
    bank::v1beta1::{
        MsgSend, QueryAllBalancesRequest, QueryAllBalancesResponse, QueryBalanceRequest,
        QueryBalanceResponse,
    },
    base::v1beta1::{Coin, SendCoins},
};
use proto_types::{AccAddress, Denom};
use serde::{Deserialize, Serialize};
use tendermint_informal::abci::{Event, EventAttributeIndexExt};

use crate::{
    error::AppError,
    store::{KVStore, MutablePrefixStore, Store},
    types::{Context, QueryContext},
    x::auth::{Auth, Module},
};

use super::Params;

const SUPPLY_KEY: [u8; 1] = [0];
const ADDRESS_BALANCES_STORE_PREFIX: [u8; 1] = [2];

#[derive(Debug, Clone)]
pub struct Bank {}

// TODO: should remove total supply since it can be derived from the balances
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct GenesisState {
    pub balances: Vec<Balance>,
    pub params: Params,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Balance {
    pub address: AccAddress,
    pub coins: Vec<Coin>,
}

impl Bank {
    pub fn init_genesis<T: Database>(ctx: &mut Context<T>, genesis: GenesisState) {
        // TODO:
        // 1. cosmos SDK sorts the balances first
        // 2. Need to confirm that the SDK does not validate list of coins in each balance (validates order, denom etc.)
        // 3. Need to set denom metadata
        Params::set(ctx, genesis.params);

        let bank_store = ctx.get_mutable_kv_store(Store::Bank);

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
            Bank::set_supply(
                ctx,
                Coin {
                    denom: coin.0,
                    amount: coin.1,
                },
            );
        }
    }

    pub fn query_balance<T: Database>(
        ctx: &QueryContext<T>,
        req: QueryBalanceRequest,
    ) -> QueryBalanceResponse {
        let bank_store = ctx.get_kv_store(Store::Bank);
        let prefix = create_denom_balance_prefix(req.address);

        let account_store = bank_store.get_immutable_prefix_store(prefix);
        let bal = account_store.get(req.denom.to_string().as_bytes());

        match bal {
            Some(amount) => QueryBalanceResponse {
                balance: Some(
                    Coin::decode::<Bytes>(amount.to_owned().into())
                        .expect("invalid data in database - possible database corruption"),
                ),
            },
            None => QueryBalanceResponse { balance: None },
        }
    }

    pub fn query_all_balances<T: Database>(
        ctx: &QueryContext<T>,
        req: QueryAllBalancesRequest,
    ) -> QueryAllBalancesResponse {
        let bank_store = ctx.get_kv_store(Store::Bank);
        let prefix = create_denom_balance_prefix(req.address);
        let account_store = bank_store.get_immutable_prefix_store(prefix);

        let mut balances = vec![];

        for (_, coin) in account_store.range(..) {
            let coin: Coin = Coin::decode::<Bytes>(coin.to_owned().into())
                .expect("invalid data in database - possible database corruption");
            balances.push(coin);
        }

        QueryAllBalancesResponse {
            balances,
            pagination: None,
        }
    }

    /// Gets the total supply of every denom
    // TODO: should be paginated
    // TODO: should ignore coins with zero balance
    // TODO: does this method guarantee that coins are sorted?
    pub fn get_paginated_total_supply<T: Database>(ctx: &QueryContext<T>) -> Vec<Coin> {
        let bank_store = ctx.get_kv_store(Store::Bank);
        let supply_store = bank_store.get_immutable_prefix_store(SUPPLY_KEY.into());

        supply_store
            .range(..)
            .map(|raw_coin| {
                let denom = Denom::from_str(&String::from_utf8_lossy(&raw_coin.0))
                    .expect("invalid data in database - possible database corruption");
                let amount = Uint256::from_str(&String::from_utf8_lossy(&raw_coin.1))
                    .expect("invalid data in database - possible database corruption");
                Coin { denom, amount }
            })
            .collect()
    }

    pub fn send_coins_from_account_to_module<T: Database>(
        ctx: &mut Context<T>,
        from_address: AccAddress,
        to_module: Module,
        amount: SendCoins,
    ) -> Result<(), AppError> {
        Auth::check_create_new_module_account(ctx, &to_module);

        let msg = MsgSend {
            from_address,
            to_address: to_module.get_address(),
            amount,
        };

        Bank::send_coins(ctx, msg)
    }

    pub fn send_coins_from_account_to_account<T: Database>(
        ctx: &mut Context<T>,
        msg: MsgSend,
    ) -> Result<(), AppError> {
        Bank::send_coins(ctx, msg.clone())?;

        // Create account if recipient does not exist
        if !Auth::has_account(ctx, &msg.to_address) {
            Auth::create_new_base_account(ctx, &msg.to_address);
        };

        Ok(())
    }

    fn send_coins<T: Database>(ctx: &mut Context<T>, msg: MsgSend) -> Result<(), AppError> {
        // TODO: refactor this to subtract all amounts before adding all amounts

        let bank_store = ctx.get_mutable_kv_store(Store::Bank);
        let mut events = vec![];

        let from_address = msg.from_address;
        let to_address = msg.to_address;

        for send_coin in msg.amount {
            let mut from_account_store =
                Bank::get_address_balances_store(bank_store, &from_address);
            let from_balance = from_account_store
                .get(send_coin.denom.to_string().as_bytes())
                .ok_or(AppError::Send("Insufficient funds".into()))?;

            let mut from_balance: Coin = Coin::decode::<Bytes>(from_balance.to_owned().into())
                .expect("invalid data in database - possible database corruption");

            if from_balance.amount < send_coin.amount {
                return Err(AppError::Send("Insufficient funds".into()));
            }

            from_balance.amount = from_balance.amount - send_coin.amount;

            from_account_store.set(
                send_coin.denom.clone().to_string().into(),
                from_balance.encode_vec(),
            );

            //TODO: if balance == 0 then denom should be removed from store

            let mut to_account_store = Bank::get_address_balances_store(bank_store, &to_address);
            let to_balance = to_account_store.get(send_coin.denom.to_string().as_bytes());

            let mut to_balance: Coin = match to_balance {
                Some(to_balance) => Coin::decode::<Bytes>(to_balance.to_owned().into())
                    .expect("invalid data in database - possible database corruption"),
                None => Coin {
                    denom: send_coin.denom.clone(),
                    amount: Uint256::zero(),
                },
            };

            to_balance.amount = to_balance.amount + send_coin.amount;

            to_account_store.set(send_coin.denom.to_string().into(), to_balance.encode_vec());

            events.push(Event::new(
                "transfer",
                vec![
                    ("recipient", String::from(to_address.clone())).index(),
                    ("sender", String::from(from_address.clone())).index(),
                    ("amount", send_coin.amount.into()).index(),
                ],
            ));
        }

        ctx.append_events(events);

        return Ok(());
    }

    pub fn set_supply<T: Database>(ctx: &mut Context<T>, coin: Coin) {
        // TODO: need to delete coins with zero balance

        let bank_store = ctx.get_mutable_kv_store(Store::Bank);
        let mut supply_store = bank_store.get_mutable_prefix_store(SUPPLY_KEY.into());

        supply_store.set(
            coin.denom.to_string().into(),
            coin.amount.to_string().into(),
        );
    }

    fn get_address_balances_store<'a, T: Database>(
        bank_store: &'a mut KVStore<T>,
        address: &AccAddress,
    ) -> MutablePrefixStore<'a, T> {
        let prefix = create_denom_balance_prefix(address.to_owned());
        bank_store.get_mutable_prefix_store(prefix)
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

#[cfg(test)]
mod tests {

    use std::{str::FromStr, vec};

    use crate::{
        store::MultiStore, types::InitContext, x::bank::_DEFAULT_PARAMS as DEFAULT_PARAMS,
    };
    use database::MemDB;
    use proto_types::Denom;

    use super::*;

    #[test]
    fn create_account_balances_prefix_works() {
        let expected = vec![2, 4, 97, 98, 99, 100];
        let acc_address = AccAddress::try_from(vec![97, 98, 99, 100]).unwrap();
        let res = create_denom_balance_prefix(acc_address);

        assert_eq!(expected, res);
    }

    #[test]
    fn query_balance_works() {
        let db = MemDB::new();
        let mut store = MultiStore::new(db);
        let genesis = GenesisState {
            balances: vec![Balance {
                address: AccAddress::from_bech32("cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux")
                    .unwrap(),
                coins: vec![Coin {
                    denom: "coinA".to_string().try_into().unwrap(),
                    amount: Uint256::from_str("123").unwrap(),
                }],
            }],
            params: DEFAULT_PARAMS,
        };

        let mut ctx = InitContext::new(&mut store, 0, "".into());
        Bank::init_genesis(&mut ctx.as_any(), genesis);

        let ctx = QueryContext::new(&store, 0);

        let req = QueryBalanceRequest {
            address: AccAddress::from_bech32("cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux")
                .unwrap(),
            denom: Denom::try_from(String::from("coinA")).unwrap(),
        };

        let res = Bank::query_balance(&ctx, req);

        let expected_res = QueryBalanceResponse {
            balance: Some(Coin {
                amount: Uint256::from_str("123").unwrap(),
                denom: "coinA".to_string().try_into().unwrap(),
            }),
        };

        assert_eq!(expected_res, res);
    }

    #[test]
    fn query_all_balances_works() {
        let db = MemDB::new();
        let mut store = MultiStore::new(db);
        let genesis = GenesisState {
            balances: vec![Balance {
                address: AccAddress::from_bech32("cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux")
                    .unwrap(),
                coins: vec![Coin {
                    denom: "coinA".to_string().try_into().unwrap(),
                    amount: Uint256::from_str("123").unwrap(),
                }],
            }],
            params: DEFAULT_PARAMS,
        };

        let req = QueryAllBalancesRequest {
            address: AccAddress::from_bech32("cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux")
                .unwrap(),
            pagination: None,
        };

        let mut ctx = InitContext::new(&mut store, 0, "".into());
        Bank::init_genesis(&mut ctx.as_any(), genesis);
        ctx.multi_store.commit(); //TODO: this won't be needed once the KVStore iterator correctly incorporates cached values

        let ctx = QueryContext::new(&store, 0);
        let res = Bank::query_all_balances(&ctx, req);

        let expected_res = QueryAllBalancesResponse {
            balances: vec![Coin {
                denom: "coinA".to_string().try_into().unwrap(),
                amount: Uint256::from_str("123").unwrap(),
            }],
            pagination: None,
        };

        assert_eq!(expected_res, res);
    }

    #[test]
    fn get_paginated_total_supply_works() {
        let db = MemDB::new();
        let mut store = MultiStore::new(db);
        let genesis = GenesisState {
            balances: vec![Balance {
                address: AccAddress::from_bech32("cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux")
                    .unwrap(),
                coins: vec![
                    Coin {
                        denom: "coinA".to_string().try_into().unwrap(),
                        amount: Uint256::from_str("123").unwrap(),
                    },
                    Coin {
                        denom: "coinB".to_string().try_into().unwrap(),
                        amount: Uint256::from_str("321").unwrap(),
                    },
                ],
            }],
            params: DEFAULT_PARAMS,
        };

        let mut ctx = InitContext::new(&mut store, 0, "".into());
        Bank::init_genesis(&mut ctx.as_any(), genesis);
        ctx.multi_store.commit(); //TODO: this won't be needed once the KVStore iterator correctly incorporates cached values

        let ctx = QueryContext::new(&store, 0);
        let res = Bank::get_paginated_total_supply(&ctx);

        let expected_res = vec![
            Coin {
                denom: "coinA".to_string().try_into().unwrap(),
                amount: Uint256::from_str("123").unwrap(),
            },
            Coin {
                denom: "coinB".to_string().try_into().unwrap(),
                amount: Uint256::from_str("321").unwrap(),
            },
        ];

        assert_eq!(expected_res, res);
    }
}
