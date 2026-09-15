use crate::checksum::adler32;
use crate::codec::{ChecksumKind, CodecError, CodecLimits, CodecStage, WorkBudget};
use crate::deflate::decode_deflate_with_work;

/// Decode one RFC 1950 zlib stream containing RFC 1951 DEFLATE data.
pub fn decode_zlib(input: &[u8], limits: CodecLimits) -> Result<Vec<u8>, CodecError> {
    let mut work = WorkBudget::new(limits.work);
    decode_zlib_with_work(input, limits, &mut work)
}

pub(crate) fn decode_zlib_with_work(
    input: &[u8],
    limits: CodecLimits,
    work: &mut WorkBudget,
) -> Result<Vec<u8>, CodecError> {
    work.charge(2)?;
    let (&cmf, &flg) = input
        .first()
        .zip(input.get(1))
        .ok_or(CodecError::UnexpectedEnd {
            stage: CodecStage::Zlib,
        })?;
    if cmf & 0x0f != 8 {
        return Err(CodecError::InvalidZlib {
            reason: "compression method is not DEFLATE",
        });
    }
    if cmf >> 4 > 7 {
        return Err(CodecError::InvalidZlib {
            reason: "window size exceeds 32 KiB",
        });
    }
    if (u16::from(cmf) << 8 | u16::from(flg)) % 31 != 0 {
        return Err(CodecError::InvalidZlib {
            reason: "header check bits are invalid",
        });
    }
    if flg & 0x20 != 0 {
        return Err(CodecError::UnsupportedZlib {
            feature: "preset zlib dictionary",
        });
    }
    if input.len() < 6 {
        return Err(CodecError::UnexpectedEnd {
            stage: CodecStage::Zlib,
        });
    }

    let payload = &input[2..input.len() - 4];
    let decoded = decode_deflate_with_work(payload, limits, work)?;
    if decoded.consumed_bytes != payload.len() {
        return Err(CodecError::InvalidZlib {
            reason: "trailing bytes follow the DEFLATE stream",
        });
    }
    let expected = u32::from_be_bytes(input[input.len() - 4..].try_into().expect("four-byte tail"));
    work.charge(decoded.bytes.len())?;
    let actual = adler32(&decoded.bytes);
    if actual != expected {
        return Err(CodecError::ChecksumMismatch {
            kind: ChecksumKind::Adler32,
            expected,
            actual,
        });
    }
    Ok(decoded.bytes)
}
