use std::fmt;

use serde::{de, Deserialize, Deserializer};

use super::fixed_hash::H256;

#[derive(Debug, Deserialize)]
pub struct Block {
    #[serde(deserialize_with = "H256::deserialize_hex")]
    pub hash: H256,
    pub confirmations: i32,
    pub size: u32,
    pub strippedsize: u32,
    pub weight: u32,
    pub height: u32,
    pub version: u32,
    #[serde(deserialize_with = "hex::deserialize", rename = "versionHex")]
    pub version_hex: Vec<u8>,
    #[serde(deserialize_with = "H256::deserialize_hex")]
    pub merkleroot: H256,
    #[serde(rename = "tx")]
    pub transactions: Vec<Transaction>,
    pub time: u32,
    pub mediantime: u32,
    pub nonce: u32,
    #[serde(deserialize_with = "hex::deserialize")]
    pub bits: Vec<u8>,
    pub difficulty: f64,
    #[serde(deserialize_with = "H256::deserialize_hex")]
    pub chainwork: H256,
    #[serde(rename = "nTx")]
    pub n_tx: u32,
    #[serde(deserialize_with = "H256::deserialize_hex_some", default)]
    pub previousblockhash: Option<H256>,
    #[serde(deserialize_with = "H256::deserialize_hex_some", default)]
    pub nextblockhash: Option<H256>,
}

#[derive(Debug, Deserialize)]
pub struct Transaction {
    #[serde(deserialize_with = "hex::deserialize")]
    pub hex: Vec<u8>,
    #[serde(deserialize_with = "H256::deserialize_hex")]
    pub txid: H256,
    #[serde(deserialize_with = "H256::deserialize_hex")]
    pub hash: H256,
    pub size: u32,
    pub vsize: u32,
    pub weight: u32,
    pub version: u32,
    pub locktime: u32,
    pub vin: Vec<TransactionInput>,
    pub vout: Vec<TransactionOutput>,
}

#[derive(Debug)]
pub enum TransactionInput {
    Coinbase {
        hex: Vec<u8>,
        sequence: u32,
    },
    Usual {
        txid: Option<H256>,
        vout: u32,
        script: TransactionInputScript,
        sequence: u32,
        txinwitness: Option<Vec<Vec<u8>>>,
    },
}

impl<'de> Deserialize<'de> for TransactionInput {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<TransactionInput, D::Error> {
        struct Visitor;

        impl<'de> de::Visitor<'de> for Visitor {
            type Value = TransactionInput;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a JSON object as transaction input")
            }

            fn visit_map<V>(self, mut visitor: V) -> Result<TransactionInput, V::Error>
            where
                V: de::MapAccess<'de>,
            {
                let mut coinbase = None;
                let mut txid = None;
                let mut vout = None;
                let mut sequence = None;
                let mut script = None;
                let mut txinwitness = None;

                macro_rules! check_duplicate {
                    ($var:ident, $name:expr) => {
                        if $var.is_some() {
                            return Err(de::Error::duplicate_field($name));
                        }
                    };
                }

                while let Some(key) = visitor.next_key()? {
                    match key {
                        "coinbase" => {
                            check_duplicate!(coinbase, "coinbase");
                            let value = visitor.next_value()?;
                            coinbase = Some(hex::decode(value).map_err(|_| {
                                de::Error::invalid_value(de::Unexpected::Str(value), &self)
                            })?);
                        }
                        "txid" => {
                            check_duplicate!(txid, "txid");
                            let value = visitor.next_value()?;
                            let mut hash = H256::zero();
                            hex::decode_to_slice(value, &mut hash.0 as &mut [u8]).map_err(
                                |_| de::Error::invalid_value(de::Unexpected::Str(value), &self),
                            )?;
                            txid = Some(Some(hash));
                        }
                        "vout" => {
                            check_duplicate!(vout, "vout");
                            vout = Some(visitor.next_value::<u32>()?);
                        }
                        "sequence" => {
                            check_duplicate!(sequence, "sequence");
                            sequence = Some(visitor.next_value::<u32>()?);
                        }
                        "scriptSig" => {
                            check_duplicate!(script, "script");
                            script = Some(visitor.next_value::<TransactionInputScript>()?);
                        }
                        "txinwitness" => {
                            check_duplicate!(txinwitness, "txinwitness");
                            let items = visitor.next_value::<Vec<&str>>()?;
                            let mut witness = Vec::with_capacity(items.len());
                            for item in items.iter() {
                                witness.push(hex::decode(item).map_err(de::Error::custom)?)
                            }
                            txinwitness = Some(witness);
                        }
                        _ => {
                            return Err(de::Error::unknown_field(key, &[]));
                        }
                    }
                }

                macro_rules! extra_field {
                    ($var:ident, $name:expr, $expected:expr) => {
                        if $var.is_some() {
                            return Err(de::Error::unknown_field($name, $expected));
                        }
                    };
                }

                Ok(if coinbase.is_some() {
                    let coinbase_fields = &["coinbase", "sequence"];
                    extra_field!(txid, "txid", coinbase_fields);
                    extra_field!(vout, "vout", coinbase_fields);
                    extra_field!(script, "script", coinbase_fields);
                    extra_field!(txinwitness, "txinwitness", coinbase_fields);

                    TransactionInput::Coinbase {
                        hex: coinbase.ok_or_else(|| de::Error::missing_field("coinbase"))?,
                        sequence: sequence.unwrap(),
                    }
                } else {
                    let usual_fields = &["txid", "vout", "scriptSig", "sequence", "txinwitness"];
                    extra_field!(coinbase, "coinbase", usual_fields);

                    TransactionInput::Usual {
                        txid: txid.ok_or_else(|| de::Error::missing_field("txid"))?,
                        vout: vout.ok_or_else(|| de::Error::missing_field("vout"))?,
                        script: script.ok_or_else(|| de::Error::missing_field("scriptSig"))?,
                        sequence: sequence.unwrap(),
                        txinwitness,
                    }
                })
            }
        }

        deserializer.deserialize_map(Visitor)
    }
}

#[derive(Debug, Deserialize)]
pub struct TransactionInputScript {
    pub asm: String,
    #[serde(deserialize_with = "hex::deserialize")]
    pub hex: Vec<u8>,
}

#[derive(Debug, Deserialize)]
pub struct TransactionOutput {
    // Require `serde_json` with feature `arbitrary_precision`
    #[serde(deserialize_with = "de_vout_value")]
    pub value: String,
    pub n: u32,
    #[serde(rename(deserialize = "scriptPubKey"))]
    pub script: TransactionOutputScript,
}

fn de_vout_value<'de, D: Deserializer<'de>>(deserializer: D) -> Result<String, D::Error> {
    struct Visitor;

    impl<'de> de::Visitor<'de> for Visitor {
        type Value = String;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a JSON number")
        }

        fn visit_map<V>(self, mut visitor: V) -> Result<String, V::Error>
        where
            V: de::MapAccess<'de>,
        {
            let value = visitor.next_key::<String>()?;
            if value.is_none() {
                return Err(de::Error::invalid_type(de::Unexpected::Map, &self));
            }
            visitor.next_value()
        }
    }

    deserializer.deserialize_any(Visitor)
}

#[derive(Debug, Deserialize)]
pub struct TransactionOutputScript {
    pub asm: String,
    #[serde(deserialize_with = "hex::deserialize")]
    pub hex: Vec<u8>,
    #[serde(rename = "reqSigs")]
    pub req_sigs: Option<u32>,
    pub r#type: String,
    pub addresses: Option<Vec<String>>,
}
