use ibc_proto::protobuf::Protobuf;
use nutype::nutype;
use prost::Message;
use serde::{Deserialize, Serialize};

use crate::{
    cosmos::crypto::secp256k1::v1beta1::{PubKey, RawPubKey},
    Error,
};

#[derive(Clone, PartialEq, Message)]
pub struct SignerDataRaw {
    #[prost(string, tag = "1")]
    pub address: String,
    #[prost(string)]
    pub chain_id: String,
    #[prost(fixed64)]
    pub account_number: u64,
    #[prost(fixed64)]
    pub sequence: u64,
    #[prost(message, required)]
    pub pub_key: RawPubKey,
}

impl TryFrom<SignerDataRaw> for SignerData {
    type Error = Error;

    fn try_from(value: SignerDataRaw) -> Result<Self, Self::Error> {
        let SignerDataRaw {
            address,
            chain_id,
            account_number,
            sequence,
            pub_key,
        } = value;

        let var = Self {
            address,
            chain_id: ChainId::new(chain_id).map_err(|e| Error::DecodeGeneral(e.to_string()))?,
            account_number,
            sequence,
            pub_key: pub_key.try_into()?,
        };

        Ok(var)
    }
}

impl From<SignerData> for SignerDataRaw {
    fn from(value: SignerData) -> Self {
        let SignerData {
            address,
            chain_id,
            account_number,
            sequence,
            pub_key,
        } = value;
        Self {
            address,
            chain_id: chain_id.into_inner(),
            account_number,
            sequence,
            pub_key: pub_key.into(),
        }
    }
}

impl Protobuf<SignerDataRaw> for SignerData {}

#[nutype(validate(not_empty))]
#[derive(*, Serialize, Deserialize)]
pub struct ChainId(String);

/// SignerData is the specific information needed to sign a transaction that generally
/// isn't included in the transaction body itself
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignerData {
    /// The address of the signer.
    ///
    /// In case of multisigs, this should be the multisig's address.
    pub address: String,

    /// ChainID is the chain that this transaction is targeting.
    pub chain_id: ChainId,

    /// AccountNumber is the account number of the signer.
    ///
    /// In case of multisigs, this should be the multisig account number.
    pub account_number: u64,

    /// Sequence is the account sequence number of the signer that is used
    /// for replay protection. This field is only useful for Legacy Amino signing,
    /// since in SIGN_MODE_DIRECT the account sequence is already in the signer info.
    ///
    /// In case of multisigs, this should be the multisig sequence.
    pub sequence: u64,

    pub pub_key: PubKey,
}
