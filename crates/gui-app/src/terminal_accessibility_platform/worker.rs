//! Background lifecycle for Datum's Linux accessibility-bus service.

use std::collections::VecDeque;
use std::io::{self, Read, Write};
use std::os::fd::AsRawFd;
use std::os::unix::net::UnixStream;
use std::sync::{Arc, Mutex};

use crate::console_accessibility::AccessibilityAnnouncement;
use crate::selection_accessibility::SelectionAccessibilitySnapshot;
use crate::terminal_accessibility::TerminalAccessibilitySnapshot;
use crate::terminal_accessibility_bridge::TerminalAccessibilityEvent;
use datum_gui_protocol::GlobalPreferencesAccessibleNode;
use datum_gui_protocol::gui_menu_model::accessibility::MenuAccessibleNode;

use super::atspi::{REGISTRY_NAME, REGISTRY_PATH, ROOT_PATH, ServiceState};
use super::connection::{BusConnection, object_reference_body};
use super::dbus::{Message, MessageType};
use super::events;

const SOCKET_INTERFACE: &str = "org.a11y.atspi.Socket";
const WAKE_BYTES: usize = 64;
const ANNOUNCEMENT_CAPACITY: usize = 64;

struct PendingUpdate {
    snapshot: Arc<TerminalAccessibilitySnapshot>,
    events: Vec<TerminalAccessibilityEvent>,
    terminal_available: bool,
    announcements: VecDeque<AccessibilityAnnouncement>,
    preferences: Vec<GlobalPreferencesAccessibleNode>,
    menus: Vec<MenuAccessibleNode>,
    selection: Option<SelectionAccessibilitySnapshot>,
}

struct Shared {
    pending: Option<PendingUpdate>,
    latest_snapshot: Arc<TerminalAccessibilitySnapshot>,
    terminal_available: bool,
    latest_preferences: Vec<GlobalPreferencesAccessibleNode>,
    latest_menus: Vec<MenuAccessibleNode>,
    latest_selection: Option<SelectionAccessibilitySnapshot>,
}

pub(crate) struct PlatformBridge {
    shared: Arc<Mutex<Shared>>,
    wake: UnixStream,
}

impl PlatformBridge {
    pub(crate) fn start(
        snapshot: Arc<TerminalAccessibilitySnapshot>,
        events: Vec<TerminalAccessibilityEvent>,
    ) -> io::Result<Self> {
        Self::start_update(PendingUpdate {
            snapshot,
            events,
            terminal_available: true,
            announcements: VecDeque::new(),
            preferences: Vec::new(),
            menus: Vec::new(),
            selection: None,
        })
    }

    fn start_update(update: PendingUpdate) -> io::Result<Self> {
        let (wake, worker_wake) = UnixStream::pair()?;
        wake.set_nonblocking(true)?;
        worker_wake.set_nonblocking(true)?;
        let latest_snapshot = update.snapshot.clone();
        let terminal_available = update.terminal_available;
        let latest_preferences = update.preferences.clone();
        let latest_menus = update.menus.clone();
        let latest_selection = update.selection.clone();
        let shared = Arc::new(Mutex::new(Shared {
            pending: Some(update),
            latest_snapshot,
            terminal_available,
            latest_preferences,
            latest_menus,
            latest_selection,
        }));
        let worker_shared = Arc::clone(&shared);
        std::thread::Builder::new()
            .name("datum-atspi".into())
            .spawn(move || run(worker_shared, worker_wake))?;
        let mut bridge = Self { shared, wake };
        bridge.notify();
        Ok(bridge)
    }

    pub(crate) fn start_announcement(announcement: AccessibilityAnnouncement) -> io::Result<Self> {
        let mut announcements = VecDeque::new();
        push_bounded_announcement(&mut announcements, announcement);
        Self::start_update(PendingUpdate {
            snapshot: empty_snapshot(),
            events: Vec::new(),
            terminal_available: false,
            announcements,
            preferences: Vec::new(),
            menus: Vec::new(),
            selection: None,
        })
    }

