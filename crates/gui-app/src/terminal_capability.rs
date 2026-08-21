//! Truthful shell capability identity for Datum's native terminal.
//!
//! The database bytes are generated from the adjacent Datum-authored source
//! during development and embedded in the application. Runtime installation is
//! session-local, bounded, atomic, and does not invoke `tic` or depend on host
//! terminal packages.

use anyhow::{Context, Result};
use std::{fs, path::Path};

pub(crate) const DATUM_TERM: &str = "datum-256color";
pub(crate) const DATUM_TERM_PROGRAM: &str = "Datum";

const COMPILED_ENTRY: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/terminfo/compiled/d/datum-256color"
));

/// Materialize the embedded terminfo entry beside, never at, the durable JSON
/// session record and return its database root.
pub(crate) fn install_session_terminfo(session_path: &Path) -> Result<std::path::PathBuf> {
    let runtime = session_path.with_extension("terminal-runtime");
    let root = runtime.join("terminfo");
    let family = root.join("d");
    fs::create_dir_all(&family)
        .with_context(|| format!("create Datum terminfo directory {}", family.display()))?;
    let destination = family.join(DATUM_TERM);
    let temporary = family.join(format!(".{DATUM_TERM}.tmp"));
    fs::write(&temporary, COMPILED_ENTRY)
        .with_context(|| format!("write Datum terminfo entry {}", temporary.display()))?;
    fs::rename(&temporary, &destination).with_context(|| {
        format!(
            "install Datum terminfo entry {} -> {}",
            temporary.display(),
            destination.display()
        )
    })?;
    Ok(root)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn embedded_entry_has_a_bounded_compiled_terminfo_header() {
        assert!(COMPILED_ENTRY.len() < 64 * 1024);
        assert!(matches!(
            COMPILED_ENTRY.get(..2),
            Some(magic) if magic == [0x1a, 0x01] || magic == [0x1e, 0x02]
        ));
        let names_size = u16::from_le_bytes([COMPILED_ENTRY[2], COMPILED_ENTRY[3]]) as usize;
        let names = COMPILED_ENTRY
            .get(12..12 + names_size)
            .expect("compiled entry has its declared names table");
        assert!(names.starts_with(b"datum-256color|Datum EDA terminal with 256 colors\0"));
    }

    #[test]
    fn session_install_materializes_only_the_embedded_datum_entry() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock follows the epoch")
            .as_nanos();
        let fixture =
            std::env::temp_dir().join(format!("datum-terminfo-{}-{nonce}", std::process::id()));
        let session = fixture.join("session.json");
        let root = install_session_terminfo(&session).expect("install embedded terminfo");
        assert_eq!(root, fixture.join("session.terminal-runtime/terminfo"));
        assert!(
            !session.is_dir(),
            "the durable JSON session path must never become a terminfo directory"
        );
        let installed = root.join("d").join(DATUM_TERM);
        assert_eq!(
            fs::read(installed).expect("read installed entry"),
            COMPILED_ENTRY
        );
        assert_eq!(
            fs::read_dir(root.join("d"))
                .expect("read terminfo family")
                .count(),
            1
        );
        fs::remove_dir_all(fixture).expect("remove test session tree");
    }
}
