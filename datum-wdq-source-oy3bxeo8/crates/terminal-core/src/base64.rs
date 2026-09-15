use crate::LimitError;
use crate::codec::{Base64Limits, CodecError, WorkBudget};

/// Decode canonical RFC 4648 Base64 without accepting whitespace or aliases.
pub fn decode_base64(input: &[u8], limits: Base64Limits) -> Result<Vec<u8>, CodecError> {
    let mut work = WorkBudget::new(limits.work);
    decode_base64_with_work(input, limits, &mut work)
}

pub(crate) fn decode_base64_with_work(
    input: &[u8],
    limits: Base64Limits,
    work: &mut WorkBudget,
) -> Result<Vec<u8>, CodecError> {
    work.charge(input.len())?;
    if input.is_empty() {
        return Ok(Vec::new());
    }
    if !input.len().is_multiple_of(4) {
        return Err(CodecError::InvalidBase64Padding {
            offset: input.len(),
        });
    }

    let padding = match &input[input.len() - 2..] {
        [b'=', b'='] => 2,
        [_, b'='] => 1,
        _ => 0,
    };
    let groups = input.len() / 4;
    let decoded = groups
        .checked_mul(3)
        .and_then(|value| value.checked_sub(padding))
        .ok_or(LimitError::ArithmeticOverflow { kind: limits.kind })?;
    if decoded > limits.decoded_bytes {
        return Err(LimitError::Exceeded {
            kind: limits.kind,
            requested: decoded,
            maximum: limits.decoded_bytes,
        }
        .into());
    }

    let mut output = Vec::with_capacity(decoded);
    for (group, quartet) in input.chunks_exact(4).enumerate() {
        let offset = group * 4;
        let final_group = group + 1 == groups;
        let a = value(quartet[0], offset)?;
        let b = value(quartet[1], offset + 1)?;

        if quartet[2] == b'=' {
            if quartet[3] != b'=' {
                return Err(CodecError::InvalidBase64 { offset: offset + 2 });
            }
            if !final_group {
                return Err(CodecError::InvalidBase64Padding { offset: offset + 2 });
            }
            if b & 0x0f != 0 {
                return Err(CodecError::NonCanonicalBase64 { offset: offset + 1 });
            }
            output.push((a << 2) | (b >> 4));
            continue;
        }

        let c = value(quartet[2], offset + 2)?;
        if quartet[3] == b'=' {
            if !final_group {
                return Err(CodecError::InvalidBase64Padding { offset: offset + 3 });
            }
            if c & 0x03 != 0 {
                return Err(CodecError::NonCanonicalBase64 { offset: offset + 2 });
            }
            output.push((a << 2) | (b >> 4));
            output.push((b << 4) | (c >> 2));
            continue;
        }

        let d = value(quartet[3], offset + 3)?;
        output.push((a << 2) | (b >> 4));
        output.push((b << 4) | (c >> 2));
        output.push((c << 6) | d);
    }
    debug_assert_eq!(output.len(), decoded);
    Ok(output)
}

fn value(byte: u8, offset: usize) -> Result<u8, CodecError> {
    match byte {
        b'A'..=b'Z' => Ok(byte - b'A'),
        b'a'..=b'z' => Ok(byte - b'a' + 26),
        b'0'..=b'9' => Ok(byte - b'0' + 52),
        b'+' => Ok(62),
        b'/' => Ok(63),
        _ => Err(CodecError::InvalidBase64 { offset }),
    }
}
