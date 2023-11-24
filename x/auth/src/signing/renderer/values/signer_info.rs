use database::RocksDB;
use gears::types::context::context::Context;
use proto_messages::cosmos::tx::v1beta1::{
    screen::{Content, Indent, Screen},
    signer::SignerInfo,
};
use store::StoreKey;

use crate::signing::renderer::value_renderer::{
    DefaultPrimitiveRenderer, PrimitiveValueRenderer, ValueRenderer,
};

impl<DefaultValueRenderer, SK: StoreKey> ValueRenderer<DefaultValueRenderer, SK> for SignerInfo {
    fn format(
        &self,
        ctx: &Context<'_, '_, RocksDB, SK>,
    ) -> Result<Vec<Screen>, Box<dyn std::error::Error>> {
        let SignerInfo {
            public_key,
            mode_info,
            sequence,
        } = &self;

        let mut final_screens = Vec::<Screen>::new();
        if let Some(public_key) = public_key {
            final_screens.append(&mut ValueRenderer::<DefaultValueRenderer, SK>::format(
                public_key, ctx,
            )?)
        }

        if let Some(mode_info) = mode_info {
            final_screens.append(&mut ValueRenderer::<DefaultValueRenderer, SK>::format(
                mode_info, ctx,
            )?)
        }

        final_screens.push(Screen {
            title: "Sequence".to_string(),
            content: Content::new(DefaultPrimitiveRenderer::format(*sequence))?,
            indent: Some(Indent::new(2)?),
            expert: true,
        });

        Ok(final_screens)
    }
}
