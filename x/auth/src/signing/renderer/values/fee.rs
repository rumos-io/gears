use database::RocksDB;
use gears::types::context::context::Context;
use proto_messages::cosmos::tx::v1beta1::{
    fee::Fee,
    screen::{Content, Screen},
};
use store::StoreKey;

use crate::signing::renderer::value_renderer::{
    DefaultPrimitiveRenderer, PrimitiveValueRenderer, ValueRenderer,
};

impl<DefaultValueRenderer, SK: StoreKey> ValueRenderer<DefaultValueRenderer, SK> for Fee {
    fn format(
        &self,
        ctx: &Context<'_, '_, RocksDB, SK>,
    ) -> Result<Vec<Screen>, Box<dyn std::error::Error>> {
        let Fee {
            amount,
            gas_limit,
            payer,
            granter,
        } = &self;

        let mut screens = Vec::<Screen>::new();
        if let Some(amount) = amount {
            screens.append(&mut ValueRenderer::<DefaultValueRenderer, SK>::format(
                amount, ctx,
            )?);
        }
        if let Some(payer) = payer {
            screens.push(Screen {
                title: "Fee payer".to_string(),
                content: Content::new(payer.as_hex())?,
                indent: None,
                expert: true,
            });
        }
        if let Ok(granter) = Content::new(granter) {
            screens.push(Screen {
                title: "Fee granter".to_string(),
                content: granter,
                indent: None,
                expert: true,
            });
        }

        screens.push(Screen {
            title: "Gas limit".to_string(),
            content: Content::new(DefaultPrimitiveRenderer::format(*gas_limit))?,
            indent: None,
            expert: true,
        });

        Ok(screens)
    }
}
