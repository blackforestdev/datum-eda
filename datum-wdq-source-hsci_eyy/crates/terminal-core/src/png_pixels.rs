use crate::codec::{CodecError, CodecLimits, WorkBudget, checked_product};
use crate::png::{PngHeader, Rgba8};
use crate::{LimitError, LimitKind};

#[derive(Clone, Copy)]
struct Pass {
    x: usize,
    y: usize,
    dx: usize,
    dy: usize,
}

const FULL: Pass = Pass {
    x: 0,
    y: 0,
    dx: 1,
    dy: 1,
};
const ADAM7: [Pass; 7] = [
    Pass {
        x: 0,
        y: 0,
        dx: 8,
        dy: 8,
    },
    Pass {
        x: 4,
        y: 0,
        dx: 8,
        dy: 8,
    },
    Pass {
        x: 0,
        y: 4,
        dx: 4,
        dy: 8,
    },
    Pass {
        x: 2,
        y: 0,
        dx: 4,
        dy: 4,
    },
    Pass {
        x: 0,
        y: 2,
        dx: 2,
        dy: 4,
    },
    Pass {
        x: 1,
        y: 0,
        dx: 2,
        dy: 2,
    },
    Pass {
        x: 0,
        y: 1,
        dx: 1,
        dy: 2,
    },
];

pub(crate) fn decode_png_pixels(
    header: PngHeader,
    palette: Option<&[[u8; 3]]>,
    transparency: Option<&[u8]>,
    inflated: &[u8],
    compressed_bytes: usize,
    limits: CodecLimits,
    work: &mut WorkBudget,
) -> Result<Vec<Rgba8>, CodecError> {
    validate_transparency(header, transparency)?;
    let width = header.width as usize;
    let height = header.height as usize;
    let channels = channels(header.color_type)?;
    let bits_per_pixel = channels * usize::from(header.bit_depth);
    let passes: &[Pass] = if header.interlace == 0 {
        &[FULL]
    } else {
        &ADAM7
    };

    let mut expected = 0usize;
    let mut maximum_row = 0usize;
    for &pass in passes {
        let pass_width = pass_extent(width, pass.x, pass.dx);
        let pass_height = pass_extent(height, pass.y, pass.dy);
        if pass_width == 0 || pass_height == 0 {
            continue;
        }
        let row_bytes = scanline_bytes(pass_width, bits_per_pixel)?;
        maximum_row = maximum_row.max(row_bytes);
        let pass_bytes = checked_product(
            pass_height,
            row_bytes
                .checked_add(1)
                .ok_or(LimitError::ArithmeticOverflow {
                    kind: LimitKind::GraphicDecodedBytes,
                })?,
            LimitKind::GraphicDecodedBytes,
        )?;
        expected = expected
            .checked_add(pass_bytes)
            .ok_or(LimitError::ArithmeticOverflow {
                kind: LimitKind::GraphicDecodedBytes,
            })?;
    }
    if inflated.len() != expected {
        return Err(CodecError::InvalidPng {
            reason: "inflated scanline length does not match IHDR",
        });
    }

    let pixel_count = checked_product(width, height, LimitKind::GraphicPixels)?;
    limits.pixels.check(pixel_count)?;
    let output_bytes = checked_product(pixel_count, 4, LimitKind::GraphicDecodedBytes)?;
    let row_scratch = checked_product(maximum_row, 2, LimitKind::GraphicDecodedBytes)?;
    let resident = compressed_bytes
        .checked_add(inflated.len())
        .and_then(|value| value.checked_add(output_bytes))
        .and_then(|value| value.checked_add(row_scratch))
        .ok_or(LimitError::ArithmeticOverflow {
            kind: LimitKind::GraphicDecodedBytes,
        })?;
    limits.decoded_bytes.check(resident)?;
    work.charge(pixel_count)?;

    let mut pixels = vec![
        Rgba8 {
            red: 0,
            green: 0,
            blue: 0,
            alpha: 0
        };
        pixel_count
    ];
    let filter_bytes_per_pixel = bits_per_pixel.div_ceil(8).max(1);
    let mut source = 0usize;
    for &pass in passes {
        let pass_width = pass_extent(width, pass.x, pass.dx);
        let pass_height = pass_extent(height, pass.y, pass.dy);
        if pass_width == 0 || pass_height == 0 {
            continue;
        }
        let row_bytes = scanline_bytes(pass_width, bits_per_pixel)?;
        let mut previous = vec![0u8; row_bytes];
        for pass_row in 0..pass_height {
            let filter = inflated[source];
            source += 1;
            let mut row = inflated[source..source + row_bytes].to_vec();
            source += row_bytes;
            unfilter(filter, &mut row, &previous, filter_bytes_per_pixel, work)?;
            for pass_column in 0..pass_width {
                let pixel = decode_pixel(&row, pass_column, header, palette, transparency)?;
                let x = pass.x + pass_column * pass.dx;
                let y = pass.y + pass_row * pass.dy;
                pixels[y * width + x] = pixel;
            }
            previous = row;
        }
    }
    debug_assert_eq!(source, inflated.len());
    Ok(pixels)
}

