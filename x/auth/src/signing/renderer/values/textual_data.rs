use database::RocksDB;
use gears::types::context::context::Context;
use proto_messages::cosmos::tx::v1beta1::{
    message::Message,
    screen::{Content, Screen},
    textual_data::TextualData,
};
use store::StoreKey;

use crate::signing::{hasher::hash_get, renderer::value_renderer::ValueRenderer};

impl<DefaultValueRenderer, SK: StoreKey, M: Message + ValueRenderer<DefaultValueRenderer, SK>>
    ValueRenderer<DefaultValueRenderer, SK> for TextualData<M>
{
    fn format(
        &self,
        ctx: &Context<'_, '_, RocksDB, SK>,
    ) -> Result<Vec<Screen>, Box<dyn std::error::Error>> {
        let TextualData {
            body,
            auth_info,
            signer_data,
            body_bytes,
            auth_info_bytes,
        } = &self; // we need to remember using all fields

        let messages_count = body.messages.len();

        let mut screens = Vec::<Screen>::new();

        // =========================
        screens.append(&mut ValueRenderer::<DefaultValueRenderer, SK>::format(
            signer_data,
            ctx,
        )?);

        // Transaction message section
        screens.push(Screen {
            title: String::new(),
            content: Content::new(match messages_count {
                1 => format!("This transaction has 1 Message"),
                _ => format!("This transaction has {} Messages", body.messages.len()),
            })?,
            indent: None,
            expert: false,
        });

        for (i, ms) in body.messages.iter().enumerate() {
            screens.push(Screen {
                title: format!("Message ({}/{messages_count})", i + 1),
                content: Content::new(ms.type_url().to_string())?,
                indent: None,
                expert: false,
            });
            screens.append(&mut ValueRenderer::<DefaultValueRenderer, SK>::format(
                ms, ctx,
            )?);
        }
        screens.push(Screen {
            title: String::new(),
            content: Content::new("End of Message".to_string())?,
            indent: None,
            expert: false,
        });
        if let Ok(memo) = Content::new(body.memo.clone()) {
            screens.push(Screen {
                title: "Memo".to_string(),
                content: memo,
                indent: None,
                expert: false,
            });
        }

        // =========================
        screens.append(&mut ValueRenderer::<DefaultValueRenderer, SK>::format(
            auth_info, ctx,
        )?);

        // =========================
        screens.push(Screen {
            title: "Hash of raw bytes".to_string(),
            content: Content::new(hash_get(body_bytes, auth_info_bytes))?,
            indent: None,
            expert: true,
        });

        Ok(screens)
    }
}
