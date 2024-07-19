use gears::application::handlers::node::{ErrorCode, ModuleInfo, TxError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StakingTxError {
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl StakingTxError {
    pub fn into<MI: ModuleInfo>(self) -> TxError {
        let code = match &self {
            StakingTxError::Other(_) => 1,
        };

        TxError {
            msg: self.to_string(),
            code: ErrorCode::try_new(code).expect("all > 0"),
            codespace: MI::NAME,
        }
    }
}
