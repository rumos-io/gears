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

fn prefix_encode<W: Write>(major: u8, arg: u64, writter: &mut W) -> Result<(), std::io::Error> {
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

pub trait Cbor {
    fn encode<W: Write>(&self, writter: &mut W) -> Result<(), std::io::Error>;
}

impl Cbor for u64 {
    fn encode<W: Write>(&self, writter: &mut W) -> Result<(), std::io::Error> {
        prefix_encode(MAJOR_U64, *self, writter)
    }
}

impl Cbor for &str {
    fn encode<W: Write>(&self, writter: &mut W) -> Result<(), std::io::Error> {
        prefix_encode(MAJOR_TEXT_STRING, self.len() as u64, writter)
    }
}

impl<T: Cbor> Cbor for &[T] {
    fn encode<W: Write>(&self, writter: &mut W) -> Result<(), std::io::Error> {
        prefix_encode(MAJOR_ARRAY, self.len() as u64, writter)?;

        for item in self.iter() {
            item.encode(writter)?;
        }

        Ok(())
    }
}

impl<T: Cbor + Eq + PartialEq + Hash + Ord, U: Cbor> Cbor for HashMap<T, U> {
    fn encode<W: Write>(&self, writter: &mut W) -> Result<(), std::io::Error> {
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
    fn encode<W: Write>(&self, writter: &mut W) -> Result<(), std::io::Error> {
        let arg =
            u64::try_from(*self).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        prefix_encode(MAJOR_SIMPLE, arg, writter)
    }
}

// TODO: tests
