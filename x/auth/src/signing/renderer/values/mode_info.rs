use database::RocksDB;
use gears::types::context::context::Context;
use proto_messages::cosmos::{tx::v1beta1::screen::Screen, ibc_types::tx::ModeInfo};
use store::StoreKey;

use crate::signing::renderer::value_renderer::ValueRenderer;

impl<DefaultValueRenderer, SK: StoreKey> ValueRenderer<DefaultValueRenderer, SK> for ModeInfo {
    fn format(
        &self,
        _ctx: &Context<'_, '_, RocksDB, SK>,
    ) -> Result<Vec<Screen>, Box<dyn std::error::Error>> {
        // I don't see that mode ino is used in screen formatin for now, but leave this as things may change
        Ok(Vec::new())
    }
}
