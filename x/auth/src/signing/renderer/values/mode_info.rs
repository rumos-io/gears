use database::RocksDB;
use gears::types::context::context::Context;
use ibc_proto::cosmos::tx::v1beta1::ModeInfo;
use proto_messages::cosmos::tx::v1beta1::screen::Screen;
use store::StoreKey;

use crate::signing::renderer::value_renderer::ValueRenderer;

impl<DefaultValueRenderer, SK: StoreKey> ValueRenderer<DefaultValueRenderer, SK> for ModeInfo {
    fn format(
        &self,
        _ctx: &Context<'_, '_, RocksDB, SK>,
    ) -> Result<Vec<Screen>, Box<dyn std::error::Error>> {
        unimplemented!("TODO: Ask Kevin about it")
    }
}
