#[cfg(feature = "rocksdb")]
pub use self::inner::*;

#[cfg(feature = "rocksdb")]
mod inner {
    use thiserror::Error;

    #[derive(Error, Debug, PartialEq, Eq)]
    pub enum Error {
        #[error(transparent)]
        Decode(#[from] rocksdb::Error),
    }
}
