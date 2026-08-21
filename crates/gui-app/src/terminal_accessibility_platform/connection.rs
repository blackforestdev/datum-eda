//! Bounded Unix D-Bus connection/authentication for the AT-SPI bridge.

use super::body::BodyWriter;
use super::dbus::{FrameBuffer, Message, MessageType};
use std::env;
use std::io::{self, Read, Write};
use std::os::fd::AsRawFd;
use std::os::linux::net::SocketAddrExt;
use std::os::unix::net::{SocketAddr, UnixStream};
use std::time::{Duration, Instant};

const AUTH_LINE_BYTES: usize = 4096;
const IO_TIMEOUT: Duration = Duration::from_secs(2);
const CALL_TIMEOUT: Duration = Duration::from_secs(5);
const READ_CHUNK_BYTES: usize = 16 * 1024;

pub(super) struct BusConnection {
    stream: UnixStream,
    frames: FrameBuffer,
    serial: u32,
    unique_name: String,
}

impl BusConnection {
    pub(super) fn connect(address: &str) -> io::Result<Self> {
        let mut stream = connect_unix_address(address)?;
        stream.set_nonblocking(true)?;
        authenticate(&mut stream)?;
        let mut connection = Self {
            stream,
            frames: FrameBuffer::default(),
            serial: 1,
            unique_name: String::new(),
        };
        let serial = connection.take_serial();
        let reply = connection.call_blocking(Message::method_call(
            serial,
            "org.freedesktop.DBus",
            "/org/freedesktop/DBus",
            "org.freedesktop.DBus",
            "Hello",
            "",
            Vec::new(),
        ))?;
        let mut body = reply.body_reader();
        connection.unique_name = body.string()?;
        if !connection.unique_name.starts_with(':') {
            return Err(invalid("D-Bus Hello returned invalid unique name"));
        }
        Ok(connection)
    }

    pub(super) fn accessibility_address() -> io::Result<String> {
        if let Some(address) = env::var_os("AT_SPI_BUS_ADDRESS") {
            let address = address
                .into_string()
                .map_err(|_| invalid("AT_SPI_BUS_ADDRESS is not UTF-8"))?;
            if !address.is_empty() {
                return Ok(address);
            }
        }
        let session = env::var("DBUS_SESSION_BUS_ADDRESS")
            .map_err(|_| io::Error::new(io::ErrorKind::NotFound, "session D-Bus unavailable"))?;
        let mut connection = Self::connect(&session)?;
        let serial = connection.take_serial();
        let reply = connection.call_blocking(Message::method_call(
            serial,
            "org.a11y.Bus",
            "/org/a11y/bus",
            "org.a11y.Bus",
            "GetAddress",
            "",
            Vec::new(),
        ))?;
        reply.body_reader().string()
    }

    pub(super) fn unique_name(&self) -> &str {
        &self.unique_name
    }

    pub(super) fn take_serial(&mut self) -> u32 {
        let serial = self.serial;
        self.serial = self
            .serial
            .checked_add(1)
            .filter(|value| *value != 0)
            .unwrap_or(1);
        serial
    }

