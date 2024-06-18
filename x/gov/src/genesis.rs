use gears::{
    baseapp::genesis::Genesis,
    types::{address::AccAddress, base::send::SendCoins},
};
use serde::{Deserialize, Serialize};

use crate::{
    msg::{deposit::MsgDeposit, weighted_vote::MsgVoteWeighted},
    params::GovParams,
    types::proposal::Proposal,
};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GovGenesisState {
    pub starting_proposal_id: u64,
    pub deposits: Vec<MsgDeposit>,
    pub votes: Vec<MsgVoteWeighted>,
    pub proposals: Vec<Proposal>, 
    pub params: GovParams,
}

impl Genesis for GovGenesisState {
    fn add_genesis_account(
        &mut self,
        _address: AccAddress,
        _coins: SendCoins,
    ) -> Result<(), gears::error::AppError> {
        todo!()
    }
}

impl Default for GovGenesisState {
    fn default() -> Self {
        Self {
            starting_proposal_id: 1,
            deposits: Vec::new(),
            votes: Vec::new(),
            proposals: Vec::new(),
            params: GovParams {
                tally: Default::default(),
                voting: Default::default(),
                deposit: Default::default(),
            },
        }
    }
}
