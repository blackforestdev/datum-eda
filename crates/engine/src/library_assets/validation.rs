use std::collections::BTreeMap;

use serde_json::Value;

use super::invalid;
use crate::error::EngineError;

pub(super) fn extensions(fields: Option<&BTreeMap<String, Value>>) -> Result<(), EngineError> {
    if let Some(fields) = fields {
        for key in fields.keys() {
            let labels: Vec<_> = key.split('.').collect();
            if labels.len() < 2
                || labels.iter().any(|label| {
                    label.is_empty()
                        || !label
                            .bytes()
                            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-'))
                })
            {
                return Err(invalid(
                    "extension key requires a nonempty qualified namespace",
                ));
            }
        }
    }
    Ok(())
}

/// Calendar dates or extended ISO calendar timestamps with an explicit offset.
/// Unsupported representations refuse, without inventing precision or a zone.
/// This checks representation/ranges, not the factual truth of a claimed time.
pub(super) fn timestamp(value: Option<&str>) -> Result<(), EngineError> {
    let Some(value) = value else {
        return Ok(());
    };
    let b = value.as_bytes();
    let valid = valid_date(b.get(..10).unwrap_or_default())
        && (b.len() == 10
            || b.get(10).is_some_and(|c| matches!(c, b'T' | b't')) && valid_time(&b[11..]));
    if valid {
        Ok(())
    } else {
        Err(invalid("unsupported or invalid ISO calendar timestamp"))
    }
}

fn decimal(bytes: &[u8]) -> Option<u32> {
    if bytes.is_empty() {
        return None;
    }
    bytes.iter().try_fold(0u32, |v, b| {
        b.is_ascii_digit().then_some(())?;
        v.checked_mul(10)?.checked_add(u32::from(b - b'0'))
    })
}

fn valid_date(b: &[u8]) -> bool {
    if b.len() != 10 || b[4] != b'-' || b[7] != b'-' {
        return false;
    }
    let (Some(year), Some(month), Some(day)) =
        (decimal(&b[..4]), decimal(&b[5..7]), decimal(&b[8..10]))
    else {
        return false;
    };
    let days = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if year % 400 == 0 || year % 4 == 0 && year % 100 != 0 => 29,
        2 => 28,
        _ => return false,
    };
    (1..=days).contains(&day)
}

fn valid_time(b: &[u8]) -> bool {
    let split = if matches!(b.last(), Some(b'Z' | b'z')) {
        b.len() - 1
    } else {
        let Some(split) = b.iter().position(|b| matches!(b, b'+' | b'-')) else {
            return false;
        };
        let offset = &b[split..];
        if offset.len() != 6
            || offset[3] != b':'
            || !decimal(&offset[1..3]).is_some_and(|h| h <= 23)
            || !decimal(&offset[4..]).is_some_and(|m| m <= 59)
        {
            return false;
        }
        split
    };
    let time = &b[..split];
    if time.len() < 5
        || time[2] != b':'
        || !decimal(&time[..2]).is_some_and(|h| h <= 23)
        || !decimal(&time[3..5]).is_some_and(|m| m <= 59)
    {
        return false;
    }
    if time.len() == 5 {
        return true;
    }
    if time.len() < 8 || time[5] != b':' || !decimal(&time[6..8]).is_some_and(|s| s <= 60) {
        return false;
    }
    time.len() == 8 || time[8] == b'.' && time.len() > 9 && time[9..].iter().all(u8::is_ascii_digit)
}
