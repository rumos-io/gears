use bytes::Bytes;
use gears::{
    application::handlers::node::ABCIHandler,
    context::{init::InitContext, query::QueryContext, tx::TxContext, TransactionalContext},
    error::AppError,
    params::ParamsSubspaceKey,
    store::{database::Database, StoreKey},
    tendermint::types::{
        proto::event::{Event, EventAttribute},
        request::query::RequestQuery,
    },
    types::tx::raw::TxWithRaw,
    x::{keepers::bank::BankKeeper, module::Module},
};

use crate::{
    genesis::GovGenesisState,
    keeper::GovKeeper,
    msg::GovMsg,
    query::{GovQueryRequest, GovQueryResponse},
};

#[derive(Debug, Clone)]
pub struct GovAbciHandler<SK: StoreKey, PSK: ParamsSubspaceKey, M: Module, BK: BankKeeper<SK, M>> {
    keeper: GovKeeper<SK, PSK, M, BK>,
}

impl<SK: StoreKey, PSK: ParamsSubspaceKey, M: Module, BK: BankKeeper<SK, M>>
    GovAbciHandler<SK, PSK, M, BK>
{
    pub fn new(keeper: GovKeeper<SK, PSK, M, BK>) -> Self {
        Self { keeper }
    }
}

impl<SK: StoreKey, PSK: ParamsSubspaceKey, M: Module, BK: BankKeeper<SK, M>> ABCIHandler
    for GovAbciHandler<SK, PSK, M, BK>
{
    type Message = GovMsg;

    type Genesis = GovGenesisState;

    type StoreKey = SK;

    type QReq = GovQueryRequest;

    type QRes = GovQueryResponse;

    fn typed_query<DB: Database>(
        &self,
        _ctx: &QueryContext<DB, Self::StoreKey>,
        _query: Self::QReq,
    ) -> Self::QRes {
        todo!()
    }

    fn run_ante_checks<DB: Database>(
        &self,
        _ctx: &mut TxContext<'_, DB, Self::StoreKey>,
        _tx: &TxWithRaw<Self::Message>,
    ) -> Result<(), AppError> {
        Ok(())
    }

    fn tx<DB: Database + Sync + Send>(
        &self,
        ctx: &mut TxContext<'_, DB, Self::StoreKey>,
        msg: &Self::Message,
    ) -> Result<(), AppError> {
        match msg {
            GovMsg::Deposit(msg) => {
                let is_voting_started = self
                    .keeper
                    .deposit_add(ctx, msg.clone())
                    .map_err(|e| AppError::Custom(e.to_string()))?;

                ctx.push_event(Event::new(
                    "message",
                    vec![
                        EventAttribute::new("module".into(), "governance".into(), false),
                        EventAttribute::new(
                            "sender".into(),
                            msg.depositor.to_string().into(),
                            false,
                        ),
                    ],
                ));

                if is_voting_started {
                    ctx.push_event(Event::new(
                        "proposal_deposit",
                        vec![EventAttribute::new(
                            "voting_period_start".into(),
                            msg.proposal_id.to_string().into(),
                            false,
                        )],
                    ));
                }

                Ok(())
            }
            GovMsg::Vote(msg) => {
                self.keeper
                    .vote_add(ctx, msg.clone().into())
                    .map_err(|e| AppError::Custom(e.to_string()))?;

                ctx.push_event(Event::new(
                    "message",
                    vec![
                        EventAttribute::new("module".into(), "governance".into(), false),
                        EventAttribute::new("sender".into(), msg.voter.to_string().into(), false),
                    ],
                ));

                Ok(())
            }
            GovMsg::Weighted(_msg) => todo!(),
        }
    }

    fn init_genesis<DB: Database>(
        &self,
        ctx: &mut InitContext<'_, DB, Self::StoreKey>,
        genesis: Self::Genesis,
    ) {
        self.keeper.init_genesis(ctx, genesis)
    }

    fn query<DB: Database>(
        &self,
        _ctx: &QueryContext<DB, Self::StoreKey>,
        _query: RequestQuery,
    ) -> Result<Bytes, AppError> {
        todo!()
    }
}
