use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum KVStoreError {
    #[error(transparent)]
    Tree(#[from] trees::Error),
}

pub const KEY_EXISTS_MSG: &str = "a store for every key is guaranteed to exist";
pub const POISONED_LOCK: &str = "poisoned lock";
