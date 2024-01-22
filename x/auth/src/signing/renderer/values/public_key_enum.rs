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

#[cfg(test)]
mod tests {
    use gears::types::context::context::Context;
    use proto_messages::cosmos::{
        crypto::secp256k1::v1beta1::PubKey,
        tx::v1beta1::screen::{Content, Indent, Screen},
    };

    use crate::signing::renderer::{
        value_renderer::{DefaultValueRenderer, ValueRenderer},
        KeyMock, MockContext,
    };

    #[test]
    fn secp256_pubkey_formating() -> anyhow::Result<()> {
        let key: PubKey = serde_json::from_str(
            r#"{
            "@type": "/cosmos.crypto.secp256k1.PubKey",
            "key": "Auvdf+T963bciiBe9l15DNMOijdaXCUo6zqSOvH7TXlN"
        }"#,
        )?;

        let expected_screens = vec![
            Screen {
                title: "Public key".to_string(),
                content: Content::new("/cosmos.crypto.secp256k1.PubKey")?,
                indent: None,
                expert: true,
            },
            Screen {
                title: "Key".to_string(),
                content: Content::new("02EB DD7F E4FD EB76 DC8A 205E F65D 790C D30E 8A37 5A5C 2528 EB3A 923A F1FB 4D79 4D")?,
                indent: Some(Indent::new(1)?),
                expert: true,
            },
        ];

        let mut ctx = MockContext;

        let context: Context<'_, '_, database::RocksDB, KeyMock> =
            Context::DynamicContext(&mut ctx);

        let actual_screens = ValueRenderer::<DefaultValueRenderer, KeyMock>::format(&key, &context)
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;

        assert_eq!(expected_screens, actual_screens);

        Ok(())
    }
}
