use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum Error {
    #[error(transparent)]
    Decode(#[from] rocksdb::Error),
}
