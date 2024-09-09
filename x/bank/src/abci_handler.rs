use std::marker::PhantomData;

use gears::application::handlers::node::{ABCIHandler, ModuleInfo, TxError};
use gears::baseapp::errors::QueryError;
use gears::baseapp::QueryRequest;
use gears::context::{init::InitContext, query::QueryContext, tx::TxContext};
use gears::core::Protobuf;
use gears::derive::Query;
use gears::ext::Pagination;
use gears::params::ParamsSubspaceKey;
use gears::store::database::Database;
use gears::store::StoreKey;
use gears::tendermint::types::request::query::RequestQuery;
use gears::types::pagination::response::PaginationResponse;
use gears::types::store::gas::ext::GasResultExt;
use gears::x::keepers::auth::AuthKeeper;
use gears::x::keepers::bank::BankKeeper;
use gears::x::module::Module;
use serde::Serialize;

use crate::errors::BankTxError;
use crate::types::query::{
    QueryAllBalancesRequest, QueryAllBalancesResponse, QueryBalanceRequest, QueryBalanceResponse,
    QueryDenomMetadataRequest, QueryDenomMetadataResponse, QueryDenomsMetadataRequest,
    QueryDenomsMetadataResponse, QueryParamsRequest, QueryParamsResponse,
    QuerySpendableBalancesRequest, QuerySpendableBalancesResponse, QuerySupplyOfRequest,
    QuerySupplyOfResponse, QueryTotalSupplyRequest, QueryTotalSupplyResponse,
};
use crate::{GenesisState, Keeper, Message};

#[derive(Debug, Clone)]
pub struct BankABCIHandler<
    SK: StoreKey,
    PSK: ParamsSubspaceKey,
    AK: AuthKeeper<SK, M>,
    M: Module,
    MI,
> {
    keeper: Keeper<SK, PSK, AK, M>,
    phantom_data: PhantomData<MI>,
}

#[derive(Clone, Debug, Query)]
pub enum BankNodeQueryRequest {
    Balance(QueryBalanceRequest),
    AllBalances(QueryAllBalancesRequest),
    TotalSupply(QueryTotalSupplyRequest),
    DenomsMetadata(QueryDenomsMetadataRequest),
    DenomMetadata(QueryDenomMetadataRequest),
    Params(QueryParamsRequest),
    SupplyOf(QuerySupplyOfRequest),
    Spendable(QuerySpendableBalancesRequest),
}

impl QueryRequest for BankNodeQueryRequest {
    fn height(&self) -> u32 {
        todo!()
    }
}

#[derive(Clone, Debug, Serialize, Query)]
#[serde(untagged)]
pub enum BankNodeQueryResponse {
    Balance(QueryBalanceResponse),
    AllBalances(QueryAllBalancesResponse),
    TotalSupply(QueryTotalSupplyResponse),
    DenomsMetadata(QueryDenomsMetadataResponse),
    DenomMetadata(QueryDenomMetadataResponse),
    Params(QueryParamsResponse),
    SupplyOf(QuerySupplyOfResponse),
    Spendable(QuerySpendableBalancesResponse),
}

