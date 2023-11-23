use database::RocksDB;
use gears::types::context::context::Context;
use ibc_proto::protobuf::Protobuf;
use proto_messages::cosmos::tx::v1beta1::{
    screen::{Content, Screen},
    signer_data::SignerData,
};
use store::StoreKey;

use crate::signing::renderer::value_renderer::ValueRenderer;

impl<DefaultValueRenderer, SK: StoreKey> ValueRenderer<DefaultValueRenderer, SK> for SignerData {
    fn format(
        &self,
        _: &Context<'_, '_, RocksDB, SK>,
    ) -> Result<Vec<Screen>, Box<dyn std::error::Error>> {
        let screens = vec![
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
            Screen {
                title: "Public key".to_string(),
                content: Content::new(self.pub_key.encode_to_hex_string())?,
                indent: None,
                expert: true,
            },
        ];

        Ok(screens)
    }
}
