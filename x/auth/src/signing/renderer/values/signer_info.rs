use database::Database;
use gears::types::context::context::Context;
use proto_messages::cosmos::tx::v1beta1::{
    screen::{Content, Indent, Screen},
    signer::SignerInfo,
};
use store::StoreKey;

use crate::signing::renderer::value_renderer::{
    DefaultPrimitiveRenderer, PrimitiveValueRenderer, ValueRenderer,
};

impl<DefaultValueRenderer, SK: StoreKey, DB: Database> ValueRenderer<DefaultValueRenderer, SK, DB>
    for SignerInfo
{
    fn format(
        &self,
        ctx: &Context<'_, '_, DB, SK>,
    ) -> Result<Vec<Screen>, Box<dyn std::error::Error>> {
        let SignerInfo {
            public_key,
            mode_info,
            sequence,
        } = &self;

        let mut final_screens = Vec::<Screen>::new();
        if let Some(public_key) = public_key {
            final_screens.append(&mut ValueRenderer::<DefaultValueRenderer, SK, DB>::format(
                public_key, ctx,
            )?)
        }

        final_screens.append(&mut ValueRenderer::<DefaultValueRenderer, SK, DB>::format(
            mode_info, ctx,
        )?);

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
    use gears::types::context::context::Context;
    use proto_messages::cosmos::tx::v1beta1::{
        mode_info::{ModeInfo, SignMode},
        screen::{Content, Indent, Screen},
        signer::SignerInfo,
    };

    use crate::signing::renderer::{
        value_renderer::{DefaultValueRenderer, ValueRenderer},
        KeyMock, MockContext,
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

        let mut ctx = MockContext;

        let context: Context<'_, '_, database::RocksDB, KeyMock> =
            Context::DynamicContext(&mut ctx);

        let actuals_screens =
            ValueRenderer::<DefaultValueRenderer, KeyMock, _>::format(&info, &context)
                .map_err(|e| anyhow::anyhow!(e.to_string()))?;

        assert_eq!(expected_screens, actuals_screens);

        Ok(())
    }
}
