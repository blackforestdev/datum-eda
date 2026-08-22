//! Persistent user preferences owned by the output-only Datum Console.
//!
//! This state is machine-local UI configuration: never project state, never a
//! journal input, and never an alternate design mutation path.

use anyhow::{Context, Result, bail};
use datum_gui_protocol::ConsoleFeedbackDuration;
use std::path::{Path, PathBuf};

const PREFERENCE_ENV: &str = "DATUM_GUI_PREFERENCES_PATH";

pub(super) fn load_duration_preference() -> Option<ConsoleFeedbackDuration> {
    let path = preference_path()?;
    load_duration_at(&path).ok().flatten()
}

pub(super) fn persist_duration_preference(duration: ConsoleFeedbackDuration) -> Result<()> {
    let path = preference_path().context("user configuration directory is unavailable")?;
    persist_duration_at(&path, duration)
}

fn preference_path() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os(PREFERENCE_ENV) {
        return Some(PathBuf::from(path));
    }
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))?;
    Some(base.join("datum").join("gui-preferences.json"))
}

fn load_duration_at(path: &Path) -> Result<Option<ConsoleFeedbackDuration>> {
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error).with_context(|| format!("read {}", path.display())),
    };
    let value: serde_json::Value =
        serde_json::from_str(&text).with_context(|| format!("parse {}", path.display()))?;
    let duration = value
        .get("console_duration")
        .and_then(serde_json::Value::as_str)
        .context("console_duration is missing or is not a string")?;
    Ok(Some(parse_duration(duration)?))
}

fn persist_duration_at(path: &Path, duration: ConsoleFeedbackDuration) -> Result<()> {
    let parent = path
        .parent()
        .context("GUI preference path has no parent directory")?;
    std::fs::create_dir_all(parent)
        .with_context(|| format!("create preference directory {}", parent.display()))?;
    let text = format!(
        "{{\n  \"schema\": \"datum_gui_preferences_v1\",\n  \"console_duration\": \"{}\"\n}}\n",
        duration_name(duration)
    );
    let temp = path.with_extension(format!("json.tmp-{}", std::process::id()));
    std::fs::write(&temp, text).with_context(|| format!("write {}", temp.display()))?;
    std::fs::rename(&temp, path)
        .with_context(|| format!("replace GUI preference file {}", path.display()))
}

fn duration_name(duration: ConsoleFeedbackDuration) -> &'static str {
    match duration {
        ConsoleFeedbackDuration::FourSeconds => "4s",
        ConsoleFeedbackDuration::SixSeconds => "6s",
        ConsoleFeedbackDuration::TenSeconds => "10s",
        ConsoleFeedbackDuration::Never => "never",
    }
}

fn parse_duration(value: &str) -> Result<ConsoleFeedbackDuration> {
    match value {
        "4s" => Ok(ConsoleFeedbackDuration::FourSeconds),
        "6s" => Ok(ConsoleFeedbackDuration::SixSeconds),
        "10s" => Ok(ConsoleFeedbackDuration::TenSeconds),
        "never" => Ok(ConsoleFeedbackDuration::Never),
        other => bail!("unsupported Console duration {other:?}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "datum-console-preferences-{}-{name}.json",
            std::process::id()
        ))
    }

    #[test]
    fn every_duration_round_trips_through_the_owned_preference_file() {
        let path = temp_path("round-trip");
        let _ = std::fs::remove_file(&path);
        for duration in [
            ConsoleFeedbackDuration::FourSeconds,
            ConsoleFeedbackDuration::SixSeconds,
            ConsoleFeedbackDuration::TenSeconds,
            ConsoleFeedbackDuration::Never,
        ] {
            persist_duration_at(&path, duration).unwrap();
            assert_eq!(load_duration_at(&path).unwrap(), Some(duration));
        }
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn absent_file_defaults_and_unknown_values_fail_closed() {
        let path = temp_path("invalid");
        let _ = std::fs::remove_file(&path);
        assert_eq!(load_duration_at(&path).unwrap(), None);
        std::fs::write(&path, r#"{"console_duration":"forever"}"#).unwrap();
        assert!(load_duration_at(&path).is_err());
        std::fs::remove_file(path).unwrap();
    }
}
