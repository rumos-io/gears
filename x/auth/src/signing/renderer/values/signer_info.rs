use proto_messages::cosmos::tx::v1beta1::{
    screen::{Content, Indent, Screen},
    signer::SignerInfo,
    tx_metadata::Metadata,
};
use proto_types::Denom;

use crate::signing::renderer::value_renderer::{
    DefaultPrimitiveRenderer, PrimitiveValueRenderer, ValueRenderer,
};

impl ValueRenderer for SignerInfo {
    fn format<F: Fn(&Denom) -> Option<Metadata>>(
        &self,
        get_metadata: &F,
    ) -> Result<Vec<Screen>, Box<dyn std::error::Error>> {
        let SignerInfo {
            public_key,
            mode_info,
            sequence,
        } = &self;

        let mut final_screens = Vec::<Screen>::new();
        if let Some(public_key) = public_key {
            final_screens.append(&mut ValueRenderer::format(public_key, get_metadata)?)
        }

        final_screens.append(&mut ValueRenderer::format(mode_info, get_metadata)?);

        final_screens.push(Screen {
            title: "Sequence".to_string(),
            content: Content::new(DefaultPrimitiveRenderer::format(*sequence))?,
            indent: Some(Indent::new(2)?),
            expert: true,
        });

        Ok(final_screens)
    }
}

#[cfg(test)]
mod tests {
    use proto_messages::cosmos::tx::v1beta1::{
        mode_info::{ModeInfo, SignMode},
        screen::{Content, Indent, Screen},
        signer::SignerInfo,
    };

    use crate::signing::renderer::{
        value_renderer::ValueRenderer, values::test_functions::get_metadata,
    };

    #[test]
    fn signer_info_formatting() -> anyhow::Result<()> {
        let info = SignerInfo {
            public_key: Some(serde_json::from_str(
                r#"{
                        "@type": "/cosmos.crypto.secp256k1.PubKey",
                        "key": "Auvdf+T963bciiBe9l15DNMOijdaXCUo6zqSOvH7TXlN"
                    }"#,
            )?),
            mode_info: ModeInfo::Single(SignMode::Direct),
            sequence: 2,
        };

        let expected_screens = vec![
            Screen {
                title: "Public key".to_string(),
                content: Content::new("/cosmos.crypto.secp256k1.PubKey")?,
                indent: None,
                expert: true,
            },
            Screen {
                title: "Key".to_string(),
                content: Content::new( "02EB DD7F E4FD EB76 DC8A 205E F65D 790C D30E 8A37 5A5C 2528 EB3A 923A F1FB 4D79 4D" )?,
                indent: Some(Indent::new(1)?),
                expert: true,
            },
            Screen {
                title: "Sequence".to_string(),
                content: Content::new(2.to_string())?,
                indent: Some(Indent::new(2)?),
                expert: true,
            },
        ];

        let actuals_screens = ValueRenderer::format(&info, &get_metadata)
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;

        assert_eq!(expected_screens, actuals_screens);

        Ok(())
    }
}
