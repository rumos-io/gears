use std::borrow::Cow;

use database::Database;

use crate::types::store::gas::errors::GasStoreErrors;

use super::{GasRange, RangeBackend};

#[derive(Debug)]
pub struct RangeIter<'a, DB> {
    range: GasRange<'a, DB>,
    err: Option<GasStoreErrors>,
}

impl<DB> RangeIter<'_, DB> {
    pub fn error(&self) -> Option<&GasStoreErrors> {
        self.err.as_ref()
    }
}

impl<'a, DB> From<GasRange<'a, DB>> for RangeIter<'a, DB> {
    fn from(value: GasRange<'a, DB>) -> Self {
        Self {
            range: value,
            err: None,
        }
    }
}

impl<'a, DB: Database> Iterator for RangeIter<'a, DB> {
    type Item = (Cow<'a, Vec<u8>>, Cow<'a, Vec<u8>>);

    fn next(&mut self) -> Option<Self::Item> {
        let next = match &mut self.range.inner {
            RangeBackend::Kv(var) => var.next(),
            RangeBackend::Prefix(var) => var.next(),
        };

        let err = self.range.guard.range(
            next.as_ref()
                .map(|(key, val)| (key.len(), val.len(), &***key)),
        );

        if let Err(err) = err {
            self.err = Some(err);

            None
        } else {
            next
        }
    }
}
