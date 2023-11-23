use database::RocksDB;
use gears::types::context::context::Context;
use proto_messages::cosmos::tx::v1beta1::{auth_info::AuthInfo, screen::Screen};
use store::StoreKey;

use crate::signing::renderer::value_renderer::ValueRenderer;

impl<DefaultValueRenderer, SK: StoreKey> ValueRenderer<DefaultValueRenderer, SK> for AuthInfo {
    fn format(
        &self,
        ctx: &Context<'_, '_, RocksDB, SK>,
    ) -> Result<Vec<Screen>, Box<dyn std::error::Error>> {
        let AuthInfo {
            signer_infos,
            fee,
            tip,
        } = &self;
        let mut final_screens = Vec::<Screen>::new();

        final_screens.append(&mut ValueRenderer::<DefaultValueRenderer, SK>::format(
            fee, ctx,
        )?);
        if let Some(tip) = tip {
            final_screens.append(&mut ValueRenderer::<DefaultValueRenderer, SK>::format(
                tip, ctx,
            )?);
        }

        Ok(final_screens)
    }
}
