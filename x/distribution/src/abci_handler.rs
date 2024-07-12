use crate::{GenesisState, Keeper};
use gears::{
    context::{block::BlockContext, init::InitContext, QueryableContext},
    params::ParamsSubspaceKey,
    store::{database::Database, StoreKey},
    tendermint::types::request::begin_block::RequestBeginBlock,
    types::address::ConsAddress,
    x::{
        keepers::{
            auth::AuthKeeper, bank::StakingBankKeeper as BankKeeper, staking::SlashingStakingKeeper,
        },
        module::Module,
    },
};

#[derive(Debug, Clone)]
pub struct ABCIHandler<
    SK: StoreKey,
    PSK: ParamsSubspaceKey,
    AK: AuthKeeper<SK, M>,
    BK: BankKeeper<SK, M>,
    SSK: SlashingStakingKeeper<SK, M>,
    M: Module,
> {
    keeper: Keeper<SK, PSK, AK, BK, SSK, M>,
}

impl<
        SK: StoreKey,
        PSK: ParamsSubspaceKey,
        AK: AuthKeeper<SK, M>,
        BK: BankKeeper<SK, M>,
        SSK: SlashingStakingKeeper<SK, M>,
        M: Module,
    > ABCIHandler<SK, PSK, AK, BK, SSK, M>
{
    pub fn new(keeper: Keeper<SK, PSK, AK, BK, SSK, M>) -> Self {
        ABCIHandler { keeper }
    }

    pub fn genesis<DB: Database>(&self, ctx: &mut InitContext<'_, DB, SK>, genesis: GenesisState) {
        if let Err(e) = self.keeper.init_genesis(ctx, genesis) {
            panic!("Initialization of genesis failed with error:\n{e}")
        }
    }

    /// begin_block sets the proposer for determining distribution during end_block
    /// and distribute rewards for the previous block
    pub fn begin_block<DB: Database>(
        &self,
        ctx: &mut BlockContext<'_, DB, SK>,
        request: RequestBeginBlock,
    ) {
        // determine the total power signing the block
        let mut sum_previous_precommit_power: u64 = 0;
        let previous_total_power = request.last_commit_info.votes.iter().fold(0, |acc, vote| {
            let power = u64::from(vote.validator.power);
            if vote.signed_last_block {
                sum_previous_precommit_power += power;
            }
            acc + power
        });

        // TODO this is Tendermint-dependent
        // ref https://github.com/cosmos/cosmos-sdk/issues/3095

        if ctx.height() > 1 {
            if let Some(previous_proposer) = self.keeper.previous_proposer_cons_addr(ctx) {
                if let Err(e) = self.keeper.allocate_tokens(
                    ctx,
                    sum_previous_precommit_power,
                    previous_total_power,
                    &previous_proposer,
                    &request.last_commit_info.votes,
                ) {
                    panic!("Error thrown in begin_block method: \n{e}");
                }
            } else {
                panic!("previous proposer not set");
            }
        }

        // record the proposer for when we payout on the next block
        // TODO:ME consider to change request header structure to have ConsAddress
        let cons_addr = match ConsAddress::try_from(request.header.proposer_address) {
            Ok(addr) => addr,
            Err(e) => panic!("{e}"),
        };
        self.keeper.set_previous_proposer_cons_addr(ctx, &cons_addr);
    }
}
