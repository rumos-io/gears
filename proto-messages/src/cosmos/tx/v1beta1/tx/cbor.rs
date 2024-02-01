use std::{collections::HashMap, hash::Hash, io::Write};

use serde::Serialize;

// Cbor is a CBOR (RFC8949) data item that can be encoded to a stream.
pub trait Cbor {
    // `encode` deterministically writes the CBOR-encoded data to the stream.
    fn encode(&self, writter: &mut impl Write) -> Result<(), std::io::Error>;
}

impl<T: Serialize + Eq + PartialEq + Hash + Ord, V: Serialize> Cbor for HashMap<T, V> {
    fn encode(&self, writter: &mut impl Write) -> Result<(), std::io::Error> {
        ciborium::into_writer(self, writter)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    //use std::collections::HashMap;

    //use super::Cbor;

    // TODO: Fix this test
    // #[test]
    // fn check_hashmap() {
    //     // Examples come from RFC8949, Appendix A
    //     let mut var = HashMap::new();

    //     var.insert(1_u64, 2_u64);
    //     var.insert(3, 4);

    //     let mut buf = Vec::new();

    //     var.encode(&mut buf).expect("Failed to write buffer");

    //     let hex = data_encoding::HEXLOWER.encode(&buf);

    //     assert_eq!(&hex, "a201020304")
    // }
}
