use anyhow::{Context, Result};
use std::fs;
use std::io::Write;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

static ATOMIC_WRITE_NONCE: AtomicU64 = AtomicU64::new(0);

pub(super) fn atomic_write_text(path: &Path, text: &str) -> Result<()> {
    let parent = path.parent().context("atomic write path has no parent")?;
    fs::create_dir_all(parent)?;
    let filename = path
        .file_name()
        .and_then(|name| name.to_str())
        .context("atomic write path has no file name")?;
    let temp_path = parent.join(format!(
        ".{filename}.tmp-{}-{}-{}",
        std::process::id(),
        unix_time_ms()?,
        ATOMIC_WRITE_NONCE.fetch_add(1, Ordering::Relaxed),
    ));
    {
        let mut file = fs::File::create(&temp_path)?;
        file.write_all(text.as_bytes())?;
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

pub(super) fn atomic_write_texts(entries: &[(&Path, &str)]) -> Result<()> {
    std::thread::scope(|scope| {
        let handles = entries
            .iter()
            .map(|&(path, text)| scope.spawn(move || atomic_write_text(path, text)))
            .collect::<Vec<_>>();
        for handle in handles {
            handle
                .join()
                .map_err(|_| anyhow::anyhow!("atomic write worker panicked"))??;
        }
        Ok(())
    })
}

fn unix_time_ms() -> Result<u128> {
    Ok(std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .context("terminal context timestamp")?
        .as_millis())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn concurrent_atomic_context_bundle_uses_unique_temporary_files() {
        let root = std::env::temp_dir().join(format!(
            "datum-context-bundle-{}-{}",
            std::process::id(),
            ATOMIC_WRITE_NONCE.fetch_add(1, Ordering::Relaxed),
        ));
        fs::create_dir_all(&root).unwrap();
        let paths = [
            root.join("context.json"),
            root.join("latest.json"),
            root.join("session.json"),
        ];
        atomic_write_texts(&[
            (&paths[0], "{\"kind\":\"context\"}\n"),
            (&paths[1], "{\"kind\":\"latest\"}\n"),
            (&paths[2], "{\"kind\":\"session\"}\n"),
        ])
        .unwrap();
        assert!(paths.iter().all(|path| path.exists()));
        assert!(fs::read_dir(&root).unwrap().all(|entry| {
            !entry
                .unwrap()
                .file_name()
                .to_string_lossy()
                .contains(".tmp-")
        }));
        let _ = fs::remove_dir_all(root);
    }
}
