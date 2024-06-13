use deposit::Deposit;
use gears::{
    core::{any::google::Any, errors::CoreError},
    types::tx::TxMessage,
};
use serde::Serialize;
use vote::Vote;
use weighted_vote::VoteWeighted;

pub mod deposit;
pub mod proposal;
pub mod vote;
pub mod weighted_vote;

#[derive(Debug, Clone, Serialize)]
pub enum GovMsg {
    Deposit(Deposit),
    Vote(Vote),
    VoteWeighted(VoteWeighted),
}

impl TryFrom<Any> for GovMsg {
    type Error = CoreError;

    fn try_from(_value: Any) -> Result<Self, Self::Error> {
        todo!()
    }
}

impl From<GovMsg> for Any {
    fn from(_value: GovMsg) -> Self {
        todo!()
    }
}

impl TxMessage for GovMsg {
    fn get_signers(&self) -> Vec<&gears::types::address::AccAddress> {
        todo!()
    }

    fn validate_basic(&self) -> Result<(), String> {
        todo!()
    }

    fn type_url(&self) -> &'static str {
        todo!()
    }
}
