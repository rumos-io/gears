use database::Database;
use serde::{de::DeserializeOwned, Serialize};
use store_crate::{types::prefix::mutable::MutablePrefixStore, WritePrefixStore};

use super::{errors::ParamsError, space::ParamsSpace, ParamsPath};

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
    pub fn params<T: DeserializeOwned, PP: ParamsPath>(&self, path: &PP) -> Result<T, ParamsError> {
        self.to_immutable().params(path)
    }

    pub fn params_field<T: DeserializeOwned, F, PP: ParamsPath, L: FnOnce(T) -> Option<F>>(
        &self,
        path: &PP,
        ex: L,
    ) -> Result<F, ParamsError> {
        self.to_immutable().params_field(path, ex)
    }

    pub fn params_set<T: Serialize, PP: ParamsPath>(
        &mut self,
        path: &PP,
        params: &T,
    ) -> Result<(), ParamsError> {
        let params_json = serde_json::to_string(params)?;

        self.inner.set(
            path.key().as_bytes().into_iter().cloned(),
            params_json.into_bytes(),
        );

        Ok(())
    }
}
