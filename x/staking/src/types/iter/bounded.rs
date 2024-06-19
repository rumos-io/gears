use std::borrow::Cow;

use gears::{
    store::database::Database,
    types::store::{gas::errors::GasStoreErrors, kv::Store, range::StoreRange},
};

use crate::{
    consts::{error::SERDE_ENCODING_DOMAIN_TYPE, keeper::VALIDATORS_BY_POWER_INDEX_KEY},
    BondStatus, Validator,
};

#[derive(Debug)]
pub struct BoundedValidatorsIterator<'a, DB> {
    inner: StoreRange<'a, DB>,
    position: usize,
    max_validator: usize,
}

impl<'a, DB: Database> BoundedValidatorsIterator<'a, DB> {
    pub fn new(store: Store<'a, DB>, max_validator: usize) -> BoundedValidatorsIterator<'a, DB> {
        BoundedValidatorsIterator {
            inner: store
                .prefix_store(VALIDATORS_BY_POWER_INDEX_KEY)
                .into_range(..),
            position: 0,
            max_validator,
        }
    }
}

impl<'a, DB: Database> Iterator for BoundedValidatorsIterator<'a, DB> {
    type Item = Result<(Cow<'a, Vec<u8>>, Validator), GasStoreErrors>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.position >= self.max_validator {
            return None; // TODO: Option or Error?
        }

        if let Some(var) = self.inner.next() {
            match var {
                Ok((key, value)) => {
                    let validator: Validator =
                        serde_json::from_slice(&value).expect(SERDE_ENCODING_DOMAIN_TYPE);

                    if validator.status == BondStatus::Bonded {
                        self.position += 1;
                        Some(Ok((key, validator)))
                    } else {
                        self.next()
                    }
                }
                Err(err) => Some(Err(err)),
            }
        } else {
            None
        }
    }
}
