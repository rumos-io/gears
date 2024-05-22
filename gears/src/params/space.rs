use std::collections::HashMap;

use database::Database;
use store_crate::{types::prefix::immutable::ImmutablePrefixStore, ReadPrefixStore};

use super::{parse_primitive, ParamString, ParamsDeserialize};

pub struct ParamsSpace<'a, DB> {
    pub(super) inner: ImmutablePrefixStore<'a, DB>,
}

impl<DB: Database> ParamsSpace<'_, DB> {
    /// Return whole serialized structure.
    ///
    /// It's recommended to use `Self::params_field` 'cause it requires less writing parsing code from you
    pub fn params<T: ParamsDeserialize>(&self) -> Option<T> {
        let keys = T::keys();
        let mut params_fields = HashMap::with_capacity(keys.len());

        for key in keys {
            params_fields.insert(key, self.inner.get(key)?.into());
        }

        Some(T::from_raw(params_fields))
    }

    /// Return only field from structure.
    pub fn params_field<F: From<ParamString>>(&self, path: &str) -> Option<F> {
        let param_string = parse_primitive(self.inner.get(path).map(|this| this.into()))?;

        Some(F::from(param_string))
    }
}
