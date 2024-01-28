use database::Database;
use gears::types::context::init_context::InitContext;
use gears::types::context::query_context::QueryContext;
use gears::types::context::tx_context::TxContext;
use gears::{error::AppError, x::params::ParamsSubspaceKey};
use proto_messages::cosmos::bank::v1beta1::{
    QueryAllBalancesRequest, QueryBalanceRequest, QueryTotalSupplyResponse,
};
use proto_messages::cosmos::ibc_types::protobuf::Protobuf;
use store::StoreKey;

use crate::{GenesisState, Keeper, Message};

#[derive(Debug, Clone)]
pub struct ABCIHandler<SK: StoreKey, PSK: ParamsSubspaceKey> {
    keeper: Keeper<SK, PSK>,
}

impl<SK: StoreKey, PSK: ParamsSubspaceKey> ABCIHandler<SK, PSK> {
    pub fn new(keeper: Keeper<SK, PSK>) -> Self {
        ABCIHandler { keeper }
    }

    pub fn tx<DB: Database>(
        &self,
        ctx: &mut TxContext<'_, DB, SK>,
        msg: &Message,
    ) -> Result<(), AppError> {
        match msg {
            Message::Send(msg_send) => self
                .keeper
                .send_coins_from_account_to_account(&mut ctx.as_any(), msg_send),
        }
    }

    pub fn query<DB: Database>(
        &self,
        ctx: &QueryContext<'_, DB, SK>,
        query: tendermint::proto::abci::RequestQuery,
    ) -> std::result::Result<bytes::Bytes, AppError> {
        match query.path.as_str() {
            "/cosmos.bank.v1beta1.Query/AllBalances" => {
                let req = QueryAllBalancesRequest::decode(query.data)?;

                Ok(self.keeper.query_all_balances(ctx, req).encode_vec().into())
            }
            "/cosmos.bank.v1beta1.Query/TotalSupply" => Ok(QueryTotalSupplyResponse {
                supply: self.keeper.get_paginated_total_supply(ctx),
                pagination: None,
            }
            .encode_vec()
            .into()),
            "/cosmos.bank.v1beta1.Query/Balance" => {
                let req = QueryBalanceRequest::decode(query.data)?;

                Ok(self.keeper.query_balance(ctx, req).encode_vec().into())
            }
            "/cosmos.bank.v1beta1.Query/DenomsMetadata" => {
                Ok(self.keeper.query_denoms_metadata(ctx).encode_vec().into())
            }
            _ => Err(AppError::InvalidRequest("query path not found".into())),
        }
    }

    pub fn genesis<DB: Database>(&self, ctx: &mut InitContext<'_, DB, SK>, genesis: GenesisState) {
        self.keeper.init_genesis(ctx, genesis)
    }
}
