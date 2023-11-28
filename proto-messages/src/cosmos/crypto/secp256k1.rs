pub mod v1beta1 {
    use std::fmt;

    use base64::{
        engine::general_purpose::{self},
        Engine,
    };
    use ibc_proto::protobuf::Protobuf;
    use proto_types::AccAddress;
    use ripemd::Ripemd160;
    use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
    use sha2::{Digest, Sha256};

    use crate::Error;

    pub use secp256k1::PublicKey as Secp256k1PubKey;

    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct RawPubKey {
        #[prost(bytes = "vec", tag = "1")]
        pub key: Vec<u8>,
    }

    #[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
    pub struct PubKey {
        // a custom serde is needed since the Secp256k1 serde uses hex encoding and not base64
        #[serde(serialize_with = "serialize_key", deserialize_with = "deserialize_key")]
        key: Secp256k1PubKey,
    }

    impl TryFrom<RawPubKey> for PubKey {
        type Error = Error;

        fn try_from(raw: RawPubKey) -> Result<Self, Self::Error> {
            PubKey::try_from(raw.key)
        }
    }

    impl From<PubKey> for RawPubKey {
        fn from(key: PubKey) -> RawPubKey {
            RawPubKey {
                key: Vec::from(key),
            }
        }
    }

    impl Protobuf<RawPubKey> for PubKey {}

    impl PubKey {
        /// Returns a Bitcoin style addresses: RIPEMD160(SHA256(pubkey))
        pub fn get_address(&self) -> AccAddress {
            let mut hasher = Sha256::new();
            hasher.update(&Vec::from(self.to_owned()));
            let hash = hasher.finalize();

            let mut hasher = Ripemd160::new();
            hasher.update(hash);
            let hash = hasher.finalize();

            let res: AccAddress = hash.as_slice().try_into().expect(
                "ripemd160 digest size is 160 bytes which is less than AccAddress::MAX_ADDR_LEN",
            );

            res
        }

        #[cfg(feature = "testing")]
        /// Function for creating of test object
        pub fn new(key: Secp256k1PubKey) -> Self {
            Self { key }
        }
    }

    impl From<PubKey> for Vec<u8> {
        fn from(key: PubKey) -> Vec<u8> {
            key.key.serialize().to_vec()
        }
    }

    impl TryFrom<Vec<u8>> for PubKey {
        type Error = Error;

        fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
            let key = Secp256k1PubKey::from_slice(&value)
                .map_err(|e| Error::DecodeGeneral(format!("invalid key: {e}")))?;

            Ok(PubKey { key })
        }
    }

    fn serialize_key<S>(key: &Secp256k1PubKey, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        s.serialize_str(&general_purpose::STANDARD.encode(key.serialize()))
    }

    fn deserialize_key<'de, D>(deserializer: D) -> Result<Secp256k1PubKey, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(Secp256k1Visitor)
    }

    struct Secp256k1Visitor;

    impl<'de> de::Visitor<'de> for Secp256k1Visitor {
        type Value = Secp256k1PubKey;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("string-encoded secp256k1 public key")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            let key = general_purpose::STANDARD
                .decode(v)
                .map_err(|e| E::custom(format!("Error parsing public key '{}': {}", v, e)))?;

            Secp256k1PubKey::from_slice(&key)
                .map_err(|e| E::custom(format!("Error parsing public key '{}': {}", v, e)))
        }
    }
}

#[cfg(test)]
mod tests {

    use super::v1beta1::*;

    #[test]
    fn get_address_works() {
        let key = hex::decode("02950e1cdfcb133d6024109fd489f734eeb4502418e538c28481f22bce276f248c")
            .unwrap();
        let raw = RawPubKey { key };
        let key: PubKey = raw.try_into().unwrap();
        let address = key.get_address();
        let address: Vec<u8> = address.into();

        assert_eq!(
            hex::encode(address),
            "7c2bb42a8be69791ec763e51f5a49bcd41e82237"
        )
    }
}
