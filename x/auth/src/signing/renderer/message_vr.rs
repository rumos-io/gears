use database::RocksDB;
use gears::types::context::context::Context;
use proto_messages::cosmos::tx::v1beta1::screen::Screen;
use store::StoreKey;

use crate::signing::errors::SigningErrors;

use super::vr_trait::ValueRendererTrait;

pub trait MessageValueRendererTrait<V, SK: StoreKey>:
    ValueRendererTrait<V, SK>
{
}

pub struct MessageValueRenderer;

impl<V, SK: StoreKey> MessageValueRendererTrait<V, SK> for MessageValueRenderer {}

impl<Envelope, SK: StoreKey> ValueRendererTrait<Envelope, SK>
    for MessageValueRenderer
{
    fn format(
        _ctx: &Context<'_, '_, RocksDB, SK>,
        _value: Envelope,
    ) -> Result<Vec<Screen>, SigningErrors> {
        // _Note:_ Cosmos.SDK checks that value name == expected name(in terms of protobuf), but we implemented this trait only for Envelope struct so we could omit this
        let screens = Vec::new();

        let _first_screen_content = format!( "{} object", ""  ); //TODO: its seems like here we need protoreflect name e.g. "google.protobuf.Any"

        Ok( screens)
    }

    fn parse(
        _ctx: &Context<'_, '_, database::RocksDB, SK>,
        _screens: impl IntoIterator<Item = Screen>,
    ) -> Result<Envelope, SigningErrors> {
        unimplemented!()
    }
}
