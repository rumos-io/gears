use keyring::key_pair::KeyPair;
use prost::Message;
use proto_messages::cosmos::{
    ibc_types::{
        protobuf::Protobuf,
        tx::{SignDoc, TxRaw},
    },
    tx::v1beta1::{
        auth_info::AuthInfo,
        fee::Fee,
        message::Message as SDKMessage,
        mode_info::{ModeInfo, SignMode},
        signer::SignerInfo,
        tip::Tip,
        tx_body::TxBody,
    },
};
use tendermint::informal::chain::Id;

/// Contains info required to sign a Tx
pub struct SigningInfo {
    pub key: KeyPair,
    pub sequence: u64,
    pub account_number: u64,
}

pub fn create_signed_transaction<M: SDKMessage>(
    signing_infos: Vec<SigningInfo>,
    tx_body: TxBody<M>,
    fee: Fee,
    tip: Option<Tip>,
    chain_id: Id,
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

    let body_bytes = tx_body.encode_vec();
    let auth_info_bytes = auth_info.encode_vec();

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
