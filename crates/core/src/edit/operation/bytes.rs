//! A file's bytes in an operation: written as bytes — a `Uint8Array` through
//! `serde-wasm-bindgen`, an array of numbers in JSON — and read from either.

use std::fmt;

use serde::de::{SeqAccess, Visitor};
use serde::{Deserializer, Serializer};

pub(super) fn serialize<S: Serializer>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_bytes(bytes)
}

pub(super) fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<u8>, D::Error> {
    deserializer.deserialize_byte_buf(BytesVisitor)
}

struct BytesVisitor;

impl<'de> Visitor<'de> for BytesVisitor {
    type Value = Vec<u8>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("bytes, or an array of numbers from 0 to 255")
    }

    fn visit_bytes<E>(self, bytes: &[u8]) -> Result<Self::Value, E> {
        Ok(bytes.to_vec())
    }

    fn visit_byte_buf<E>(self, bytes: Vec<u8>) -> Result<Self::Value, E> {
        Ok(bytes)
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut sequence: A) -> Result<Self::Value, A::Error> {
        let mut bytes = Vec::with_capacity(sequence.size_hint().unwrap_or_default());
        while let Some(byte) = sequence.next_element::<u8>()? {
            bytes.push(byte);
        }
        Ok(bytes)
    }
}
