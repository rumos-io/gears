use database::Database;
use gears::types::context::context::Context;
use gears::types::context::init_context::InitContext;
use gears::types::context::query_context::QueryContext;
use gears::types::context::tx_context::TxContext;
use gears::{error::AppError, x::params::ParamsSubspaceKey};
use ibc_proto::protobuf::Protobuf;
use proto_messages::cosmos::{
    bank::v1beta1::{QueryAllBalancesRequest, QueryBalanceRequest, QueryTotalSupplyResponse},
    base::v1beta1::SendCoins,
};
use proto_types::AccAddress;
use store::StoreKey;

use crate::{Balance, GenesisState, Keeper, Message};

#[derive(Debug, Clone)]
pub struct Handler<SK: StoreKey, PSK: ParamsSubspaceKey> {
    keeper: Keeper<SK, PSK>,
}

impl<SK: StoreKey, PSK: ParamsSubspaceKey> Handler<SK, PSK> {
    pub fn new(keeper: Keeper<SK, PSK>) -> Self {
        Handler { keeper }
    }

    pub fn handle<DB: Database>(
        &self,
        ctx: &mut TxContext<'_, DB, SK>,
        msg: &Message,
    ) -> Result<(), AppError> {
        let mut ctx = Context::TxContext(ctx);

        match msg {
            Message::Send(msg_send) => self
                .keeper
                .send_coins_from_account_to_account(&mut ctx, msg_send),
        }
    }

    pub fn handle_query<DB: Database>(
        &self,
        ctx: &QueryContext<'_, DB, SK>,
        query: tendermint_proto::abci::RequestQuery,
    ) -> std::result::Result<bytes::Bytes, AppError> {
        match query.path.as_str() {
            "/cosmos.bank.v1beta1.Query/AllBalances" => {
                let req = QueryAllBalancesRequest::decode(query.data)?;

                Ok(self
                    .keeper
                    .query_all_balances(&ctx, req)
                    .encode_vec()
                    .into())
            }
            "/cosmos.bank.v1beta1.Query/TotalSupply" => Ok(QueryTotalSupplyResponse {
                supply: self.keeper.get_paginated_total_supply(&ctx),
                pagination: None,
            }
            .encode_vec()
            .into()),
            "/cosmos.bank.v1beta1.Query/Balance" => {
                let req = QueryBalanceRequest::decode(query.data)?;

                Ok(self.keeper.query_balance(&ctx, req).encode_vec().into())
            }

            _ => Err(AppError::InvalidRequest("query path not found".into())),
        }
    }

    pub fn init_genesis<DB: Database>(&self, ctx: &mut InitContext<'_, DB, SK>, genesis: GenesisState) {
        self.keeper.init_genesis(ctx, genesis)
    }

    /// NOTE: If the genesis_state already contains an entry for the given address then this method
    /// will add another entry to the list i.e. it does not merge entries
    pub fn handle_add_genesis_account(
        &self,
        genesis_state: &mut GenesisState,
        address: AccAddress,
        coins: SendCoins,
    ) {
        genesis_state.balances.push(Balance { address, coins })
    }
}
