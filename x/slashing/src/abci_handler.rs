use crate::{GenesisState, Keeper};
use gears::{
    context::{block::BlockContext, init::InitContext},
    params::ParamsSubspaceKey,
    store::{database::Database, StoreKey},
    tendermint::types::request::begin_block::RequestBeginBlock,
    x::{keepers::staking::SlashingStakingKeeper, module::Module},
};

#[derive(Debug, Clone)]
pub struct ABCIHandler<
    SK: StoreKey,
    PSK: ParamsSubspaceKey,
    SSK: SlashingStakingKeeper<SK, M>,
    M: Module,
> {
    keeper: Keeper<SK, PSK, SSK, M>,
}

impl<SK: StoreKey, PSK: ParamsSubspaceKey, SSK: SlashingStakingKeeper<SK, M>, M: Module>
    ABCIHandler<SK, PSK, SSK, M>
{
    pub fn new(keeper: Keeper<SK, PSK, SSK, M>) -> Self {
        ABCIHandler { keeper }
    }

    pub fn genesis<DB: Database>(&self, ctx: &mut InitContext<'_, DB, SK>, genesis: GenesisState) {
        self.keeper.init_genesis(ctx, genesis)
    }

    /// begin_block check for infraction evidence or downtime of validators
    /// on every begin block
    pub fn begin_block<DB: Database>(
        &self,
        ctx: &mut BlockContext<'_, DB, SK>,
        request: RequestBeginBlock,
    ) {
        // Iterate over all the validators which *should* have signed this block
        // store whether or not they have actually signed it and slash/unbond any
        // which have missed too many blocks in a row (downtime slashing)
        if let Some(vote_info) = request.last_commit_info {
            for vote in vote_info.votes {
                // TODO: seems like tendermint type has optional value in order of using prost
                let validator = vote.validator.unwrap();
                self.keeper.handle_validator_signature(
                    ctx,
                    validator.address,
                    validator.power as u32,
                    vote.signed_last_block,
                );
            }
        }
    }
}
