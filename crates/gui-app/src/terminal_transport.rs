//! Process transport boundary for Datum's embedded terminal.
//!
//! `portable-pty` exclusively owns PTY allocation, child spawn, master I/O and
//! resize. Terminal parsing, cells, selection, chrome and Datum context
//! projections must never enter this module.

use anyhow::{Context, Result};
use portable_pty::{Child, CommandBuilder, MasterPty, NativePtySystem, PtySize, PtySystem};
use std::{
    collections::BTreeMap,
    ffi::{OsStr, OsString},
    io::{Read, Write},
    path::{Path, PathBuf},
};

pub(crate) const INITIAL_TERMINAL_SIZE: PtySize = PtySize {
    rows: 24,
    cols: 80,
    pixel_width: 0,
    pixel_height: 0,
};

/// Complete process-launch request at the transport boundary. The caller owns
/// Datum-specific context construction; this type guarantees that cwd,
/// inherited credentials/environment overrides and arbitrary argv all reach
/// the same portable spawn path.
#[derive(Debug, Clone)]
pub(super) struct TerminalTransportLaunch {
    program: OsString,
    args: Vec<OsString>,
    cwd: PathBuf,
    env: BTreeMap<OsString, OsString>,
    size: PtySize,
}

impl TerminalTransportLaunch {
    pub(super) fn new(program: impl AsRef<OsStr>, cwd: impl AsRef<Path>) -> Self {
        Self {
            program: program.as_ref().to_owned(),
            args: Vec::new(),
            cwd: cwd.as_ref().to_owned(),
            env: BTreeMap::new(),
            size: INITIAL_TERMINAL_SIZE,
        }
    }

    pub(super) fn args<I, S>(&mut self, args: I)
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.args
            .extend(args.into_iter().map(|arg| arg.as_ref().to_owned()));
    }

    pub(super) fn env(&mut self, key: impl AsRef<OsStr>, value: impl AsRef<OsStr>) {
        self.env
            .insert(key.as_ref().to_owned(), value.as_ref().to_owned());
    }

    #[cfg(test)]
    pub(super) fn set_size(&mut self, size: PtySize) {
        self.size = size;
    }

    fn command(&self) -> CommandBuilder {
        let mut command = CommandBuilder::new(&self.program);
        command.args(&self.args);
        command.cwd(&self.cwd);
        for (key, value) in &self.env {
            command.env(key, value);
        }
        command
    }
}

pub(super) struct PortablePtyProcess {
    pub(super) master: Box<dyn MasterPty + Send>,
    pub(super) reader: Box<dyn Read + Send>,
    pub(super) writer: Box<dyn Write + Send>,
    pub(super) child: Box<dyn Child + Send + Sync>,
}

pub(super) fn spawn_portable_pty(launch: &TerminalTransportLaunch) -> Result<PortablePtyProcess> {
    let pair = NativePtySystem::default()
        .openpty(launch.size)
        .context("allocate portable terminal PTY")?;
    let child = pair
        .slave
        .spawn_command(launch.command())
        .with_context(|| format!("spawn PTY child {:?}", launch.program))?;
    drop(pair.slave);
    let reader = pair
        .master
        .try_clone_reader()
        .context("clone portable PTY master reader")?;
    let writer = pair
        .master
        .take_writer()
        .context("take portable PTY master writer")?;
    Ok(PortablePtyProcess {
        master: pair.master,
        reader,
        writer,
        child,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn launch_maps_arbitrary_argv_cwd_environment_and_size() {
        let mut launch = TerminalTransportLaunch::new("/bin/printf", "/tmp");
        launch.args(["%s", "portable"]);
        launch.env("DATUM_TRANSPORT_PROBE", "mapped");
        launch.set_size(PtySize {
            rows: 31,
            cols: 97,
            pixel_width: 970,
            pixel_height: 620,
        });
        let command = launch.command();
        assert_eq!(
            command.get_argv(),
            &["/bin/printf", "%s", "portable"]
                .into_iter()
                .map(OsString::from)
                .collect::<Vec<_>>()
        );
        assert_eq!(command.get_cwd(), Some(&OsString::from("/tmp")));
        assert_eq!(
            command.get_env("DATUM_TRANSPORT_PROBE"),
            Some(OsStr::new("mapped"))
        );
        assert!(command.get_controlling_tty());
        assert_eq!(launch.size.rows, 31);
        assert_eq!(launch.size.cols, 97);
    }
}
