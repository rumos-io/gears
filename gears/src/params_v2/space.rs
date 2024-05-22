use database::Database;
use serde::de::DeserializeOwned;
use store_crate::{types::prefix::immutable::ImmutablePrefixStore, ReadPrefixStore};

use super::{errors::ParamsError, ParamsPath};

pub struct ParamsSpace<'a, DB> {
    pub(super) inner: ImmutablePrefixStore<'a, DB>,
}

impl<DB: Database> ParamsSpace<'_, DB> {
    /// Return whole serialized structure.
    pub fn params<T: DeserializeOwned, PP: ParamsPath>(&self, path: &PP) -> Result<T, ParamsError> {
        let params_json = self.inner.get(path.key()).ok_or(ParamsError::NotFound)?;
        let params = serde_json::from_slice::<T>(&params_json)?;

        Ok(params)
    }

    /// Return only field from structure. Make sense to use if structure have all fields optional.
    ///
    /// # Example
    ///
    /// ```
    /// struct Params // This structure should be able to deserialize always
    /// {
    ///     field_a : Option<i32>,
    ///     field_b : Option<u32>,
    /// }
    /// ```
    pub fn params_field<T: DeserializeOwned, F, PP: ParamsPath, L: FnOnce(T) -> Option<F>>(
        &self,
        path: &PP,
        ex: L,
    ) -> Result<F, ParamsError> {
        let params_json = self.inner.get(path.key()).ok_or(ParamsError::NotFound)?;
        let params = serde_json::from_slice::<T>(&params_json)?;

        let field = ex(params).ok_or(ParamsError::MissingField)?;

        Ok(field)
    }
}
