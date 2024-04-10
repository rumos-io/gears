use keyring::key_pair::KeyPair;
use prost::Message;
use proto_messages::cosmos::{
    ibc::{
        protobuf::Protobuf,
        tx::{SignDoc, TxRaw},
    },
    tx::v1beta1::{auth_info::AuthInfo, message::Message as SDKMessage, tx_body::TxBody},
};
use tendermint::informal::chain::Id;

/// Contains info required to sign a Tx
pub struct SigningInfo {
    pub key: KeyPair,
    pub sequence: u64,
    pub account_number: u64,
}

pub fn create_signed_transaction<M: SDKMessage>(
    signing_infos: &[SigningInfo],
    auth_info: AuthInfo,
    tx_body: TxBody<M>,
    chain_id: Id,
) -> TxRaw {
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

            sign_bytes(&s.key, sign_doc.encode_to_vec())
        })
        .collect();

    TxRaw {
        body_bytes,
        auth_info_bytes,
        signatures,
    }
}

/// Create signature for random slice of bytes.
// force inline because it is very simple code. It can be moved to KeyPair
#[inline]
pub fn sign_bytes<B: AsRef<[u8]>>(key: &KeyPair, bytes: B) -> Vec<u8> {
    key.sign(bytes.as_ref())
}
