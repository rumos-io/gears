use super::Database;
use gears::{
    store::database::{ext::UnwrapCorrupt, prefix::PrefixDB},
    types::{
        address::ValAddress,
        store::{gas::errors::GasStoreErrors, kv::Store, range::StoreRange},
    },
};
use std::{borrow::Cow, ops::Bound};

#[derive(Debug)]
struct StoreIterator<'a, DB>(StoreRange<'a, PrefixDB<DB>>);

impl<'a, DB: Database> StoreIterator<'a, DB> {
    pub fn new(
        store: Store<'a, PrefixDB<DB>>,
        start: Vec<u8>,
        end: Vec<u8>,
    ) -> StoreIterator<'a, DB> {
        Self(store.into_range((
            Bound::Included(start.clone()),
            Bound::Excluded([start, end].concat().to_vec()),
        )))
    }
}

impl<'a, DB: Database> Iterator for StoreIterator<'a, DB> {
    type Item = Result<(Cow<'a, Vec<u8>>, Cow<'a, Vec<u8>>), GasStoreErrors>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(var) = self.0.next() {
            match var {
                Ok((key, value)) => Some(Ok((key, value))),
                Err(err) => Some(Err(err)),
            }
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct UnbondingValidatorsIterator<'a, DB>(StoreIterator<'a, DB>);
impl<'a, DB: Database> UnbondingValidatorsIterator<'a, DB> {
    pub fn new(
        store: Store<'a, PrefixDB<DB>>,
        start: Vec<u8>,
        mut end: Vec<u8>,
    ) -> UnbondingValidatorsIterator<'a, DB> {
        end.push(0);
        UnbondingValidatorsIterator(StoreIterator::new(store, start, end))
    }
}

impl<'a, DB: Database> Iterator for UnbondingValidatorsIterator<'a, DB> {
    type Item = Result<(Vec<u8>, Vec<ValAddress>), GasStoreErrors>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0
            .next()
            .map(|r| r.map(|(k, v)| (k.to_vec(), serde_json::from_slice(&v).unwrap_or_corrupt())))
    }
}
