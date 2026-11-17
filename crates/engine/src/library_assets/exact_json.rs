//! Duplicate-safe structural comparison without rounding unknown JSON numbers.
use std::{
    collections::{BTreeMap, VecDeque},
    fmt,
};

use serde::de::{DeserializeSeed, MapAccess, SeqAccess, Visitor};

use super::invalid;
use crate::error::EngineError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum ExactValue {
    Null,
    Bool(bool),
    String(String),
    Number {
        negative: bool,
        digits: String,
        exponent: i64,
    },
    Array(Vec<Self>),
    Object(BTreeMap<String, Self>),
}

pub(super) struct ParsedJson {
    pub(super) value: ExactValue,
    fractional_token: bool,
}

impl ParsedJson {
    pub(super) fn has_fractional_token(&self) -> bool {
        self.fractional_token
    }
}

pub(super) fn parse(bytes: &[u8]) -> Result<ParsedJson, EngineError> {
    // serde validates syntax/Unicode and depth; tokens preserve the original
    // decimal value even when serde's numeric visitor receives a rounded f64.
    let mut numbers = number_tokens(bytes)?;
    let fractional_token = numbers.iter().any(|n| n.contains(['.', 'e', 'E']));
    let mut de = serde_json::Deserializer::from_slice(bytes);
    let value = ExactSeed {
        numbers: &mut numbers,
    }
    .deserialize(&mut de)?;
    de.end()?;
    if !numbers.is_empty() {
        return Err(invalid("unconsumed numeric token"));
    }
    Ok(ParsedJson {
        value,
        fractional_token,
    })
}

fn number_tokens(bytes: &[u8]) -> Result<VecDeque<String>, EngineError> {
    let mut result = VecDeque::new();
    let (mut quoted, mut escaped, mut i) = (false, false, 0);
    while i < bytes.len() {
        let b = bytes[i];
        if quoted {
            if escaped {
                escaped = false;
            } else if b == b'\\' {
                escaped = true;
            } else if b == b'"' {
                quoted = false;
            }
            i += 1;
        } else if b == b'"' {
            quoted = true;
            i += 1;
        } else if b == b'-' || b.is_ascii_digit() {
            let start = i;
            while i < bytes.len()
                && matches!(bytes[i], b'0'..=b'9' | b'-' | b'+' | b'.' | b'e' | b'E')
            {
                i += 1;
            }
            result.push_back(
                std::str::from_utf8(&bytes[start..i])
                    .map_err(|_| invalid("invalid numeric token encoding"))?
                    .to_owned(),
            );
        } else {
            i += 1;
        }
    }
    Ok(result)
}

struct ExactSeed<'a> {
    numbers: &'a mut VecDeque<String>,
}

impl<'de> DeserializeSeed<'de> for ExactSeed<'_> {
    type Value = ExactValue;
    fn deserialize<D: serde::Deserializer<'de>>(self, de: D) -> Result<Self::Value, D::Error> {
        de.deserialize_any(self)
    }
}

impl ExactSeed<'_> {
    fn number<E: serde::de::Error>(&mut self) -> Result<ExactValue, E> {
        let raw = self
            .numbers
            .pop_front()
            .ok_or_else(|| E::custom("missing numeric token"))?;
        decimal_value(&raw).map_err(E::custom)
    }
}

impl<'de> Visitor<'de> for ExactSeed<'_> {
    type Value = ExactValue;
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("unambiguous JSON")
    }
    fn visit_unit<E: serde::de::Error>(self) -> Result<Self::Value, E> {
        Ok(ExactValue::Null)
    }
    fn visit_bool<E: serde::de::Error>(self, value: bool) -> Result<Self::Value, E> {
        Ok(ExactValue::Bool(value))
    }
    fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Self::Value, E> {
        Ok(ExactValue::String(value.to_owned()))
    }
    fn visit_string<E: serde::de::Error>(self, value: String) -> Result<Self::Value, E> {
        Ok(ExactValue::String(value))
    }
    fn visit_i64<E: serde::de::Error>(mut self, _: i64) -> Result<Self::Value, E> {
        self.number()
    }
    fn visit_u64<E: serde::de::Error>(mut self, _: u64) -> Result<Self::Value, E> {
        self.number()
    }
    fn visit_f64<E: serde::de::Error>(mut self, _: f64) -> Result<Self::Value, E> {
        self.number()
    }
    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        let mut values = Vec::new();
        while let Some(value) = seq.next_element_seed(ExactSeed {
            numbers: &mut *self.numbers,
        })? {
            values.push(value);
        }
        Ok(ExactValue::Array(values))
    }
    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
        let mut fields = BTreeMap::new();
        while let Some(key) = map.next_key::<String>()? {
            if fields.contains_key(&key) {
                return Err(serde::de::Error::custom(format!(
                    "duplicate JSON key {key:?}"
                )));
            }
            let value = map.next_value_seed(ExactSeed {
                numbers: &mut *self.numbers,
            })?;
            fields.insert(key, value);
        }
        Ok(ExactValue::Object(fields))
    }
}

fn decimal_value(raw: &str) -> Result<ExactValue, EngineError> {
    let negative = raw.starts_with('-');
    let unsigned = raw.strip_prefix('-').unwrap_or(raw);
    let mut parts = unsigned.split(['e', 'E']);
    let mantissa = parts
        .next()
        .ok_or_else(|| invalid("missing number mantissa"))?;
    let explicit_exponent = parts
        .next()
        .map(str::parse::<i64>)
        .transpose()
        .map_err(|_| invalid("unsupported numeric exponent"))?
        .unwrap_or(0);
    let fractional = mantissa
        .split_once('.')
        .map_or(0, |(_, fraction)| fraction.len());
    let mut exponent = explicit_exponent
        .checked_sub(
            i64::try_from(fractional).map_err(|_| invalid("numeric fraction is too long"))?,
        )
        .ok_or_else(|| invalid("numeric exponent overflow"))?;
    let digits: String = mantissa.chars().filter(|c| *c != '.').collect();
    let mut digits = digits.trim_start_matches('0').to_owned();
    if digits.is_empty() {
        return Ok(ExactValue::Number {
            negative: false,
            digits: "0".into(),
            exponent: 0,
        });
    }
    while digits.ends_with('0') {
        digits.pop();
        exponent = exponent
            .checked_add(1)
            .ok_or_else(|| invalid("numeric exponent overflow"))?;
    }
    Ok(ExactValue::Number {
        negative,
        digits,
        exponent,
    })
}
