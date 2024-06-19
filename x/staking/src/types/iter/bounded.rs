use gears::{
    store::database::Database,
    types::store::{gas::errors::GasStoreErrors, kv::Store},
};

use crate::{
    consts::{error::SERDE_ENCODING_DOMAIN_TYPE, keeper::VALIDATORS_BY_POWER_INDEX_KEY},
    BondStatus, Validator,
};

#[derive(Debug)]
pub struct BoundedValidatorsIterator {
    inner: Vec<Result<Validator, GasStoreErrors>>,
    position: usize,
    max_validator: usize,
}

impl BoundedValidatorsIterator {
    pub fn new<DB: Database>(
        store: Store<'_, DB>,
        max_validator: usize,
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
        if self.position >= self.max_validator {
            return None; // TODO: Option or Error?
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