fn channels(color_type: u8) -> Result<usize, CodecError> {
    match color_type {
        0 | 3 => Ok(1),
        2 => Ok(3),
        4 => Ok(2),
        6 => Ok(4),
        _ => Err(CodecError::InvalidPng {
            reason: "invalid color type",
        }),
    }
}

fn pass_extent(full: usize, start: usize, step: usize) -> usize {
    if full <= start {
        0
    } else {
        (full - start).div_ceil(step)
    }
}

fn scanline_bytes(width: usize, bits_per_pixel: usize) -> Result<usize, CodecError> {
    checked_product(width, bits_per_pixel, LimitKind::GraphicDecodedBytes)
        .map(|bits| bits.div_ceil(8))
}

fn unfilter(
    filter: u8,
    row: &mut [u8],
    previous: &[u8],
    bytes_per_pixel: usize,
    work: &mut WorkBudget,
) -> Result<(), CodecError> {
    work.charge(row.len())?;
    for index in 0..row.len() {
        let left = index
            .checked_sub(bytes_per_pixel)
            .map_or(0, |left| row[left]);
        let above = previous[index];
        let upper_left = index
            .checked_sub(bytes_per_pixel)
            .map_or(0, |left| previous[left]);
        let predictor = match filter {
            0 => 0,
            1 => left,
            2 => above,
            3 => ((u16::from(left) + u16::from(above)) / 2) as u8,
            4 => paeth(left, above, upper_left),
            _ => {
                return Err(CodecError::InvalidPng {
                    reason: "unknown scanline filter",
                });
            }
        };
        row[index] = row[index].wrapping_add(predictor);
    }
    Ok(())
}

fn paeth(left: u8, above: u8, upper_left: u8) -> u8 {
    let left = i32::from(left);
    let above = i32::from(above);
    let upper_left = i32::from(upper_left);
    let estimate = left + above - upper_left;
    let left_distance = (estimate - left).abs();
    let above_distance = (estimate - above).abs();
    let diagonal_distance = (estimate - upper_left).abs();
    if left_distance <= above_distance && left_distance <= diagonal_distance {
        left as u8
    } else if above_distance <= diagonal_distance {
        above as u8
    } else {
        upper_left as u8
    }
}

