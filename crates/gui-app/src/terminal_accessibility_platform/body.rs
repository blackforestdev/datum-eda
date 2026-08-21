//! Bounded D-Bus body marshalling for Datum's AT-SPI bridge.

use std::io;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Endian {
    Little,
    Big,
}

impl Endian {
    pub(super) fn from_marker(marker: u8) -> io::Result<Self> {
        match marker {
            b'l' => Ok(Self::Little),
            b'B' => Ok(Self::Big),
            _ => Err(invalid("invalid D-Bus byte order")),
        }
    }

    pub(super) fn u32(self, bytes: &[u8]) -> u32 {
        let bytes = [bytes[0], bytes[1], bytes[2], bytes[3]];
        match self {
            Self::Little => u32::from_le_bytes(bytes),
            Self::Big => u32::from_be_bytes(bytes),
        }
    }
}

pub(super) struct BodyWriter {
    bytes: Vec<u8>,
}

impl BodyWriter {
    pub(super) fn new() -> Self {
        Self { bytes: Vec::new() }
    }

    pub(super) fn finish(self) -> Vec<u8> {
        self.bytes
    }

    pub(super) fn byte(&mut self, value: u8) {
        self.bytes.push(value);
    }

    pub(super) fn bool(&mut self, value: bool) {
        self.u32(u32::from(value));
    }

    pub(super) fn i16(&mut self, value: i16) {
        self.align(2);
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    pub(super) fn i32(&mut self, value: i32) {
        self.align(4);
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    pub(super) fn u32(&mut self, value: u32) {
        self.align(4);
        put_u32(&mut self.bytes, value);
    }

    pub(super) fn f64(&mut self, value: f64) {
        self.align(8);
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    pub(super) fn string(&mut self, value: &str) {
        self.align(4);
        put_u32(&mut self.bytes, value.len() as u32);
        self.bytes.extend_from_slice(value.as_bytes());
        self.bytes.push(0);
    }

    pub(super) fn object_path(&mut self, value: &str) {
        self.string(value);
    }

    pub(super) fn signature(&mut self, value: &str) {
        debug_assert!(value.len() <= u8::MAX as usize);
        self.bytes.push(value.len() as u8);
        self.bytes.extend_from_slice(value.as_bytes());
        self.bytes.push(0);
    }

    pub(super) fn structure(&mut self, write: impl FnOnce(&mut Self)) {
        self.align(8);
        write(self);
    }

    pub(super) fn array(&mut self, element_alignment: usize, write: impl FnOnce(&mut Self)) {
        self.align(4);
        let length_at = self.bytes.len();
        put_u32(&mut self.bytes, 0);
        self.align(element_alignment);
        let start = self.bytes.len();
        write(self);
        let length = self.bytes.len() - start;
        self.bytes[length_at..length_at + 4].copy_from_slice(&(length as u32).to_le_bytes());
    }

    pub(super) fn variant(&mut self, signature: &str, write: impl FnOnce(&mut Self)) {
        self.signature(signature);
        self.align(alignment_for(signature));
        write(self);
    }

    pub(super) fn header_field_string(&mut self, code: u8, signature: &str, value: &str) {
        self.structure(|writer| {
            writer.byte(code);
            writer.variant(signature, |writer| writer.string(value));
        });
    }

    pub(super) fn header_field_u32(&mut self, code: u8, value: u32) {
        self.structure(|writer| {
            writer.byte(code);
            writer.variant("u", |writer| writer.u32(value));
        });
    }

    pub(super) fn header_field_signature(&mut self, code: u8, value: &str) {
        self.structure(|writer| {
            writer.byte(code);
            writer.variant("g", |writer| writer.signature(value));
        });
    }

    fn align(&mut self, alignment: usize) {
        pad(&mut self.bytes, alignment);
    }
}

pub(super) struct BodyReader<'a> {
    cursor: Cursor<'a>,
}

impl<'a> BodyReader<'a> {
    #[cfg(test)]
    pub(super) fn new(bytes: &'a [u8]) -> Self {
        Self::with_endian(bytes, Endian::Little)
    }

    pub(super) fn with_endian(bytes: &'a [u8], endian: Endian) -> Self {
        Self {
            cursor: Cursor::new(bytes, endian),
        }
    }

    pub(super) fn string(&mut self) -> io::Result<String> {
        self.cursor.string()
    }

    pub(super) fn u32(&mut self) -> io::Result<u32> {
        self.cursor.u32()
    }

    pub(super) fn i32(&mut self) -> io::Result<i32> {
        self.cursor.i32()
    }

    pub(super) fn object_path(&mut self) -> io::Result<String> {
        self.cursor.string()
    }

    pub(super) fn is_done(&self) -> bool {
        self.cursor.is_done()
    }

    pub(super) fn variant_i32(&mut self) -> io::Result<i32> {
        let signature = self.cursor.signature()?;
        if signature != "i" {
            return Err(invalid("expected D-Bus int32 variant"));
        }
        self.cursor.i32()
    }

    #[cfg(test)]
    pub(super) fn variant_string(&mut self) -> io::Result<String> {
        let signature = self.cursor.signature()?;
        if signature != "s" {
            return Err(invalid("expected D-Bus string variant"));
        }
        self.cursor.string()
    }

    #[cfg(test)]
    pub(super) fn variant_bool(&mut self) -> io::Result<bool> {
        let signature = self.cursor.signature()?;
        if signature != "b" {
            return Err(invalid("expected D-Bus boolean variant"));
        }
        match self.cursor.u32()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(invalid("invalid D-Bus boolean value")),
        }
    }

    #[cfg(test)]
    pub(super) fn u32_array(&mut self) -> io::Result<Vec<u32>> {
        let byte_count = self.cursor.u32()? as usize;
        self.cursor.align(4)?;
        if !byte_count.is_multiple_of(4) {
            return Err(invalid("invalid D-Bus uint32 array length"));
        }
        let mut values = Vec::with_capacity(byte_count / 4);
        for _ in 0..byte_count / 4 {
            values.push(self.cursor.u32()?);
        }
        Ok(values)
    }
}

pub(super) struct Cursor<'a> {
    bytes: &'a [u8],
    offset: usize,
    endian: Endian,
}

