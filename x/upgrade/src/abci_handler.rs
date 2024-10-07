use std::{
    fmt::{Debug, Display},
    marker::PhantomData,
};

use gears::{
    application::handlers::node::{ABCIHandler, ModuleInfo},
    baseapp::genesis::NullGenesis,
    context::QueryableContext,
    params::ParamsSubspaceKey,
    store::StoreKey,
    types::tx::NullTxMsg,
    x::module::Module,
};
use tracing::info;

use crate::{
    handler::DummyHandler,
    keeper::{downgrade_verified, set_downgrade_verified, UpgradeKeeper},
    types::query::{UpgradeQueryRequest, UpgradeQueryResponse},
};

#[derive(Debug, Clone)]
pub struct UpgradeAbciHandler<SK: StoreKey, PSK: ParamsSubspaceKey, M: Module, MI> {
    keeper: UpgradeKeeper<SK, M, DummyHandler>,
    _marker: PhantomData<(MI, SK, PSK, M)>,
}

impl<
        SK: StoreKey,
        PSK: ParamsSubspaceKey,
        M: Module + TryFrom<Vec<u8>> + std::cmp::Eq + std::hash::Hash,
        MI: ModuleInfo,
    > ABCIHandler for UpgradeAbciHandler<SK, PSK, M, MI>
where
    <M as TryFrom<Vec<u8>>>::Error: Display + Debug,
{
    type Message = NullTxMsg;

    type Genesis = NullGenesis;

    type StoreKey = SK;

    type QReq = UpgradeQueryRequest;

    type QRes = UpgradeQueryResponse;

    fn typed_query<DB: gears::store::database::Database>(
        &self,
        _ctx: &gears::context::query::QueryContext<DB, Self::StoreKey>,
        _query: Self::QReq,
    ) -> Self::QRes {
        todo!()
    }

    fn run_ante_checks<DB: gears::store::database::Database>(
        &self,
        _: &mut gears::context::tx::TxContext<'_, DB, Self::StoreKey>,
        _: &gears::types::tx::raw::TxWithRaw<Self::Message>,
        _: bool,
    ) -> Result<(), gears::application::handlers::node::TxError> {
        Ok(())
    }

    fn msg<DB: gears::store::database::Database>(
        &self,
        _: &mut gears::context::tx::TxContext<'_, DB, Self::StoreKey>,
        _: &Self::Message,
    ) -> Result<(), gears::application::handlers::node::TxError> {
        Ok(())
    }

    fn init_genesis<DB: gears::store::database::Database>(
        &self,
        _: &mut gears::context::init::InitContext<'_, DB, Self::StoreKey>,
        _: Self::Genesis,
    ) -> Vec<gears::tendermint::types::proto::validator::ValidatorUpdate> {
        Vec::new()
    }

    fn query<DB: gears::store::database::Database + Send + Sync>(
        &self,
        _ctx: &gears::context::query::QueryContext<DB, Self::StoreKey>,
        _query: gears::tendermint::types::request::query::RequestQuery,
    ) -> Result<Vec<u8>, gears::baseapp::errors::QueryError> {
        todo!()
    }

    fn begin_block<'b, DB: gears::store::database::Database>(
        &self,
        ctx: &mut gears::context::block::BlockContext<'_, DB, Self::StoreKey>,
        _request: gears::tendermint::request::RequestBeginBlock,
    ) {
        let plan = self.keeper.upgrade_plan(ctx);

        if !downgrade_verified() {
            set_downgrade_verified(true);
            let last_applied_plan = self.keeper.last_completed_upgrade(ctx);

            let is_none = plan.is_none();
            let should_execute = plan
                .as_ref()
                .map(|this| this.should_execute(ctx))
                .unwrap_or_default();

            // This check will make sure that we are using a valid binary.
            // It'll panic in these cases if there is no upgrade handler registered for the last applied upgrade.
            // 1. If there is no scheduled upgrade.
            // 2. If the plan is not ready.
            // 3. If the plan is ready and skip upgrade height is set for current height.
            if is_none
                || !should_execute
                || (should_execute && self.keeper.is_skip_height(ctx.height()))
            {
                match last_applied_plan {
                    Some(upg) if self.keeper.has_handler(&upg.name) => panic!(
                        "Wrong app version {}, upgrade handler is missing for {} upgrade plan",
                        1, // TODO: consensus params should have version?
                        upg.name,
                    ),
                    _ => (),
                }
            }
        }

        let plan = match plan {
            Some(plan) => plan,
            None => return,
        };

        // To make sure clear upgrade is executed at the same block
        if plan.should_execute(ctx) {
            // If skip upgrade has been set for current height, we clear the upgrade plan
            if self.keeper.is_skip_height(ctx.height()) {
                info!(
                    "UPGRADE `{}` SKIPPED at {}: {}",
                    plan.name, plan.height, plan.info
                );

                self.keeper.delete_upgrade_plan(ctx);
                return;
            }

            if !self.keeper.has_handler(&plan.name) {
                // TODO: store info https://github.com/cosmos/cosmos-sdk/blob/d3f09c222243bb3da3464969f0366330dcb977a8/x/upgrade/keeper/keeper.go#L375-L396

                // We don't have an upgrade handler for this upgrade name, meaning this software is out of date so shutdown
                // "UPGRADE \"%s\" NEEDED at %s: %s", plan.Name, plan.DueAt(), plan.Info
                let msg = format!(
                    "UPGRADE `{}` NEEDED at height: {}: {}",
                    plan.name, plan.height, plan.info
                );
                let log_msg = msg.clone();

                tracing::error!("{log_msg}");
                panic!("{msg}");
            }

            tracing::info!(
                "applying upgrade `{}` at height: {}",
                plan.name,
                plan.height
            );
            // todo: why they need gas https://github.com/cosmos/cosmos-sdk/blob/d3f09c222243bb3da3464969f0366330dcb977a8/x/upgrade/abci.go#L75
            match self.keeper.apply_upgrade(ctx, plan) {
                Ok(_) => return,
                Err(err) => panic!("{err}"),
            }
        }

        if self.keeper.has_handler(&plan.name) {
            let msg = format!(
                "BINARY UPDATED BEFORE TRIGGER! UPGRADE `{}` - in binary but not executed on chain",
                plan.name
            );
            let log_msg = msg.clone();
            tracing::error!("{log_msg}");
            panic!("{msg}");
        }
    }
}
