use anyhow::{Context, Result};
use std::fs;
use std::io::Write;
use std::path::Path;

pub(super) fn atomic_write_text(path: &Path, text: &str) -> Result<()> {
    let parent = path.parent().context("atomic write path has no parent")?;
    fs::create_dir_all(parent)?;
    let filename = path
        .file_name()
        .and_then(|name| name.to_str())
        .context("atomic write path has no file name")?;
    let temp_path = parent.join(format!(
        ".{filename}.tmp-{}-{}",
        std::process::id(),
        unix_time_ms()?
    ));
    {
        let mut file = fs::File::create(&temp_path)?;
        file.write_all(text.as_bytes())?;
        file.sync_all()?;
    }
    if let Err(err) = fs::rename(&temp_path, path) {
        let _ = fs::remove_file(&temp_path);
        return Err(err).with_context(|| {
            format!(
                "rename atomic temp {} to {}",
                temp_path.display(),
                path.display()
            )
        });
    }
    Ok(())
}

fn unix_time_ms() -> Result<u128> {
    Ok(std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .context("terminal context timestamp")?
        .as_millis())
}
