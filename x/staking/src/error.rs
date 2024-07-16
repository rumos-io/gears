use gears::{
    application::handlers::node::{ErrorCode, ModuleInfo, TxError},
    error::AppError,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StakingTxError<MI> {
    #[error(transparent)]
    Other(#[from] AppError), //TODO: stop using AppError
    #[error("phantom error")]
    Phantom((MI, std::convert::Infallible)),
}

impl<MI: ModuleInfo> From<StakingTxError<MI>> for TxError {
    fn from(error: StakingTxError<MI>) -> Self {
        let code = match &error {
            StakingTxError::Other(_) => 1,
            StakingTxError::Phantom(_) => unreachable!(),
        };

        TxError {
            msg: error.to_string(),
            code: ErrorCode::try_new(code).expect("all > 0"),
            codespace: MI::NAME,
        }
    }
}
