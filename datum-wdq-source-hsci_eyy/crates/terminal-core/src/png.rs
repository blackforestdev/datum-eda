use crate::checksum::Crc32;
use crate::codec::{ChecksumKind, CodecError, CodecLimits, CodecStage, WorkBudget};
use crate::png_pixels::decode_png_pixels;
use crate::{GraphicDecodedBytesLimit, LimitError, LimitKind};

const PNG_SIGNATURE: &[u8; 8] = b"\x89PNG\r\n\x1a\n";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Rgba8 {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PngImage {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<Rgba8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PngHeader {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) bit_depth: u8,
    pub(crate) color_type: u8,
    pub(crate) interlace: u8,
}

/// Decode the static reference image in a conforming PNG datastream to RGBA8.
pub fn decode_png(input: &[u8], limits: CodecLimits) -> Result<PngImage, CodecError> {
    let mut work = WorkBudget::new(limits.work);
    if input.get(..PNG_SIGNATURE.len()) != Some(PNG_SIGNATURE) {
        return Err(CodecError::InvalidPng {
            reason: "signature mismatch",
        });
    }
    let mut offset = PNG_SIGNATURE.len();
    let mut header = None;
    let mut palette: Option<Vec<[u8; 3]>> = None;
    let mut transparency: Option<Vec<u8>> = None;
    let mut idat = Vec::new();
    let mut seen_idat = false;
    let mut idat_ended = false;
    let mut seen_iend = false;

    while offset < input.len() {
        work.charge(12)?;
        let length_bytes = take(input, &mut offset, 4)?;
        let length = u32::from_be_bytes(length_bytes.try_into().expect("four-byte length"));
        if length > 0x7fff_ffff {
            return Err(CodecError::InvalidPng {
                reason: "chunk length exceeds PNG's signed interoperability bound",
            });
        }
        let length = usize::try_from(length).map_err(|_| LimitError::ArithmeticOverflow {
            kind: LimitKind::GraphicDecodedBytes,
        })?;
        let kind: [u8; 4] = take(input, &mut offset, 4)?
            .try_into()
            .expect("four-byte chunk type");
        if !kind.iter().all(u8::is_ascii_alphabetic) {
            return Err(CodecError::InvalidPng {
                reason: "chunk type contains a non-letter byte",
            });
        }
        if kind[2] & 0x20 != 0 {
            return Err(CodecError::InvalidPng {
                reason: "chunk type sets the reserved bit",
            });
        }
        let data = take(input, &mut offset, length)?;
        let expected = u32::from_be_bytes(
            take(input, &mut offset, 4)?
                .try_into()
                .expect("four-byte CRC"),
        );
        work.charge(length)?;
        let mut checksum = Crc32::new();
        checksum.update(&kind);
        checksum.update(data);
        let actual = checksum.finish();
        if actual != expected {
            return Err(CodecError::ChecksumMismatch {
                kind: ChecksumKind::Crc32,
                expected,
                actual,
            });
        }

        if header.is_none() && &kind != b"IHDR" {
            return Err(CodecError::InvalidPng {
                reason: "IHDR is not the first chunk",
            });
        }
        if seen_idat && &kind != b"IDAT" && &kind != b"IEND" {
            idat_ended = true;
        }
        match &kind {
            b"IHDR" => {
                if header.is_some() || seen_idat {
                    return Err(CodecError::InvalidPng {
                        reason: "duplicate or misplaced IHDR",
                    });
                }
                header = Some(parse_header(data, limits)?);
            }
            b"PLTE" => {
                let value = header.expect("IHDR checked above");
                if palette.is_some()
                    || transparency.is_some()
                    || seen_idat
                    || matches!(value.color_type, 0 | 4)
                {
                    return Err(CodecError::InvalidPng {
                        reason: "duplicate, forbidden, or misplaced PLTE",
                    });
                }
                if data.is_empty() || data.len() % 3 != 0 || data.len() > 768 {
                    return Err(CodecError::InvalidPng {
                        reason: "PLTE length is invalid",
                    });
                }
                let entries: Vec<[u8; 3]> = data
                    .chunks_exact(3)
                    .map(|entry| [entry[0], entry[1], entry[2]])
                    .collect();
                if value.color_type == 3 && entries.len() > (1usize << value.bit_depth) {
                    return Err(CodecError::InvalidPng {
                        reason: "palette has more entries than the indexed bit depth permits",
                    });
                }
                palette = Some(entries);
            }
            b"tRNS" => {
                let value = header.expect("IHDR checked above");
                if transparency.is_some() || seen_idat || matches!(value.color_type, 4 | 6) {
                    return Err(CodecError::InvalidPng {
                        reason: "duplicate, forbidden, or misplaced tRNS",
                    });
                }
                match value.color_type {
                    0 if data.len() == 2 => {}
                    2 if data.len() == 6 => {}
                    3 if palette
                        .as_ref()
                        .is_some_and(|entries| data.len() <= entries.len()) => {}
                    _ => {
                        return Err(CodecError::InvalidPng {
                            reason: "tRNS payload does not match the PNG color type",
                        });
                    }
                }
                transparency = Some(data.to_vec());
            }
            b"IDAT" => {
                if idat_ended {
                    return Err(CodecError::InvalidPng {
                        reason: "IDAT chunks are not consecutive",
                    });
                }
                let total =
                    idat.len()
                        .checked_add(data.len())
                        .ok_or(LimitError::ArithmeticOverflow {
                            kind: LimitKind::GraphicDecodedBytes,
                        })?;
                limits.decoded_bytes.check(total)?;
                idat.extend_from_slice(data);
                seen_idat = true;
            }
            b"IEND" => {
                if !seen_idat || !data.is_empty() {
                    return Err(CodecError::InvalidPng {
                        reason: "IEND is nonempty or precedes IDAT",
                    });
                }
                seen_iend = true;
                break;
            }
            _ if kind[0] & 0x20 == 0 => {
                return Err(CodecError::UnsupportedPng {
                    feature: "unknown critical chunk",
                });
            }
            _ => {}
        }
    }

    if !seen_iend || offset != input.len() {
        return Err(CodecError::InvalidPng {
            reason: "missing IEND or trailing bytes after it",
        });
    }
    let header = header.expect("IEND requires the earlier IHDR");
    if header.color_type == 3 && palette.is_none() {
        return Err(CodecError::InvalidPng {
            reason: "indexed-color image has no PLTE",
        });
    }
    if idat.len() >= limits.decoded_bytes.get() {
        return Err(LimitError::Exceeded {
            kind: LimitKind::GraphicDecodedBytes,
            requested: idat.len().saturating_add(1),
            maximum: limits.decoded_bytes.get(),
        }
        .into());
    }
    let inflate_limit = GraphicDecodedBytesLimit::new(limits.decoded_bytes.get() - idat.len())?;
    let inflate_limits = CodecLimits {
        decoded_bytes: inflate_limit,
        ..limits
    };
    let inflated = crate::zlib::decode_zlib_with_work(&idat, inflate_limits, &mut work)?;
    let pixels = decode_png_pixels(
        header,
        palette.as_deref(),
        transparency.as_deref(),
        &inflated,
        idat.len(),
        limits,
        &mut work,
    )?;
    Ok(PngImage {
        width: header.width,
        height: header.height,
        pixels,
    })
}

fn parse_header(data: &[u8], limits: CodecLimits) -> Result<PngHeader, CodecError> {
    if data.len() != 13 {
        return Err(CodecError::InvalidPng {
            reason: "IHDR length is not 13",
        });
    }
    let width = u32::from_be_bytes(data[0..4].try_into().expect("width"));
    let height = u32::from_be_bytes(data[4..8].try_into().expect("height"));
    if width == 0 || height == 0 || width > 0x7fff_ffff || height > 0x7fff_ffff {
        return Err(CodecError::InvalidPng {
            reason: "image dimensions are zero or exceed PNG bounds",
        });
    }
    let pixels = usize::try_from(u64::from(width) * u64::from(height)).map_err(|_| {
        LimitError::ArithmeticOverflow {
            kind: LimitKind::GraphicPixels,
        }
    })?;
    limits.pixels.check(pixels)?;
    let bit_depth = data[8];
    let color_type = data[9];
    let valid_depth = match color_type {
        0 => matches!(bit_depth, 1 | 2 | 4 | 8 | 16),
        2 => matches!(bit_depth, 8 | 16),
        3 => matches!(bit_depth, 1 | 2 | 4 | 8),
        4 | 6 => matches!(bit_depth, 8 | 16),
        _ => false,
    };
    if !valid_depth {
        return Err(CodecError::InvalidPng {
            reason: "color type and bit depth combination is invalid",
        });
    }
    if data[10] != 0 || data[11] != 0 || !matches!(data[12], 0 | 1) {
        return Err(CodecError::UnsupportedPng {
            feature: "compression, filter, or interlace method",
        });
    }
    Ok(PngHeader {
        width,
        height,
        bit_depth,
        color_type,
        interlace: data[12],
    })
}

fn take<'a>(input: &'a [u8], offset: &mut usize, length: usize) -> Result<&'a [u8], CodecError> {
    let end = offset
        .checked_add(length)
        .ok_or(LimitError::ArithmeticOverflow {
            kind: LimitKind::GraphicDecodedBytes,
        })?;
    let bytes = input.get(*offset..end).ok_or(CodecError::UnexpectedEnd {
        stage: CodecStage::Png,
    })?;
    *offset = end;
    Ok(bytes)
}
