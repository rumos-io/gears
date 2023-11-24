use database::RocksDB;
use gears::types::context::context::Context;
use proto_messages::cosmos::tx::v1beta1::{
    auth_info::AuthInfo,
    screen::{Content, Indent, Screen},
};
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

        let signer_count = signer_infos.len();
        final_screens.push(Screen {
            title: "Other signer".to_string(),
            content: Content::new(match signer_count {
                1 => format!("1 SignerInfo"),
                _ => format!("{signer_count} SignerInfos"),
            })?,
            indent: None,
            expert: true,
        });

        for (i, info) in signer_infos.iter().enumerate() {
            final_screens.push(Screen {
                title: format!("{}/{signer_count}", i + 1),
                content: Content::new("SignerInfo object")?,
                indent: Some(Indent::new(1)?),
                expert: true,
            });
            final_screens.append(&mut ValueRenderer::<DefaultValueRenderer, SK>::format(
                info, ctx,
            )?);
        }

        final_screens.push(Screen {
            title: String::new(),
            content: Content::new("End of Other signer")?,
            indent: None,
            expert: true,
        });

        Ok(final_screens)
    }
}
