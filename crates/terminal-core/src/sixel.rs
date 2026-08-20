use crate::{
    GraphicDecodedBytesLimit, GraphicPixelsLimit, LimitError, LimitKind, ParserWorkLimit,
    PixelAspect, Rgba8,
};
use std::error::Error;
use std::fmt;

const REGISTER_COUNT: usize = 256;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SixelColorRegisters {
    colors: [Rgba8; REGISTER_COUNT],
}

impl Default for SixelColorRegisters {
    fn default() -> Self {
        let mut colors = [rgba(0, 0, 0); REGISTER_COUNT];
        let values = [
            (0, 0, 0),
            (0, 0, 100),
            (100, 0, 0),
            (0, 100, 0),
            (100, 0, 100),
            (0, 100, 100),
            (100, 100, 0),
            (50, 50, 50),
            (25, 25, 25),
            (0, 0, 50),
            (50, 0, 0),
            (0, 50, 0),
            (50, 0, 50),
            (0, 50, 50),
            (50, 50, 0),
            (75, 75, 75),
        ];
        for (target, (red, green, blue)) in colors.iter_mut().zip(values) {
            *target = rgb_percent(red, green, blue);
        }
        Self { colors }
    }
}

impl SixelColorRegisters {
    pub const fn color(&self, register: u8) -> Rgba8 {
        self.colors[register as usize]
    }

