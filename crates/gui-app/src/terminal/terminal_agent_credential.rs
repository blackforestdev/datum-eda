//! Session-lifetime credential and revocation state for Datum agent brokers.

use crate::terminal_context_io::atomic_write_text;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};

const CREDENTIAL_SCHEMA: &str = "datum_agent_credential_v1";
const AUTHORITY_SCHEMA: &str = "datum_agent_authority_v1";

#[derive(Debug, Serialize)]
struct CredentialFile<'a> {
    schema: &'static str,
    credential_id: &'a str,
    terminal_session_id: &'a str,
    project_root: &'a str,
    secret: &'a str,
}

#[derive(Debug, Serialize, Deserialize)]
struct AuthorityDescriptor {
    schema: String,
    credential_id: String,
    terminal_session_id: String,
    project_root: String,
    state: String,
    issued_unix_ms: u128,
    revoked_unix_ms: Option<u128>,
}

pub(super) fn credential_path(context_path: &Path, session_id: &str) -> PathBuf {
    context_path.with_file_name(format!(".{session_id}.agent-credential.json"))
}

pub(super) fn authority_descriptor_path(context_path: &Path, session_id: &str) -> PathBuf {
    context_path.with_file_name(format!("{session_id}.agent-authority.json"))
}

pub(super) fn agent_launch_id(context_id: &str) -> String {
    format!("agent-launch-{context_id}")
}

pub(super) fn create_session_authority(
    context_path: &Path,
    session_id: &str,
    project_root: &Path,
    issued_unix_ms: u128,
) -> Result<()> {
    let parent = context_path
        .parent()
        .context("terminal context path has no parent")?;
    fs::set_permissions(parent, fs::Permissions::from_mode(0o700))
        .with_context(|| format!("protect terminal context dir {}", parent.display()))?;

    let credential_id = random_hex(16)?;
    let secret = random_hex(32)?;
    let project_root = project_root.display().to_string();
    let credential = CredentialFile {
        schema: CREDENTIAL_SCHEMA,
        credential_id: &credential_id,
        terminal_session_id: session_id,
        project_root: &project_root,
        secret: &secret,
    };
    let credential_text = format!(
        "{}\n",
        serde_json::to_string(&credential).context("serialize agent credential")?
    );
    let credential_path = credential_path(context_path, session_id);
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&credential_path)
        .with_context(|| format!("create agent credential {}", credential_path.display()))?;
    file.write_all(credential_text.as_bytes())
        .with_context(|| format!("write agent credential {}", credential_path.display()))?;
    file.sync_all()
        .with_context(|| format!("sync agent credential {}", credential_path.display()))?;

    let descriptor = AuthorityDescriptor {
        schema: AUTHORITY_SCHEMA.to_string(),
        credential_id,
        terminal_session_id: session_id.to_string(),
        project_root,
        state: "active".to_string(),
        issued_unix_ms,
        revoked_unix_ms: None,
    };
    write_descriptor(context_path, session_id, &descriptor)
}

pub(super) fn revoke_session_authority(
    context_path: &Path,
    session_id: &str,
    revoked_unix_ms: u128,
) -> Result<()> {
    let descriptor_path = authority_descriptor_path(context_path, session_id);
    let Ok(text) = fs::read_to_string(&descriptor_path) else {
        let _ = fs::remove_file(credential_path(context_path, session_id));
        return Ok(());
    };
    let mut descriptor: AuthorityDescriptor = serde_json::from_str(&text)
        .with_context(|| format!("parse agent authority {}", descriptor_path.display()))?;
    if descriptor.state != "revoked" {
        descriptor.state = "revoked".to_string();
        descriptor.revoked_unix_ms = Some(revoked_unix_ms);
        write_descriptor(context_path, session_id, &descriptor)?;
    }
    match fs::remove_file(credential_path(context_path, session_id)) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error).context("remove revoked agent credential"),
    }
}

fn write_descriptor(
    context_path: &Path,
    session_id: &str,
    descriptor: &AuthorityDescriptor,
) -> Result<()> {
    let path = authority_descriptor_path(context_path, session_id);
    let text = format!(
        "{}\n",
        serde_json::to_string_pretty(descriptor).context("serialize agent authority")?
    );
    atomic_write_text(&path, &text)
        .with_context(|| format!("publish agent authority {}", path.display()))?;
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600))
        .with_context(|| format!("protect agent authority {}", path.display()))
}

fn random_hex(byte_count: usize) -> Result<String> {
    let mut bytes = vec![0_u8; byte_count];
    File::open("/dev/urandom")
        .context("open Linux random source for agent credential")?
        .read_exact(&mut bytes)
        .context("read Linux random source for agent credential")?;
    let mut encoded = String::with_capacity(byte_count * 2);
    for byte in bytes {
        use std::fmt::Write as _;
        write!(&mut encoded, "{byte:02x}").expect("write to String cannot fail");
    }
    Ok(encoded)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn revocation_removes_secret_and_publishes_only_inert_authority() {
        let root = std::env::temp_dir().join(format!(
            "datum-agent-credential-{}-{}",
            std::process::id(),
            std::thread::current().name().unwrap_or("test")
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("credential test root should create");
        let context_path = root.join("session.json");
        create_session_authority(&context_path, "terminal-7", &root, 10)
            .expect("session authority should create");
        let credential = credential_path(&context_path, "terminal-7");
        let descriptor = authority_descriptor_path(&context_path, "terminal-7");
        assert_eq!(
            fs::metadata(&credential)
                .expect("credential metadata should read")
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
        let secret_document = fs::read_to_string(&credential).expect("credential should read");
        assert!(secret_document.contains("\"secret\""));
        assert!(
            !fs::read_to_string(&descriptor)
                .expect("descriptor should read")
                .contains("\"secret\"")
        );

        revoke_session_authority(&context_path, "terminal-7", 20)
            .expect("session authority should revoke");
        assert!(!credential.exists());
        let revoked: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(&descriptor).expect("revoked descriptor should read"),
        )
        .expect("revoked descriptor should parse");
        assert_eq!(revoked["state"], "revoked");
        assert_eq!(revoked["revoked_unix_ms"], 20);
        let _ = fs::remove_dir_all(&root);
    }
}
