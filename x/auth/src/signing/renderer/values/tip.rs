use database::RocksDB;
use gears::types::context::context::Context;
use proto_messages::cosmos::tx::v1beta1::{
    screen::{Content, Screen},
    tip::Tip,
};
use store::StoreKey;

use crate::signing::renderer::value_renderer::ValueRenderer;

impl<DefaultValueRenderer, SK: StoreKey> ValueRenderer<DefaultValueRenderer, SK> for Tip {
    fn format(
        &self,
        ctx: &Context<'_, '_, RocksDB, SK>,
    ) -> Result<Vec<Screen>, Box<dyn std::error::Error>> {
        let Tip { amount, tipper } = &self;

        if let Some(amount) = amount {
            let mut screens = ValueRenderer::<DefaultValueRenderer, SK>::format(amount, ctx)?;

            screens.push(Screen {
                title: "Tipper".to_string(),
                content: Content::new(tipper.as_hex())?,
                indent: None,
                expert: false,
            });

            Ok(screens)
        } else {
            Ok(Vec::new())
        }
    }
}
