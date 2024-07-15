use gears::context::{init::InitContext, query::QueryContext, tx::TxContext};
use gears::core::errors::CoreError as IbcError;
use gears::error::AppError;
use gears::error::IBC_ENCODE_UNWRAP;
use gears::ext::Pagination;
use gears::params::ParamsSubspaceKey;
use gears::rest::response::PaginationResponse;
use gears::store::database::Database;
use gears::store::StoreKey;
use gears::tendermint::types::proto::Protobuf;
use gears::tendermint::types::request::query::RequestQuery;
use gears::types::query::metadata::{
    QueryDenomMetadataRequest, QueryDenomMetadataResponse, QueryDenomsMetadataRequest,
};
use gears::types::store::gas::ext::GasResultExt;
use gears::x::keepers::auth::AuthKeeper;
use gears::x::keepers::bank::BankKeeper;
use gears::x::module::Module;
use serde::Serialize;

use crate::types::query::{
    QueryAllBalancesRequest, QueryAllBalancesResponse, QueryBalanceRequest, QueryBalanceResponse,
    QueryDenomsMetadataResponse, QueryTotalSupplyRequest, QueryTotalSupplyResponse,
};
use crate::{GenesisState, Keeper, Message};

#[derive(Debug, Clone)]
pub struct ABCIHandler<SK: StoreKey, PSK: ParamsSubspaceKey, AK: AuthKeeper<SK, M>, M: Module> {
    keeper: Keeper<SK, PSK, AK, M>,
}

#[derive(Clone)]
pub enum BankNodeQueryRequest {
    Balance(QueryBalanceRequest),
    AllBalances(QueryAllBalancesRequest),
    TotalSupply(QueryTotalSupplyRequest),
    DenomsMetadata(QueryDenomsMetadataRequest),
    DenomMetadata(QueryDenomMetadataRequest),
}

#[derive(Clone, Serialize)]
#[serde(untagged)]
pub enum BankNodeQueryResponse {
    Balance(QueryBalanceResponse),
    AllBalances(QueryAllBalancesResponse),
    TotalSupply(QueryTotalSupplyResponse),
    DenomsMetadata(QueryDenomsMetadataResponse),
    DenomMetadata(QueryDenomMetadataResponse),
}

impl<'a, SK: StoreKey, PSK: ParamsSubspaceKey, AK: AuthKeeper<SK, M>, M: Module>
    ABCIHandler<SK, PSK, AK, M>
{
    pub fn new(keeper: Keeper<SK, PSK, AK, M>) -> Self {
        ABCIHandler { keeper }
    }

    pub fn typed_query<DB: Database + Send + Sync>(
        &self,
        ctx: &QueryContext<DB, SK>,
        query: BankNodeQueryRequest,
    ) -> BankNodeQueryResponse {
        match query {
            BankNodeQueryRequest::Balance(req) => {
                let res = self.keeper.query_balance(ctx, req);
                BankNodeQueryResponse::Balance(res)
            }
            BankNodeQueryRequest::AllBalances(req) => {
                BankNodeQueryResponse::AllBalances(self.query_balances(ctx, req))
            }
            BankNodeQueryRequest::TotalSupply(req) => {
                BankNodeQueryResponse::TotalSupply(self.query_total_supply(ctx, req))
            }
            BankNodeQueryRequest::DenomsMetadata(req) => {
                BankNodeQueryResponse::DenomsMetadata(self.query_denoms(ctx, req))
            }
            BankNodeQueryRequest::DenomMetadata(req) => {
                let metadata = self
                    .keeper
                    .get_denom_metadata(ctx, &req.denom)
                    .expect("Query ctx doesn't have any gas");
                BankNodeQueryResponse::DenomMetadata(QueryDenomMetadataResponse { metadata })
            }
        }
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
        ctx: &QueryContext<DB, SK>,
        query: RequestQuery,
    ) -> std::result::Result<bytes::Bytes, AppError> {
        match query.path.as_str() {
            QueryAllBalancesRequest::TYPE_URL => {
                let req = QueryAllBalancesRequest::decode(query.data)
                    .map_err(|e| IbcError::DecodeProtobuf(e.to_string()))?;

                let result = self.query_balances(ctx, req);

                Ok(result.encode_vec().expect(IBC_ENCODE_UNWRAP).into())
            }
            QueryTotalSupplyRequest::TYPE_URL => {
                let req = QueryTotalSupplyRequest::decode(query.data)
                    .map_err(|e| IbcError::DecodeProtobuf(e.to_string()))?;

                Ok(self
                    .query_total_supply(ctx, req)
                    .encode_vec()
                    .expect(IBC_ENCODE_UNWRAP)
                    .into())
            }
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
            QueryDenomsMetadataRequest::TYPE_URL => {
                let req = QueryDenomsMetadataRequest::decode(query.data)
                    .map_err(|e| IbcError::DecodeProtobuf(e.to_string()))?;

                let result = self
                    .query_denoms(ctx, req)
                    .encode_vec()
                    .expect(IBC_ENCODE_UNWRAP);

                Ok(result.into())
            }
            "/cosmos.bank.v1beta1.Query/DenomMetadata" => {
                let req = QueryDenomMetadataRequest::decode(query.data)
                    .map_err(|e| IbcError::DecodeProtobuf(e.to_string()))?;
                let metadata = self
                    .keeper
                    .get_denom_metadata(ctx, &req.denom)
                    .expect("Query ctx doesn't have any gas");
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

    fn query_balances<DB: Database>(
        &self,
        ctx: &QueryContext<DB, SK>,
        QueryAllBalancesRequest {
            address,
            pagination,
        }: QueryAllBalancesRequest,
    ) -> QueryAllBalancesResponse {
        let paginate = pagination.is_some();
        let (total, balances) = self
            .keeper
            .all_balances(ctx, address, pagination.map(Pagination::from))
            .unwrap_gas();

        QueryAllBalancesResponse {
            balances,
            pagination: match paginate {
                true => Some(PaginationResponse::new(total)),
                false => None,
            },
        }
    }

    fn query_denoms<DB: Database>(
        &self,
        ctx: &QueryContext<DB, SK>,
        QueryDenomsMetadataRequest { pagination }: QueryDenomsMetadataRequest,
    ) -> QueryDenomsMetadataResponse {
        let paginate = pagination.is_some();

        let (total, metadatas) = self
            .keeper
            .denoms_metadata(ctx, pagination.map(Pagination::from));

        QueryDenomsMetadataResponse {
            metadatas,
            pagination: match paginate {
                true => Some(PaginationResponse::new(total)),
                false => None,
            },
        }
    }

    fn query_total_supply<DB: Database>(
        &self,
        ctx: &QueryContext<DB, SK>,
        QueryTotalSupplyRequest { pagination }: QueryTotalSupplyRequest,
    ) -> QueryTotalSupplyResponse {
        let paginate = pagination.is_some();

        let (total, supply) = self
            .keeper
            .total_supply(ctx, pagination.map(Pagination::from));

        QueryTotalSupplyResponse {
            supply,
            pagination: match paginate {
                true => Some(PaginationResponse::new(total)),
                false => None,
            },
        }
    }
}
