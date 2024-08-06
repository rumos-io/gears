#[derive(thiserror::Error, Debug)]
#[error("evidence router already exists")]
pub struct RouterAlreadyExistsError;