    pub(crate) fn start_preferences(
        preferences: Vec<GlobalPreferencesAccessibleNode>,
    ) -> io::Result<Self> {
        Self::start_update(PendingUpdate {
            snapshot: empty_snapshot(),
            events: Vec::new(),
            terminal_available: false,
            announcements: VecDeque::new(),
            preferences,
            menus: Vec::new(),
            selection: None,
        })
    }

    pub(crate) fn publish(
        &mut self,
        snapshot: Arc<TerminalAccessibilitySnapshot>,
        events: Vec<TerminalAccessibilityEvent>,
    ) {
        if let Ok(mut shared) = self.shared.lock() {
            shared.latest_snapshot = snapshot.clone();
            shared.terminal_available = true;
            match &mut shared.pending {
                Some(pending) => {
                    pending.snapshot = snapshot;
                    pending.terminal_available = true;
                    for event in events {
                        if !pending.events.contains(&event) {
                            pending.events.push(event);
                        }
                    }
                }
                None => {
                    let preferences = shared.latest_preferences.clone();
                    let menus = shared.latest_menus.clone();
                    let selection = shared.latest_selection.clone();
                    shared.pending = Some(PendingUpdate {
                        snapshot,
                        events,
                        terminal_available: true,
                        announcements: VecDeque::new(),
                        preferences,
                        menus,
                        selection,
                    })
                }
            }
        }
        self.notify();
    }

    pub(crate) fn publish_announcement(&mut self, announcement: AccessibilityAnnouncement) {
        if let Ok(mut shared) = self.shared.lock() {
            let snapshot = shared.latest_snapshot.clone();
            let terminal_available = shared.terminal_available;
            let preferences = shared.latest_preferences.clone();
            let menus = shared.latest_menus.clone();
            let selection = shared.latest_selection.clone();
            let pending = shared.pending.get_or_insert_with(|| PendingUpdate {
                snapshot,
                events: Vec::new(),
                terminal_available,
                announcements: VecDeque::new(),
                preferences,
                menus,
                selection,
            });
            push_bounded_announcement(&mut pending.announcements, announcement);
        }
        self.notify();
    }

    pub(crate) fn publish_preferences(
        &mut self,
        preferences: Vec<GlobalPreferencesAccessibleNode>,
    ) {
        if let Ok(mut shared) = self.shared.lock() {
            shared.latest_preferences = preferences.clone();
            let snapshot = shared.latest_snapshot.clone();
            let terminal_available = shared.terminal_available;
            let menus = shared.latest_menus.clone();
            let selection = shared.latest_selection.clone();
            let pending = shared.pending.get_or_insert_with(|| PendingUpdate {
                snapshot,
                events: Vec::new(),
                terminal_available,
                announcements: VecDeque::new(),
                preferences: Vec::new(),
                menus,
                selection,
            });
            pending.preferences = preferences;
        }
        self.notify();
    }

    pub(crate) fn start_menus(menus: Vec<MenuAccessibleNode>) -> io::Result<Self> {
        Self::start_update(PendingUpdate {
            snapshot: empty_snapshot(),
            events: Vec::new(),
            terminal_available: false,
            announcements: VecDeque::new(),
            preferences: Vec::new(),
            menus,
            selection: None,
        })
    }

    pub(crate) fn publish_menus(&mut self, menus: Vec<MenuAccessibleNode>) {
        if let Ok(mut shared) = self.shared.lock() {
            shared.latest_menus = menus.clone();
            let snapshot = shared.latest_snapshot.clone();
            let terminal_available = shared.terminal_available;
            let preferences = shared.latest_preferences.clone();
            let selection = shared.latest_selection.clone();
            let pending = shared.pending.get_or_insert_with(|| PendingUpdate {
                snapshot,
                events: Vec::new(),
                terminal_available,
                announcements: VecDeque::new(),
                preferences,
                menus: Vec::new(),
                selection,
            });
            pending.menus = menus;
        }
        self.notify();
    }

