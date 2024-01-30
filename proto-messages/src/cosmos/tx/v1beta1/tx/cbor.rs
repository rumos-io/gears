// const (
// 	majorUint       byte = 0
// 	majorNegInt     byte = 1
// 	majorByteString byte = 2
// 	majorTextString byte = 3
// 	majorArray      byte = 4
// 	majorMap        byte = 5
// 	majorTagged     byte = 6
// 	majorSimple     byte = 7
// )

use std::{collections::HashMap, hash::Hash, io::Write, u8};

use byteorder::{BigEndian, WriteBytesExt};
use serde::Serialize;

const MAJOR_U64: u8 = 0;
// const MAJOR_TEXT_STRING: u8 = 3;
// const MAJOR_ARRAY: u8 = 4;
// const MAJOR_MAP: u8 = 5;
// const MAJOR_SIMPLE: u8 = 7;

// Cbor is a CBOR (RFC8949) data item that can be encoded to a stream.
pub trait Cbor {
    // `encode` deterministically writes the CBOR-encoded data to the stream.
    fn encode(&self, writter: &mut impl Write) -> Result<(), std::io::Error>;
}

fn first_byte_encode(major: u8, extra: u8) -> u8 {
    (major << 5) | extra & 0x1F
}

fn prefix_encode(major: u8, arg: u64, writter: &mut impl Write) -> Result<(), std::io::Error> {
    const U8_MAX: u64 = u8::MAX as u64;
    const U16_MAX: u64 = u16::MAX as u64;
    const U32_MAX: u64 = u32::MAX as u64;

    match arg {
        ..=U8_MAX => (arg as u8).encode(writter),
        ..=U16_MAX => {
            writter.write_u8(first_byte_encode(MAJOR_U64, 25))?;
            writter.write_u16::<BigEndian>(arg as u16)
        }
        ..=U32_MAX => {
            writter.write_u8(first_byte_encode(MAJOR_U64, 26))?;
            writter.write_u32::<BigEndian>(arg as u32)
        }
        _ => {
            writter.write_u8(first_byte_encode(major, 27))?;
            writter.write_u64::<BigEndian>(arg)
        }
    }
}

impl Cbor for u8 {
    fn encode(&self, writter: &mut impl Write) -> Result<(), std::io::Error> {
        match *self {
            ..=23 => writter.write_u8(first_byte_encode(MAJOR_U64, *self)),
            _ => writter.write_all(&[first_byte_encode(MAJOR_U64, 24), *self]),
        }
    }
}

impl Cbor for u16 {
    fn encode(&self, writter: &mut impl Write) -> Result<(), std::io::Error> {
        (*self as u64).encode(writter)
    }
}

impl Cbor for u32 {
    fn encode(&self, writter: &mut impl Write) -> Result<(), std::io::Error> {
        (*self as u64).encode(writter)
    }
}

impl Cbor for u64 {
    fn encode(&self, writter: &mut impl Write) -> Result<(), std::io::Error> {
        prefix_encode(MAJOR_U64, *self, writter)
    }
}

impl Cbor for bool {
    fn encode(&self, writter: &mut impl Write) -> Result<(), std::io::Error> {
        ciborium::into_writer(self, writter)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
    }
}

#[derive(Eq, PartialEq, Hash, Ord, PartialOrd, Serialize)]
pub enum CborPrimitivies {
    Uint8(u8),
    Uint16(u16),
    Uint32(u32),
    Uint64(u64),
    String(String),
    Bool(bool),
}

impl Cbor for CborPrimitivies {
    fn encode(&self, writter: &mut impl Write) -> Result<(), std::io::Error> {
        match self {
            CborPrimitivies::Uint8(var) => var.encode(writter),
            CborPrimitivies::Uint16(var) => var.encode(writter),
            CborPrimitivies::Uint32(var) => var.encode(writter),
            CborPrimitivies::Uint64(var) => var.encode(writter),
            CborPrimitivies::String(var) => var.encode(writter),
            CborPrimitivies::Bool(var) => var.encode(writter),
        }
    }
}

impl Cbor for &str {
    fn encode(&self, writter: &mut impl Write) -> Result<(), std::io::Error> {
        ciborium::into_writer(self, writter)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
    }
}

impl Cbor for String {
    fn encode(&self, writter: &mut impl Write) -> Result<(), std::io::Error> {
        ciborium::into_writer(self, writter)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
    }
}

// impl<T: Serialize, U: Iterator<Item = T>> Cbor for U{
//     fn encode(&self, writter: &mut impl Write) -> Result<(), std::io::Error> {
//         ciborium::into_writer(self, writter)
//         .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
//     }
// }

impl<T: Serialize> Cbor for &[T] {
    fn encode(&self, writter: &mut impl Write) -> Result<(), std::io::Error> {
        ciborium::into_writer(self, writter)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
    }
}

// impl<T: Serialize> Cbor for Vec<T> {
//     fn encode(&self, writter: &mut impl Write) -> Result<(), std::io::Error> {
//         AsRef::<[T]>::as_ref(self).encode(writter)
//     }
// }

impl<T: Serialize + Eq + PartialEq + Hash + Ord, V: Serialize> Cbor for HashMap<T, V> {
    fn encode(&self, writter: &mut impl Write) -> Result<(), std::io::Error> {
        ciborium::into_writer(self, writter)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::Cbor;

    #[test]
    fn check_u8() {
        // Examples come from RFC8949, Appendix A
        let var = [
            (0_u8, "00"),
            (1, "01"),
            (10, "0a"),
            (23, "17"),
            (24, "1818"),
            (25, "1819"),
            (100, "1864"),
        ];

        validate_result(var)
    }

    #[test]
    fn check_u16() {
        // Examples come from RFC8949, Appendix A
        let var = [
            (0_u16, "00"),
            (1, "01"),
            (10, "0a"),
            (23, "17"),
            (24, "1818"),
            (25, "1819"),
            (100, "1864"),
            (1000, "1903e8"),
        ];

        validate_result(var)
    }

    #[test]
    fn check_u32() {
        // Examples come from RFC8949, Appendix A
        let var = [
            (0_u32, "00"),
            (1, "01"),
            (10, "0a"),
            (23, "17"),
            (24, "1818"),
            (25, "1819"),
            (100, "1864"),
            (1000, "1903e8"),
            (1000000, "1a000f4240"),
        ];

        validate_result(var)
    }

    #[test]
    fn check_u64() {
        // Examples come from RFC8949, Appendix A
        let var = [
            (0_u64, "00"),
            (1, "01"),
            (10, "0a"),
            (23, "17"),
            (24, "1818"),
            (25, "1819"),
            (100, "1864"),
            (1000, "1903e8"),
            (1000000, "1a000f4240"),
            (1000000000000, "1b000000e8d4a51000"),
            (18446744073709551615, "1bffffffffffffffff"),
        ];

        validate_result(var)
    }

    #[test]
    fn check_bool() {
        // Examples come from RFC8949, Appendix A
        let var = [(false, "f4"), (true, "f5")];

        validate_result(var)
    }

    #[test]
    fn check_str() {
        // Examples come from RFC8949, Appendix A
        let var = [
            ("", "60"),
            ("a", "6161"),
            ("IETF", "6449455446"),
            // (r#"\"\\"#, "62225c"),
            // (r#"\u00fc"#, "62c3bc"),
            // (r#"\u6c34"#, "63e6b0b4"),
        ];

        validate_result(var)
    }

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