    pub(super) fn send(&mut self, message: &Message) -> io::Result<()> {
        let encoded = message.encode()?;
        let deadline = Instant::now() + IO_TIMEOUT;
        let mut offset = 0;
        while offset < encoded.len() {
            match self.stream.write(&encoded[offset..]) {
                Ok(0) => return Err(io::Error::new(io::ErrorKind::WriteZero, "D-Bus socket")),
                Ok(count) => offset += count,
                Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                    wait_fd(self.stream.as_raw_fd(), libc::POLLOUT, deadline)?;
                }
                Err(error) => return Err(error),
            }
        }
        Ok(())
    }

    pub(super) fn enter_nonblocking(&self) -> io::Result<()> {
        self.stream.set_nonblocking(true)
    }

    pub(super) fn raw_fd(&self) -> libc::c_int {
        self.stream.as_raw_fd()
    }

    pub(super) fn receive_available(&mut self) -> io::Result<Vec<Message>> {
        let mut messages = Vec::new();
        let mut buffer = [0_u8; READ_CHUNK_BYTES];
        loop {
            match self.stream.read(&mut buffer) {
                Ok(0) => return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "D-Bus closed")),
                Ok(count) => messages.extend(self.frames.push(&buffer[..count])?),
                Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => return Ok(messages),
                Err(error) => return Err(error),
            }
        }
    }

    pub(super) fn call_blocking(&mut self, request: Message) -> io::Result<Message> {
        self.call_blocking_with(request, |_, _| None)
    }

    pub(super) fn call_blocking_with(
        &mut self,
        request: Message,
        mut dispatch: impl FnMut(u32, &Message) -> Option<Message>,
    ) -> io::Result<Message> {
        let request_serial = request.serial;
        self.send(&request)?;
        let deadline = Instant::now() + CALL_TIMEOUT;
        let mut buffer = [0_u8; READ_CHUNK_BYTES];
        loop {
            if Instant::now() >= deadline {
                return Err(io::Error::new(io::ErrorKind::TimedOut, "D-Bus method call"));
            }
            match self.stream.read(&mut buffer) {
                Ok(0) => {
                    return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "D-Bus closed"));
                }
                Ok(count) => {
                    for message in self.frames.push(&buffer[..count])? {
                        if message.header.reply_serial == Some(request_serial) {
                            return match message.kind {
                                MessageType::MethodReturn => Ok(message),
                                MessageType::Error => Err(remote_error(&message)),
                                _ => Err(invalid("invalid D-Bus method reply")),
                            };
                        } else if message.kind == MessageType::MethodCall {
                            let serial = self.take_serial();
                            if let Some(reply) = dispatch(serial, &message) {
                                self.send(&reply)?;
                            }
                        }
                    }
                }
                Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                    wait_fd(self.stream.as_raw_fd(), libc::POLLIN, deadline)?;
                }
                Err(error) => return Err(error),
            }
        }
    }
}

pub(super) fn object_reference_body(bus_name: &str, path: &str) -> Vec<u8> {
    let mut body = BodyWriter::new();
    body.structure(|body| {
        body.string(bus_name);
        body.object_path(path);
    });
    body.finish()
}

fn authenticate(stream: &mut UnixStream) -> io::Result<()> {
    let uid = unsafe { libc::geteuid() }.to_string();
    let encoded_uid = uid
        .bytes()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let deadline = Instant::now() + IO_TIMEOUT;
    write_with_deadline(stream, &[0], deadline)?;
    write_with_deadline(
        stream,
        format!("AUTH EXTERNAL {encoded_uid}\r\n").as_bytes(),
        deadline,
    )?;
    let response = read_auth_line(stream, deadline)?;
    if !response.starts_with("OK ") {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "D-Bus EXTERNAL authentication rejected",
        ));
    }
    write_with_deadline(stream, b"BEGIN\r\n", deadline)?;
    Ok(())
}

fn read_auth_line(stream: &mut UnixStream, deadline: Instant) -> io::Result<String> {
    let mut bytes = Vec::new();
    while bytes.len() < AUTH_LINE_BYTES {
        let mut byte = [0_u8; 1];
        match stream.read(&mut byte) {
            Ok(0) => return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "D-Bus closed")),
            Ok(_) => {
                bytes.push(byte[0]);
                if bytes.ends_with(b"\r\n") {
                    bytes.truncate(bytes.len() - 2);
                    return String::from_utf8(bytes)
                        .map_err(|_| invalid("D-Bus auth is not ASCII"));
                }
            }
            Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                wait_fd(stream.as_raw_fd(), libc::POLLIN, deadline)?;
            }
            Err(error) => return Err(error),
        }
    }
    Err(invalid("D-Bus authentication line exceeds Datum limit"))
}

fn write_with_deadline(stream: &mut UnixStream, bytes: &[u8], deadline: Instant) -> io::Result<()> {
    let mut offset = 0;
    while offset < bytes.len() {
        match stream.write(&bytes[offset..]) {
            Ok(0) => return Err(io::Error::new(io::ErrorKind::WriteZero, "D-Bus socket")),
            Ok(count) => offset += count,
            Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                wait_fd(stream.as_raw_fd(), libc::POLLOUT, deadline)?;
            }
            Err(error) => return Err(error),
        }
    }
    Ok(())
}

