use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum Error {
    #[error("database error")]
    Decode(#[from] rocksdb::Error),
}
