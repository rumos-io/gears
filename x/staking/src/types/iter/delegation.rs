use crate::{consts::keeper::DELEGATION_KEY, Delegation};
use gears::{
    core::Protobuf,
    extensions::corruption::UnwrapCorrupt,
    store::database::Database,
    types::{
        address::AccAddress,
        store::{gas::errors::GasStoreErrors, kv::Store, range::StoreRange},
    },
};
use std::{borrow::Cow, ops::Bound};

#[derive(Debug)]
pub struct DelegationIterator<'a, DB>(
    StoreRange<'a, DB, Vec<u8>, (Bound<Vec<u8>>, Bound<Vec<u8>>)>,
);

impl<'a, DB: Database> DelegationIterator<'a, DB> {
    pub fn new(store: Store<'a, DB>, address: &AccAddress) -> DelegationIterator<'a, DB> {
        let prefix =
            store.prefix_store([DELEGATION_KEY.as_slice(), &address.prefix_len_bytes()].concat());

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
                    Delegation::decode_vec(&value).unwrap_or_corrupt(),
                ))),
                Err(err) => Some(Err(err)),
            }
        } else {
            None
        }
    }
}
