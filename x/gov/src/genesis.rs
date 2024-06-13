use crate::{
    params::GovParams,
    types::{deposit::Deposit, proposal::Proposal, vote_weighted::VoteWeighted},
};

pub struct GovGenesisState {
    pub starting_proposal_id: u64,
    pub deposits: Vec<Deposit>,
    pub votes: Vec<VoteWeighted>,
    pub proposals: Vec<Proposal>,
    pub params: GovParams,
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
