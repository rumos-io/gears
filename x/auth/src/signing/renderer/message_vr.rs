use database::RocksDB;
use gears::types::context::context::Context;
use proto_messages::cosmos::tx::v1beta1::{message::Message, screen::Screen};
use store::StoreKey;

use crate::signing::errors::SigningErrors;

use super::vr_trait::ValueRendererTrait;

pub trait MessageValueRendererTrait<V, SK: StoreKey, M: Message>:
    ValueRendererTrait<V, SK, M>
{
}

pub struct MessageValueRenderer;

impl<V, SK: StoreKey, M: Message> MessageValueRendererTrait<V, SK, M> for MessageValueRenderer {}

impl<Envelope, SK: StoreKey, M: Message> ValueRendererTrait<Envelope, SK, M>
    for MessageValueRenderer
{
    fn format(
        _ctx: &Context<'_, '_, RocksDB, SK>,
        _value: Envelope,
    ) -> Result<Vec<Screen>, SigningErrors> {
        todo!()
    }

    fn parse(
        _ctx: &Context<'_, '_, database::RocksDB, SK>,
        _screens: impl IntoIterator<Item = Screen>,
    ) -> Result<Envelope, SigningErrors> {
        todo!()
    }
}
