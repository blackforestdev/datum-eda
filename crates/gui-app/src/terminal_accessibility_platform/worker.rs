//! Background lifecycle for Datum's Linux accessibility-bus service.

use std::io::{self, Read, Write};
use std::os::fd::AsRawFd;
use std::os::unix::net::UnixStream;
use std::sync::{Arc, Mutex};

use crate::terminal_accessibility::TerminalAccessibilitySnapshot;
use crate::terminal_accessibility_bridge::TerminalAccessibilityEvent;

use super::atspi::{REGISTRY_NAME, REGISTRY_PATH, ROOT_PATH, ServiceState};
use super::connection::{BusConnection, object_reference_body};
use super::dbus::{Message, MessageType};
use super::events;

const SOCKET_INTERFACE: &str = "org.a11y.atspi.Socket";
const WAKE_BYTES: usize = 64;

struct PendingUpdate {
    snapshot: TerminalAccessibilitySnapshot,
    events: Vec<TerminalAccessibilityEvent>,
}

struct Shared {
    pending: Option<PendingUpdate>,
}

pub(crate) struct PlatformBridge {
    shared: Arc<Mutex<Shared>>,
    wake: UnixStream,
}

impl PlatformBridge {
    pub(crate) fn start(
        snapshot: TerminalAccessibilitySnapshot,
        events: Vec<TerminalAccessibilityEvent>,
    ) -> io::Result<Self> {
        let (wake, worker_wake) = UnixStream::pair()?;
        wake.set_nonblocking(true)?;
        worker_wake.set_nonblocking(true)?;
        let shared = Arc::new(Mutex::new(Shared {
            pending: Some(PendingUpdate { snapshot, events }),
        }));
        let worker_shared = Arc::clone(&shared);
        std::thread::Builder::new()
            .name("datum-atspi".into())
            .spawn(move || run(worker_shared, worker_wake))?;
        let mut bridge = Self { shared, wake };
        bridge.notify();
        Ok(bridge)
    }

    pub(crate) fn publish(
        &mut self,
        snapshot: TerminalAccessibilitySnapshot,
        events: Vec<TerminalAccessibilityEvent>,
    ) {
        if let Ok(mut shared) = self.shared.lock() {
            match &mut shared.pending {
                Some(pending) => {
                    pending.snapshot = snapshot;
                    for event in events {
                        if !pending.events.contains(&event) {
                            pending.events.push(event);
                        }
                    }
                }
                None => shared.pending = Some(PendingUpdate { snapshot, events }),
            }
        }
        self.notify();
    }

    fn notify(&mut self) {
        match self.wake.write(&[1]) {
            Ok(_) => {}
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {}
            Err(_) => {}
        }
    }
}

fn run(shared: Arc<Mutex<Shared>>, mut wake: UnixStream) {
    let mut connection = None;
    let mut service = None;
    loop {
        let update = take_update(&shared);
        if let Some(update) = update {
            let mut newly_connected = false;
            if connection.is_none() {
                match connect(&update.snapshot) {
                    Ok((next_connection, next_service)) => {
                        connection = Some(next_connection);
                        service = Some(next_service);
                        newly_connected = true;
                    }
                    Err(_) => {
                        if !wait_for_wake(&mut wake, None) {
                            return;
                        }
                        continue;
                    }
                }
            }
            let send_failed = if let (Some(active), Some(state)) = (&mut connection, &mut service) {
                let previous = (!newly_connected).then(|| state.snapshot.clone());
                state.snapshot = update.snapshot;
                let messages = events::messages(
                    || active.take_serial(),
                    previous.as_ref(),
                    &state.snapshot,
                    &update.events,
                );
                messages
                    .iter()
                    .try_for_each(|message| active.send(message))
                    .is_err()
            } else {
                false
            };
            if send_failed {
                connection.take();
                service.take();
            }
        }

        let Some(active) = connection.as_mut() else {
            if !wait_for_wake(&mut wake, None) {
                return;
            }
            continue;
        };
        if !wait_for_wake(&mut wake, Some(active.raw_fd())) {
            return;
        }
        drain_wake(&mut wake);
        match active.receive_available() {
            Ok(messages) => {
                let Some(state) = service.as_mut() else {
                    continue;
                };
                for call in messages
                    .into_iter()
                    .filter(|message| message.kind == MessageType::MethodCall)
                {
                    let reply = state.dispatch(active.take_serial(), &call);
                    if active.send(&reply).is_err() {
                        connection.take();
                        service.take();
                        break;
                    }
                }
            }
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {}
            Err(_) => {
                connection.take();
                service.take();
            }
        }
    }
}

