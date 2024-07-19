use gears::application::handlers::node::{ErrorCode, TxError};
use thiserror::Error;

pub const SERDE_JSON_CONVERSION: &str = "conversion to json shouldn't fail";
pub const EXISTS: &str = "value guaranteed to exists";

#[derive(Error, Debug)]
pub enum GovTxError {
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl From<GovTxError> for TxError {
    fn from(value: GovTxError) -> Self {
        match value {
            GovTxError::Other(e) => TxError {
                msg: format!("{e}"),
                code: ErrorCode::try_new(1).expect("1 > 0"),
                codespace: "",
            },
        }
    }
}
