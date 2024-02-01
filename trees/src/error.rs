use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum Error {
    #[error("rotate error")]
    RotateError,
    #[error("could not find requested version in DB")]
    VersionNotFound,
    #[error("unable to deserialize bytes to Node")]
    NodeDeserialize,
    #[error("cannot overwrite existing version")]
    Overwrite,
    #[error("requested node is not exists")]
    NodeNotExists, // TODO: More specific and special errors for removing node
}
