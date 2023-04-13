pub mod v1beta1 {
    use ibc_proto::protobuf::Protobuf;
    use proto_types::AccAddress;
    use ripemd::Ripemd160;
    use serde::{Deserialize, Serialize};
    use sha2::{Digest, Sha256};

    use crate::Error;

    // PubKeySize is comprised of 32 bytes for one field element
    // (the x-coordinate), plus one byte for the parity of the y-coordinate.
    const PUB_KEY_SIZE: usize = 33;

    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct RawPubKey {
        #[prost(bytes = "vec", tag = "1")]
        pub key: Vec<u8>,
    }

    #[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
    pub struct PubKey {
        key: Vec<u8>,
    }

    impl TryFrom<RawPubKey> for PubKey {
        type Error = Error;

        fn try_from(raw: RawPubKey) -> Result<Self, Self::Error> {
            if raw.key.len() != PUB_KEY_SIZE {
                return Err(Error::DecodeGeneral("invalid key length".into()));
            }
            Ok(PubKey { key: raw.key })
        }
    }

    impl From<PubKey> for RawPubKey {
        fn from(key: PubKey) -> RawPubKey {
            RawPubKey { key: key.key }
        }
    }

    impl Protobuf<RawPubKey> for PubKey {}

    impl PubKey {
        /// Returns a Bitcoin style addresses: RIPEMD160(SHA256(pubkey))
        pub fn get_address(&self) -> AccAddress {
            let mut hasher = Sha256::new();
            hasher.update(&self.key);
            let hash = hasher.finalize();

            let mut hasher = Ripemd160::new();
            hasher.update(hash);
            let hash = hasher.finalize();

            let res: AccAddress = hash.as_slice().try_into().expect(
                "ripemd160 digest size is 160 bytes which is less than AccAddress::MAX_ADDR_LEN",
            );

            res
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