impl<
        SK: StoreKey,
        PSK: ParamsSubspaceKey,
        AK: AuthKeeper<SK, M> + Send + Sync + 'static,
        M: Module,
        MI: ModuleInfo + Clone + Send + Sync + 'static,
    > ABCIHandler for BankABCIHandler<SK, PSK, AK, M, MI>
{
    type Message = Message;

    type Genesis = GenesisState;

    type StoreKey = SK;

    type QReq = BankNodeQueryRequest;

    type QRes = BankNodeQueryResponse;

    fn typed_query<DB: Database>(
        &self,
        ctx: &QueryContext<DB, Self::StoreKey>,
        query: Self::QReq,
    ) -> Self::QRes {
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
            BankNodeQueryRequest::Params(_req) => {
                BankNodeQueryResponse::Params(QueryParamsResponse {
                    params: self.keeper.params(ctx),
                })
            }
            BankNodeQueryRequest::SupplyOf(req) => {
                BankNodeQueryResponse::SupplyOf(self.query_supply_of(ctx, req))
            }
            BankNodeQueryRequest::Spendable(QuerySpendableBalancesRequest {
                address,
                pagination,
            }) => {
                // TODO: edit error "handling"
                let (spendable, pagination_result) = self
                    .keeper
                    .spendable_coins(ctx, &address, pagination.map(Pagination::from))
                    .map(|(spendable, _, pag)| {
                        (spendable.map(Vec::from), pag.map(PaginationResponse::from))
                    })
                    .unwrap_or_default();

                BankNodeQueryResponse::Spendable(QuerySpendableBalancesResponse {
                    balances: spendable.unwrap_or_default(),
                    pagination: pagination_result,
                })
            }
        }
    }

    fn run_ante_checks<DB: Database>(
        &self,
        _: &mut TxContext<'_, DB, Self::StoreKey>,
        _: &gears::types::tx::raw::TxWithRaw<Self::Message>,
        _: bool,
    ) -> Result<(), TxError> {
        Ok(())
    }

    fn msg<DB: Database>(
        &self,
        ctx: &mut TxContext<'_, DB, Self::StoreKey>,
        msg: &Self::Message,
    ) -> Result<(), TxError> {
        let result = match msg {
            Message::Send(msg_send) => self
                .keeper
                .send_coins_from_account_to_account(ctx, msg_send),
        };

        result.map_err(|e| Into::<BankTxError>::into(e).into::<MI>())
    }

    fn init_genesis<DB: Database>(
        &self,
        ctx: &mut InitContext<'_, DB, Self::StoreKey>,
        genesis: Self::Genesis,
    ) -> Vec<gears::tendermint::types::proto::validator::ValidatorUpdate> {
        self.genesis(ctx, genesis);

        Vec::new()
    }

    fn query<DB: Database + Send + Sync>(
        &self,
        ctx: &QueryContext<DB, Self::StoreKey>,
        query: RequestQuery,
    ) -> Result<Vec<u8>, QueryError> {
        match query.path.as_str() {
            QueryAllBalancesRequest::QUERY_URL => {
                let req = QueryAllBalancesRequest::decode(query.data)?;

                let result = self.query_balances(ctx, req);

                Ok(result.encode_vec().into())
            }
            QueryTotalSupplyRequest::QUERY_URL => {
                let req = QueryTotalSupplyRequest::decode(query.data)?;

                Ok(self.query_total_supply(ctx, req).encode_vec().into())
            }
            "/cosmos.bank.v1beta1.Query/Balance" => {
                let req = QueryBalanceRequest::decode(query.data)?;

                Ok(self.keeper.query_balance(ctx, req).encode_vec().into())
            }
            QueryDenomsMetadataRequest::QUERY_URL => {
                let req = QueryDenomsMetadataRequest::decode(query.data)?;

                let result = self.query_denoms(ctx, req).encode_vec();

                Ok(result.into())
            }
            "/cosmos.bank.v1beta1.Query/DenomMetadata" => {
                let req = QueryDenomMetadataRequest::decode(query.data)?;
                let metadata = self.keeper.get_denom_metadata(ctx, &req.denom).unwrap_gas();
                Ok(QueryDenomMetadataResponse { metadata }.encode_vec().into())
            }
            "/cosmos.bank.v1beta1.Query/Params" => {
                // a kind of type check
                let _req = QueryParamsRequest::decode(query.data)?;
                let params = self.keeper.params(ctx);
                Ok(QueryParamsResponse { params }.encode_vec().into())
            }
            _ => Err(QueryError::PathNotFound),
        }
    }
}

impl<SK: StoreKey, PSK: ParamsSubspaceKey, AK: AuthKeeper<SK, M>, M: Module, MI: ModuleInfo>
    BankABCIHandler<SK, PSK, AK, M, MI>
{
    pub fn new(keeper: Keeper<SK, PSK, AK, M>) -> Self {
        BankABCIHandler {
            keeper,
            phantom_data: PhantomData,
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
        let (p_result, balances) = self
            .keeper
            .all_balances(ctx, address, pagination.map(Pagination::from))
            .unwrap_gas();

        QueryAllBalancesResponse {
            balances,
            pagination: p_result.map(PaginationResponse::from),
        }
    }

    fn query_denoms<DB: Database>(
        &self,
        ctx: &QueryContext<DB, SK>,
        QueryDenomsMetadataRequest { pagination }: QueryDenomsMetadataRequest,
    ) -> QueryDenomsMetadataResponse {
        let (p_result, metadatas) = self
            .keeper
            .denoms_metadata(ctx, pagination.map(Pagination::from));

        QueryDenomsMetadataResponse {
            metadatas,
            pagination: p_result.map(PaginationResponse::from),
        }
    }

    fn query_total_supply<DB: Database>(
        &self,
        ctx: &QueryContext<DB, SK>,
        QueryTotalSupplyRequest { pagination }: QueryTotalSupplyRequest,
    ) -> QueryTotalSupplyResponse {
        let (p_result, supply) = self
            .keeper
            .total_supply(ctx, pagination.map(Pagination::from));

        QueryTotalSupplyResponse {
            supply,
            pagination: p_result.map(PaginationResponse::from),
        }
    }

    fn query_supply_of<DB: Database>(
        &self,
        ctx: &QueryContext<DB, SK>,
        QuerySupplyOfRequest { denom }: QuerySupplyOfRequest,
    ) -> QuerySupplyOfResponse {
        let supply = self.keeper.supply(ctx, &denom).unwrap_gas();
        QuerySupplyOfResponse { amount: supply }
    }
}
