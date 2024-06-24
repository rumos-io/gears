use std::borrow::Cow;

use gears::{
    store::database::Database,
    types::{
        address::AccAddress,
        store::{gas::errors::GasStoreErrors, kv::Store, range::StoreRange},
    },
};

use crate::{consts::{error::SERDE_ENCODING_DOMAIN_TYPE, keeper::DELEGATION_KEY}, Delegation};

#[derive(Debug)]
pub struct DelegationIterator<'a, DB>(StoreRange<'a, DB>);

impl<'a, DB: Database> DelegationIterator<'a, DB> {
    pub fn new(store: Store<'a, DB>, address: &AccAddress) -> DelegationIterator<'a, DB> {
        let prefix = store.prefix_store([DELEGATION_KEY.as_slice(), address.as_ref()].concat());

        let range = prefix.into_range(..);

        DelegationIterator(range)
    }
}

impl<'a, DB: Database> Iterator for DelegationIterator<'a, DB> {
    type Item = Result<(Cow<'a, Vec<u8>>, Delegation), GasStoreErrors>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(var) = self.0.next() {
            match var {
                Ok((key, value)) => Some(Ok((
                    key,
                    serde_json::from_slice(&value).expect(SERDE_ENCODING_DOMAIN_TYPE),
                ))),
                Err(err) => Some(Err(err)),
            }
        } else {
            None
        }
    }
}
