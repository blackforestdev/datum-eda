use std::error::Error;
use std::fmt;

use crate::{
    ClipboardBytesLimit, CompressionRatioLimit, CoreLimits, GraphicDecodedBytesLimit,
    GraphicPixelsLimit, LimitError, LimitKind, ParserWorkLimit,
};

/// A Base64 output budget tied to the resource family that owns the payload.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Base64Limits {
    pub work: ParserWorkLimit,
    pub(crate) decoded_bytes: usize,
    pub(crate) kind: LimitKind,
}

impl Base64Limits {
    pub const fn graphics(decoded_bytes: GraphicDecodedBytesLimit, work: ParserWorkLimit) -> Self {
        Self {
            work,
            decoded_bytes: decoded_bytes.get(),
            kind: LimitKind::GraphicDecodedBytes,
        }
    }

    pub const fn clipboard(decoded_bytes: ClipboardBytesLimit, work: ParserWorkLimit) -> Self {
        Self {
            work,
            decoded_bytes: decoded_bytes.get(),
            kind: LimitKind::ClipboardBytes,
        }
    }
}

/// Owner-supplied limits shared by the Datum-owned binary codecs.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CodecLimits {
    pub decoded_bytes: GraphicDecodedBytesLimit,
    pub pixels: GraphicPixelsLimit,
    pub compression_ratio: CompressionRatioLimit,
    pub work: ParserWorkLimit,
}

impl From<CoreLimits> for CodecLimits {
    fn from(limits: CoreLimits) -> Self {
        Self {
            decoded_bytes: limits.graphic_decoded_bytes,
            pixels: limits.graphic_pixels,
            compression_ratio: limits.compression_ratio,
            work: limits.parser_work,
        }
    }
}

/// Closed failure surface for Base64, DEFLATE, zlib, and PNG decoding.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CodecError {
    Limit(LimitError),
    UnexpectedEnd {
        stage: CodecStage,
    },
    InvalidBase64 {
        offset: usize,
    },
    InvalidBase64Padding {
        offset: usize,
    },
    NonCanonicalBase64 {
        offset: usize,
    },
    InvalidDeflate {
        reason: &'static str,
    },
    InvalidZlib {
        reason: &'static str,
    },
    UnsupportedZlib {
        feature: &'static str,
    },
    InvalidPng {
        reason: &'static str,
    },
    UnsupportedPng {
        feature: &'static str,
    },
    ChecksumMismatch {
        kind: ChecksumKind,
        expected: u32,
        actual: u32,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CodecStage {
    Base64,
    Deflate,
    Zlib,
    Png,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChecksumKind {
    Adler32,
    Crc32,
}

impl fmt::Display for CodecError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Limit(error) => error.fmt(formatter),
            Self::UnexpectedEnd { stage } => write!(formatter, "unexpected end of {stage:?} data"),
            Self::InvalidBase64 { offset } => {
                write!(formatter, "invalid Base64 byte at offset {offset}")
            }
            Self::InvalidBase64Padding { offset } => {
                write!(formatter, "invalid Base64 padding at offset {offset}")
            }
            Self::NonCanonicalBase64 { offset } => {
                write!(formatter, "non-canonical Base64 tail at offset {offset}")
            }
            Self::InvalidDeflate { reason } => {
                write!(formatter, "invalid DEFLATE stream: {reason}")
            }
            Self::InvalidZlib { reason } => write!(formatter, "invalid zlib stream: {reason}"),
            Self::UnsupportedZlib { feature } => {
                write!(formatter, "unsupported zlib feature: {feature}")
            }
            Self::InvalidPng { reason } => write!(formatter, "invalid PNG datastream: {reason}"),
            Self::UnsupportedPng { feature } => {
                write!(formatter, "unsupported PNG feature: {feature}")
            }
            Self::ChecksumMismatch {
                kind,
                expected,
                actual,
            } => write!(
                formatter,
                "{kind:?} mismatch: expected {expected:08x}, computed {actual:08x}"
            ),
        }
    }
}

impl Error for CodecError {}

impl From<LimitError> for CodecError {
    fn from(error: LimitError) -> Self {
        Self::Limit(error)
    }
}

#[derive(Debug)]
pub(crate) struct WorkBudget {
    used: usize,
    maximum: usize,
}

impl WorkBudget {
    pub(crate) fn new(limit: ParserWorkLimit) -> Self {
        Self {
            used: 0,
            maximum: limit.get(),
        }
    }

    pub(crate) fn charge(&mut self, amount: usize) -> Result<(), CodecError> {
        self.used = self
            .used
            .checked_add(amount)
            .ok_or(LimitError::ArithmeticOverflow {
                kind: LimitKind::ParserWork,
            })?;
        if self.used > self.maximum {
            return Err(LimitError::Exceeded {
                kind: LimitKind::ParserWork,
                requested: self.used,
                maximum: self.maximum,
            }
            .into());
        }
        Ok(())
    }
}

pub(crate) fn checked_product(
    left: usize,
    right: usize,
    kind: LimitKind,
) -> Result<usize, CodecError> {
    left.checked_mul(right)
        .ok_or(LimitError::ArithmeticOverflow { kind }.into())
}