impl<'a> Cursor<'a> {
    pub(super) fn new(bytes: &'a [u8], endian: Endian) -> Self {
        Self {
            bytes,
            offset: 0,
            endian,
        }
    }

    pub(super) fn is_done(&self) -> bool {
        self.offset == self.bytes.len()
    }

    pub(super) fn align(&mut self, alignment: usize) -> io::Result<()> {
        let next = aligned(self.offset, alignment);
        if next > self.bytes.len() {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "D-Bus alignment",
            ));
        }
        self.offset = next;
        Ok(())
    }

    fn take(&mut self, count: usize) -> io::Result<&'a [u8]> {
        let end = self
            .offset
            .checked_add(count)
            .ok_or_else(|| invalid("D-Bus offset overflow"))?;
        let value = self
            .bytes
            .get(self.offset..end)
            .ok_or_else(|| io::Error::new(io::ErrorKind::UnexpectedEof, "truncated D-Bus value"))?;
        self.offset = end;
        Ok(value)
    }

    pub(super) fn byte(&mut self) -> io::Result<u8> {
        Ok(self.take(1)?[0])
    }

    pub(super) fn u32(&mut self) -> io::Result<u32> {
        self.align(4)?;
        let endian = self.endian;
        Ok(endian.u32(self.take(4)?))
    }

    pub(super) fn i32(&mut self) -> io::Result<i32> {
        Ok(self.u32()? as i32)
    }

    pub(super) fn string(&mut self) -> io::Result<String> {
        let length = self.u32()? as usize;
        let bytes = self.take(length)?;
        if self.byte()? != 0 {
            return Err(invalid("D-Bus string lacks NUL terminator"));
        }
        String::from_utf8(bytes.to_vec()).map_err(|_| invalid("D-Bus string is not UTF-8"))
    }

    pub(super) fn signature(&mut self) -> io::Result<String> {
        let length = self.byte()? as usize;
        let bytes = self.take(length)?;
        if self.byte()? != 0 {
            return Err(invalid("D-Bus signature lacks NUL terminator"));
        }
        let value = std::str::from_utf8(bytes)
            .map_err(|_| invalid("D-Bus signature is not ASCII"))?
            .to_owned();
        validate_signature(&value)?;
        Ok(value)
    }

    pub(super) fn skip_value(&mut self, signature: &str) -> io::Result<()> {
        match signature {
            "s" | "o" => {
                self.string()?;
            }
            "g" => {
                self.signature()?;
            }
            "u" | "i" | "b" => {
                self.u32()?;
            }
            _ => return Err(invalid("unsupported D-Bus header-field type")),
        }
        Ok(())
    }
}

