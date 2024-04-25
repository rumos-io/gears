use gears::core::errors::Error as IbcError;
use gears::error::IBC_ENCODE_UNWRAP;
use gears::store::database::Database;
use gears::store::StoreKey;
use gears::tendermint::types::proto::Protobuf;
use gears::tendermint::types::request::query::RequestQuery;
use gears::types::context::init::InitContext;
use gears::types::context::query::QueryContext;
use gears::types::context::TransactionalContext;
use gears::types::query::metadata::{QueryDenomMetadataRequest, QueryDenomMetadataResponse};
use gears::x::keepers::auth::AuthKeeper;
use gears::x::keepers::bank::BankKeeper;
use gears::{error::AppError, params::ParamsSubspaceKey};

use crate::types::query::{QueryAllBalancesRequest, QueryBalanceRequest, QueryTotalSupplyResponse};
use crate::{GenesisState, Keeper, Message};

#[derive(Debug, Clone)]
pub struct ABCIHandler<SK: StoreKey, PSK: ParamsSubspaceKey, AK: AuthKeeper<SK>> {
    keeper: Keeper<SK, PSK, AK>,
}

impl<'a, SK: StoreKey, PSK: ParamsSubspaceKey, AK: AuthKeeper<SK>> ABCIHandler<SK, PSK, AK> {
    pub fn new(keeper: Keeper<SK, PSK, AK>) -> Self {
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
                .send_coins_from_account_to_account(ctx, msg_send),
        }
    }

    pub fn query<DB: Database>(
        &self,
        ctx: &QueryContext<'a, DB, SK>,
        query: RequestQuery,
    ) -> std::result::Result<bytes::Bytes, AppError> {
        match query.path.as_str() {
            "/cosmos.bank.v1beta1.Query/AllBalances" => {
                let req = QueryAllBalancesRequest::decode(query.data)
                    .map_err(|e| IbcError::DecodeProtobuf(e.to_string()))?;

                Ok(self
                    .keeper
                    .query_all_balances(ctx, req)
                    .encode_vec()
                    .expect(IBC_ENCODE_UNWRAP)
                    .into()) // TODO:IBC
            }
            "/cosmos.bank.v1beta1.Query/TotalSupply" => Ok(QueryTotalSupplyResponse {
                supply: self.keeper.get_paginated_total_supply(ctx),
                pagination: None,
            }
            .encode_vec()
            .expect(IBC_ENCODE_UNWRAP)
            .into()), // TODO:IBC
            "/cosmos.bank.v1beta1.Query/Balance" => {
                let req = QueryBalanceRequest::decode(query.data)
                    .map_err(|e| IbcError::DecodeProtobuf(e.to_string()))?;

                Ok(self
                    .keeper
                    .query_balance(ctx, req)
                    .encode_vec()
                    .expect(IBC_ENCODE_UNWRAP)
                    .into()) // TODO:IBC
            }
            "/cosmos.bank.v1beta1.Query/DenomsMetadata" => {
                Ok(self
                    .keeper
                    .query_denoms_metadata(ctx)
                    .encode_vec()
                    .expect(IBC_ENCODE_UNWRAP)
                    .into()) // TODO:IBC
            }
            "/cosmos.bank.v1beta1.Query/DenomMetadata" => {
                let req = QueryDenomMetadataRequest::decode(query.data)
                    .map_err(|e| IbcError::DecodeProtobuf(e.to_string()))?;
                let metadata = self.keeper.get_denom_metadata(ctx, &req.denom);
                Ok(QueryDenomMetadataResponse { metadata }
                    .encode_vec()
                    .expect(IBC_ENCODE_UNWRAP)
                    .into()) // TODO:IBC
            }
            _ => Err(AppError::InvalidRequest("query path not found".into())),
        }
    }

    pub fn genesis<DB: Database>(&self, ctx: &mut InitContext<'_, DB, SK>, genesis: GenesisState) {
        self.keeper.init_genesis(ctx, genesis)
    }
}
