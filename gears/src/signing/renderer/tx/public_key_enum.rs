use crate::crypto::public::PublicKey;
use crate::signing::handler::MetadataGetter;
use crate::signing::renderer::value_renderer::{RenderError, ValueRenderer};
use crate::types::rendering::screen::Screen;

impl ValueRenderer for PublicKey {
    fn format<MG: MetadataGetter>(&self, get_metadata: &MG) -> Result<Vec<Screen>, RenderError> {
        match self {
            PublicKey::Secp256k1(key) => ValueRenderer::format(key, get_metadata),
            PublicKey::Ed25519(_) => Err(RenderError::NotImplemented),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::crypto::secp256k1::Secp256k1PubKey;
    use crate::signing::renderer::test_functions::TestMetadataGetter;
    use crate::signing::renderer::value_renderer::ValueRenderer;
    use crate::types::rendering::screen::{Content, Indent, Screen};

    #[test]
    fn secp256_pubkey_formating() -> anyhow::Result<()> {
        let key: Secp256k1PubKey = serde_json::from_str(
            r#"{
            "@type": "/cosmos.crypto.secp256k1.PubKey",
            "key": "Auvdf+T963bciiBe9l15DNMOijdaXCUo6zqSOvH7TXlN"
        }"#,
        )?;

        let expected_screens = vec![
            Screen {
                title: "Public key".to_string(),
                content: Content::try_new("/cosmos.crypto.secp256k1.PubKey")?,
                indent: None,
                expert: true,
            },
            Screen {
                title: "Key".to_string(),
                content: Content::try_new("02EB DD7F E4FD EB76 DC8A 205E F65D 790C D30E 8A37 5A5C 2528 EB3A 923A F1FB 4D79 4D")?,
                indent: Some(Indent::one()),
                expert: true,
            },
        ];

        let actual_screens = ValueRenderer::format(&key, &TestMetadataGetter)
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;

        assert_eq!(expected_screens, actual_screens);

        Ok(())
    }
}
