use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum Error {
    #[error(transparent)]
    Database(#[from] trees::Error),
}
