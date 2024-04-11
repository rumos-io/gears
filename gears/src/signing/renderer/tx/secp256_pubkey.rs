use keyring::key::secp256k1::Secp256k1PubKey;

use crate::types::{
    denom::Denom,
    rendering::screen::{Indent, Screen},
    tx::metadata::Metadata,
};

use crate::signing::renderer::value_renderer::{
    DefaultPrimitiveRenderer, RenderError, TryPrimitiveValueRenderer, ValueRenderer,
};

const TYPE_URL: &str = "/cosmos.crypto.secp256k1.PubKey";

impl ValueRenderer for Secp256k1PubKey {
    fn format<F: Fn(&Denom) -> Option<Metadata>>(
        &self,
        _get_metadata: &F,
    ) -> Result<Vec<Screen>, RenderError> {
        Ok(vec![
            Screen {
                title: "Public key".to_string(),
                content: DefaultPrimitiveRenderer::try_format(TYPE_URL)
                    .expect("hard coded type URL is not empty"),
                indent: None,
                expert: true,
            },
            Screen {
                title: "Key".to_string(),
                content: DefaultPrimitiveRenderer::try_format(
                    Vec::<u8>::from(self.clone()).as_slice(),
                )
                .expect("key is not empty so it will never fail to parse"),
                indent: Some(Indent::one()),
                expert: true,
            },
        ])
    }
}

#[cfg(test)]
mod tests {
    use keyring::key::secp256k1::Secp256k1PubKey;

    use crate::types::rendering::screen::{Content, Indent, Screen};

    use crate::signing::renderer::{test_functions::get_metadata, value_renderer::ValueRenderer};

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
                content: Content::new("/cosmos.crypto.secp256k1.PubKey")?,
                indent: None,
                expert: true,
            },
            Screen {
                title: "Key".to_string(),
                content: Content::new("02EB DD7F E4FD EB76 DC8A 205E F65D 790C D30E 8A37 5A5C 2528 EB3A 923A F1FB 4D79 4D")?,
                indent: Some(Indent::one()),
                expert: true,
            },
        ];

        let actual_screens = ValueRenderer::format(&key, &get_metadata)
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;

        assert_eq!(expected_screens, actual_screens);

        Ok(())
    }
}