fn validate_signature(signature: &str) -> io::Result<()> {
    if signature.len() > u8::MAX as usize || !signature.is_ascii() {
        return Err(invalid("invalid D-Bus signature"));
    }
    let bytes = signature.as_bytes();
    let mut offset = 0;
    while offset < bytes.len() {
        parse_complete_type(bytes, &mut offset, 0, false)?;
    }
    Ok(())
}

fn parse_complete_type(
    bytes: &[u8],
    offset: &mut usize,
    depth: usize,
    dictionary_allowed: bool,
) -> io::Result<()> {
    if depth >= 32 {
        return Err(invalid("D-Bus signature nesting exceeds Datum limit"));
    }
    let value = *bytes
        .get(*offset)
        .ok_or_else(|| invalid("incomplete D-Bus signature"))?;
    *offset += 1;
    match value {
        b'y' | b'b' | b'n' | b'q' | b'i' | b'u' | b'x' | b't' | b'd' | b'h' | b's' | b'o'
        | b'g' | b'v' => Ok(()),
        b'a' => parse_complete_type(bytes, offset, depth + 1, true),
        b'(' => {
            let start = *offset;
            while bytes.get(*offset) != Some(&b')') {
                parse_complete_type(bytes, offset, depth + 1, false)?;
            }
            if *offset == start {
                return Err(invalid("empty D-Bus structure signature"));
            }
            *offset += 1;
            Ok(())
        }
        b'{' if dictionary_allowed => {
            let key = *bytes
                .get(*offset)
                .ok_or_else(|| invalid("incomplete D-Bus dictionary signature"))?;
            if !matches!(
                key,
                b'y' | b'b'
                    | b'n'
                    | b'q'
                    | b'i'
                    | b'u'
                    | b'x'
                    | b't'
                    | b'd'
                    | b'h'
                    | b's'
                    | b'o'
                    | b'g'
            ) {
                return Err(invalid("invalid D-Bus dictionary key type"));
            }
            *offset += 1;
            parse_complete_type(bytes, offset, depth + 1, false)?;
            if bytes.get(*offset) != Some(&b'}') {
                return Err(invalid("unterminated D-Bus dictionary signature"));
            }
            *offset += 1;
            Ok(())
        }
        _ => Err(invalid("invalid D-Bus signature")),
    }
}

fn alignment_for(signature: &str) -> usize {
    match signature.as_bytes().first().copied() {
        Some(b'y' | b'g' | b'v') => 1,
        Some(b'n' | b'q') => 2,
        Some(b'i' | b'u' | b'b' | b's' | b'o' | b'a' | b'h') => 4,
        Some(b'x' | b't' | b'd' | b'(' | b'{') => 8,
        _ => 1,
    }
}

pub(super) fn aligned(value: usize, alignment: usize) -> usize {
    (value + alignment - 1) & !(alignment - 1)
}

pub(super) fn pad(bytes: &mut Vec<u8>, alignment: usize) {
    bytes.resize(aligned(bytes.len(), alignment), 0);
}

pub(super) fn put_u32(bytes: &mut Vec<u8>, value: u32) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

pub(super) fn invalid(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signature_grammar_accepts_atspi_shapes_and_rejects_hostile_nesting() {
        for valid in ["", "siiva{sv}", "a((so)(so)(so)iiassusau)", "(iiii)"] {
            validate_signature(valid).unwrap();
        }
        for invalid in ["a", "()", "{sv}", "a{vs}", "(ii", "a{ss"] {
            assert_eq!(
                validate_signature(invalid).unwrap_err().kind(),
                io::ErrorKind::InvalidData
            );
        }
        let deep = format!("{}u{}", "a(".repeat(33), ")".repeat(33));
        assert_eq!(
            validate_signature(&deep).unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );
    }

    #[test]
    fn scalar_array_structure_and_variant_alignment_is_deterministic() {
        let mut scalar = BodyWriter::new();
        scalar.byte(1);
        scalar.i32(7);
        assert_eq!(scalar.finish(), vec![1, 0, 0, 0, 7, 0, 0, 0]);

        let mut variant = BodyWriter::new();
        variant.byte(9);
        variant.variant("i", |body| body.i32(11));
        assert_eq!(variant.finish(), vec![9, 1, b'i', 0, 11, 0, 0, 0]);

        let mut array = BodyWriter::new();
        array.array(8, |body| {
            body.structure(|body| {
                body.i32(3);
                body.i32(4);
            });
        });
        assert_eq!(
            array.finish(),
            vec![8, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0]
        );
    }
}
