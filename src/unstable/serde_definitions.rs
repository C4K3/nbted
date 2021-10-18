use crate::data::NBT;


use std::str::from_utf8;
use serde::{Serialize, Serializer, ser::{SerializeMap, SerializeSeq, Error}};


impl Serialize for NBT {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            NBT::End => return Err(Error::custom("cannot serialize NBT End tag")),
            NBT::Byte(x) => serializer.serialize_i8(*x),
            NBT::Short(x) => serializer.serialize_i16(*x),
            NBT::Int(x) => serializer.serialize_i32(*x),
            NBT::Long(x) => serializer.serialize_i64(*x),
            NBT::Float(x) => serializer.serialize_f32(*x),
            NBT::Double(x) => serializer.serialize_f64(*x),
            NBT::ByteArray(x) => {
                let mut seq = serializer.serialize_seq(Some(x.len()))?;
                for b in x {
                    seq.serialize_element(b)?;
                }
                seq.end()
            },
            NBT::String(x) => {
                let keystr = match from_utf8(x) {
                    Ok(s) => s,
                    Err(error) => return Err(Error::custom(format!("invalid UTF-8 string after `{}`", from_utf8(&x[..error.valid_up_to()]).unwrap())))
                };
                serializer.serialize_str(keystr)
            },
            NBT::List(x) => {
                let mut seq = serializer.serialize_seq(Some(x.len()))?;
                for b in x {
                    seq.serialize_element(b)?;
                }
                seq.end()
            },
            NBT::Compound(x) => {
                let mut comp = serializer.serialize_map(Some(x.len()))?;
                for (k, v) in x {
                    let keystr = match from_utf8(k) {
                        Ok(s) => s,
                        Err(error) => return Err(Error::custom(format!("invalid UTF-8 string after '{}'", from_utf8(&k[..error.valid_up_to()]).unwrap())))
                    };
                    comp.serialize_entry(keystr, v)?;
                }
                comp.end()
            },
            NBT::IntArray(x) => {
                let mut seq = serializer.serialize_seq(Some(x.len()))?;
                for b in x {
                    seq.serialize_element(b)?;
                }
                seq.end()
            },
            NBT::LongArray(x) => {
                let mut seq = serializer.serialize_seq(Some(x.len()))?;
                for b in x {
                    seq.serialize_element(b)?;
                }
                seq.end()
            },
        }
    }
}

