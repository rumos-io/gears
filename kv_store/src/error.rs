use thiserror::Error;

use crate::StoreKey;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum KVStoreError {
    #[error(transparent)]
    Tree(#[from] trees::Error),
}

#[derive(Debug, PartialEq, Eq)]

pub struct MultiStoreError<SK: StoreKey> {
    pub sk: SK,
    pub err: KVStoreError,
}

impl<SK: StoreKey> std::fmt::Display for MultiStoreError<SK> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "failed to init for {} with error: {}",
            self.sk.name(),
            self.err
        )
    }
}

pub const KEY_EXISTS_MSG: &str = "a store for every key is guaranteed to exist";
pub const POISONED_LOCK: &str = "poisoned lock";
