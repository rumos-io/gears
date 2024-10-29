use std::marker::PhantomData;

use gears::{
    application::{
        handlers::node::{ABCIHandler, ModuleInfo},
        keepers::params::ParamsKeeper,
    },
    baseapp::{errors::QueryError, QueryResponse},
    context::{query::QueryContext, InfallibleContext, TransactionalContext},
    core::Protobuf,
    params::ParamsSubspaceKey,
    store::{database::Database, StoreKey},
    tendermint::types::{
        proto::event::{Event, EventAttribute},
        request::query::RequestQuery,
    },
    types::{
        base::coins::UnsignedCoins,
        decimal256::{CosmosDecimalProtoString, Decimal256},
        tx::NullTxMsg,
    },
    x::{
        keepers::mint::{MintingBankKeeper, MintingStakingKeeper},
        module::Module,
    },
};

use crate::{
    genesis::MintGenesis,
    keeper::MintKeeper,
    params::MintParamsKeeper,
    types::query::{
        request::{
            MintQueryRequest, QueryAnnualProvisionsRequest, QueryInflationRequest,
            QueryParamsRequest,
        },
        response::{
            MintQueryResponse, QueryAnnualProvisionsResponse, QueryInflationResponse,
            QueryParamsResponse,
        },
    },
};

const MISSING_MINTER_ERR_MSG: &str =
    "Failed to get minter. Minter should be set during init genesis";

#[derive(Debug, Clone)]
pub struct MintAbciHandler<SK, PSK, BK, STK, M, MI> {
    keeper: MintKeeper<SK, BK, STK, M>,
    params_keeper: MintParamsKeeper<PSK>,
    _marker: PhantomData<MI>,
}

impl<SK, PSK, BK, STK, M, MI> MintAbciHandler<SK, PSK, BK, STK, M, MI> {
    pub fn new(keeper: MintKeeper<SK, BK, STK, M>, params_subspace_key: PSK) -> Self {
        Self {
            keeper,
            params_keeper: MintParamsKeeper {
                params_subspace_key,
            },
            _marker: PhantomData,
        }
    }
}

impl<
        SK: StoreKey,
        PSK: ParamsSubspaceKey,
        BK: MintingBankKeeper<SK, M>,
        STK: MintingStakingKeeper<SK, M>,
        M: Module,
        MI: ModuleInfo,
    > MintAbciHandler<SK, PSK, BK, STK, M, MI>
{
    pub fn query_params<CTX: InfallibleContext<DB, SK>, DB: Database>(
        &self,
        ctx: &CTX,
    ) -> QueryParamsResponse {
        QueryParamsResponse {
            params: self.params_keeper.get(ctx),
        }
    }

    pub fn query_inflation<CTX: InfallibleContext<DB, SK>, DB: Database>(
        &self,
        ctx: &CTX,
    ) -> QueryInflationResponse {
        let inflation = match self.keeper.minter(ctx) {
            Some(minter) => minter.inflation,
            None => panic!("{MISSING_MINTER_ERR_MSG}"),
        };

        QueryInflationResponse { inflation }
    }

    pub fn query_annual_provisions<CTX: InfallibleContext<DB, SK>, DB: Database>(
        &self,
        ctx: &CTX,
    ) -> QueryAnnualProvisionsResponse {
        let annual_provisions = match self.keeper.minter(ctx) {
            Some(minter) => minter.annual_provisions,
            None => panic!("{MISSING_MINTER_ERR_MSG}"),
        };

        QueryAnnualProvisionsResponse { annual_provisions }
    }
}