    pub(crate) fn start_selection(
        selection: Option<SelectionAccessibilitySnapshot>,
    ) -> io::Result<Self> {
        Self::start_update(PendingUpdate {
            snapshot: empty_snapshot(),
            events: Vec::new(),
            terminal_available: false,
            announcements: VecDeque::new(),
            preferences: Vec::new(),
            menus: Vec::new(),
            selection,
        })
    }
    pub(crate) fn publish_selection(&mut self, selection: Option<SelectionAccessibilitySnapshot>) {
        if let Ok(mut shared) = self.shared.lock() {
            shared.latest_selection = selection.clone();
            let snapshot = shared.latest_snapshot.clone();
            let terminal_available = shared.terminal_available;
            let preferences = shared.latest_preferences.clone();
            let menus = shared.latest_menus.clone();
            let pending = shared.pending.get_or_insert_with(|| PendingUpdate {
                snapshot,
                events: Vec::new(),
                terminal_available,
                announcements: VecDeque::new(),
                preferences,
                menus,
                selection: None,
            });
            pending.selection = selection;
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

fn push_bounded_announcement(
    announcements: &mut VecDeque<AccessibilityAnnouncement>,
    announcement: AccessibilityAnnouncement,
) {
    if announcements.len() == ANNOUNCEMENT_CAPACITY {
        announcements.pop_front();
    }
    announcements.push_back(announcement);
}

fn run(shared: Arc<Mutex<Shared>>, mut wake: UnixStream) {
    let mut connection = None;
    let mut service = None;
    loop {
        let update = take_update(&shared);
        if let Some(update) = update {
            let mut newly_connected = false;
            if connection.is_none() {
                match connect(
                    &update.snapshot,
                    update.terminal_available,
                    update.preferences.clone(),
                    update.menus.clone(),
                ) {
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
                let previous_menus = if newly_connected {
                    Vec::new()
                } else {
                    state.menus.clone()
                };
                let previous_root_offset = usize::from(state.terminal_available)
                    + usize::from(!state.preferences.is_empty());
                let previous_selection = if newly_connected {
                    None
                } else {
                    state.selection.clone()
                };
                state.selection = update.selection;
                state.snapshot = update.snapshot;
                state.terminal_available = update.terminal_available;
                state.preferences = update.preferences;
                state.menus = update.menus;
                let messages = events::messages(
                    || active.take_serial(),
                    previous.as_deref(),
                    &state.snapshot,
                    &update.events,
                );
                let terminal_failed = messages
                    .iter()
                    .try_for_each(|message| active.send(message))
                    .is_err();
                let menu_messages = super::atspi::menu_messages(
                    || active.take_serial(),
                    &state.bus_name,
                    &previous_menus,
                    &state.menus,
                    previous_root_offset,
                    usize::from(state.terminal_available)
                        + usize::from(!state.preferences.is_empty()),
                );
                let menu_failed = menu_messages
                    .iter()
                    .try_for_each(|message| active.send(message))
                    .is_err();
                let selection_messages = super::atspi::selection_messages(
                    || active.take_serial(),
                    &state.bus_name,
                    previous_selection.as_ref(),
                    state.selection.as_ref(),
                    state,
                );
                let selection_failed = selection_messages
                    .iter()
                    .try_for_each(|message| active.send(message))
                    .is_err();
                let mut announcement_failed = false;
                for announcement in &update.announcements {
                    let message = events::announcement_message(active.take_serial(), announcement);
                    if active.send(&message).is_err() {
                        announcement_failed = true;
                        break;
                    }
                }
                terminal_failed || menu_failed || selection_failed || announcement_failed
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

fn connect(
    snapshot: &Arc<TerminalAccessibilitySnapshot>,
    terminal_available: bool,
    preferences: Vec<GlobalPreferencesAccessibleNode>,
    menus: Vec<MenuAccessibleNode>,
) -> io::Result<(BusConnection, ServiceState)> {
    let address = BusConnection::accessibility_address()?;
    let mut connection = BusConnection::connect(&address)?;
    let mut service = ServiceState::new(snapshot.clone(), terminal_available, preferences);
    service.menus = menus;
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

pub(super) fn empty_snapshot() -> Arc<TerminalAccessibilitySnapshot> {
    Arc::new(TerminalAccessibilitySnapshot {
        session_id: String::new(),
        title: "Terminal".to_string(),
        text: String::new(),
        caret: 0,
        selection: None,
        links: Vec::new(),
        focused: false,
        bell_count: 0,
        bounds: Default::default(),
    })
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
#[path = "worker/tests.rs"]
mod tests;