fn decode_pixel(
    row: &[u8],
    column: usize,
    header: PngHeader,
    palette: Option<&[[u8; 3]]>,
    transparency: Option<&[u8]>,
) -> Result<Rgba8, CodecError> {
    let depth = header.bit_depth;
    match header.color_type {
        0 => {
            let grey = sample(row, column, depth)?;
            let alpha = if transparent_grey(transparency) == Some(grey) {
                0
            } else {
                255
            };
            let grey = scale_sample(grey, depth);
            Ok(Rgba8 {
                red: grey,
                green: grey,
                blue: grey,
                alpha,
            })
        }
        2 => {
            let base = column * 3;
            let raw = [
                sample(row, base, depth)?,
                sample(row, base + 1, depth)?,
                sample(row, base + 2, depth)?,
            ];
            let alpha = if transparent_rgb(transparency) == Some(raw) {
                0
            } else {
                255
            };
            Ok(Rgba8 {
                red: scale_sample(raw[0], depth),
                green: scale_sample(raw[1], depth),
                blue: scale_sample(raw[2], depth),
                alpha,
            })
        }
        3 => {
            let index = usize::from(sample(row, column, depth)?);
            let color =
                palette
                    .and_then(|entries| entries.get(index))
                    .ok_or(CodecError::InvalidPng {
                        reason: "palette index is out of range",
                    })?;
            let alpha = transparency
                .and_then(|values| values.get(index))
                .copied()
                .unwrap_or(255);
            Ok(Rgba8 {
                red: color[0],
                green: color[1],
                blue: color[2],
                alpha,
            })
        }
        4 => {
            let base = column * 2;
            let grey = scale_sample(sample(row, base, depth)?, depth);
            let alpha = scale_sample(sample(row, base + 1, depth)?, depth);
            Ok(Rgba8 {
                red: grey,
                green: grey,
                blue: grey,
                alpha,
            })
        }
        6 => {
            let base = column * 4;
            Ok(Rgba8 {
                red: scale_sample(sample(row, base, depth)?, depth),
                green: scale_sample(sample(row, base + 1, depth)?, depth),
                blue: scale_sample(sample(row, base + 2, depth)?, depth),
                alpha: scale_sample(sample(row, base + 3, depth)?, depth),
            })
        }
        _ => Err(CodecError::InvalidPng {
            reason: "invalid color type",
        }),
    }
}

fn sample(row: &[u8], index: usize, depth: u8) -> Result<u16, CodecError> {
    match depth {
        1 | 2 | 4 => {
            let depth = usize::from(depth);
            let bit = index * depth;
            let byte = *row.get(bit / 8).ok_or(CodecError::InvalidPng {
                reason: "packed sample exceeds scanline",
            })?;
            let shift = 8 - depth - (bit % 8);
            Ok(u16::from((byte >> shift) & ((1u8 << depth) - 1)))
        }
        8 => row
            .get(index)
            .copied()
            .map(u16::from)
            .ok_or(CodecError::InvalidPng {
                reason: "sample exceeds scanline",
            }),
        16 => {
            let byte = index.checked_mul(2).ok_or(LimitError::ArithmeticOverflow {
                kind: LimitKind::GraphicDecodedBytes,
            })?;
            let sample = row.get(byte..byte + 2).ok_or(CodecError::InvalidPng {
                reason: "16-bit sample exceeds scanline",
            })?;
            Ok(u16::from_be_bytes([sample[0], sample[1]]))
        }
        _ => Err(CodecError::InvalidPng {
            reason: "invalid sample depth",
        }),
    }
}

fn scale_sample(value: u16, depth: u8) -> u8 {
    let maximum = if depth == 16 {
        65_535
    } else {
        (1u32 << depth) - 1
    };
    ((u32::from(value) * 255 + maximum / 2) / maximum) as u8
}

fn transparent_grey(bytes: Option<&[u8]>) -> Option<u16> {
    bytes.map(|value| u16::from_be_bytes([value[0], value[1]]))
}

fn transparent_rgb(bytes: Option<&[u8]>) -> Option<[u16; 3]> {
    bytes.map(|value| {
        [
            u16::from_be_bytes([value[0], value[1]]),
            u16::from_be_bytes([value[2], value[3]]),
            u16::from_be_bytes([value[4], value[5]]),
        ]
    })
}

fn validate_transparency(header: PngHeader, bytes: Option<&[u8]>) -> Result<(), CodecError> {
    let Some(bytes) = bytes else {
        return Ok(());
    };
    let maximum = if header.bit_depth == 16 {
        65_535
    } else {
        (1u32 << header.bit_depth) - 1
    };
    let valid = match header.color_type {
        0 => u32::from(transparent_grey(Some(bytes)).expect("validated length")) <= maximum,
        2 => transparent_rgb(Some(bytes))
            .expect("validated length")
            .into_iter()
            .all(|value| u32::from(value) <= maximum),
        3 => true,
        _ => false,
    };
    if !valid {
        return Err(CodecError::InvalidPng {
            reason: "tRNS sample exceeds the image bit depth",
        });
    }
    Ok(())
}
