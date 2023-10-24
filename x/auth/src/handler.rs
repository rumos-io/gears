use database::Database;
use gears::types::context::context::Context;
use gears::types::context::init_context::InitContext;
use gears::types::context::query_context::QueryContext;
use gears::{error::AppError, x::params::ParamsSubspaceKey};
use ibc_proto::protobuf::Protobuf;
use proto_messages::cosmos::auth::v1beta1::{BaseAccount, QueryAccountRequest};
use proto_types::AccAddress;
use store::StoreKey;

use crate::{GenesisState, Keeper, Message};

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
        _ctx: &mut Context<DB, SK>,
        _msg: &Message,
    ) -> Result<(), AppError> {
        Ok(())
    }

    pub fn handle_query<DB: Database>(
        &self,
        ctx: &QueryContext<DB, SK>,
        query: tendermint_proto::abci::RequestQuery,
    ) -> std::result::Result<bytes::Bytes, AppError> {
        match query.path.as_str() {
            "/cosmos.auth.v1beta1.Query/Account" => {
                let req = QueryAccountRequest::decode(query.data)
                    .map_err(|e| AppError::InvalidRequest(e.to_string()))?;

                self.keeper
                    .query_account(&ctx, req)
                    .map(|res| res.encode_vec().into())
            }
            _ => Err(AppError::InvalidRequest("query path not found".into())),
        }
    }

    pub fn init_genesis<DB: Database>(&self, ctx: &mut InitContext<DB, SK>, genesis: GenesisState) {
        self.keeper.init_genesis(ctx, genesis)
    }

    pub fn handle_add_genesis_account(
        &self,
        genesis_state: &mut GenesisState,
        address: AccAddress,
    ) -> Result<(), AppError> {
        let mut contains = false;
        for acct in &genesis_state.accounts {
            if acct.address == address {
                contains = true;
                break;
            }
        }

        if !contains {
            genesis_state.accounts.push(BaseAccount {
                address,
                pub_key: None,
                account_number: 0, // This is ignored when initializing from genesis
                sequence: 0,       //TODO: make a BaseAccount constructor
            });
            Ok(())
        } else {
            Err(AppError::Genesis(format!(
                "cannot add account at existing address {}",
                address
            )))
        }
    }
}