impl<
        SK: StoreKey,
        PSK: ParamsSubspaceKey,
        BK: MintingBankKeeper<SK, M>,
        STK: MintingStakingKeeper<SK, M>,
        M: Module,
        MI: ModuleInfo,
    > ABCIHandler for MintAbciHandler<SK, PSK, BK, STK, M, MI>
{
    type Message = NullTxMsg;

    type Genesis = MintGenesis;

    type StoreKey = SK;

    type QReq = MintQueryRequest;

    type QRes = MintQueryResponse;

    fn typed_query<DB: Database>(
        &self,
        ctx: &QueryContext<DB, Self::StoreKey>,
        query: Self::QReq,
    ) -> Self::QRes {
        match query {
            MintQueryRequest::Params(_) => Self::QRes::Params(self.query_params(ctx)),
            MintQueryRequest::Inflation(_) => Self::QRes::Inflation(self.query_inflation(ctx)),
            MintQueryRequest::AnnualProvisions(_) => {
                Self::QRes::AnnualProvisions(self.query_annual_provisions(ctx))
            }
        }
    }

    fn msg<DB: Database>(
        &self,
        _ctx: &mut gears::context::tx::TxContext<'_, DB, Self::StoreKey>,
        _msg: &Self::Message,
    ) -> Result<(), gears::application::handlers::node::TxError> {
        unreachable!("Module {} doesn't have any tx", MI::NAME)
    }

    fn init_genesis<DB: Database>(
        &self,
        ctx: &mut gears::context::init::InitContext<'_, DB, Self::StoreKey>,
        Self::Genesis { minter, params }: Self::Genesis,
    ) -> Vec<gears::tendermint::types::proto::validator::ValidatorUpdate> {
        self.keeper.minter_set(ctx, &minter);
        self.params_keeper.set(ctx, params);

        Vec::new()
    }

    fn query<DB: gears::store::database::Database + Send + Sync>(
        &self,
        ctx: &QueryContext<DB, Self::StoreKey>,
        RequestQuery { data, path, .. }: RequestQuery,
    ) -> Result<Vec<u8>, gears::baseapp::errors::QueryError> {
        let query = match path.as_str() {
            QueryParamsRequest::QUERY_URL => {
                Self::QReq::Params(QueryParamsRequest::decode_vec(&data)?)
            }
            QueryInflationRequest::QUERY_URL => {
                Self::QReq::Inflation(QueryInflationRequest::decode_vec(&data)?)
            }
            QueryAnnualProvisionsRequest::QUERY_URL => {
                Self::QReq::AnnualProvisions(QueryAnnualProvisionsRequest::decode_vec(&data)?)
            }
            _ => Err(QueryError::PathNotFound)?,
        };

        Ok(self.typed_query(ctx, query).into_bytes())
    }

    fn begin_block<'a, DB: gears::store::database::Database>(
        &self,
        ctx: &mut gears::context::block::BlockContext<'_, DB, Self::StoreKey>,
        _request: gears::tendermint::request::RequestBeginBlock,
    ) {
        let mut minter = match self.keeper.minter(ctx) {
            Some(minter) => minter,
            None => panic!(
                "Failed to `begin_block` in {} Reason: {MISSING_MINTER_ERR_MSG}",
                MI::NAME
            ), // This should never happen
        };
        let params = self.params_keeper.get(ctx);
        let total_staking_supply = self
            .keeper
            .staking_token_supply(ctx)
            .map(|this| this.amount)
            .unwrap_or_default();

        let bonded_ration = self.keeper.bonded_ratio(ctx);

        //
        minter.inflation = match minter.next_inflation_rate(&params, bonded_ration) {
            Some(inflation) => inflation,
            None => panic!(
                "Failed to `begin_block` in {} Reason: overflow while calculate inflation",
                MI::NAME
            ),
        };

        minter.annual_provisions =
            match minter.next_annual_provision(Decimal256::new(total_staking_supply)) {
                Some(provisions) => provisions,
                None => panic!(
                    "Failed to `begin_block` in {} Reason: overflow while calculate next annual provision",
                    MI::NAME
                ),
            };

        self.keeper.minter_set(ctx, &minter);

        //
        let minted_coin = minter.block_provision(&params).expect("overflow");
        let minted_attribute =
            EventAttribute::new("amount".into(), minted_coin.amount.to_string().into(), true);

        let minted_coins = match UnsignedCoins::new([minted_coin]) {
            Ok(minted_coins) => minted_coins,
            Err(_) => {
                tracing::info!("No suitable coin for minting found");

                return;
            }
        };

        if let Err(err) = self.keeper.mint_coins(ctx, minted_coins.clone()) {
            panic!(
                "Failed to `begin_block` in {} Reason: error minting coins {err}",
                MI::NAME
            )
        }

        if let Err(err) = self.keeper.collect_fees(ctx, minted_coins) {
            panic!(
                "Failed to `begin_block` in {} Reason: error collecting fees: {err}",
                MI::NAME
            )
        }

        ctx.push_event(Event::new(
            "mint",
            [
                EventAttribute::new(
                    "bonded_ratio".into(),
                    bonded_ration.to_cosmos_proto_string().into(),
                    true,
                ),
                EventAttribute::new(
                    "inflation".into(),
                    minter.inflation.to_cosmos_proto_string().into(),
                    true,
                ),
                EventAttribute::new(
                    "annual_provisions".into(),
                    minter.annual_provisions.to_cosmos_proto_string().into(),
                    true,
                ),
                minted_attribute,
            ],
        ));
    }
}
