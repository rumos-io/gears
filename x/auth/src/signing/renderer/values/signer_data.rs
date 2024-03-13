use proto_messages::cosmos::tx::v1beta1::{
    screen::Screen, signer_data::SignerData, tx_metadata::Metadata,
};
use proto_types::Denom;

use crate::signing::renderer::value_renderer::{
    DefaultPrimitiveRenderer, Error, PrimitiveValueRenderer, TryPrimitiveValueRenderer,
    ValueRenderer,
};

impl ValueRenderer for SignerData {
    fn format<F: Fn(&Denom) -> Option<Metadata>>(
        &self,
        get_metadata: &F,
    ) -> Result<Vec<Screen>, Error> {
        let mut screens = vec![
            Screen {
                title: "Chain id".to_string(),
                content: DefaultPrimitiveRenderer::try_format(self.chain_id.clone().to_string())?,
                indent: None,
                expert: false,
            },
            Screen {
                title: "Account number".to_string(),
                content: DefaultPrimitiveRenderer::format(self.account_number),
                indent: None,
                expert: false,
            },
            Screen {
                title: "Sequence".to_string(),
                content: DefaultPrimitiveRenderer::format(self.sequence),
                indent: None,
                expert: false,
            },
            Screen {
                title: "Address".to_string(),
                content: DefaultPrimitiveRenderer::format(self.address.to_owned()),
                indent: None,
                expert: true,
            },
        ];

        screens.append(&mut ValueRenderer::format(&self.pub_key, get_metadata)?);

        Ok(screens)
    }
}

#[cfg(test)]
mod tests {
    use proto_messages::cosmos::tx::v1beta1::{
        screen::{Content, Indent, Screen},
        signer_data::SignerData,
    };
    use proto_types::AccAddress;
    use tendermint::informal::chain::Id;

    use crate::signing::renderer::{
        value_renderer::ValueRenderer, values::test_functions::get_metadata,
    };

    #[test]
    fn signer_data_formatting() -> anyhow::Result<()> {
        let signer_data = SignerData {
            address: AccAddress::from_bech32("cosmos1ulav3hsenupswqfkw2y3sup5kgtqwnvqa8eyhs")?,
            chain_id: Id::try_from("my-chain".to_string()).expect("this is a valid chain id"),
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
                indent: Some(Indent::one()),
                expert: true,
            },
        ];

        let actuals_screens = ValueRenderer::format(&signer_data, &get_metadata)
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;

        assert_eq!(expected_screens, actuals_screens);

        Ok(())
    }
}
