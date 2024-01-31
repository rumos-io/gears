use std::{collections::HashMap, hash::Hash, io::Write};

use serde::Serialize;

// Cbor is a CBOR (RFC8949) data item that can be encoded to a stream.
pub trait Cbor {
    // `encode` deterministically writes the CBOR-encoded data to the stream.
    fn encode(&self, writter: &mut impl Write) -> Result<(), std::io::Error>;
}

impl<T: Serialize> Cbor for &[T] {
    fn encode(&self, writter: &mut impl Write) -> Result<(), std::io::Error> {
        ciborium::into_writer(self, writter)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
    }
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

    use super::Cbor;

    #[test]
    fn check_array() {
        // Examples come from RFC8949, Appendix A
        let var = [
            ([].as_ref(), "80"),
            (&[1_u64, 2, 3], "83010203"),
            (
                &[
                    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22,
                    23, 24, 25,
                ],
                "98190102030405060708090a0b0c0d0e0f101112131415161718181819",
            ),
        ];

        validate_result(var)
    }

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

    fn validate_result<'a, T: Cbor>(value: impl IntoIterator<Item = (T, &'a str)>) {
        for (i, expected) in value {
            let mut buf = Vec::new();

            i.encode(&mut buf).expect("Failed to write buffer");

            let expected = data_encoding::HEXLOWER.decode(expected.as_bytes()).unwrap();
            assert_eq!(buf, expected, "{buf:02x?} != {expected:02x?}");
        }
    }
}
