//! Real daemon transport/lifetime regression. The test retains the daemon Child
//! for reliable reaping; a killed stand-in holds its lifetime pipe. Actual GUI
//! ancestry is covered by the GUI proof; normal NativeProjectClient::Drop retains
//! its existing focused evidence and unchanged kill/wait path.

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use uuid::Uuid;

const LIFETIME_ENV: &str = "DATUM_NATIVE_EDIT_LIFETIME_STDIN";
const DEADLINE: Duration = Duration::from_secs(5);

struct Daemon {
    child: Child,
    root: PathBuf,
    socket: PathBuf,
}

impl Daemon {
    fn spawn(owned: bool, piped: bool) -> Self {
        let root = std::env::temp_dir().join(format!("datum-companion-{}", Uuid::new_v4()));
        std::fs::create_dir(&root).unwrap();
        let socket = root.join("engine.sock");
        let child_socket = socket.clone();
        let config = root.join("config");
        // The real GUI starts companions on temporary workers. Joining this
        // worker must not terminate the daemon while its process still owns it.
        let child = std::thread::spawn(move || {
            let mut command = Command::new(env!("CARGO_BIN_EXE_eda-engine-daemon"));
            command
                .arg("--socket")
                .arg(child_socket)
                .env("XDG_CONFIG_HOME", config)
                .env_remove("DATUM_GUI_PREFERENCES_PATH")
                .env_remove(LIFETIME_ENV)
                .stdin(if piped { Stdio::piped() } else { Stdio::null() })
                .stdout(Stdio::null())
                .stderr(Stdio::inherit());
            if owned {
                command.env(LIFETIME_ENV, "1");
            }
            command.spawn().unwrap()
        })
        .join()
        .unwrap();
        Self {
            child,
            root,
            socket,
        }
    }

    fn connect(&mut self) -> UnixStream {
        let until = Instant::now() + DEADLINE;
        loop {
            if let Ok(stream) = UnixStream::connect(&self.socket) {
                stream.set_read_timeout(Some(DEADLINE)).unwrap();
                stream.set_write_timeout(Some(DEADLINE)).unwrap();
                return stream;
            }
            assert!(
                self.child.try_wait().unwrap().is_none(),
                "daemon exited before ready"
            );
            assert!(Instant::now() < until, "daemon did not bind its socket");
            std::thread::sleep(Duration::from_millis(5));
        }
    }

    fn exited(&mut self) -> ExitStatus {
        let until = Instant::now() + DEADLINE;
        loop {
            if let Some(status) = self.child.try_wait().unwrap() {
                return status;
            }
            assert!(
                Instant::now() < until,
                "owned daemon survived lifetime-pipe closure"
            );
            std::thread::sleep(Duration::from_millis(5));
        }
    }
}

impl Drop for Daemon {
    fn drop(&mut self) {
        // Only the exact task-owned Child is signalled, including assertion failures.
        let _ = self.child.kill();
        let _ = self.child.wait();
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

struct PipeOwner(Child);

impl PipeOwner {
    fn take_from(daemon: &mut Daemon) -> Self {
        let writer = daemon.child.stdin.take().unwrap();
        // This stand-in merely retains the write descriptor as stdin; it never
        // reads or writes it. The daemon remains our Child so no orphan cleanup
        // relies on PID lookups, external signals or the system child reaper.
        Self(
            Command::new("sleep")
                .arg("60")
                .stdin(Stdio::from(writer))
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .unwrap(),
        )
    }

    fn kill(&mut self) {
        self.0.kill().unwrap();
        assert!(!self.0.wait().unwrap().success());
    }
}

impl Drop for PipeOwner {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn query(stream: &mut UnixStream) {
    let request = json!({
        "jsonrpc":"2.0", "id":"lifetime-probe", "method":"close_project", "params":{}
    });
    serde_json::to_writer(&mut *stream, &request).unwrap();
    stream.write_all(b"\n").unwrap();
    let mut line = String::new();
    BufReader::new(&mut *stream).read_line(&mut line).unwrap();
    let response: Value = serde_json::from_str(&line).unwrap();
    assert_eq!(response["id"], request["id"]);
    assert_eq!(response["result"], json!({"closed":true}));
    assert!(response["error"].is_null());
}

#[test]
fn killed_pipe_owner_stops_accept_idle_and_partial_read_after_worker_exits() {
    for state in ["accept", "idle", "partial"] {
        let mut daemon = Daemon::spawn(true, true);
        let mut owner = PipeOwner::take_from(&mut daemon);
        let mut stream = daemon.connect();
        query(&mut stream);
        assert!(
            daemon.child.try_wait().unwrap().is_none(),
            "spawning-thread exit killed daemon"
        );
        let held = match state {
            "accept" => {
                drop(stream);
                None
            }
            "idle" => Some(stream),
            "partial" => {
                stream.write_all(b"{\"jsonrpc\":\"2.0\"").unwrap();
                Some(stream)
            }
            _ => unreachable!(),
        };
        owner.kill();
        assert!(
            daemon.exited().success(),
            "shutdown failed in {state} state"
        );
        // Keep the accepted client alive until AFTER exit: an idle/partial peer
        // must not keep a companion running after its owning process disappears.
        drop(held);
    }
}

#[test]
fn owned_companion_exits_when_stdin_is_already_closed_at_startup() {
    let mut daemon = Daemon::spawn(true, false);
    assert!(daemon.exited().success());
}

#[test]
fn standalone_daemon_remains_usable_with_closed_stdin() {
    let mut daemon = Daemon::spawn(false, false);
    let mut stream = daemon.connect();
    query(&mut stream);
    query(&mut stream);
    assert!(daemon.child.try_wait().unwrap().is_none());
}
