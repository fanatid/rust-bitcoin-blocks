use std::fmt;

use bytes::Buf;
use fixed_hash::construct_fixed_hash;
use serde::{de, Deserializer};

use super::block_hex::StructFromBytes;

construct_fixed_hash! {
    pub struct H256(32);
}

macro_rules! impl_serde_hex {
    ($name:ident, $len:expr) => {
        impl $name {
            pub fn deserialize_hex<'de, D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct Visitor;

                impl<'de> de::Visitor<'de> for Visitor {
                    type Value = H256;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        write!(formatter, "a hex encoded string with len 64")
                    }

                    fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
                        let mut hash = $name::zero();
                        match hex::decode_to_slice(v, &mut hash.0 as &mut [u8]) {
                            Ok(_) => Ok(hash),
                            Err(_) => Err(E::invalid_length(v.len(), &self)),
                        }
                    }
                }

                deserializer.deserialize_str(Visitor)
            }

            pub fn deserialize_hex_some<'de, D>(deserializer: D) -> Result<Option<Self>, D::Error>
            where
                D: Deserializer<'de>,
            {
                Self::deserialize_hex(deserializer).map(Some)
            }
        }

        impl StructFromBytes for $name {
            type Output = $name;

            fn from_bytes(bytes: &mut &[u8]) -> Self::Output {
                let mut hash = $name::zero();
                bytes.copy_to_slice(hash.as_bytes_mut());
                hash
            }
        }
    };
}

impl_serde_hex!(H256, 32);
