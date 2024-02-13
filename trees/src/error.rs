use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum Error {
    #[error("rotate error: {0}")]
    RotateError(String),
    #[error("could not find requested version in DB")]
    VersionNotFound,
    #[error("unable to deserialize bytes to Node")]
    NodeDeserialize,
    #[error("cannot overwrite existing version")]
    Overwrite,
    #[error("requested node is not exists")]
    NodeNotExists, // TODO: More specific and special errors for removing node
    #[error("custom error: {0}")]
    CustomError(String),
}

pub mod constants {
    pub const LEAF_ROTATE_ERROR: &str = "can't rotate a leaf node";
}
