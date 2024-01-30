use database::Database;
use gears::types::context::context::Context;
use proto_messages::cosmos::tx::v1beta1::{
    screen::{Content, Screen},
    signer_data::SignerData,
};
use store::StoreKey;

use crate::signing::renderer::value_renderer::ValueRenderer;

impl<DefaultValueRenderer, SK: StoreKey, DB: Database> ValueRenderer<DefaultValueRenderer, SK, DB>
    for SignerData
{
    fn format(
        &self,
        ctx: &Context<'_, '_, DB, SK>,
    ) -> Result<Vec<Screen>, Box<dyn std::error::Error>> {
        let mut screens = vec![
            Screen {
                title: "Chain id".to_string(),
                content: Content::new(self.chain_id.clone().into_inner())?,
                indent: None,
                expert: false,
            },
            Screen {
                title: "Account number".to_string(),
                content: Content::new(self.account_number.to_string())?,
                indent: None,
                expert: false,
            },
            Screen {
                title: "Sequence".to_string(),
                content: Content::new(self.sequence.to_string())?,
                indent: None,
                expert: false,
            },
            Screen {
                title: "Address".to_string(),
                content: Content::new(self.address.clone())?,
                indent: None,
                expert: true,
            },
        ];

        screens.append(&mut ValueRenderer::<DefaultValueRenderer, SK, DB>::format(
            &self.pub_key,
            ctx,
        )?);

        Ok(screens)
    }
}

#[cfg(test)]
mod tests {
    use gears::types::context::context::Context;
    use proto_messages::cosmos::tx::v1beta1::{
        screen::{Content, Indent, Screen},
        signer_data::{ChainId, SignerData},
    };
    use proto_types::AccAddress;

    use crate::signing::renderer::{
        value_renderer::{DefaultValueRenderer, ValueRenderer},
        KeyMock, MockContext,
    };

    #[test]
    fn signer_data_formating() -> anyhow::Result<()> {
        let signer_data = SignerData {
            address: AccAddress::from_bech32("cosmos1ulav3hsenupswqfkw2y3sup5kgtqwnvqa8eyhs")?,
            chain_id: ChainId::new("my-chain".to_string())?,
            account_number: 1,
            sequence: 2,
            pub_key: serde_json::from_str(
                r#"{
				"@type": "/cosmos.crypto.secp256k1.PubKey",
				"key": "Auvdf+T963bciiBe9l15DNMOijdaXCUo6zqSOvH7TXlN"
			}"#,
            )?,
        };

        let expected_screens = vec![
            Screen {
                title: "Chain id".to_string(),
                content: Content::new("my-chain".to_string())?,
                indent: None,
                expert: false,
            },
            Screen {
                title: "Account number".to_string(),
                content: Content::new(1.to_string())?,
                indent: None,
                expert: false,
            },
            Screen {
                title: "Sequence".to_string(),
                content: Content::new(2.to_string())?,
                indent: None,
                expert: false,
            },
            Screen {
                title: "Address".to_string(),
                content: Content::new("cosmos1ulav3hsenupswqfkw2y3sup5kgtqwnvqa8eyhs")?,
                indent: None,
                expert: true,
            },
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
        ];

        let mut ctx = MockContext;

        let context: Context<'_, '_, database::RocksDB, KeyMock> =
            Context::DynamicContext(&mut ctx);

        let actuals_screens =
            ValueRenderer::<DefaultValueRenderer, KeyMock, _>::format(&signer_data, &context)
                .map_err(|e| anyhow::anyhow!(e.to_string()))?;

        assert_eq!(expected_screens, actuals_screens);

        Ok(())
    }
}
