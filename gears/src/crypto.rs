use ibc_proto::{
    cosmos::tx::v1beta1::{
        mode_info::{Single, Sum},
        ModeInfo, SignDoc, TxRaw,
    },
    protobuf::Protobuf,
};
use ibc_relayer::keyring::{Secp256k1KeyPair, SigningKeyPair};
use prost::Message;
use proto_messages::cosmos::{
    crypto::secp256k1::v1beta1::{PubKey, RawPubKey},
    tx::v1beta1::{
        auth_info::AuthInfo, fee::Fee, message::Message as SDKMessage, public_key::PublicKey,
        signer::SignerInfo, tip::Tip, tx_body::TxBody,
    },
};
use tendermint_informal::chain::Id;

/// Contains info required to sign a Tx
pub struct SigningInfo {
    pub key: Secp256k1KeyPair,
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
            let public_key = s.key.public_key.serialize().to_vec();
            let public_key = RawPubKey { key: public_key }; //TODO: add method to PubKey to make this easier?
            let public_key: PubKey = public_key
                .try_into()
                .expect("converting the secp256k1 library's public key will always succeed");

            SignerInfo {
                public_key: Some(PublicKey::Secp256k1(public_key)),
                mode_info: Some(ModeInfo {
                    sum: Some(Sum::Single(Single { mode: 1 })),
                }),
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

            s.key
                .sign(&sign_doc.encode_to_vec())
                .expect("library method can never fail")
        })
        .collect();

    TxRaw {
        body_bytes,
        auth_info_bytes,
        signatures,
    }
}
