use database::Database;
use store_crate::{types::prefix::immutable::ImmutablePrefixStore, ReadPrefixStore};

use super::{parsed::Params, ParamKind, ParamsDeserialize};

pub struct ParamsSpace<'a, DB> {
    pub(super) inner: ImmutablePrefixStore<'a, DB>,
}

impl<DB: Database> ParamsSpace<'_, DB> {
    /// Return whole serialized structure.
    pub fn params<T: ParamsDeserialize>(&self) -> Option<T> {
        let keys = T::keys();
        let mut params_fields = Vec::with_capacity(keys.len());

        for (key, _) in keys {
            params_fields.push((key, self.inner.get(key).ok()?));
        }

        Some(T::from_raw(params_fields.into_iter().collect()))
    }

    /// Return only field from structure.
    pub fn params_field(&self, path: &str, kind: ParamKind) -> Option<Params> {
        Some(kind.parse_param(self.inner.get(path).ok()?))
    }
}
