use database::Database;
use gears::{error::AppError, types::context_v2::Context};
use store::StoreKey;

use crate::{Keeper, Message};

#[derive(Debug, Clone)]
pub struct Handler<SK: StoreKey> {
    keeper: Keeper<SK>,
}

impl<SK: StoreKey> Handler<SK> {
    pub fn new(keeper: Keeper<SK>) -> Self {
        Handler { keeper }
    }

    pub fn handle<DB: Database>(
        &self,
        ctx: &mut Context<DB, SK>,
        msg: &Message,
    ) -> Result<(), AppError> {
        match msg {
            Message::Send(msg_send) => self.keeper.send_coins(ctx, msg_send),
        }
    }
}
