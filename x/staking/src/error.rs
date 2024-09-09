use gears::{
    application::handlers::node::{ModuleInfo, TxError},
    tendermint::error::Error,
    types::{address::ValAddress, base::coin::UnsignedCoin},
    x::types::validator::BondStatus,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StakingGenesisError {
    #[error("invalid validator status {0}")]
    InvalidStatus(BondStatus),
    #[error("bonded pool balance is different from bonded coins: {0:?} <-> {1:?}")]
    WrongBondedPoolBalance(Vec<UnsignedCoin>, Vec<UnsignedCoin>),
    #[error("not bonded pool balance is different from not bonded coins: {0:?} <-> {1:?}")]
    WrongUnbondedPoolBalance(Vec<UnsignedCoin>, Vec<UnsignedCoin>),
    #[error("invalid genesis file: validator {0} in `last_validator_powers` list not found in `validators` list")]
    ValidatorNotFound(ValAddress),
    #[error("{0}")]
    VotingPower(#[from] Error),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[derive(Error, Debug)]
pub enum StakingTxError {
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl StakingTxError {
    pub fn into<MI: ModuleInfo>(self) -> TxError {
        let code = match &self {
            StakingTxError::Other(_) => nz::u16!(1),
        };

        TxError {
            msg: self.to_string().into(),
            code,
            codespace: MI::NAME,
        }
    }
}
