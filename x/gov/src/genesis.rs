use crate::{
    params::{DepositParams, TallyParams, VotingParams},
    types::{deposit::Deposit, proposal::Proposal, vote::Vote},
};

pub struct GovGenesisState {
    pub starting_proposal_id: u64,
    pub deposits: Vec<Deposit>,
    pub votes: Vec<Vote>,
    pub proposals: Vec<Proposal>,
    pub deposit: DepositParams,
    pub voting: VotingParams,
    pub tally: TallyParams,
}

impl Default for GovGenesisState {
    fn default() -> Self {
        Self {
            starting_proposal_id: 1,
            deposits: Vec::new(),
            votes: Vec::new(),
            proposals: Vec::new(),
            deposit: Default::default(),
            voting: Default::default(),
            tally: Default::default(),
        }
    }
}
