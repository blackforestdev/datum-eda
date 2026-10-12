//! Opt-in constant frame-preparation checkpoints, independent of stderr records.
use std::io::Write;
use std::path::Path;

pub(crate) fn phase(stage: &'static str) {
    if std::env::var_os("DATUM_GUI_VERBOSE_LOG").is_none() {
        return;
    }
    let path = std::env::var_os("DATUM_GUI_LOG")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::env::temp_dir().join("datum-gui-last.log"));
    append(&path, stage);
}

fn append(path: &Path, stage: &'static str) {
    let line = format!(
        "[{:?}] renderer preparation phase {stage}\n",
        std::time::SystemTime::now()
    );
    let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
    else {
        return;
    };
    // One small append request; other app loggers have independent coordination.
    // Diagnostics remain nonfatal and never wait for the stderr record lock.
    let _ = file.write_all(line.as_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preparation_checkpoints_append_in_order_and_ignore_unavailable_sink() {
        let directory = std::env::temp_dir().join(format!(
            "datum-render-preparation-phases-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&directory).unwrap();
        let path = directory.join("phases.log");
        std::fs::write(&path, "existing diagnostic\n").unwrap();
        append(&path, "inspector begin");
        append(&path, "inspector end");
        let output = std::fs::read_to_string(&path).unwrap();
        let lines: Vec<_> = output.lines().collect();
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "existing diagnostic");
        for (line, stage) in lines[1..].iter().zip(["inspector begin", "inspector end"]) {
            assert!(line.starts_with("[SystemTime { tv_sec: "));
            assert!(line.contains("tv_nsec: "));
            assert!(line.ends_with(&format!("] renderer preparation phase {stage}")));
        }
        append(&directory.join("absent-parent/phases.log"), "install begin");
        assert!(!directory.join("absent-parent").exists());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), output);
        std::fs::remove_dir_all(directory).unwrap();
    }
}
