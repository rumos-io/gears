use database::Database;

use crate::{
    params::{parsed::Params, ParamKind, ParamsDeserialize},
    types::store::{errors::StoreErrors, prefix::PrefixStore},
};

pub struct GasParamsSpace<'a, DB> {
    pub(super) inner: PrefixStore<'a, DB>,
}

impl<DB: Database> GasParamsSpace<'_, DB> {
    /// Return whole serialized structure.
    pub fn params<T: ParamsDeserialize>(&self) -> Result<Option<T>, StoreErrors> {
        let keys = T::keys();
        let mut params_fields = Vec::with_capacity(keys.len());

        for (key, _) in keys {
            if let Some(value) = self.inner.get(key)? {
                params_fields.push((key, value));
            } else {
                return Ok(None);
            }
        }

        Ok(Some(T::from_raw(params_fields.into_iter().collect())))
    }

    /// Return only field from structure.
    pub fn params_field(&self, path: &str, kind: ParamKind) -> Result<Option<Params>, StoreErrors> {
        if let Some(value) = self.inner.get(path)? {
            Ok(Some(kind.parse_param(value)))
        } else {
            Ok(None)
        }
    }
}
