//! Minimal bounded D-Bus wire codec used only by Datum's Linux AT-SPI bridge.
//!
//! This is not a general D-Bus library. It implements the standard message
//! envelope and the exact scalar/container encodings required by the AT-SPI
//! interfaces served by Datum.

use std::io;

use super::body::{BodyReader, BodyWriter, Cursor, Endian, aligned, invalid, pad, put_u32};

pub(super) const MAX_MESSAGE_BYTES: usize = 16 * 1024 * 1024;
const FIXED_HEADER_BYTES: usize = 16;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum MessageType {
    MethodCall = 1,
    MethodReturn = 2,
    Error = 3,
    Signal = 4,
}

impl TryFrom<u8> for MessageType {
    type Error = io::Error;

    fn try_from(value: u8) -> io::Result<Self> {
        match value {
            1 => Ok(Self::MethodCall),
            2 => Ok(Self::MethodReturn),
            3 => Ok(Self::Error),
            4 => Ok(Self::Signal),
            _ => Err(invalid("unknown D-Bus message type")),
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(super) struct Header {
    pub(super) path: Option<String>,
    pub(super) interface: Option<String>,
    pub(super) member: Option<String>,
    pub(super) error_name: Option<String>,
    pub(super) reply_serial: Option<u32>,
    pub(super) destination: Option<String>,
    pub(super) sender: Option<String>,
    pub(super) signature: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct Message {
    pub(super) kind: MessageType,
    pub(super) flags: u8,
    pub(super) serial: u32,
    pub(super) header: Header,
    pub(super) body: Vec<u8>,
    endian: Endian,
}

impl Message {
    pub(super) fn method_call(
        serial: u32,
        destination: &str,
        path: &str,
        interface: &str,
        member: &str,
        signature: &str,
        body: Vec<u8>,
    ) -> Self {
        Self {
            kind: MessageType::MethodCall,
            flags: 0,
            serial,
            header: Header {
                path: Some(path.into()),
                interface: Some(interface.into()),
                member: Some(member.into()),
                destination: Some(destination.into()),
                signature: signature.into(),
                ..Header::default()
            },
            body,
            endian: Endian::Little,
        }
    }

    pub(super) fn method_return(
        serial: u32,
        reply_serial: u32,
        destination: Option<&str>,
        signature: &str,
        body: Vec<u8>,
    ) -> Self {
        Self {
            kind: MessageType::MethodReturn,
            flags: 0,
            serial,
            header: Header {
                reply_serial: Some(reply_serial),
                destination: destination.map(str::to_owned),
                signature: signature.into(),
                ..Header::default()
            },
            body,
            endian: Endian::Little,
        }
    }

    pub(super) fn error(
        serial: u32,
        reply_serial: u32,
        destination: Option<&str>,
        name: &str,
        description: &str,
    ) -> Self {
        let mut body = BodyWriter::new();
        body.string(description);
        Self {
            kind: MessageType::Error,
            flags: 0,
            serial,
            header: Header {
                error_name: Some(name.into()),
                reply_serial: Some(reply_serial),
                destination: destination.map(str::to_owned),
                signature: "s".into(),
                ..Header::default()
            },
            body: body.finish(),
            endian: Endian::Little,
        }
    }

    pub(super) fn signal(
        serial: u32,
        path: &str,
        interface: &str,
        member: &str,
        signature: &str,
        body: Vec<u8>,
    ) -> Self {
        Self {
            kind: MessageType::Signal,
            flags: 0,
            serial,
            header: Header {
                path: Some(path.into()),
                interface: Some(interface.into()),
                member: Some(member.into()),
                signature: signature.into(),
                ..Header::default()
            },
            body,
            endian: Endian::Little,
        }
    }

    pub(super) fn encode(&self) -> io::Result<Vec<u8>> {
        if self.serial == 0 {
            return Err(invalid("D-Bus serial must not be zero"));
        }
        if self.body.len() > MAX_MESSAGE_BYTES {
            return Err(invalid("D-Bus body exceeds Datum limit"));
        }
        let mut fields = BodyWriter::new();
        if let Some(value) = &self.header.path {
            fields.header_field_string(1, "o", value);
        }
        if let Some(value) = &self.header.interface {
            fields.header_field_string(2, "s", value);
        }
        if let Some(value) = &self.header.member {
            fields.header_field_string(3, "s", value);
        }
        if let Some(value) = &self.header.error_name {
            fields.header_field_string(4, "s", value);
        }
        if let Some(value) = self.header.reply_serial {
            fields.header_field_u32(5, value);
        }
        if let Some(value) = &self.header.destination {
            fields.header_field_string(6, "s", value);
        }
        if let Some(value) = &self.header.sender {
            fields.header_field_string(7, "s", value);
        }
        if !self.header.signature.is_empty() {
            fields.header_field_signature(8, &self.header.signature);
        }
        let fields = fields.finish();
        let total = aligned(FIXED_HEADER_BYTES + fields.len(), 8)
            .checked_add(self.body.len())
            .ok_or_else(|| invalid("D-Bus message length overflow"))?;
        if total > MAX_MESSAGE_BYTES {
            return Err(invalid("D-Bus message exceeds Datum limit"));
        }
        let mut out = Vec::with_capacity(total);
        out.extend_from_slice(&[b'l', self.kind as u8, self.flags, 1]);
        put_u32(&mut out, self.body.len() as u32);
        put_u32(&mut out, self.serial);
        put_u32(&mut out, fields.len() as u32);
        out.extend_from_slice(&fields);
        pad(&mut out, 8);
        out.extend_from_slice(&self.body);
        Ok(out)
    }

    fn decode(frame: &[u8]) -> io::Result<Self> {
        let layout = FrameLayout::read(frame)?;
        if frame.len() != layout.total {
            return Err(invalid("D-Bus frame length mismatch"));
        }
        let endian = Endian::from_marker(frame[0])?;
        let kind = MessageType::try_from(frame[1])?;
        if frame[3] != 1 {
            return Err(invalid("unsupported D-Bus major version"));
        }
        let serial = endian.u32(&frame[8..12]);
        if serial == 0 {
            return Err(invalid("D-Bus serial must not be zero"));
        }
        let mut cursor = Cursor::new(&frame[16..16 + layout.fields], endian);
        let mut header = Header::default();
        while !cursor.is_done() {
            cursor.align(8)?;
            if cursor.is_done() {
                break;
            }
            let code = cursor.byte()?;
            let signature = cursor.signature()?;
            match (code, signature.as_str()) {
                (1, "o") => header.path = Some(cursor.string()?),
                (2, "s") => header.interface = Some(cursor.string()?),
                (3, "s") => header.member = Some(cursor.string()?),
                (4, "s") => header.error_name = Some(cursor.string()?),
                (5, "u") => header.reply_serial = Some(cursor.u32()?),
                (6, "s") => header.destination = Some(cursor.string()?),
                (7, "s") => header.sender = Some(cursor.string()?),
                (8, "g") => header.signature = cursor.signature()?,
                (_, signature) => cursor.skip_value(signature)?,
            }
        }
        Ok(Self {
            kind,
            flags: frame[2],
            serial,
            header,
            body: frame[layout.body_start..layout.total].to_vec(),
            endian,
        })
    }

    pub(super) fn body_reader(&self) -> BodyReader<'_> {
        BodyReader::with_endian(&self.body, self.endian)
    }
}

#[derive(Default)]
pub(super) struct FrameBuffer {
    bytes: Vec<u8>,
}

impl FrameBuffer {
    pub(super) fn push(&mut self, bytes: &[u8]) -> io::Result<Vec<Message>> {
        let mut messages = Vec::new();
        let mut incoming = bytes;
        while !incoming.is_empty() {
            let available = MAX_MESSAGE_BYTES.saturating_sub(self.bytes.len());
            if available == 0 {
                return Err(invalid("D-Bus receive buffer exceeds Datum limit"));
            }
            let count = available.min(incoming.len());
            self.bytes.extend_from_slice(&incoming[..count]);
            incoming = &incoming[count..];
            self.drain_complete(&mut messages)?;
            if self.bytes.len() == MAX_MESSAGE_BYTES && !self.bytes.is_empty() {
                return Err(invalid("D-Bus receive buffer exceeds Datum limit"));
            }
        }
        Ok(messages)
    }

    fn drain_complete(&mut self, messages: &mut Vec<Message>) -> io::Result<()> {
        while self.bytes.len() >= FIXED_HEADER_BYTES {
            let layout = FrameLayout::read(&self.bytes)?;
            if self.bytes.len() < layout.total {
                return Ok(());
            }
            let frame: Vec<u8> = self.bytes.drain(..layout.total).collect();
            messages.push(Message::decode(&frame)?);
        }
        Ok(())
    }
}

struct FrameLayout {
    fields: usize,
    body_start: usize,
    total: usize,
}

impl FrameLayout {
    fn read(bytes: &[u8]) -> io::Result<Self> {
        if bytes.len() < FIXED_HEADER_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "short D-Bus header",
            ));
        }
        let endian = Endian::from_marker(bytes[0])?;
        let body = endian.u32(&bytes[4..8]) as usize;
        let fields = endian.u32(&bytes[12..16]) as usize;
        let body_start = aligned(
            FIXED_HEADER_BYTES
                .checked_add(fields)
                .ok_or_else(|| invalid("D-Bus header length overflow"))?,
            8,
        );
        let total = body_start
            .checked_add(body)
            .ok_or_else(|| invalid("D-Bus frame length overflow"))?;
        if total > MAX_MESSAGE_BYTES {
            return Err(invalid("D-Bus frame exceeds Datum limit"));
        }
        Ok(Self {
            fields,
            body_start,
            total,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terminal_accessibility_platform::body::BodyWriter;

    #[test]
    fn method_call_round_trips_with_fragmented_input() {
        let mut body = BodyWriter::new();
        body.string("org.a11y.Bus");
        let message = Message::method_call(
            7,
            "org.freedesktop.DBus",
            "/org/freedesktop/DBus",
            "org.freedesktop.DBus",
            "Hello",
            "s",
            body.finish(),
        );
        let encoded = message.encode().unwrap();
        let mut frames = FrameBuffer::default();
        let mut decoded = Vec::new();
        for byte in encoded {
            decoded.extend(frames.push(&[byte]).unwrap());
        }
        assert_eq!(decoded, vec![message]);
    }

    #[test]
    fn malformed_and_oversize_frames_fail_closed() {
        let mut invalid = vec![0; 16];
        invalid[0] = b'?';
        assert_eq!(
            FrameBuffer::default().push(&invalid).unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );
        let mut oversize = vec![b'l', 1, 0, 1];
        oversize.extend_from_slice(&((MAX_MESSAGE_BYTES as u32) + 1).to_le_bytes());
        oversize.extend_from_slice(&1_u32.to_le_bytes());
        oversize.extend_from_slice(&0_u32.to_le_bytes());
        assert_eq!(
            FrameBuffer::default().push(&oversize).unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );
    }
}
