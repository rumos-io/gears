use serde::{Deserialize, Serialize};

use crate::{cosmos::tx::v1beta1::message::Message, Error};

use super::{auth_info::AuthInfo, signer_data::SignerData, tx_body::TxBody, tx_data::TxData};

/// `TextualData` represents all the information needed to generate
/// the textual SignDoc (which is []Screen encoded to XBOR). It is meant to be
/// used as an internal type in Textual's implementations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextualData<M: Message> {
    /// body_bytes is a protobuf serialization of a TxBody that matches the
    /// representation in SignDoc.
    pub body: TxBody<M>,
    /// auth_info_bytes is a protobuf serialization of an AuthInfo that matches the
    /// representation in SignDoc.
    pub auth_info: AuthInfo,
    // signer_data represents all data in Textual's SignDoc that are not
    // inside the Tx body and auth_info.
    pub signer_data: SignerData,
}

impl<M: Message> TextualData<M> {
    pub fn new(signer_data: SignerData, tx_data: TxData<M>) -> Result<Self, Error> {
        let data = TextualData {
            signer_data,
            body: tx_data.body,
            auth_info: tx_data.auth_info,
        };

        Ok(data)
    }
}
