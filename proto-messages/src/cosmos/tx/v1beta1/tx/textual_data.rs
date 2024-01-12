use ibc_proto::{
    cosmos::tx::v1beta1::AuthInfo as RawAuthInfo, cosmos::tx::v1beta1::TxBody as RawTxBody,
    protobuf::Protobuf,
};
use prost::{bytes::Bytes, Message as ProstMessage};
use serde::{Deserialize, Serialize};

use crate::{cosmos::tx::v1beta1::message::Message, Error};

use super::{
    auth_info::AuthInfo,
    signer_data::{SignerData, SignerDataRaw},
    tx_body::TxBody,
    tx_data::TxData,
};

#[derive(Clone, PartialEq, ProstMessage)]
pub(crate) struct TextualDataRaw {
    #[prost(bytes, required, tag = "1")]
    pub body_bytes: Bytes,
    #[prost(bytes, required, tag = "2")]
    pub auth_info_bytes: Bytes,
    #[prost(message, required, tag = "3")]
    pub signer_data: SignerDataRaw,
}

impl<M: Message> TryFrom<TextualDataRaw> for TextualData<M> {
    type Error = Error;

    fn try_from(value: TextualDataRaw) -> Result<Self, Self::Error> {
        let TextualDataRaw {
            body_bytes,
            auth_info_bytes,
            signer_data,
        } = value;

        let body: RawTxBody = prost::Message::decode(body_bytes.clone())?;
        let auth_info: RawAuthInfo = prost::Message::decode(auth_info_bytes.clone())?;

        let var = Self {
            signer_data: signer_data.try_into()?,
            body: body.try_into()?,
            auth_info: auth_info.try_into()?,
        };

        Ok(var)
    }
}

impl<M: Message> From<TextualData<M>> for TextualDataRaw {
    fn from(value: TextualData<M>) -> Self {
        let TextualData {
            signer_data,
            body,
            auth_info,
        } = value;

        Self {
            body_bytes: Bytes::from_iter( body.encode_vec() ),
            auth_info_bytes: Bytes::from_iter( auth_info.encode_vec() ),
            signer_data: signer_data.into(),
        }
    }
}

impl<M: Message> Protobuf<TextualDataRaw> for TextualData<M> {}

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
