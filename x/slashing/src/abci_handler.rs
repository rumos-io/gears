use crate::{GenesisState, Keeper};
use gears::{
    context::init::InitContext,
    params::ParamsSubspaceKey,
    store::{database::Database, StoreKey},
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
}