fn connect_unix_address(addresses: &str) -> io::Result<UnixStream> {
    let mut last_error = None;
    for address in addresses.split(';') {
        let Some(fields) = address.strip_prefix("unix:") else {
            continue;
        };
        let mut path = None;
        let mut abstract_name = None;
        for field in fields.split(',') {
            if let Some(value) = field.strip_prefix("path=") {
                path = Some(percent_decode(value)?);
            } else if let Some(value) = field.strip_prefix("abstract=") {
                abstract_name = Some(percent_decode(value)?);
            }
        }
        let result = if let Some(path) = path {
            UnixStream::connect(std::path::PathBuf::from(bytes_to_os_string(path)))
        } else if let Some(name) = abstract_name {
            SocketAddr::from_abstract_name(&name)
                .and_then(|address| UnixStream::connect_addr(&address))
        } else {
            continue;
        };
        match result {
            Ok(stream) => return Ok(stream),
            Err(error) => last_error = Some(error),
        }
    }
    Err(last_error.unwrap_or_else(|| invalid("no supported Unix D-Bus address")))
}

fn percent_decode(value: &str) -> io::Result<Vec<u8>> {
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            let pair = bytes
                .get(index + 1..index + 3)
                .ok_or_else(|| invalid("truncated D-Bus percent escape"))?;
            let pair = std::str::from_utf8(pair).map_err(|_| invalid("invalid D-Bus escape"))?;
            decoded
                .push(u8::from_str_radix(pair, 16).map_err(|_| invalid("invalid D-Bus escape"))?);
            index += 3;
        } else {
            decoded.push(bytes[index]);
            index += 1;
        }
    }
    if decoded.contains(&0) {
        return Err(invalid("D-Bus address contains NUL"));
    }
    Ok(decoded)
}

fn bytes_to_os_string(bytes: Vec<u8>) -> std::ffi::OsString {
    use std::os::unix::ffi::OsStringExt;
    std::ffi::OsString::from_vec(bytes)
}

fn wait_fd(fd: libc::c_int, events: libc::c_short, deadline: Instant) -> io::Result<()> {
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "D-Bus socket readiness",
            ));
        }
        let mut pollfd = libc::pollfd {
            fd,
            events,
            revents: 0,
        };
        let timeout = remaining.as_millis().min(libc::c_int::MAX as u128) as libc::c_int;
        let result = unsafe { libc::poll(&mut pollfd, 1, timeout.max(1)) };
        if result > 0 {
            if pollfd.revents & events != 0 {
                return Ok(());
            }
            if pollfd.revents & (libc::POLLERR | libc::POLLHUP | libc::POLLNVAL) != 0 {
                return Err(io::Error::new(
                    io::ErrorKind::BrokenPipe,
                    "D-Bus socket closed",
                ));
            }
            continue;
        }
        if result == 0 {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "D-Bus socket readiness",
            ));
        }
        let error = io::Error::last_os_error();
        if error.kind() != io::ErrorKind::Interrupted {
            return Err(error);
        }
    }
}

fn remote_error(message: &Message) -> io::Error {
    let description = message
        .body_reader()
        .string()
        .unwrap_or_else(|_| "D-Bus remote error".into());
    io::Error::other(format!(
        "{}: {description}",
        message
            .header
            .error_name
            .as_deref()
            .unwrap_or("D-Bus.Error")
    ))
}

