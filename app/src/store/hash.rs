use integer_encoding::VarInt;
use sha2::{Digest, Sha256};

#[derive(Debug)]
pub struct StoreInfo {
    pub name: String,
    pub hash: [u8; 32],
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct Pair {
    key: Vec<u8>,
    value: Vec<u8>,
}

impl Pair {
    // TODO: is this the same as plain protobuf encoding?
    fn to_bytes(&self) -> Vec<u8> {
        let key_length = self.key.len().encode_var_vec();
        let value_length = self.value.len().encode_var_vec();

        [
            key_length,
            self.key.clone(),
            value_length,
            self.value.clone(),
        ]
        .concat()
    }
}

impl From<StoreInfo> for Pair {
    fn from(info: StoreInfo) -> Self {
        Pair {
            key: info.name.into(),
            value: Sha256::digest(info.hash).to_vec(),
        }
    }
}

pub fn hash_store_infos(store_infos: Vec<StoreInfo>) -> [u8; 32] {
    if store_infos.len() == 0 {
        panic!("must contain at least one store")
    };

    let mut pairs: Vec<Pair> = store_infos.into_iter().map(|info| info.into()).collect();
    pairs.sort();
    let byte_pairs: Vec<Vec<u8>> = pairs.into_iter().map(|pair| pair.to_bytes()).collect();
    trees::merkle::root_hash(&byte_pairs)
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn hash_store_infos_works() {
        let store_infos = vec![StoreInfo {
            name: "bob".to_string(),
            hash: hex::decode("45aa73be3d99644509f273acc713717f7c49caacd64226216e6263fdd8a3296c")
                .unwrap()
                .try_into()
                .unwrap(),
        }];
        assert_eq!(
            hex::encode(hash_store_infos(store_infos)),
            "08e21bd642c10dfca8510b6ca47b22b3a388817d0874ca23b506bc53105fbf18"
        );

        let store_infos = vec![
            StoreInfo {
                name: "bob".to_string(),
                hash: hex::decode(
                    "45aa73be3d99644509f273acc713717f7c49caacd64226216e6263fdd8a3296c",
                )
                .unwrap()
                .try_into()
                .unwrap(),
            },
            StoreInfo {
                name: "alice".to_string(),
                hash: hex::decode(
                    "c70e5a44aceeb02764ce49920ddd7c7abe0d2bb28be890764d6912c187144520",
                )
                .unwrap()
                .try_into()
                .unwrap(),
            },
        ];
        assert_eq!(
            hex::encode(hash_store_infos(store_infos)),
            "9328960b097a043bd62b6d22075084251688dff84d004743d0666f4ecdd5b86d"
        );
    }
}
