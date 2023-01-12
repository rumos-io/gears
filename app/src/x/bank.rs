use std::str::FromStr;

use cosmwasm_std::Uint256;
use ibc_proto::cosmos::{
    bank::v1beta1::{
        MsgSend, QueryAllBalancesRequest, QueryAllBalancesResponse, QueryBalanceRequest,
        QueryBalanceResponse,
    },
    base::v1beta1::Coin,
};

use crate::{
    baseapp::BANK_STORE_PREFIX,
    error::AppError,
    store::MutableSubStore,
    types::{AccAddress, Context},
};

const ADDRESS_BALANCES_STORE_PREFIX: [u8; 1] = [2];

#[derive(Debug, Clone)]
pub struct Bank {}

pub struct GenesisState {
    pub balances: Vec<Balance>,
}

pub struct Balance {
    pub address: AccAddress,
    pub coins: Vec<Coin>,
}

impl Bank {
    pub fn init_genesis(ctx: &mut Context, genesis: GenesisState) {
        let mut bank_store = ctx
            .get_mutable_store()
            .get_mutable_sub_store(BANK_STORE_PREFIX.into());

        for balance in genesis.balances {
            let prefix = create_denom_balance_prefix(balance.address);
            let mut denom_balance_store = bank_store.get_sub_store(prefix);

            for coin in balance.coins {
                denom_balance_store.set(
                    coin.denom.as_bytes().to_vec(),
                    coin.amount.to_string().into(),
                );
            }
        }
    }

    pub fn query_balance(
        ctx: &Context,
        req: QueryBalanceRequest,
    ) -> Result<QueryBalanceResponse, AppError> {
        let address = AccAddress::from_bech32(&req.address)?;

        let bank_store = ctx
            .get_multi_store()
            .get_immutable_sub_store(BANK_STORE_PREFIX.into());
        let prefix = create_denom_balance_prefix(address);

        let account_store = bank_store.get_sub_store(prefix);
        let bal = account_store.get(req.denom.as_bytes());

        match bal {
            Some(amount) => Ok(QueryBalanceResponse {
                balance: Some(Coin {
                    denom: req.denom,
                    amount: Uint256::from_str(
                        &String::from_utf8(amount.to_owned()).expect("Should be valid Uint256"),
                    )
                    .expect("Should be valid utf8"),
                }),
            }),
            None => Ok(QueryBalanceResponse { balance: None }),
        }
    }

    pub fn query_all_balances(
        ctx: &Context,
        req: QueryAllBalancesRequest,
    ) -> Result<QueryAllBalancesResponse, AppError> {
        let address = AccAddress::from_bech32(&req.address)?;

        let bank_store = ctx
            .get_multi_store()
            .get_immutable_sub_store(BANK_STORE_PREFIX.into());
        let prefix = create_denom_balance_prefix(address);
        let account_store = bank_store.get_sub_store(prefix);

        let mut balances = vec![];

        for (denom, amount) in account_store {
            let denom = String::from_utf8(denom).expect("Should be valid utf8");
            let amount =
                Uint256::from_str(&String::from_utf8(amount).expect("Should be valid Uint256"))
                    .expect("Should be valid utf8");

            let coin = Coin { denom, amount };
            balances.push(coin);
        }

        return Ok(QueryAllBalancesResponse {
            balances,
            pagination: None,
        });
    }

    pub fn send_coins(ctx: &mut Context, msg: MsgSend) -> Result<(), AppError> {
        let mut bank_store = ctx
            .get_mutable_store()
            .get_mutable_sub_store(BANK_STORE_PREFIX.into());

        let from_address = AccAddress::from_bech32(&msg.from_address)?;

        let to_address = AccAddress::from_bech32(&msg.to_address)?;

        for send_coin in msg.amount {
            let mut from_account_store =
                Bank::get_address_balances_store(&mut bank_store, &from_address);
            let from_balance = from_account_store
                .get(send_coin.denom.as_bytes())
                .ok_or(AppError::Send("Insufficient funds".into()))?;

            let from_balance = Uint256::from_str(
                &String::from_utf8(from_balance.to_owned()).expect("Should be valid Uint256"),
            )
            .expect("Should be valid utf8");

            if from_balance < send_coin.amount {
                return Err(AppError::Send("Insufficient funds".into()));
            }

            from_account_store.set(
                send_coin.denom.clone().into(),
                (from_balance - send_coin.amount).to_string().into(),
            );

            let mut to_account_store =
                Bank::get_address_balances_store(&mut bank_store, &to_address);
            let to_balance = to_account_store.get(send_coin.denom.as_bytes());
            let to_balance = match to_balance {
                Some(to_balance) => Uint256::from_str(
                    &String::from_utf8(to_balance.to_owned()).expect("Should be valid Uint256"),
                )
                .expect("Should be valid utf8"),
                None => Uint256::zero(),
            };

            to_account_store.set(
                send_coin.denom.into(),
                (to_balance + send_coin.amount).to_string().into(),
            );
        }

        //TODO:
        // Create account if recipient does not exist

        return Ok(());
    }

    fn get_address_balances_store<'a>(
        bank_store: &'a mut MutableSubStore,
        address: &AccAddress,
    ) -> MutableSubStore<'a> {
        let prefix = create_denom_balance_prefix(address.to_owned());
        bank_store.get_sub_store(prefix)
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

    use crate::store::MultiStore;

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
        let store = MultiStore::new();
        let genesis = GenesisState {
            balances: vec![Balance {
                address: AccAddress::from_bech32("cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux")
                    .unwrap(),
                coins: vec![Coin {
                    denom: "coinA".into(),
                    amount: Uint256::from_str("123").unwrap(),
                }],
            }],
        };

        let mut ctx = Context::new(store);
        Bank::init_genesis(&mut ctx, genesis);

        let req = QueryBalanceRequest {
            address: "cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux".to_string(),
            denom: "coinA".to_string(),
        };

        let res = Bank::query_balance(&ctx, req).unwrap();

        let expected_res = QueryBalanceResponse {
            balance: Some(Coin {
                amount: Uint256::from_str("123").unwrap(),
                denom: "coinA".to_string(),
            }),
        };

        assert_eq!(expected_res, res);
    }

    #[test]
    fn query_all_balances_works() {
        let store = MultiStore::new();
        let genesis = GenesisState {
            balances: vec![Balance {
                address: AccAddress::from_bech32("cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux")
                    .unwrap(),
                coins: vec![Coin {
                    denom: "coinA".into(),
                    amount: Uint256::from_str("123").unwrap(),
                }],
            }],
        };

        let req = QueryAllBalancesRequest {
            address: "cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux".to_string(),
            pagination: None,
        };

        let mut ctx = Context::new(store);
        Bank::init_genesis(&mut ctx, genesis);

        let res = Bank::query_all_balances(&ctx, req).unwrap();

        let expected_res = QueryAllBalancesResponse {
            balances: vec![Coin {
                denom: "coinA".to_string(),
                amount: Uint256::from_str("123").unwrap(),
            }],
            pagination: None,
        };

        assert_eq!(expected_res, res);
    }
}