fn connect(snapshot: &TerminalAccessibilitySnapshot) -> io::Result<(BusConnection, ServiceState)> {
    let address = BusConnection::accessibility_address()?;
    let mut connection = BusConnection::connect(&address)?;
    let mut service = ServiceState::new(snapshot.clone());
    service.set_bus_name(connection.unique_name().to_owned());
    let serial = connection.take_serial();
    let request = Message::method_call(
        serial,
        REGISTRY_NAME,
        REGISTRY_PATH,
        SOCKET_INTERFACE,
        "Embed",
        "(so)",
        object_reference_body(connection.unique_name(), ROOT_PATH),
    );
    let reply = connection
        .call_blocking_with(request, |serial, call| Some(service.dispatch(serial, call)))?;
    let mut body = reply.body_reader();
    let bus_name = body.string()?;
    let path = body.object_path()?;
    if !body.is_done() || !path.starts_with('/') {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid AT-SPI registry reference",
        ));
    }
    service.registry_parent = (bus_name, path);
    connection.enter_nonblocking()?;
    Ok((connection, service))
}

fn take_update(shared: &Mutex<Shared>) -> Option<PendingUpdate> {
    shared.lock().ok()?.pending.take()
}

fn wait_for_wake(wake: &mut UnixStream, bus_fd: Option<libc::c_int>) -> bool {
    let mut fds = [
        libc::pollfd {
            fd: wake.as_raw_fd(),
            events: libc::POLLIN,
            revents: 0,
        },
        libc::pollfd {
            fd: bus_fd.unwrap_or(-1),
            events: libc::POLLIN,
            revents: 0,
        },
    ];
    loop {
        let result = unsafe { libc::poll(fds.as_mut_ptr(), fds.len() as libc::nfds_t, -1) };
        if result > 0 {
            if fds[0].revents & (libc::POLLHUP | libc::POLLERR | libc::POLLNVAL) != 0 {
                return false;
            }
            return true;
        }
        if result == 0 {
            continue;
        }
        if io::Error::last_os_error().kind() != io::ErrorKind::Interrupted {
            return false;
        }
    }
}

fn drain_wake(wake: &mut UnixStream) {
    let mut bytes = [0_u8; WAKE_BYTES];
    loop {
        match wake.read(&mut bytes) {
            Ok(0) => return,
            Ok(_) => {}
            Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => return,
            Err(_) => return,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terminal_accessibility::TerminalAccessibilityBounds;

    fn snapshot(text: &str) -> TerminalAccessibilitySnapshot {
        TerminalAccessibilitySnapshot {
            session_id: "s".into(),
            title: "Terminal".into(),
            text: text.into(),
            caret: text.chars().count(),
            selection: None,
            links: Vec::new(),
            focused: true,
            bell_count: 0,
            bounds: TerminalAccessibilityBounds::default(),
        }
    }

    #[test]
    fn pending_updates_replace_snapshot_and_coalesce_event_kinds() {
        let shared = Mutex::new(Shared {
            pending: Some(PendingUpdate {
                snapshot: snapshot("a"),
                events: vec![TerminalAccessibilityEvent::TextChanged],
            }),
        });
        {
            let mut state = shared.lock().unwrap();
            let pending = state.pending.as_mut().unwrap();
            pending.snapshot = snapshot("abc");
            if !pending
                .events
                .contains(&TerminalAccessibilityEvent::CaretMoved)
            {
                pending.events.push(TerminalAccessibilityEvent::CaretMoved);
            }
        }
        let pending = take_update(&shared).unwrap();
        assert_eq!(pending.snapshot.text, "abc");
        assert_eq!(pending.events.len(), 2);
        assert!(take_update(&shared).is_none());
    }

    #[test]
    #[ignore = "requires a live Linux accessibility bus"]
    fn real_accessibility_bus_accepts_datum_registration() {
        let (connection, service) = connect(&snapshot("Datum accessibility probe")).unwrap();
        assert!(connection.unique_name().starts_with(':'));
        assert_eq!(service.bus_name, connection.unique_name());
        assert!(!service.registry_parent.0.is_empty());
        assert_eq!(service.registry_parent.1, ROOT_PATH);
    }
}
