//! External fixture placement and informational ownership, never an execution grant.
use anyhow::{Context, Result, bail, ensure};
use std::io::Write;
use std::path::{Component, Path, PathBuf};

pub(super) fn managed_root(family: &str) -> Result<PathBuf> {
    let owner = std::env::var("DATUM_FIXTURE_OWNER")
        .context("managed fixtures require DATUM_FIXTURE_OWNER (authorized task/session label)")?;
    let base = if let Some(path) = std::env::var_os("DATUM_FIXTURE_WORKSPACE_ROOT") {
        PathBuf::from(path)
    } else if let Some(path) = std::env::var_os("XDG_STATE_HOME") {
        PathBuf::from(path).join("datum-eda/fixtures")
    } else {
        PathBuf::from(std::env::var_os("HOME").context("HOME or external workspace root required")?)
            .join(".local/state/datum-eda/fixtures")
    };
    let checkout = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()?;
    allocate(&base, family, &owner, &checkout)
}

pub(super) fn validate_output(root: &Path) -> Result<()> {
    let checkout = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()?;
    let absolute = if root.is_absolute() {
        root.to_path_buf()
    } else {
        std::env::current_dir()?.join(root)
    };
    external_path(&absolute, &checkout.join("target"))
        .context("Cargo target is disposable; fixture output must be elsewhere")
}

fn label(value: &str) -> Result<()> {
    ensure!(
        !value.is_empty()
            && value.len() <= 128
            && value
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_'),
        "workspace owner/family must be a nonempty alphanumeric, hyphen or underscore label"
    );
    Ok(())
}

fn external_path(path: &Path, checkout: &Path) -> Result<()> {
    ensure!(
        path.is_absolute(),
        "fixture workspace must be absolute and outside the checkout"
    );
    ensure!(
        !path
            .components()
            .any(|part| matches!(part, Component::ParentDir)),
        "fixture workspace cannot contain parent traversal"
    );
    // Resolve the existing ancestor so a symlink cannot disguise checkout output.
    let mut ancestor = path;
    let mut tail = Vec::new();
    while !ancestor.exists() {
        tail.push(
            ancestor
                .file_name()
                .context("workspace has no existing ancestor")?,
        );
        ancestor = ancestor
            .parent()
            .context("workspace has no existing ancestor")?;
    }
    let mut physical = ancestor.canonicalize()?;
    for component in tail.into_iter().rev() {
        physical.push(component);
    }
    ensure!(
        !physical.starts_with(checkout),
        "managed fixture output must stay outside the checkout"
    );
    Ok(())
}

fn allocate(base: &Path, family: &str, owner: &str, checkout: &Path) -> Result<PathBuf> {
    label(owner)?;
    label(family)?;
    let root = base.join(owner).join(family);
    let root_text = root
        .to_str()
        .context("managed fixture path must be UTF-8 for its ownership record")?;
    external_path(&root, checkout)?;
    if root.exists() || root.is_symlink() {
        bail!(
            "managed fixture already exists; preserve or retire released output before repeating"
        );
    }
    let parent = root.parent().context("workspace parent required")?;
    std::fs::create_dir_all(parent)?;
    external_path(&root, checkout)?;
    let receipt = parent.join(format!("{family}.owner.json"));
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&receipt)
        .context(
            "workspace reservation exists; preserve or retire released output before repeating",
        )?;
    let record = serde_json::json!({
        "schema_version": 1, "owner": owner, "family": family, "root": root_text,
        "purpose": "fixture generation attempt; informational ownership only",
        "authority": "not an execution grant, lease or acceptance receipt",
        "retention": "preserve failed and successful native data until owner release; no automatic deletion",
        "acceptance": "not established; native proof/review remain separate"
    });
    file.write_all(serde_json::to_string_pretty(&record)?.as_bytes())?;
    file.write_all(b"\n")?;
    // The recipe owns creation of the fresh project root; never pre-create it.
    Ok(root)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temporary() -> PathBuf {
        let path =
            std::env::temp_dir().join(format!("datum-fixture-placement-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&path).unwrap();
        path
    }

    #[test]
    fn explicit_output_cannot_repopulate_disposable_cargo_target() {
        let checkout = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        assert!(validate_output(&checkout.join("target/new-native-fixture")).is_err());
        let tmp = temporary();
        assert!(validate_output(&tmp.join("fresh-explicit-project")).is_ok());
        assert!(!tmp.join("fresh-explicit-project").exists());
        std::fs::remove_dir_all(tmp).unwrap();
    }

    #[test]
    fn external_reservation_is_fresh_owned_and_cannot_proliferate() {
        let tmp = temporary();
        let checkout = tmp.join("checkout");
        std::fs::create_dir(&checkout).unwrap();
        let root = allocate(
            &tmp.join("workspaces"),
            "electrical-scope",
            "task-1",
            &checkout,
        )
        .unwrap();
        assert!(!root.exists());
        let receipt = root.parent().unwrap().join("electrical-scope.owner.json");
        let bytes = std::fs::read(&receipt).unwrap();
        let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(value["owner"], "task-1");
        assert_eq!(value["root"], root.to_string_lossy().as_ref());
        assert!(
            allocate(
                &tmp.join("workspaces"),
                "electrical-scope",
                "task-1",
                &checkout
            )
            .is_err()
        );
        assert_eq!(std::fs::read(&receipt).unwrap(), bytes);
        super::super::run("electrical-scope", &root).unwrap();
        let native_before = std::fs::read(root.join("project.json")).unwrap();
        let report: serde_json::Value =
            serde_json::from_slice(&std::fs::read(root.join("s5a-fixture.json")).unwrap()).unwrap();
        assert!(
            report["acceptance"]
                .as_str()
                .unwrap()
                .contains("not established")
        );
        assert!(
            allocate(
                &tmp.join("workspaces"),
                "electrical-scope",
                "task-1",
                &checkout
            )
            .is_err()
        );
        assert_eq!(
            std::fs::read(root.join("project.json")).unwrap(),
            native_before
        );
        std::fs::remove_dir_all(tmp).unwrap();
    }

    #[test]
    fn relative_traversal_and_checkout_locations_refuse_before_writing() {
        let tmp = temporary();
        let checkout = tmp.join("checkout");
        std::fs::create_dir(&checkout).unwrap();
        for base in [
            PathBuf::from("relative"),
            checkout.clone(),
            tmp.join("outside/../checkout"),
        ] {
            assert!(allocate(&base, "electrical-scope", "task-1", &checkout).is_err());
        }
        assert!(!checkout.join("task-1").exists());
        assert!(allocate(&tmp, "electrical-scope", "../escape", &checkout).is_err());
        use std::os::unix::ffi::OsStringExt;
        let invalid = tmp.join(std::ffi::OsString::from_vec(vec![b'w', 0xff]));
        assert!(allocate(&invalid, "electrical-scope", "task-1", &checkout).is_err());
        assert!(!invalid.exists());
        std::fs::remove_dir_all(tmp).unwrap();
    }

    #[test]
    fn symlinked_checkout_ancestor_cannot_redirect_managed_output() {
        let tmp = temporary();
        let checkout = tmp.join("checkout");
        std::fs::create_dir(&checkout).unwrap();
        let alias = tmp.join("alias");
        std::os::unix::fs::symlink(&checkout, &alias).unwrap();
        assert!(
            allocate(
                &alias.join("missing"),
                "electrical-scope",
                "task-1",
                &checkout
            )
            .is_err()
        );
        assert!(!checkout.join("missing").exists());
        std::fs::remove_dir_all(tmp).unwrap();
    }
}
