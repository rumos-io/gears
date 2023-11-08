pub mod any_tx;
pub mod auth_info;
pub mod cbor;
pub mod envelope;
pub mod fee;
pub mod public_key;
#[allow(dead_code)]
pub mod screen;
pub mod signature_data;
pub mod signer;
pub mod signer_data;
pub mod textual_data;
pub mod tip;
pub mod tx;
pub mod tx_body;
pub mod tx_data;
pub mod tx_raw;

#[cfg(test)]
mod tests {

    use crate::cosmos::{
        crypto::secp256k1::v1beta1::{PubKey as Secp256k1PubKey, RawPubKey},
        tx::v1beta1::public_key::PublicKey,
    };

    #[test]
    fn serialize_pubkey_works() {
        let key = hex::decode("02950e1cdfcb133d6024109fd489f734eeb4502418e538c28481f22bce276f248c")
            .unwrap();
        let raw = RawPubKey { key };
        let key: Secp256k1PubKey = raw.try_into().unwrap();
        let key = PublicKey::Secp256k1(key);
        let key = serde_json::to_string(&key).unwrap();

        println!("{key}");

        assert_eq!(
            key,
            r#"{"@type":"/cosmos.crypto.secp256k1.PubKey","key":"ApUOHN/LEz1gJBCf1In3NO60UCQY5TjChIHyK84nbySM"}"#
        );
    }

    #[test]
    fn deserialize_pubkey_works() {
        let serialized = r#"{"@type":"/cosmos.crypto.secp256k1.PubKey","key":"ApUOHN/LEz1gJBCf1In3NO60UCQY5TjChIHyK84nbySM"}"#;
        let key: PublicKey = serde_json::from_str(serialized).unwrap();
        let PublicKey::Secp256k1(key) = key;
        assert_eq!(
            hex::encode(Vec::from(key)),
            "02950e1cdfcb133d6024109fd489f734eeb4502418e538c28481f22bce276f248c"
        );
    }
}
