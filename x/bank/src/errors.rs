use gears::{
    application::handlers::node::{ErrorCode, ModuleInfo, TxError},
    error::AppError,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BankTxError {
    #[error(transparent)]
    Other(#[from] AppError), //TODO: stop using AppError
}

impl BankTxError {
    pub fn into<MI: ModuleInfo>(self) -> TxError {
        let code = match &self {
            BankTxError::Other(_) => 1,
        };

        TxError {
            msg: self.to_string(),
            code: ErrorCode::try_new(code).expect("all > 0"),
            codespace: MI::NAME,
        }
    }
}
