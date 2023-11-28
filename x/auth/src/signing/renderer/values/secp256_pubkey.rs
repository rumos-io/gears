use database::RocksDB;
use gears::types::context::context::Context;
use proto_messages::cosmos::{
    crypto::secp256k1::v1beta1::PubKey,
    tx::v1beta1::screen::{Content, Indent, Screen},
};
use store::StoreKey;

use crate::signing::renderer::value_renderer::ValueRenderer;

const TYPE_URL: &str = "/cosmos.crypto.secp256k1.PubKey";

impl<DefaultValueRenderer, SK: StoreKey> ValueRenderer<DefaultValueRenderer, SK> for PubKey {
    fn format(
        &self,
        _ctx: &Context<'_, '_, RocksDB, SK>,
    ) -> Result<Vec<Screen>, Box<dyn std::error::Error>> {
        Ok(vec![
            Screen {
                title: "Public key".to_string(),
                content: Content::new(TYPE_URL)?,
                indent: None,
                expert: true,
            },
            Screen {
                title: "Key".to_string(),
                content: Content::new(self.get_address())?,
                indent: Some(Indent::new(1)?),
                expert: true,
            },
        ])
    }
}

#[cfg(test)]
mod tests {
    use gears::types::context::context::Context;
    use proto_messages::cosmos::{
        crypto::secp256k1::v1beta1::{PubKey, Secp256k1PubKey},
        tx::v1beta1::screen::{Content, Indent, Screen},
    };
    use rand::thread_rng;
    use secp256k1::KeyPair;

    use crate::signing::renderer::{
        value_renderer::{DefaultValueRenderer, ValueRenderer},
        KeyMock, MockContext,
    };

    #[test]
    fn secp256_pubkey_formating() -> anyhow::Result<()> {
        let mut rand = thread_rng();
        let keypair = KeyPair::new_global(&mut rand);
        let secp_key = Secp256k1PubKey::from_keypair(&keypair);
        let key = PubKey::new(secp_key);

        let expected_screens = vec![
            Screen {
                title: "Public key".to_string(),
                content: Content::new("/cosmos.crypto.secp256k1.PubKey")?,
                indent: None,
                expert: true,
            },
            Screen {
                title: "Key".to_string(),
                content: Content::new(key.get_address())?,
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
