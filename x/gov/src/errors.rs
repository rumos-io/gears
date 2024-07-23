use gears::{
    application::handlers::node::{ModuleInfo, TxError},
    types::{base::errors::CoinsError, store::gas::errors::GasStoreErrors},
    x::errors::BankKeeperError,
};

pub const SERDE_JSON_CONVERSION: &str = "conversion to json shouldn't fail";
pub const EXISTS: &str = "value guaranteed to exists";

#[derive(thiserror::Error, Debug)]
pub enum GovTxError {
    #[error(transparent)]
    Keeper(#[from] GovKeeperError),
}

impl GovTxError {
    pub fn into<MI: ModuleInfo>(self) -> TxError {
        TxError::new::<MI>(self.to_string(), nz::u16!(1))
    }
}

#[derive(thiserror::Error, Debug, Clone)]
pub enum GovKeeperError {
    #[error("gov: no handler exists for proposal type")]
    NoHandler,
    #[error("{0}")]
    Bank(#[from] BankKeeperError),
    #[error("{0}")]
    Coins(#[from] CoinsError),
    #[error("inactive proposal {0}")]
    InactiveProposal(u64),
    #[error("unknown proposal {0}")]
    ProposalUnknown(u64),
    #[error("{0}")]
    Gas(#[from] GasStoreErrors),
}
