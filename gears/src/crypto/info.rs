use core_types::{
    signing::SignDoc,
    tx::mode_info::{ModeInfo, SignMode},
};
use prost::Message;
use tendermint::types::{chain_id::ChainId, proto::Protobuf};

use std::{
    error::Error,
    fmt::{Display, Formatter},
};

use crate::{
    application::handlers::client::MetadataViaRPC,
    error::IBC_ENCODE_UNWRAP,
    signing::{
        errors::SigningErrors, handler::SignModeHandler, renderer::value_renderer::ValueRenderer,
    },
    types::{
        auth::{fee::Fee, info::AuthInfo, tip::Tip},
        signing::SignerInfo,
        tx::{body::TxBody, data::TxData, raw::TxRaw, signer::SignerData, TxMessage},
    },
};

use super::keys::{GearsPublicKey, ReadAccAddress, SigningKey};

/// Contains info required to sign a Tx
pub struct SigningInfo<K> {
    pub key: K,
    pub sequence: u64,
    pub account_number: u64,
}

pub fn create_signed_transaction_direct<K: SigningKey + GearsPublicKey>(
    signing_infos: Vec<SigningInfo<K>>,
    chain_id: ChainId,
    fee: Fee,
    tip: Option<Tip>,
    body_bytes: Vec<u8>,
) -> Result<TxRaw, K::Error> {
    let auth_info_bytes = auth_info(&signing_infos, fee, tip, Mode::Direct)
        .encode_vec()
        .expect(IBC_ENCODE_UNWRAP);

    let mut sign_doc = SignDoc {
        body_bytes: body_bytes.clone(),
        auth_info_bytes: auth_info_bytes.clone(),
        chain_id: chain_id.into(),
        account_number: 0, // This gets overwritten
    };

    let signatures = signing_infos
        .iter()
        .map(|s| {
            sign_doc.account_number = s.account_number;

            s.key.sign(&sign_doc.encode_to_vec())
        })
        .collect::<Result<Vec<Vec<u8>>, <K as crate::crypto::keys::SigningKey>::Error>>()?;

    Ok(TxRaw {
        body_bytes,
        auth_info_bytes,
        signatures,
    })
}

// NOTE: we can't implement From<K::Error> for this type
#[derive(Debug)]
pub enum TextualSigningError<K: SigningKey> {
    Rendering(SigningErrors),
    Key(K::Error),
}

impl<K: SigningKey + std::fmt::Debug> Error for TextualSigningError<K> {}

impl<K: SigningKey> Display for TextualSigningError<K> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TextualSigningError::Rendering(e) => write!(f, "{}", e),
            TextualSigningError::Key(e) => write!(f, "{}", e),
        }
    }
}

pub fn create_signed_transaction_textual<
    M: TxMessage + ValueRenderer,
    K: SigningKey + ReadAccAddress + GearsPublicKey,
>(
    signing_infos: Vec<SigningInfo<K>>,
    chain_id: ChainId,
    fee: Fee,
    tip: Option<Tip>,
    node: url::Url,
    tx_body: TxBody<M>,
) -> Result<TxRaw, TextualSigningError<K>> {
    let body_bytes = tx_body.encode_vec().expect(IBC_ENCODE_UNWRAP);
    let auth_info = auth_info(&signing_infos, fee, tip, Mode::Textual);
    let auth_info_bytes = auth_info.encode_vec().expect(IBC_ENCODE_UNWRAP);
    let tx_data = TxData {
        body: tx_body,
        auth_info,
    };

    let sign_mode_handler = SignModeHandler;

    let signatures = signing_infos
        .into_iter()
        .map(|s| {
            let signer_data = SignerData {
                address: s.key.get_address(),
                chain_id: chain_id.clone(),
                account_number: s.account_number,
                sequence: s.sequence,
                pub_key: s.key.get_gears_public_key(),
            };

            let sign_bytes = sign_mode_handler
                .sign_bytes_get(
                    &MetadataViaRPC { node: node.clone() },
                    signer_data,
                    tx_data.clone(),
                )
                .map_err(|e| TextualSigningError::Rendering(e))?;

            s.key
                .sign(&sign_bytes)
                .map_err(|e| TextualSigningError::Key(e))
        })
        .collect::<Result<Vec<Vec<u8>>, TextualSigningError<K>>>()?;

    Ok(TxRaw {
        body_bytes,
        auth_info_bytes,
        signatures,
    })
}

#[derive(Clone)]
enum Mode {
    Direct,
    Textual,
}

impl From<Mode> for SignMode {
    fn from(mode: Mode) -> Self {
        match mode {
            Mode::Direct => SignMode::Direct,
            Mode::Textual => SignMode::Textual,
        }
    }
}

fn auth_info<K: GearsPublicKey>(
    signing_infos: &Vec<SigningInfo<K>>,
    fee: Fee,
    tip: Option<Tip>,
    mode: Mode,
) -> AuthInfo {
    let signer_infos: Vec<SignerInfo> = signing_infos
        .iter()
        .map(|s| {
            let public_key = Some(s.key.get_gears_public_key());

            SignerInfo {
                public_key,
                mode_info: ModeInfo::Single(mode.clone().into()),
                sequence: s.sequence,
            }
        })
        .collect();

    AuthInfo {
        signer_infos,
        fee,
        tip,
    }
}