fn invalid(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terminal_accessibility_platform::body::BodyReader;

    fn read_message(stream: &mut UnixStream) -> Message {
        let mut frames = FrameBuffer::default();
        let mut bytes = [0_u8; 128];
        loop {
            let count = stream.read(&mut bytes).unwrap();
            assert_ne!(count, 0, "peer closed before a complete D-Bus message");
            if let Some(message) = frames.push(&bytes[..count]).unwrap().into_iter().next() {
                return message;
            }
        }
    }

    #[test]
    fn address_parser_decodes_path_and_rejects_nul() {
        assert_eq!(
            percent_decode("/run/user/1000/bus").unwrap(),
            b"/run/user/1000/bus"
        );
        assert_eq!(
            percent_decode("name%2Dwith%2Ddash").unwrap(),
            b"name-with-dash"
        );
        assert_eq!(
            percent_decode("bad%00path").unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );
    }

    #[test]
    fn object_reference_has_standard_struct_layout() {
        let body = object_reference_body(":1.42", "/org/a11y/atspi/accessible/root");
        let mut reader = BodyReader::new(&body);
        assert_eq!(reader.string().unwrap(), ":1.42");
        assert_eq!(reader.string().unwrap(), "/org/a11y/atspi/accessible/root");
    }

    #[test]
    fn external_authentication_is_bounded_and_uses_the_effective_uid() {
        let (mut client, mut server) = UnixStream::pair().unwrap();
        client.set_nonblocking(true).unwrap();
        let expected = unsafe { libc::geteuid() }
            .to_string()
            .bytes()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        let server_thread = std::thread::spawn(move || {
            let mut first = [0_u8; 1];
            server.read_exact(&mut first).unwrap();
            assert_eq!(first, [0]);
            let auth = read_auth_line(&mut server, Instant::now() + IO_TIMEOUT).unwrap();
            assert_eq!(auth, format!("AUTH EXTERNAL {expected}"));
            server.write_all(b"OK datum-test-guid\r\n").unwrap();
            assert_eq!(
                read_auth_line(&mut server, Instant::now() + IO_TIMEOUT).unwrap(),
                "BEGIN"
            );
        });
        authenticate(&mut client).unwrap();
        server_thread.join().unwrap();
    }

    #[test]
    fn blocking_call_correlates_reply_and_services_reentrant_method_call() {
        let (client, mut server) = UnixStream::pair().unwrap();
        client.set_nonblocking(true).unwrap();
        let mut connection = BusConnection {
            stream: client,
            frames: FrameBuffer::default(),
            serial: 2,
            unique_name: ":1.77".into(),
        };
        let request = Message::method_call(
            1,
            REGISTRY_TEST_NAME,
            "/registry",
            "org.a11y.atspi.Socket",
            "Embed",
            "",
            Vec::new(),
        );
        let server_thread = std::thread::spawn(move || {
            let embed = read_message(&mut server);
            assert_eq!(embed.serial, 1);
            let callback = Message::method_call(
                80,
                ":1.77",
                "/root",
                "org.freedesktop.DBus.Properties",
                "Set",
                "",
                Vec::new(),
            );
            server.write_all(&callback.encode().unwrap()).unwrap();
            let callback_reply = read_message(&mut server);
            assert_eq!(callback_reply.header.reply_serial, Some(80));
            let mut body = BodyWriter::new();
            body.string("registered");
            let reply = Message::method_return(81, embed.serial, Some(":1.77"), "s", body.finish());
            server.write_all(&reply.encode().unwrap()).unwrap();
        });
        let reply = connection
            .call_blocking_with(request, |serial, call| {
                Some(Message::method_return(
                    serial,
                    call.serial,
                    call.header.sender.as_deref(),
                    "",
                    Vec::new(),
                ))
            })
            .unwrap();
        assert_eq!(reply.body_reader().string().unwrap(), "registered");
        server_thread.join().unwrap();
    }

    #[test]
    fn disconnect_and_oversize_auth_fail_closed() {
        let (client, server) = UnixStream::pair().unwrap();
        client.set_nonblocking(true).unwrap();
        drop(server);
        let mut connection = BusConnection {
            stream: client,
            frames: FrameBuffer::default(),
            serial: 2,
            unique_name: ":1.77".into(),
        };
        let error = connection
            .call_blocking(Message::method_call(
                1,
                REGISTRY_TEST_NAME,
                "/registry",
                "org.a11y.atspi.Socket",
                "Embed",
                "",
                Vec::new(),
            ))
            .unwrap_err();
        assert!(matches!(
            error.kind(),
            io::ErrorKind::BrokenPipe | io::ErrorKind::UnexpectedEof
        ));

        let (mut client, mut server) = UnixStream::pair().unwrap();
        client.set_nonblocking(true).unwrap();
        let server_thread = std::thread::spawn(move || {
            let mut first = [0_u8; 1];
            server.read_exact(&mut first).unwrap();
            read_auth_line(&mut server, Instant::now() + IO_TIMEOUT).unwrap();
            server.write_all(&vec![b'A'; AUTH_LINE_BYTES + 2]).unwrap();
        });
        assert_eq!(
            authenticate(&mut client).unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );
        server_thread.join().unwrap();
    }

    const REGISTRY_TEST_NAME: &str = "org.a11y.atspi.Registry";
}
