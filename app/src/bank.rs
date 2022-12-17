use std::str::FromStr;

use cosmwasm_std::Uint256;
use ibc_proto::cosmos::{
    bank::v1beta1::{
        QueryAllBalancesRequest, QueryAllBalancesResponse, QueryBalanceRequest,
        QueryBalanceResponse,
    },
    base::v1beta1::Coin,
};

use crate::{error::AppError, store::Store};

const BALANCES_PREFIX: [u8; 1] = [2];

#[derive(Debug, Clone)]
pub struct Bank {
    store: Store,
}

pub struct GenesisState {
    pub balances: Vec<Balance>,
}

pub struct Balance {
    pub address: Address,
    pub coins: Vec<Coin>,
}

/// Ensures that the contained address length is less than 256
pub struct Address(String);

impl Address {
    pub fn new(addrs: String) -> Result<Self, AppError> {
        if addrs.len() > 255 {
            return Err(AppError::InvalidAddress);
        }
        return Ok(Address(addrs));
    }
}

impl Bank {
    pub fn new(store: Store, genesis: GenesisState) -> Self {
        let bank = Bank { store };

        for balance in genesis.balances {
            let prefix = create_account_balances_prefix(balance.address.0.into())
                .expect("Address guarantees that addrs length < 256");
            let denom_store = bank.store.get_sub_store(prefix);

            for coin in balance.coins {
                denom_store.set(
                    coin.denom.as_bytes().to_vec(),
                    coin.amount.to_string().into(),
                );
            }
        }

        return bank;
    }

    pub fn query_balance(&self, req: QueryBalanceRequest) -> QueryBalanceResponse {
        let prefix = create_account_balances_prefix(req.address.into());
        let prefix = match prefix {
            Ok(prefix) => prefix,
            Err(_) => return QueryBalanceResponse { balance: None },
        };

        let denom_store = self.store.get_sub_store(prefix);
        let bal = denom_store.get(req.denom.as_bytes());

        match bal {
            Some(amount) => QueryBalanceResponse {
                balance: Some(Coin {
                    denom: req.denom,
                    amount: Uint256::from_str(
                        &String::from_utf8(amount).expect("Should be valid Uint256"),
                    )
                    .expect("Should be valid utf8"),
                }),
            },
            None => QueryBalanceResponse { balance: None },
        }
    }

    pub fn query_all_balances(&self, req: QueryAllBalancesRequest) -> QueryAllBalancesResponse {
        let prefix = create_account_balances_prefix(req.address.into());
        let prefix = match prefix {
            Ok(prefix) => prefix,
            Err(_) => {
                return QueryAllBalancesResponse {
                    balances: vec![],
                    pagination: None,
                }
            }
        };

        let denom_store = self.store.get_sub_store(prefix);

        let mut balances = vec![];

        for (denom, amount) in denom_store {
            let denom = String::from_utf8(denom).expect("Should be valid utf8");
            let amount =
                Uint256::from_str(&String::from_utf8(amount).expect("Should be valid Uint256"))
                    .expect("Should be valid utf8");

            let coin = Coin { denom, amount };
            balances.push(coin);
        }

        return QueryAllBalancesResponse {
            balances,
            pagination: None,
        };
    }
}

fn create_account_balances_prefix(mut addr: Vec<u8>) -> Result<Vec<u8>, AppError> {
    let mut prefix = Vec::new();

    prefix.extend(BALANCES_PREFIX);
    let addr_len: u8 = addr.len().try_into()?;
    prefix.push(addr_len);
    prefix.append(&mut addr);

    return Ok(prefix);
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn create_account_balances_prefix_works() {
        let expected = vec![2, 4, 97, 98, 99, 100];
        let res = create_account_balances_prefix("abcd".into()).unwrap();

        assert_eq!(expected, res);
    }

    #[test]
    fn query_balance_works() {
        let store = Store::new();
        let key = vec![2, 4, 97, 98, 99, 100, 99, 111, 105, 110, 65];
        let value = "123".into();
        store.set(key, value);
        let genesis = GenesisState { balances: vec![] };

        let bank = Bank::new(store, genesis);

        let req = QueryBalanceRequest {
            address: "abcd".to_string(),
            denom: "coinA".to_string(),
        };

        let res = bank.query_balance(req);

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
        let store = Store::new();
        let key = vec![2, 4, 97, 98, 99, 100, 99, 111, 105, 110, 65];
        let value = "123".into();
        store.set(key, value);
        let genesis = GenesisState { balances: vec![] };

        let bank = Bank::new(store, genesis);

        let req = QueryAllBalancesRequest {
            address: "abcd".to_string(),
            pagination: None,
        };

        let res = bank.query_all_balances(req);

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
