use gears::{
    baseapp::genesis::{Genesis, GenesisError},
    types::{address::AccAddress, base::coins::UnsignedCoins},
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::{
    msg::{deposit::Deposit, weighted_vote::MsgVoteWeighted},
    params::GovParams,
    types::proposal::ProposalModel,
};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GovGenesisState<T> {
    pub starting_proposal_id: u64,
    pub deposits: Vec<Deposit>,
    pub votes: Vec<MsgVoteWeighted>,
    pub proposals: Vec<ProposalModel<T>>,
    pub params: GovParams,
}

impl<P: Clone + Serialize + DeserializeOwned + std::fmt::Debug + Send + Sync + 'static> Genesis
    for GovGenesisState<P>
{
    fn add_genesis_account(
        &mut self,
        _address: AccAddress,
        _coins: UnsignedCoins,
    ) -> Result<(), GenesisError> {
        todo!()
    }
}

impl<T> Default for GovGenesisState<T> {
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
