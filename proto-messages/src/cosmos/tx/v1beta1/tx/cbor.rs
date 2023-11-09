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

const MAJOR_U64: u8 = 0;
const MAJOR_TEXT_STRING: u8 = 3;
const MAJOR_ARRAY: u8 = 4;
const MAJOR_MAP: u8 = 5;
const MAJOR_SIMPLE: u8 = 7;

fn first_byte_encode(major: u8, extra: u8) -> u8 {
    (major << 5) | extra & 0x1F
}

fn prefix_encode(major: u8, arg: u64, writter: &mut impl Write) -> Result<(), std::io::Error> {
    const U8_MAX: u64 = u8::MAX as u64;
    const U16_MAX: u64 = u16::MAX as u64;
    const U32_MAX: u64 = u32::MAX as u64;

    match arg {
        ..=23 => writter.write_all(&[first_byte_encode(major, arg as u8)]),
        ..=U8_MAX => writter.write_all(&[first_byte_encode(major, 24), arg as u8]),
        ..=U16_MAX => {
            writter.write_all(&[first_byte_encode(major, 25)])?;
            writter.write_u64::<BigEndian>(arg) // TODO: go code write as U16
        }
        ..=U32_MAX => {
            writter.write_all(&[first_byte_encode(major, 26)])?;
            writter.write_u64::<BigEndian>(arg) // TODO: go code write as U32
        }
        _ => {
            writter.write_all(&[first_byte_encode(major, 27)])?;
            writter.write_u64::<BigEndian>(arg)
        }
    }
}

// Cbor is a CBOR (RFC8949) data item that can be encoded to a stream.
pub trait Cbor {
    // `encode` deterministically writes the CBOR-encoded data to the stream.
    fn encode(&self, writter: &mut impl Write) -> Result<(), std::io::Error>;
}

#[derive(Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum CborPrimitivies<'a> {
    Uint64(u64),
    String(&'a str),
    Bool(bool),
}

impl<'a> Cbor for CborPrimitivies<'_> {
    fn encode(&self, writter: &mut impl Write) -> Result<(), std::io::Error> {
        match self {
            CborPrimitivies::Uint64(var) => var.encode(writter),
            CborPrimitivies::String(var) => var.encode(writter),
            CborPrimitivies::Bool(var) => var.encode(writter),
        }
    }
}

impl Cbor for u64 {
    fn encode(&self, writter: &mut impl Write) -> Result<(), std::io::Error> {
        prefix_encode(MAJOR_U64, *self, writter)
    }
}

impl Cbor for &str {
    fn encode(&self, writter: &mut impl Write) -> Result<(), std::io::Error> {
        prefix_encode(MAJOR_TEXT_STRING, self.len() as u64, writter)
    }
}

impl<T: Cbor> Cbor for &[T] {
    fn encode(&self, writter: &mut impl Write) -> Result<(), std::io::Error> {
        prefix_encode(MAJOR_ARRAY, self.len() as u64, writter)?;

        for item in self.iter() {
            item.encode(writter)?;
        }

        Ok(())
    }
}

impl<T: Cbor> Cbor for Vec<T> {
    fn encode(&self, writter: &mut impl Write) -> Result<(), std::io::Error> {
        AsRef::<[T]>::as_ref(self).encode(writter)
    }
}

impl<T: Cbor + Eq + PartialEq + Hash + Ord, V: Cbor> Cbor for HashMap<T, V> {
    fn encode(&self, writter: &mut impl Write) -> Result<(), std::io::Error> {
        prefix_encode(MAJOR_MAP, self.len() as u64, writter)?;

        // For deterministic encoding, map entries must be sorted by their
        // encoded keys in bytewise lexicographic order (RFC 8949, section 4.2.1).

        let mut rendered_keys = Vec::<(Vec<u8>, &T)>::with_capacity(self.len());
        for (key, _) in self.iter() {
            let mut buf = Vec::new();
            key.encode(&mut buf)?;

            rendered_keys.push((buf, key));
        }

        rendered_keys.sort(); // TODO: rust default sort should do, but make sure it does

        let mut prev_key = None;
        for (bytes, idx) in rendered_keys.iter() {
            if let Some(prev_key) = prev_key {
                if prev_key == bytes {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "duplicate map keys",
                    ));
                }
            }
            prev_key = Some(bytes);

            writter.write_all(bytes)?;
            let var = self.get(idx).ok_or(std::io::Error::new(
                std::io::ErrorKind::Other,
                "failed to retrieve value",
            ))?;

            var.encode(writter)?;
        }

        Ok(())
    }
}

impl Cbor for bool {
    fn encode(&self, writter: &mut impl Write) -> Result<(), std::io::Error> {
        let arg =
            u64::try_from(*self).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        prefix_encode(MAJOR_SIMPLE, arg, writter)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::Cbor;

    #[test]
    fn check_unsigned_int() {
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
            (r#"\"\\"#, "62225c"),
            (r#"\u00fc"#, "62c3bc"),
            (r#"\u6c34"#, "63e6b0b4"),
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

    #[test]
    fn check_hashmap() {
        // Examples come from RFC8949, Appendix A
        let mut var = HashMap::new();

        var.insert( 1, 2);
        var.insert( 3, 4);

        let mut buf = Vec::new();

        var.encode( &mut buf).expect("Failed to write buffer");

        let hex = data_encoding::HEXLOWER.encode(&buf);

        assert_eq!( &hex, "a201020304")
    }

    fn validate_result<'a, T: Cbor>(value: impl IntoIterator<Item = (T, &'a str)>) {
        for (i, expected) in value {
            let mut buf = Vec::new();

            i.encode(&mut buf)
                .expect("Failed to write buffer");

            let expected = data_encoding::HEXLOWER.decode(expected.as_bytes()).unwrap();
            assert_eq!(buf, expected, "{buf:02x?} != {expected:02x?}");
        }
    }
}
