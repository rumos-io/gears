use gears::{
    store::database::Database,
    types::store::{gas::errors::GasStoreErrors, kv::Store},
    x::types::validator::BondStatus,
};

use crate::{
    consts::{error::SERDE_ENCODING_DOMAIN_TYPE, keeper::VALIDATORS_BY_POWER_INDEX_KEY},
    Validator,
};

#[derive(Debug)]
pub struct BoundedValidatorsIterator {
    inner: Vec<Result<Validator, GasStoreErrors>>, // TODO: we missing double ended iterator implementation currently so instead load all validators and read from end...
    position: usize,
    max_validator: u32,
}

impl BoundedValidatorsIterator {
    pub fn new<DB: Database>(
        store: Store<'_, DB>,
        max_validator: u32,
    ) -> BoundedValidatorsIterator {
        BoundedValidatorsIterator {
            inner: store
                .prefix_store(VALIDATORS_BY_POWER_INDEX_KEY)
                .into_range(..)
                .map(|this| {
                    this.map(|(_, value)| {
                        serde_json::from_slice(&value).expect(SERDE_ENCODING_DOMAIN_TYPE)
                    })
                })
                .collect(),
            position: 0,
            max_validator,
        }
    }
}

impl Iterator for BoundedValidatorsIterator {
    type Item = Result<Validator, GasStoreErrors>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.position >= self.max_validator as usize {
            return None; // TODO:NOW Option or Error?
        }

        if let Some(var) = self.inner.pop() {
            match var {
                Ok(validator) => {
                    if validator.status == BondStatus::Bonded {
                        self.position += 1;
                        Some(Ok(validator))
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
