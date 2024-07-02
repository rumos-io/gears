use crate::{GenesisState, Keeper, Message};
use gears::{
    context::{block::BlockContext, init::InitContext, tx::TxContext},
    error::AppError,
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

    pub fn tx<DB: Database + Sync + Send>(
        &self,
        ctx: &mut TxContext<'_, DB, SK>,
        msg: &Message,
    ) -> Result<(), AppError> {
        match msg {
            Message::Unjail(msg) => self.keeper.unjail_tx_handler(ctx, msg),
        }
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
                self.keeper
                    .handle_validator_signature(
                        ctx,
                        vote.validator.address.into(),
                        vote.validator.power as u32,
                        vote.signed_last_block,
                    )
                    .expect(
                        "method `handle_validator_signature` is called from infallible method.
                         Something wrong in the handler.",
                    );
            }
        }
    }
}