    fn set(&mut self, register: u8, color: Rgba8) {
        self.colors[register as usize] = color;
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SixelLimits {
    pub pixels: GraphicPixelsLimit,
    pub decoded_bytes: GraphicDecodedBytesLimit,
    pub work: ParserWorkLimit,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SixelImage {
    pub width: u32,
    pub height: u32,
    pub pixel_aspect: PixelAspect,
    pub pixels: Vec<Rgba8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SixelError {
    Malformed { offset: usize, reason: &'static str },
    Limit(LimitError),
    Allocation,
}

impl fmt::Display for SixelError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Malformed { offset, reason } => {
                write!(formatter, "malformed sixel at byte {offset}: {reason}")
            }
            Self::Limit(error) => error.fmt(formatter),
            Self::Allocation => formatter.write_str("sixel pixel allocation failed"),
        }
    }
}

impl Error for SixelError {}

impl From<LimitError> for SixelError {
    fn from(value: LimitError) -> Self {
        Self::Limit(value)
    }
}

#[derive(Clone, Copy, Debug)]
struct PaintRun {
    x: usize,
    y: usize,
    count: usize,
    bits: u8,
    color: Rgba8,
}

/// Decode the bytes following the DCS sixel final `q` into one bounded RGBA image.
pub fn decode_sixel(
    data: &[u8],
    background: Option<Rgba8>,
    registers: &mut SixelColorRegisters,
    initial_aspect: PixelAspect,
    limits: SixelLimits,
) -> Result<SixelImage, SixelError> {
    let mut working_registers = registers.clone();
    let mut cursor = 0usize;
    let mut x = 0usize;
    let mut y = 0usize;
    let mut maximum_x = 0usize;
    let mut maximum_y = 0usize;
    let mut declared_width = 0usize;
    let mut declared_height = 0usize;
    let mut aspect = initial_aspect;
    let mut selected = 0u8;
    let mut runs = Vec::new();
    let mut work = 0usize;
    charge(&mut work, data.len(), limits.work)?;

    while cursor < data.len() {
        charge(&mut work, 1, limits.work)?;
        match data[cursor] {
            b'?'..=b'~' => {
                let bits = data[cursor] - b'?';
                push_run(
                    &mut runs,
                    PaintRun {
                        x,
                        y,
                        count: 1,
                        bits,
                        color: working_registers.color(selected),
                    },
                )?;
                x = checked_add(x, 1, LimitKind::GraphicPixels)?;
                maximum_x = maximum_x.max(x);
                maximum_y = maximum_y.max(checked_add(y, 6, LimitKind::GraphicPixels)?);
                charge(&mut work, bits.count_ones() as usize, limits.work)?;
                cursor += 1;
            }
            b'!' => {
                let start = cursor;
                cursor += 1;
                let count = parse_number(data, &mut cursor)?.ok_or(SixelError::Malformed {
                    offset: start,
                    reason: "repeat introducer has no count",
                })?;
                if count == 0
                    || !data
                        .get(cursor)
                        .is_some_and(|byte| (b'?'..=b'~').contains(byte))
                {
                    return Err(SixelError::Malformed {
                        offset: cursor,
                        reason: "repeat count must be positive and followed by a sixel",
                    });
                }
                let bits = data[cursor] - b'?';
                push_run(
                    &mut runs,
                    PaintRun {
                        x,
                        y,
                        count,
                        bits,
                        color: working_registers.color(selected),
                    },
                )?;
                x = checked_add(x, count, LimitKind::GraphicPixels)?;
                maximum_x = maximum_x.max(x);
                maximum_y = maximum_y.max(checked_add(y, 6, LimitKind::GraphicPixels)?);
                charge(
                    &mut work,
                    count.checked_mul(bits.count_ones() as usize).ok_or(
                        LimitError::ArithmeticOverflow {
                            kind: LimitKind::ParserWork,
                        },
                    )?,
                    limits.work,
                )?;
                cursor += 1;
            }
            b'#' => {
                cursor += 1;
                let register = parse_number(data, &mut cursor)?.ok_or(SixelError::Malformed {
                    offset: cursor,
                    reason: "color introducer has no register",
                })?;
                selected = u8::try_from(register).map_err(|_| SixelError::Malformed {
                    offset: cursor,
                    reason: "color register exceeds 255",
                })?;
                if data.get(cursor) == Some(&b';') {
                    cursor += 1;
                    let model = required_number(data, &mut cursor, "color model")?;
                    require_separator(data, &mut cursor)?;
                    let first = required_number(data, &mut cursor, "first color component")?;
                    require_separator(data, &mut cursor)?;
                    let second = required_number(data, &mut cursor, "second color component")?;
                    require_separator(data, &mut cursor)?;
                    let third = required_number(data, &mut cursor, "third color component")?;
                    let color = match model {
                        1 if first <= 360 && second <= 100 && third <= 100 => {
                            hls(first as u16, second as u8, third as u8)
                        }
                        2 if first <= 100 && second <= 100 && third <= 100 => {
                            rgb_percent(first as u8, second as u8, third as u8)
                        }
                        _ => {
                            return Err(SixelError::Malformed {
                                offset: cursor,
                                reason: "color model or component is outside the DEC range",
                            });
                        }
                    };
                    working_registers.set(selected, color);
                }
            }
            b'"' => {
                cursor += 1;
                let pan = optional_number(data, &mut cursor)?.unwrap_or(0);
                require_separator(data, &mut cursor)?;
                let pad = optional_number(data, &mut cursor)?.unwrap_or(0);
                if pan != 0 && pad != 0 {
                    let pan = u32::try_from(pan).map_err(|_| SixelError::Malformed {
                        offset: cursor,
                        reason: "raster numerator exceeds 32 bits",
                    })?;
                    let pad = u32::try_from(pad).map_err(|_| SixelError::Malformed {
                        offset: cursor,
                        reason: "raster denominator exceeds 32 bits",
                    })?;
                    aspect = PixelAspect::new(pan, pad).ok_or(SixelError::Malformed {
                        offset: cursor,
                        reason: "raster aspect ratio is zero",
                    })?;
                }
                if data.get(cursor) == Some(&b';') {
                    cursor += 1;
                    declared_width = optional_number(data, &mut cursor)?.unwrap_or(0);
                    require_separator(data, &mut cursor)?;
                    declared_height = optional_number(data, &mut cursor)?.unwrap_or(0);
                }
            }
            b'$' => {
                x = 0;
                cursor += 1;
            }
            b'-' => {
                x = 0;
                y = checked_add(y, 6, LimitKind::GraphicPixels)?;
                maximum_y = maximum_y.max(y);
                cursor += 1;
            }
            b'\x00'..=b' ' | 0x7f => cursor += 1,
            _ => {
                return Err(SixelError::Malformed {
                    offset: cursor,
                    reason: "unsupported sixel command byte",
                });
            }
        }
    }

    // DECGRA's explicit raster is the drawable clip.  Without one, the
    // commands' maximum addressed extent defines the image.
    let width = if declared_width == 0 {
        maximum_x
    } else {
        declared_width
    };
    let height = if declared_height == 0 {
        maximum_y
    } else {
        declared_height
    };
    if width == 0 || height == 0 {
        *registers = working_registers;
        return Ok(SixelImage {
            width: 0,
            height: 0,
            pixel_aspect: aspect,
            pixels: Vec::new(),
        });
    }
    let pixel_count = width
        .checked_mul(height)
        .ok_or(LimitError::ArithmeticOverflow {
            kind: LimitKind::GraphicPixels,
        })?;
    limits.pixels.check(pixel_count)?;
    let decoded_bytes = pixel_count
        .checked_mul(4)
        .ok_or(LimitError::ArithmeticOverflow {
            kind: LimitKind::GraphicDecodedBytes,
        })?;
    limits.decoded_bytes.check(decoded_bytes)?;
    charge(&mut work, pixel_count, limits.work)?;
    let fill = background.unwrap_or(Rgba8 {
        red: 0,
        green: 0,
        blue: 0,
        alpha: 0,
    });
    let mut pixels = Vec::new();
    pixels
        .try_reserve_exact(pixel_count)
        .map_err(|_| SixelError::Allocation)?;
    pixels.resize(pixel_count, fill);
    for run in runs {
        for column in run.x..run.x + run.count {
            for bit in 0..6usize {
                if run.bits & (1 << bit) == 0 {
                    continue;
                }
                let row = run.y + bit;
                if column < width && row < height {
                    pixels[row * width + column] = run.color;
                }
            }
        }
    }
    *registers = working_registers;
    Ok(SixelImage {
        width: u32::try_from(width).map_err(|_| LimitError::ArithmeticOverflow {
            kind: LimitKind::GraphicPixels,
        })?,
        height: u32::try_from(height).map_err(|_| LimitError::ArithmeticOverflow {
            kind: LimitKind::GraphicPixels,
        })?,
        pixel_aspect: aspect,
        pixels,
    })
}

fn push_run(runs: &mut Vec<PaintRun>, run: PaintRun) -> Result<(), SixelError> {
    runs.try_reserve(1).map_err(|_| SixelError::Allocation)?;
    runs.push(run);
    Ok(())
}

fn parse_number(data: &[u8], cursor: &mut usize) -> Result<Option<usize>, SixelError> {
    let start = *cursor;
    let mut value = 0usize;
    while let Some(byte @ b'0'..=b'9') = data.get(*cursor).copied() {
        value = value
            .checked_mul(10)
            .and_then(|value| value.checked_add(usize::from(byte - b'0')))
            .ok_or(SixelError::Malformed {
                offset: start,
                reason: "decimal parameter overflows",
            })?;
        *cursor += 1;
    }
    Ok((*cursor != start).then_some(value))
}

fn optional_number(data: &[u8], cursor: &mut usize) -> Result<Option<usize>, SixelError> {
    parse_number(data, cursor)
}

fn required_number(
    data: &[u8],
    cursor: &mut usize,
    reason: &'static str,
) -> Result<usize, SixelError> {
    parse_number(data, cursor)?.ok_or(SixelError::Malformed {
        offset: *cursor,
        reason,
    })
}

fn require_separator(data: &[u8], cursor: &mut usize) -> Result<(), SixelError> {
    if data.get(*cursor) != Some(&b';') {
        return Err(SixelError::Malformed {
            offset: *cursor,
            reason: "missing semicolon",
        });
    }
    *cursor += 1;
    Ok(())
}

fn checked_add(left: usize, right: usize, kind: LimitKind) -> Result<usize, SixelError> {
    left.checked_add(right)
        .ok_or(SixelError::Limit(LimitError::ArithmeticOverflow { kind }))
}

fn charge(total: &mut usize, amount: usize, limit: ParserWorkLimit) -> Result<(), SixelError> {
    *total = limit.checked_total(*total, amount)?;
    Ok(())
}

const fn rgba(red: u8, green: u8, blue: u8) -> Rgba8 {
    Rgba8 {
        red,
        green,
        blue,
        alpha: 255,
    }
}

fn rgb_percent(red: u8, green: u8, blue: u8) -> Rgba8 {
    rgba(
        scale_percent(red),
        scale_percent(green),
        scale_percent(blue),
    )
}

fn scale_percent(value: u8) -> u8 {
    ((u16::from(value) * 255 + 50) / 100) as u8
}

// DEC's HLS wheel is rotated: 0° is blue, 120° is red, and 240° is green.
fn hls(hue: u16, lightness: u8, saturation: u8) -> Rgba8 {
    let hue = f64::from((u32::from(hue) + 240) % 360) / 360.0;
    let lightness = f64::from(lightness) / 100.0;
    let saturation = f64::from(saturation) / 100.0;
    if saturation == 0.0 {
        let value = (lightness * 255.0).round() as u8;
        return rgba(value, value, value);
    }
    let q = if lightness < 0.5 {
        lightness * (1.0 + saturation)
    } else {
        lightness + saturation - lightness * saturation
    };
    let p = 2.0 * lightness - q;
    rgba(
        hue_component(p, q, hue + 1.0 / 3.0),
        hue_component(p, q, hue),
        hue_component(p, q, hue - 1.0 / 3.0),
    )
}

fn hue_component(p: f64, q: f64, mut hue: f64) -> u8 {
    if hue < 0.0 {
        hue += 1.0;
    }
    if hue > 1.0 {
        hue -= 1.0;
    }
    let value = if hue < 1.0 / 6.0 {
        p + (q - p) * 6.0 * hue
    } else if hue < 0.5 {
        q
    } else if hue < 2.0 / 3.0 {
        p + (q - p) * (2.0 / 3.0 - hue) * 6.0
    } else {
        p
    };
    (value * 255.0).round() as u8
}
