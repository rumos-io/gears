use gears::{
    application::handlers::node::ABCIHandler,
    baseapp::{NullQueryRequest, NullQueryResponse},
    core::Protobuf,
    params::ParamsSubspaceKey,
    store::StoreKey,
    tendermint::types::request::deliver_tx::RequestDeliverTx,
    types::tx::NullTxMsg,
    x::{
        keepers::{
            auth::AuthKeeper,
            staking::{KeeperHooks, StakingBankKeeper},
        },
        module::Module,
    },
};

use crate::genesis::GenutilGenesis;

#[derive(Debug, Clone)]
pub struct GenutilAbciHandler<
    SK: StoreKey,
    PSK: ParamsSubspaceKey,
    AK: AuthKeeper<SK, M>,
    BK: StakingBankKeeper<SK, M>,
    KH: KeeperHooks<SK, AK, M>,
    M: Module,
> {
    staking: staking::Keeper<SK, PSK, AK, BK, KH, M>,
}

impl<
        SK: StoreKey,
        PSK: ParamsSubspaceKey,
        AK: AuthKeeper<SK, M>,
        BK: StakingBankKeeper<SK, M>,
        KH: KeeperHooks<SK, AK, M>,
        M: Module,
    > GenutilAbciHandler<SK, PSK, AK, BK, KH, M>
{
    pub fn new(staking: staking::Keeper<SK, PSK, AK, BK, KH, M>) -> Self {
        Self { staking }
    }
}

impl<
        SK: StoreKey,
        PSK: ParamsSubspaceKey,
        AK: AuthKeeper<SK, M> + Send + Sync + 'static,
        BK: StakingBankKeeper<SK, M>,
        KH: KeeperHooks<SK, AK, M>,
        M: Module,
    > ABCIHandler for GenutilAbciHandler<SK, PSK, AK, BK, KH, M>
{
    type Message = NullTxMsg;

    type Genesis = GenutilGenesis;

    type StoreKey = SK;

    type QReq = NullQueryRequest;

    type QRes = NullQueryResponse;

    fn typed_query<DB: gears::store::database::Database>(
        &self,
        _ctx: &gears::context::query::QueryContext<DB, Self::StoreKey>,
        _query: Self::QReq,
    ) -> Self::QRes {
        unreachable!()
    }

    fn run_ante_checks<DB: gears::store::database::Database>(
        &self,
        _ctx: &mut gears::context::tx::TxContext<'_, DB, Self::StoreKey>,
        _tx: &gears::types::tx::raw::TxWithRaw<Self::Message>,
    ) -> Result<(), gears::application::handlers::node::TxError> {
        Ok(())
    }

    fn msg<DB: gears::store::database::Database>(
        &self,
        _ctx: &mut gears::context::tx::TxContext<'_, DB, Self::StoreKey>,
        _msg: &Self::Message,
    ) -> Result<(), gears::application::handlers::node::TxError> {
        unreachable!()
    }

    fn init_genesis<DB: gears::store::database::Database>(
        &self,
        ctx: &mut gears::context::init::InitContext<'_, DB, Self::StoreKey>,
        genesis: Self::Genesis,
    ) -> Vec<gears::tendermint::types::proto::validator::ValidatorUpdate> {
        for tx in genesis.gen_txs {
            let result = gears::global_node::global_node()
                .expect("node should be set when we call `init_genesis`")
                .deliver_tx(RequestDeliverTx {
                    tx: tx.encode_vec().into(),
                });

            if result.code > 0 {
                panic!("log :{}, info: {}", result.log, result.info);
            }
        }

        match self.staking.apply_and_return_validator_set_updates(ctx) {
            Ok(res) => res,
            Err(err) => panic!("failed to apply validators err: {err}"),
        }
    }

    fn query<DB: gears::store::database::Database + Send + Sync>(
        &self,
        _ctx: &gears::context::query::QueryContext<DB, Self::StoreKey>,
        _query: gears::tendermint::types::request::query::RequestQuery,
    ) -> Result<Vec<u8>, gears::baseapp::errors::QueryError> {
        unreachable!()
    }
}
