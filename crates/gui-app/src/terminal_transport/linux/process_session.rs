//! Bounded, fail-closed discovery of one Datum-owned Linux terminal session.

use crate::terminal_transport::limits::{MAX_SESSION_GROUPS, MAX_SESSION_MEMBERS};
use std::{collections::BTreeMap, fmt, fs, io};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::terminal_transport) struct ProcessIdentity {
    pub pid: libc::pid_t,
    pub process_group_id: libc::pid_t,
    pub session_id: libc::pid_t,
    pub start_time: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::terminal_transport) struct OwnedSessionSnapshot {
    pub members: Vec<ProcessIdentity>,
    pub groups: Vec<(libc::pid_t, ProcessIdentity)>,
}

#[derive(Debug)]
pub(in crate::terminal_transport) enum DiscoveryError {
    Io {
        path: String,
        error: io::Error,
        observed: Vec<ProcessIdentity>,
    },
    Malformed {
        path: String,
        observed: Vec<ProcessIdentity>,
    },
    MemberLimit {
        observed: Vec<ProcessIdentity>,
    },
    GroupLimit {
        observed: Vec<ProcessIdentity>,
    },
}

impl fmt::Display for DiscoveryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, error, .. } => write!(formatter, "read {path}: {error}"),
            Self::Malformed { path, .. } => write!(formatter, "malformed process status: {path}"),
            Self::MemberLimit { .. } => write!(
                formatter,
                "owned terminal session exceeds {MAX_SESSION_MEMBERS} members"
            ),
            Self::GroupLimit { .. } => write!(
                formatter,
                "owned terminal session exceeds {MAX_SESSION_GROUPS} process groups"
            ),
        }
    }
}

impl std::error::Error for DiscoveryError {}

impl DiscoveryError {
    pub(in crate::terminal_transport) fn observed_members(&self) -> Vec<ProcessIdentity> {
        match self {
            Self::Io { observed, .. }
            | Self::Malformed { observed, .. }
            | Self::MemberLimit { observed }
            | Self::GroupLimit { observed } => observed.clone(),
        }
    }
}

pub(in crate::terminal_transport) fn discover_owned_session(
    session_id: libc::pid_t,
) -> Result<OwnedSessionSnapshot, DiscoveryError> {
    discover_in(session_id, std::path::Path::new("/proc"))
}

pub(in crate::terminal_transport) fn read_process_identity(
    pid: libc::pid_t,
) -> Result<ProcessIdentity, DiscoveryError> {
    let path = std::path::Path::new("/proc")
        .join(pid.to_string())
        .join("stat");
    let contents = fs::read_to_string(&path).map_err(|error| DiscoveryError::Io {
        path: path.display().to_string(),
        error,
        observed: Vec::new(),
    })?;
    parse_stat(pid, &contents).ok_or_else(|| DiscoveryError::Malformed {
        path: path.display().to_string(),
        observed: Vec::new(),
    })
}

fn discover_in(
    session_id: libc::pid_t,
    proc_root: &std::path::Path,
) -> Result<OwnedSessionSnapshot, DiscoveryError> {
    let entries = fs::read_dir(proc_root).map_err(|error| DiscoveryError::Io {
        path: proc_root.display().to_string(),
        error,
        observed: Vec::new(),
    })?;
    let mut members = Vec::new();
    let mut groups = BTreeMap::new();
    for entry in entries {
        let entry = entry.map_err(|error| DiscoveryError::Io {
            path: proc_root.display().to_string(),
            error,
            observed: members.clone(),
        })?;
        let Some(pid) = entry
            .file_name()
            .to_str()
            .and_then(|name| name.parse().ok())
        else {
            continue;
        };
        let path = entry.path().join("stat");
        let contents = match fs::read_to_string(&path) {
            Ok(contents) => contents,
            Err(error) if matches!(error.kind(), io::ErrorKind::NotFound) => continue,
            Err(error) => {
                return Err(DiscoveryError::Io {
                    path: path.display().to_string(),
                    error,
                    observed: members,
                });
            }
        };
        let identity = parse_stat(pid, &contents).ok_or_else(|| DiscoveryError::Malformed {
            path: path.display().to_string(),
            observed: members.clone(),
        })?;
        if identity.session_id != session_id {
            continue;
        }
        if members.len() == MAX_SESSION_MEMBERS {
            return Err(DiscoveryError::MemberLimit { observed: members });
        }
        members.push(identity);
        // Linux reserves process groups 0 and 1 from Datum's signalable
        // ownership set. Keep the member for diagnostic completeness, but
        // never turn an unsafe group identity into a teardown target.
        if identity.process_group_id > 1 {
            groups.entry(identity.process_group_id).or_insert(identity);
        }
        if groups.len() > MAX_SESSION_GROUPS {
            return Err(DiscoveryError::GroupLimit { observed: members });
        }
    }
    members.sort_by_key(|identity| identity.pid);
    Ok(OwnedSessionSnapshot {
        members,
        groups: groups.into_iter().collect(),
    })
}

fn parse_stat(pid: libc::pid_t, contents: &str) -> Option<ProcessIdentity> {
    let close = contents.rfind(')')?;
    let prefix = &contents[..=close];
    let parsed_pid: libc::pid_t = prefix.split_once(' ')?.0.parse().ok()?;
    if parsed_pid != pid {
        return None;
    }
    let fields = contents[close + 1..]
        .split_ascii_whitespace()
        .collect::<Vec<_>>();
    Some(ProcessIdentity {
        pid,
        process_group_id: fields.get(2)?.parse().ok()?,
        session_id: fields.get(3)?.parse().ok()?,
        start_time: fields.get(19)?.parse().ok()?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stat_parser_handles_spaces_and_parentheses_in_comm() {
        let stat = "42 (agent ) worker) S 1 7 9 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 12345 0";
        let identity = parse_stat(42, stat).unwrap();
        assert_eq!(identity.process_group_id, 7);
        assert_eq!(identity.session_id, 9);
        assert_eq!(identity.start_time, 12345);
    }

    #[test]
    fn current_process_is_discovered_only_in_its_real_session() {
        let pid = std::process::id() as libc::pid_t;
        let sid = unsafe { libc::getsid(pid) };
        let snapshot = discover_owned_session(sid).unwrap();
        assert!(snapshot.members.iter().any(|member| member.pid == pid));
        assert!(snapshot.groups.iter().all(|(group, _)| *group > 1));
    }

    #[test]
    fn four_thousand_ninety_seventh_member_fails_closed_before_signaling() {
        let root = std::env::temp_dir().join(format!(
            "datum-process-session-limit-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        for pid in 2..=(MAX_SESSION_MEMBERS as i32 + 2) {
            let dir = root.join(pid.to_string());
            fs::create_dir(&dir).unwrap();
            fs::write(
                dir.join("stat"),
                format!(
                    "{pid} (member) S 1 99 777 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 {} 0",
                    pid as u64
                ),
            )
            .unwrap();
        }
        assert!(matches!(
            discover_in(777, &root),
            Err(DiscoveryError::MemberLimit { .. })
        ));
        let _ = fs::remove_dir_all(root);
    }
}
