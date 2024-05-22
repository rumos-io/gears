use database::Database;
use store_crate::{types::prefix::mutable::MutablePrefixStore, WritePrefixStore};

use super::{space::ParamsSpace, ParamString, Params, ParamsDeserialize};

pub struct ParamsSpaceMut<'a, DB> {
    pub(super) inner: MutablePrefixStore<'a, DB>,
}

impl<DB> ParamsSpaceMut<'_, DB> {
    pub fn to_immutable(&self) -> ParamsSpace<'_, DB> {
        ParamsSpace {
            inner: self.inner.to_immutable(),
        }
    }
}

impl<DB: Database> ParamsSpaceMut<'_, DB> {
    pub fn params<T: ParamsDeserialize>(&self) -> Option<T> {
        self.to_immutable().params()
    }

    /// Return only field from structure.
    pub fn params_field<F: From<ParamString>>(&self, path: &str) -> Option<F> {
        self.to_immutable().params_field::<F>(path)
    }

    pub fn params_set<T: Params>(&mut self, params: &T) {
        let params = params.serialize();

        for (key, value) in params {
            self.inner.set(key.as_bytes().into_iter().cloned(), value)
        }
    }
}
