use gears::{
    application::handlers::node::{ErrorCode, ModuleInfo, TxError},
    x::errors::BankKeeperError,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BankTxError {
    #[error(transparent)]
    Keeper(#[from] BankKeeperError),
}

impl BankTxError {
    pub fn into<MI: ModuleInfo>(self) -> TxError {
        let code = match &self {
            BankTxError::Keeper(_) => 1,
        };

        TxError {
            msg: self.to_string(),
            code: ErrorCode::try_new(code).expect("all > 0"),
            codespace: MI::NAME,
        }
    }
}
