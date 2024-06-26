use std::borrow::Cow;

use gears::{
    store::database::Database,
    types::{
        address::AccAddress,
        base::coin::Coin,
        store::{gas::errors::GasStoreErrors, kv::Store, range::StoreRange},
    },
};

use crate::account_key;

#[derive(Debug)]
pub struct BalanceIterator<'a, DB>(StoreRange<'a, DB>);

impl<'a, DB: Database> BalanceIterator<'a, DB> {
    pub fn new(store: Store<'a, DB>, addr: &AccAddress) -> BalanceIterator<'a, DB> {
        let store = store.prefix_store(account_key(addr));

        // TODO: WHY https://github.com/cosmos/cosmos-sdk/blob/d3f09c222243bb3da3464969f0366330dcb977a8/store/prefix/store.go#L88-L93
        BalanceIterator(store.into_range(..))
    }
}

impl<'a, DB: Database> Iterator for BalanceIterator<'a, DB> {
    type Item = Result<(Cow<'a, Vec<u8>>, Coin), GasStoreErrors>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(var) = self.0.next() {
            match var {
                Ok((key, value)) => Some(Ok((
                    key,
                    serde_json::from_slice(&value).expect("serde encoding shouldn't fail"),
                ))),
                Err(err) => Some(Err(err)),
            }
        } else {
            None
        }
    }
}
