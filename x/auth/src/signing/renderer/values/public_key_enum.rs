use proto_messages::cosmos::tx::v1beta1::{
    public_key::PublicKey, screen::Screen, tx_metadata::Metadata,
};
use proto_types::Denom;

use crate::signing::renderer::value_renderer::ValueRenderer;

impl ValueRenderer for PublicKey {
    fn format<F: Fn(&Denom) -> Option<Metadata>>(
        &self,
        get_metadata: &F,
    ) -> Result<Vec<Screen>, Box<dyn std::error::Error>> {
        // I prefer to implement formatting for each key in own module to keep things as small as possible
        match self {
            PublicKey::Secp256k1(key) => ValueRenderer::format(key, get_metadata),
        }
    }
}

#[cfg(test)]
mod tests {
    use proto_messages::cosmos::{
        crypto::secp256k1::v1beta1::PubKey,
        tx::v1beta1::screen::{Content, Indent, Screen},
    };

    use crate::signing::renderer::{
        value_renderer::ValueRenderer, values::test_functions::get_metadata,
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

        let actual_screens = ValueRenderer::format(&key, &get_metadata)
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;

        assert_eq!(expected_screens, actual_screens);

        Ok(())
    }
}
