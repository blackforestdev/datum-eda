//! Bounded reads of regular project-owned files. No resource-policy defaults.
use crate::error::EngineError;
use std::{
    io::Read,
    path::{Path, PathBuf},
};

fn invalid(reason: &str) -> EngineError {
    EngineError::Validation(format!("source input: {reason}"))
}

/// This is the existing library relative-path/symlink contract, moved to its
/// shared source owner. Missing destinations are allowed for staging callers;
/// reading them still returns the ordinary IO error.
pub(crate) fn owned_path(root: &Path, relative: &str) -> Result<PathBuf, EngineError> {
    if relative.starts_with('/')
        || relative.contains('\\')
        || relative
            .split('/')
            .any(|p| p.is_empty() || matches!(p, "." | ".."))
    {
        return Err(invalid("unsafe relative source path"));
    }
    let mut path = root.to_path_buf();
    let parts: Vec<_> = relative.split('/').collect();
    for (index, part) in parts.iter().enumerate() {
        path.push(part);
        match std::fs::symlink_metadata(&path) {
            Ok(meta)
                if meta.file_type().is_symlink()
                    || index + 1 < parts.len() && !meta.is_dir()
                    || index + 1 == parts.len() && !meta.is_file() =>
            {
                return Err(invalid("source path has a link or unsupported file type"));
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
    }
    Ok(path)
}

pub(in crate::substrate) fn read_regular_file(
    root: &Path,
    relative: &str,
    limit: usize,
) -> Result<Vec<u8>, EngineError> {
    let path = owned_path(root, relative)?;
    let mut file = std::fs::File::open(path)?;
    let metadata = file.metadata()?;
    if !metadata.is_file() {
        return Err(invalid("source descriptor is not a regular file"));
    }
    if metadata.len() > limit as u64 {
        return Err(invalid("source exceeds explicit read limit"));
    }
    let mut bytes = Vec::new();
    let mut chunk = [0u8; 16 * 1024];
    loop {
        let n = file.read(&mut chunk)?;
        if n == 0 {
            break;
        }
        if bytes.len().checked_add(n).is_none_or(|end| end > limit) {
            return Err(invalid("source grew beyond explicit read limit"));
        }
        bytes
            .try_reserve_exact(n)
            .map_err(|_| invalid("source buffer allocation refused"))?;
        if bytes.capacity() > limit {
            return Err(invalid("source capacity exceeds explicit read limit"));
        }
        bytes.extend_from_slice(&chunk[..n]);
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bounded_read_preserves_bytes_and_refuses_oversized_source() {
        let root = std::env::temp_dir().join(format!("datum_source_io_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&root).unwrap();
        let original = b"{\"unknown\":0.10000000000000001}\n";
        std::fs::write(root.join("source.json"), original).unwrap();
        assert_eq!(
            read_regular_file(&root, "source.json", original.len()).unwrap(),
            original
        );
        assert!(read_regular_file(&root, "source.json", original.len() - 1).is_err());
        assert_eq!(std::fs::read(root.join("source.json")).unwrap(), original);
        for path in [
            "../source.json",
            "./source.json",
            "a//source.json",
            "/source.json",
            "a\\source.json",
        ] {
            assert!(owned_path(&root, path).is_err(), "{path}");
        }
        std::fs::remove_dir_all(root).unwrap();
    }
    #[cfg(unix)]
    #[test]
    fn reader_refuses_symlinked_leaf_and_ancestor() {
        let root =
            std::env::temp_dir().join(format!("datum_source_links_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(root.join("owned")).unwrap();
        std::fs::write(root.join("owned/source.json"), b"{}").unwrap();
        std::os::unix::fs::symlink(root.join("owned/source.json"), root.join("leaf.json")).unwrap();
        std::os::unix::fs::symlink(root.join("owned"), root.join("alias")).unwrap();
        assert!(read_regular_file(&root, "leaf.json", 2).is_err());
        assert!(read_regular_file(&root, "alias/source.json", 2).is_err());
        assert_eq!(
            read_regular_file(&root, "owned/source.json", 2).unwrap(),
            b"{}"
        );
        std::fs::remove_dir_all(root).unwrap();
    }
}
