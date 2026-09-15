use crate::{Base64Limits, CodecError, CodecLimits, Rgba8};
use std::error::Error;
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum KittyAction {
    Transmit,
    TransmitAndPut,
    Query,
    Put,
    Delete,
    Frame,
    Animate,
    Compose,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum KittyMedium {
    Direct,
    File,
    TemporaryFile,
    SharedMemory,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum KittyFormat {
    Rgb,
    Rgba,
    Png,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct KittyControl {
    pub(crate) action: KittyAction,
    pub(crate) medium: KittyMedium,
    pub(crate) format: KittyFormat,
    pub(crate) compression: bool,
    pub(crate) more: bool,
    pub(crate) quiet: u8,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) data_size: u32,
    pub(crate) data_offset: u32,
    pub(crate) image_id: u32,
    pub(crate) image_number: u32,
    pub(crate) placement_id: u32,
    pub(crate) usage: u32,
    pub(crate) x: u32,
    pub(crate) y: u32,
    pub(crate) crop_width: u32,
    pub(crate) crop_height: u32,
    pub(crate) offset_x: u32,
    pub(crate) offset_y: u32,
    pub(crate) columns: u32,
    pub(crate) rows: u32,
    pub(crate) no_cursor_move: bool,
    pub(crate) virtual_placement: bool,
    pub(crate) z_index: i32,
    pub(crate) z_set: bool,
    pub(crate) parent_image_id: u32,
    pub(crate) parent_placement_id: u32,
    pub(crate) horizontal_offset: i32,
    pub(crate) vertical_offset: i32,
    pub(crate) delete: u8,
    pub(crate) background: u32,
    pub(crate) composition: u32,
}

impl Default for KittyControl {
    fn default() -> Self {
        Self {
            action: KittyAction::Transmit,
            medium: KittyMedium::Direct,
            format: KittyFormat::Rgba,
            compression: false,
            more: false,
            quiet: 0,
            width: 0,
            height: 0,
            data_size: 0,
            data_offset: 0,
            image_id: 0,
            image_number: 0,
            placement_id: 0,
            usage: 0,
            x: 0,
            y: 0,
            crop_width: 0,
            crop_height: 0,
            offset_x: 0,
            offset_y: 0,
            columns: 0,
            rows: 0,
            no_cursor_move: false,
            virtual_placement: false,
            z_index: 0,
            z_set: false,
            parent_image_id: 0,
            parent_placement_id: 0,
            horizontal_offset: 0,
            vertical_offset: 0,
            delete: b'a',
            background: 0,
            composition: 0,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum KittyGraphicsError {
    Malformed { reason: &'static str },
    UnsupportedMedium,
    Codec(CodecError),
    Limit(crate::LimitError),
}

impl fmt::Display for KittyGraphicsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Malformed { reason } => {
                write!(formatter, "invalid kitty graphics command: {reason}")
            }
            Self::UnsupportedMedium => formatter.write_str(
                "kitty file and shared-memory transfers are outside TerminalCore's I/O boundary",
            ),
            Self::Codec(error) => error.fmt(formatter),
            Self::Limit(error) => error.fmt(formatter),
        }
    }
}

impl Error for KittyGraphicsError {}

impl From<CodecError> for KittyGraphicsError {
    fn from(value: CodecError) -> Self {
        Self::Codec(value)
    }
}

impl From<crate::LimitError> for KittyGraphicsError {
    fn from(value: crate::LimitError) -> Self {
        Self::Limit(value)
    }
}

pub(crate) fn parse_control(bytes: &[u8]) -> Result<KittyControl, KittyGraphicsError> {
    let mut control = KittyControl::default();
    if bytes.is_empty() {
        return Ok(control);
    }
    let mut seen = [false; 256];
    for field in bytes.split(|byte| *byte == b',') {
        let Some((&key, value)) = field.split_first() else {
            return Err(malformed("empty control field"));
        };
        if value.first() != Some(&b'=') || value.len() == 1 || seen[key as usize] {
            return Err(malformed("invalid or duplicate key/value field"));
        }
        seen[key as usize] = true;
        let value = &value[1..];
        match key {
            b'a' => control.action = parse_action(value)?,
            b't' => control.medium = parse_medium(value)?,
            b'f' => control.format = parse_format(value)?,
            b'o' => control.compression = value == b"z",
            b'm' => control.more = parse_flag(value)?,
            b'q' => control.quiet = parse_u32(value)?.min(2) as u8,
            b's' => control.width = parse_u32(value)?,
            b'v' => control.height = parse_u32(value)?,
            b'S' => control.data_size = parse_u32(value)?,
            b'O' => control.data_offset = parse_u32(value)?,
            b'i' => control.image_id = parse_u32(value)?,
            b'I' => control.image_number = parse_u32(value)?,
            b'p' => control.placement_id = parse_u32(value)?,
            b'N' => control.usage = parse_u32(value)?,
            b'x' => control.x = parse_u32(value)?,
            b'y' => control.y = parse_u32(value)?,
            b'w' => control.crop_width = parse_u32(value)?,
            b'h' => control.crop_height = parse_u32(value)?,
            b'X' => control.offset_x = parse_u32(value)?,
            b'Y' => control.offset_y = parse_u32(value)?,
            b'c' => control.columns = parse_u32(value)?,
            b'r' => control.rows = parse_u32(value)?,
            b'C' => control.composition = parse_u32(value)?,
            b'U' => control.virtual_placement = parse_flag(value)?,
            b'z' => {
                control.z_index = parse_i32(value)?;
                control.z_set = true;
            }
            b'P' => control.parent_image_id = parse_u32(value)?,
            b'Q' => control.parent_placement_id = parse_u32(value)?,
            b'H' => control.horizontal_offset = parse_i32(value)?,
            b'V' => control.vertical_offset = parse_i32(value)?,
            b'd' => {
                control.delete = *value
                    .first()
                    .ok_or_else(|| malformed("empty delete mode"))?;
                if value.len() != 1 {
                    return Err(malformed("delete mode is not one byte"));
                }
            }
            _ => return Err(malformed("unknown control key")),
        }
    }
    if control.image_id != 0 && control.image_number != 0 {
        return Err(malformed(
            "image id and image number are mutually exclusive",
        ));
    }
    match control.action {
        KittyAction::Put | KittyAction::TransmitAndPut => {
            control.no_cursor_move = seen[b'C' as usize] && control.composition == 1;
        }
        KittyAction::Frame => {
            control.composition = if seen[b'X' as usize] {
                control.offset_x
            } else {
                0
            };
            control.background = if seen[b'Y' as usize] {
                control.offset_y
            } else {
                0
            };
        }
        _ => {}
    }
    Ok(control)
}

pub(crate) fn decode_pixels(
    encoded: &[u8],
    control: &KittyControl,
    base64_limits: Base64Limits,
    codec_limits: CodecLimits,
) -> Result<(u32, u32, Vec<Rgba8>), KittyGraphicsError> {
    if control.medium != KittyMedium::Direct {
        return Err(KittyGraphicsError::UnsupportedMedium);
    }
    let mut bytes = crate::decode_base64(encoded, base64_limits)?;
    if control.compression {
        bytes = crate::decode_zlib(&bytes, codec_limits)?;
    }
    if control.data_size != 0 && bytes.len() != control.data_size as usize {
        return Err(malformed("decoded data size does not match S"));
    }
    match control.format {
        KittyFormat::Png => {
            let image = crate::decode_png(&bytes, codec_limits)?;
            Ok((image.width, image.height, image.pixels))
        }
        KittyFormat::Rgb | KittyFormat::Rgba => {
            if control.width == 0 || control.height == 0 {
                return Err(malformed("raw pixel dimensions are missing"));
            }
            let channels = if control.format == KittyFormat::Rgb {
                3
            } else {
                4
            };
            let pixels = usize::try_from(control.width)
                .ok()
                .and_then(|width| {
                    usize::try_from(control.height)
                        .ok()
                        .and_then(|height| width.checked_mul(height))
                })
                .ok_or_else(|| malformed("pixel dimensions overflow"))?;
            codec_limits.pixels.check(pixels)?;
            let expected = pixels
                .checked_mul(channels)
                .ok_or_else(|| malformed("pixel byte count overflows"))?;
            if bytes.len() != expected {
                return Err(malformed("raw pixel byte count is not exact"));
            }
            let pixels = bytes
                .chunks_exact(channels)
                .map(|pixel| Rgba8 {
                    red: pixel[0],
                    green: pixel[1],
                    blue: pixel[2],
                    alpha: if channels == 4 { pixel[3] } else { 255 },
                })
                .collect();
            Ok((control.width, control.height, pixels))
        }
    }
}

fn parse_action(value: &[u8]) -> Result<KittyAction, KittyGraphicsError> {
    match value {
        b"t" => Ok(KittyAction::Transmit),
        b"T" => Ok(KittyAction::TransmitAndPut),
        b"q" => Ok(KittyAction::Query),
        b"p" => Ok(KittyAction::Put),
        b"d" => Ok(KittyAction::Delete),
        b"f" => Ok(KittyAction::Frame),
        b"a" => Ok(KittyAction::Animate),
        b"c" => Ok(KittyAction::Compose),
        _ => Err(malformed("unknown action")),
    }
}

fn parse_medium(value: &[u8]) -> Result<KittyMedium, KittyGraphicsError> {
    match value {
        b"d" => Ok(KittyMedium::Direct),
        b"f" => Ok(KittyMedium::File),
        b"t" => Ok(KittyMedium::TemporaryFile),
        b"s" => Ok(KittyMedium::SharedMemory),
        _ => Err(malformed("unknown transmission medium")),
    }
}

fn parse_format(value: &[u8]) -> Result<KittyFormat, KittyGraphicsError> {
    match parse_u32(value)? {
        24 => Ok(KittyFormat::Rgb),
        32 => Ok(KittyFormat::Rgba),
        100 => Ok(KittyFormat::Png),
        _ => Err(malformed("unknown pixel format")),
    }
}

fn parse_flag(value: &[u8]) -> Result<bool, KittyGraphicsError> {
    match value {
        b"0" => Ok(false),
        b"1" => Ok(true),
        _ => Err(malformed("flag is not zero or one")),
    }
}

fn parse_u32(value: &[u8]) -> Result<u32, KittyGraphicsError> {
    if value.is_empty() || !value.iter().all(u8::is_ascii_digit) {
        return Err(malformed("unsigned value is malformed"));
    }
    value.iter().try_fold(0u32, |total, digit| {
        total
            .checked_mul(10)
            .and_then(|total| total.checked_add(u32::from(*digit - b'0')))
            .ok_or_else(|| malformed("unsigned value overflows"))
    })
}

fn parse_i32(value: &[u8]) -> Result<i32, KittyGraphicsError> {
    let (negative, digits) = if let Some(digits) = value.strip_prefix(b"-") {
        (true, digits)
    } else {
        (false, value)
    };
    let magnitude = parse_u32(digits)?;
    if negative {
        i32::try_from(-(i64::from(magnitude))).map_err(|_| malformed("signed value overflows"))
    } else {
        i32::try_from(magnitude).map_err(|_| malformed("signed value overflows"))
    }
}

fn malformed(reason: &'static str) -> KittyGraphicsError {
    KittyGraphicsError::Malformed { reason }
}
