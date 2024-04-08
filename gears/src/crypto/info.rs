use ibc_proto::{
    signing::SignDoc,
    tx::mode_info::{ModeInfo, SignMode},
};
use prost::Message;
use tendermint::types::{chain_id::ChainId, proto::Protobuf};

use crate::types::{
    auth::{fee::Fee, info::AuthInfo, tip::Tip},
    signing::SignerInfo,
    tx::{body::TxBody, raw::TxRaw, TxMessage},
};

use super::key::pair::KeyPair;

/// Contains info required to sign a Tx
pub struct SigningInfo {
    pub key: KeyPair,
    pub sequence: u64,
    pub account_number: u64,
}

pub fn create_signed_transaction<M: TxMessage>(
    signing_infos: Vec<SigningInfo>,
    tx_body: TxBody<M>,
    fee: Fee,
    tip: Option<Tip>,
    chain_id: ChainId,
) -> TxRaw {
    let signer_infos: Vec<SignerInfo> = signing_infos
        .iter()
        .map(|s| {
            let public_key = Some(s.key.get_gears_public_key());

            SignerInfo {
                public_key,
                mode_info: ModeInfo::Single(SignMode::Direct),
                sequence: s.sequence,
            }
        })
        .collect();

    let auth_info = AuthInfo {
        signer_infos,
        fee,
        tip,
    };

    let body_bytes = tx_body.encode_vec().expect("msg"); // TODO:NOW
    let auth_info_bytes = auth_info.encode_vec().expect("msg"); // TODO:NOW

    let mut sign_doc = SignDoc {
        body_bytes: body_bytes.clone(),
        auth_info_bytes: auth_info_bytes.clone(),
        chain_id: chain_id.into(),
        account_number: 0, // This gets overwritten
    };

    let signatures: Vec<Vec<u8>> = signing_infos
        .iter()
        .map(|s| {
            sign_doc.account_number = s.account_number;

            s.key.sign(&sign_doc.encode_to_vec())
        })
        .collect();

    TxRaw {
        body_bytes,
        auth_info_bytes,
        signatures,
    }
}
