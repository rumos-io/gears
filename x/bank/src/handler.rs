use database::Database;
use gears::{error::AppError, types::context_v2::Context, x::params::ParamsSubspaceKey};
use store::StoreKey;

use crate::{Keeper, Message};

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
        ctx: &mut Context<DB, SK>,
        msg: &Message,
    ) -> Result<(), AppError> {
        match msg {
            Message::Send(msg_send) => self
                .keeper
                .send_coins_from_account_to_account(ctx, msg_send),
        }
    }
}
