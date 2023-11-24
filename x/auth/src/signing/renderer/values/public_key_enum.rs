use database::RocksDB;
use gears::types::context::context::Context;
use proto_messages::cosmos::tx::v1beta1::{public_key::PublicKey, screen::Screen};
use store::StoreKey;

use crate::signing::renderer::value_renderer::ValueRenderer;

impl<DefaultValueRenderer, SK: StoreKey> ValueRenderer<DefaultValueRenderer, SK> for PublicKey {
    fn format(
        &self,
        ctx: &Context<'_, '_, RocksDB, SK>,
    ) -> Result<Vec<Screen>, Box<dyn std::error::Error>> {
        // I prefer to implement formating for each key in own module to keep things as small as possible
        match self {
            PublicKey::Secp256k1(key) => {
                ValueRenderer::<DefaultValueRenderer, SK>::format(key, ctx)
            }
        }
    }
}
