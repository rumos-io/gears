use bytes::Bytes;
use ibc_proto::cosmos::tx::v1beta1::{SignDoc, Tx, TxRaw};
use prost::Message;
use secp256k1::{ecdsa, hashes::sha256, PublicKey, Secp256k1};

pub fn _verify_signature(tx: Tx, tx_raw: TxRaw) -> bool {
    let sign_bytes = SignDoc {
        body_bytes: tx_raw.body_bytes,
        auth_info_bytes: tx_raw.auth_info_bytes,
        chain_id: "localnet".to_string(),
        account_number: 0,
    }
    .encode_to_vec();

    let message = secp256k1::Message::from_hashed_data::<sha256::Hash>(&sign_bytes);

    let public = tx.auth_info.clone().unwrap().signer_infos[0]
        .clone()
        .public_key
        .unwrap()
        .type_url;
    println!("################# URL:  {}", public);

    let public = tx.auth_info.clone().unwrap().signer_infos[0]
        .clone()
        .public_key
        .unwrap()
        .value;
    let public = PubKey::decode::<Bytes>(public.into()).unwrap();
    let public_key = PublicKey::from_slice(&public.key).unwrap();

    let sig = ecdsa::Signature::from_compact(&tx_raw.signatures[0]).unwrap();

    let secp = Secp256k1::verification_only();

    assert!(secp.verify_ecdsa(&message, &sig, &public_key).is_ok());

    let sig_res = secp.verify_ecdsa(&message, &sig, &public_key);

    if sig_res.is_ok() {
        println!("Sig is good!!!!")
    }

    match sig_res {
        Ok(_) => return true,
        Err(_) => return false,
    }
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PubKey {
    #[prost(bytes = "vec", tag = "1")]
    pub key: ::prost::alloc::vec::Vec<u8>,
}

#[cfg(test)]
mod tests {

    // use ibc_proto::{
    //     cosmos::{
    //         auth,
    //         tx::v1beta1::{AuthInfo, SignerInfo, TxBody},
    //     },
    //     google::protobuf::Any,
    // };

    // use super::*;

    // #[test]
    // fn verify_signature_test() {
    //     let messages = vec![Any {
    //         type_url: "/cosmos".to_string(),
    //         value: vec![1, 23, 2],
    //     }];

    //     let body = TxBody {
    //         messages,
    //         memo: "".to_string(),
    //         timeout_height: 0,
    //         extension_options: vec![],
    //         non_critical_extension_options: vec![],
    //     };

    //     let auth_info = AuthInfo {
    //         signer_infos: vec![SignerInfo {
    //             public_key: Some(Any {
    //                 type_url: "".to_string(),
    //                 value: vec![1, 23, 3],
    //             }),
    //             mode_info: None,
    //             sequence: 0,
    //         }],
    //         fee: None,
    //     };
    //     let tx = Tx {
    //         body: Some(body),
    //         auth_info: Some(auth_info),
    //         signatures: todo!(),
    //     };
    //     assert!(!verify_signature(tx))
    // }
}
