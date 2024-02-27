use std::collections::HashSet;

use proto_types::AccAddress;
use serde::{Deserialize, Serialize};

use crate::{
    cosmos::base::v1beta1::SendCoins, cosmos::tx::v1beta1::message::Message, error::Error,
};

use ibc_proto::{cosmos::tx::v1beta1::Tx as RawTx, Protobuf};

use super::{
    auth_info::AuthInfo, public_key::PublicKey, signature_data::SignatureData, tx_body::TxBody,
};

/// Tx is the standard type used for broadcasting transactions.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Tx<M: Message> {
    /// body is the processable content of the transaction
    pub body: TxBody<M>,
    /// auth_info is the authorization related content of the transaction,
    /// specifically signers, signer modes and fee
    pub auth_info: AuthInfo,
    /// signatures is a list of signatures that matches the length and order of
    /// AuthInfo's signer_infos to allow connecting signature meta information like
    /// public key and signing mode by position.
    #[serde(serialize_with = "crate::utils::serialize_vec_of_vec_to_vec_of_base64")]
    pub signatures: Vec<Vec<u8>>,
    #[serde(skip_serializing)]
    pub signatures_data: Vec<SignatureData>,
}

// TODO:
// 0. Make TxWithRaw the Tx - move methods to TxWithRaw and rename
// 1. Many more checks are needed on DecodedTx::from_bytes see https://github.com/cosmos/cosmos-sdk/blob/2582f0aab7b2cbf66ade066fe570a4622cf0b098/x/auth/tx/decoder.go#L16
// 2. Implement equality on AccAddress to avoid conversion to string in get_signers()
// 3. Consider removing the "seen" hashset in get_signers()
impl<M: Message> Tx<M> {
    pub fn get_msgs(&self) -> &Vec<M> {
        &self.body.messages
    }

    pub fn get_signers(&self) -> Vec<&AccAddress> {
        let mut signers = vec![];
        let mut seen = HashSet::new();

        for msg in &self.body.messages {
            for addr in msg.get_signers() {
                if seen.insert(addr.to_string()) {
                    signers.push(addr);
                }
            }
        }

        // ensure any specified fee payer is included in the required signers (at the end)
        let fee_payer = &self.auth_info.fee.payer;

        if let Some(addr) = fee_payer {
            if seen.insert(addr.to_string()) {
                signers.push(addr);
            }
        }

        signers
    }

    pub fn get_signatures(&self) -> &Vec<Vec<u8>> {
        &self.signatures
    }

    pub fn get_signatures_data(&self) -> &Vec<SignatureData> {
        &self.signatures_data
    }

    pub fn get_timeout_height(&self) -> u64 {
        self.body.timeout_height
    }

    pub fn get_memo(&self) -> &str {
        &self.body.memo
    }

    pub fn get_fee(&self) -> &Option<SendCoins> {
        &self.auth_info.fee.amount
    }

    pub fn get_fee_payer(&self) -> &AccAddress {
        if let Some(payer) = &self.auth_info.fee.payer {
            payer
        } else {
            // At least one signer exists due to Ante::validate_basic_ante_handler()
            return self.get_signers()[0];
        }
    }

    pub fn get_public_keys(&self) -> Vec<&Option<PublicKey>> {
        self.auth_info
            .signer_infos
            .iter()
            .map(|si| &si.public_key)
            .collect()
    }
}

impl<M: Message> TryFrom<RawTx> for Tx<M> {
    type Error = Error;

    fn try_from(raw: RawTx) -> Result<Self, Self::Error> {
        let body = raw.body.ok_or(Error::MissingField("body".into()))?;

        // This covers the SDK RejectExtensionOptions ante handler
        // https://github.com/cosmos/cosmos-sdk/blob/2582f0aab7b2cbf66ade066fe570a4622cf0b098/x/auth/ante/ext.go#L27-L36
        if !body.extension_options.is_empty() {
            return Err(Error::DecodeGeneral("unknown extension options".into()));
        }

        let auth_info: AuthInfo = raw
            .auth_info
            .ok_or(Error::MissingField("auth_info".into()))?
            .try_into()?;

        // extract signatures data when decoding - this isn't done in the SDK
        if raw.signatures.len() != auth_info.signer_infos.len() {
            return Err(Error::DecodeGeneral(
                "signatures list does not match signer_infos length".into(),
            ));
        }
        let mut signatures_data = Vec::with_capacity(raw.signatures.len());
        for (i, signature) in raw.signatures.iter().enumerate() {
            signatures_data.push(SignatureData {
                signature: signature.clone(),
                // the check above, tx.signatures.len() != tx.auth_info.signer_infos.len(), ensures that this indexing is safe
                sequence: auth_info.signer_infos[i].sequence,
                mode_info: auth_info.signer_infos[i].mode_info.clone(),
            })
        }

        Ok(Tx {
            body: body.try_into()?,
            auth_info,
            signatures: raw.signatures,
            signatures_data,
        })
    }
}

impl<M: Message> Protobuf<RawTx> for Tx<M> {}

impl<M: Message> From<Tx<M>> for RawTx {
    fn from(tx: Tx<M>) -> RawTx {
        RawTx {
            body: Some(tx.body.into()),
            auth_info: Some(tx.auth_info.into()),
            signatures: tx.signatures,
        }
    }
}
