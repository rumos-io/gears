use crate::signing::renderer::Value;

use super::{
    signer_data::SignerData,
    tx_data::{AuthBytes, BodyBytes},
};

/// `TextualData` represents all the information needed to generate
/// the textual SignDoc (which is []Screen encoded to XBOR). It is meant to be
/// used as an internal type in Textual's implementations.
#[derive(Debug)]
pub struct TextualData {
    /// body_bytes is a protobuf serialization of a TxBody that matches the
    /// representation in SignDoc.
    pub body_bytes: BodyBytes, // `protobuf:"bytes,1,opt,name=body_bytes,json=bodyBytes,proto3" json:"body_bytes,omitempty"`
    /// auth_info_bytes is a protobuf serialization of an AuthInfo that matches the
    /// representation in SignDoc.
    pub auth_info_bytes: AuthBytes, // `protobuf:"bytes,2,opt,name=auth_info_bytes,json=authInfoBytes,proto3" json:"auth_info_bytes,omitempty"`
    // signer_data represents all data in Textual's SignDoc that are not
    // inside the Tx body and auth_info.
    pub signer_data: SignerData, //`protobuf:"bytes,3,opt,name=signer_data,json=signerData,proto3" json:"signer_data,omitempty"`
}

impl TextualData {
    pub fn value_get(&self) -> Value {
        Value
    }
}
