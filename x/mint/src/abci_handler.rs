use std::marker::PhantomData;

use gears::{
    application::{
        handlers::node::{ABCIHandler, ModuleInfo},
        keepers::params::ParamsKeeper,
    },
    baseapp::{NullQueryRequest, NullQueryResponse},
    context::TransactionalContext,
    params::ParamsSubspaceKey,
    store::StoreKey,
    tendermint::types::proto::event::{Event, EventAttribute},
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

use crate::{genesis::MintGenesis, keeper::MintKeeper, params::MintParamsKeeper};

#[derive(Debug, Clone)]
pub struct MintAbciHandler<SK, PSK, BK, STK, M, MI> {
    keeper: MintKeeper<SK, BK, STK, M>,
    params_keeper: MintParamsKeeper<PSK>,
    _marker: PhantomData<(MI, SK, PSK, M)>,
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

    type QReq = NullQueryRequest;

    type QRes = NullQueryResponse;

    fn typed_query<DB: gears::store::database::Database>(
        &self,
        _ctx: &gears::context::query::QueryContext<DB, Self::StoreKey>,
        _query: Self::QReq,
    ) -> Self::QRes {
        todo!()
    }

    fn msg<DB: gears::store::database::Database>(
        &self,
        _ctx: &mut gears::context::tx::TxContext<'_, DB, Self::StoreKey>,
        _msg: &Self::Message,
    ) -> Result<(), gears::application::handlers::node::TxError> {
        todo!()
    }

    fn init_genesis<DB: gears::store::database::Database>(
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
        _ctx: &gears::context::query::QueryContext<DB, Self::StoreKey>,
        _query: gears::tendermint::types::request::query::RequestQuery,
    ) -> Result<Vec<u8>, gears::baseapp::errors::QueryError> {
        todo!()
    }

    fn begin_block<'a, DB: gears::store::database::Database>(
        &self,
        ctx: &mut gears::context::block::BlockContext<'_, DB, Self::StoreKey>,
        _request: gears::tendermint::request::RequestBeginBlock,
    ) {
        let mut minter = match self.keeper.minter(ctx) {
            Some(minter) => minter,
            None => panic!("failed to get minter"), // TODO: should this be unreachable! ?
        };
        let params = self.params_keeper.get(ctx);
        let total_staking_supply = self
            .keeper
            .staking_token_supply(ctx)
            .expect("overflow")
            .amount;
        let bonded_ration = self.keeper.bonded_ratio(ctx);

        //
        minter.inflation = minter
            .next_inflation_rate(&params, bonded_ration)
            .expect("overflow");
        minter.annual_provisions = minter
            .next_annual_provision(Decimal256::new(total_staking_supply))
            .expect("overflow");

        self.keeper.minter_set(ctx, &minter);

        //
        let minted_coin = minter.block_provision(&params).expect("overflow");
        let minted_attribute =
            EventAttribute::new("amount".into(), minted_coin.amount.to_string().into(), true);
        let minted_coins = UnsignedCoins::new(vec![minted_coin]).expect("invalid coin for minting");

        if let Err(err) = self.keeper.mint_coins(ctx, minted_coins.clone()) {
            panic!("error minting coins {err}")
        }

        if let Err(err) = self.keeper.collect_fees(ctx, minted_coins) {
            panic!("error collecting fees: {err}")
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
