//! Process lifetime for a GUI-owned native-edit companion, never a standalone daemon.
//!
//! The GUI keeps the write end of stdin open in its retained `Child`. Unlike a
//! parent-death signal, EOF follows the owning process rather than the temporary
//! worker thread that spawned it. Shutdown wakes transport waits; the server
//! thread still finishes any already-dispatched canonical operation normally.

use std::io::{self, Read};
use std::net::Shutdown;
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

#[derive(Default)]
struct State {
    stopped: bool,
    active: Option<UnixStream>,
}

pub(super) struct NativeEditCompanion {
    state: Arc<Mutex<State>>,
}

impl NativeEditCompanion {
    pub fn watch_stdin(socket: &Path) -> io::Result<Self> {
        let state = Arc::new(Mutex::new(State::default()));
        let monitor_state = Arc::clone(&state);
        let socket = socket.to_owned();
        std::thread::Builder::new()
            .name("native-edit-owner-lifetime".to_owned())
            .spawn(move || {
                let mut input = io::stdin().lock();
                let mut bytes = [0; 64];
                loop {
                    match input.read(&mut bytes) {
                        Ok(0) => break,
                        Ok(_) => continue,
                        Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                        Err(_) => break,
                    }
                }
                let active = {
                    let mut state = monitor_state.lock().unwrap_or_else(PoisonError::into_inner);
                    state.stopped = true;
                    state.active.take()
                };
                if let Some(stream) = active {
                    // Unblock an idle/partial read without interrupting a dispatch.
                    let _ = stream.shutdown(Shutdown::Both);
                }
                // Wake an idle accept. Never hold the state lock while connecting.
                let _ = UnixStream::connect(socket);
            })?;
        Ok(Self { state })
    }

    fn state(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    pub fn stopped(&self) -> bool {
        self.state().stopped
    }

    pub fn register_connection(&self, stream: &UnixStream) -> io::Result<bool> {
        let stream = stream.try_clone()?;
        let mut state = self.state();
        // Check and registration are atomic with EOF, including the accept race.
        if state.stopped {
            return Ok(false);
        }
        state.active = Some(stream);
        Ok(true)
    }

    pub fn clear_connection(&self) {
        self.state().active = None;
    }
}
