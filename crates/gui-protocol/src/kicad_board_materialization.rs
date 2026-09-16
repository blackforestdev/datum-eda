//! GUI-owned automatic import container; native project creation remains engine-owned.
use super::{LiveReviewRequest, cli_prefix, run_cli_json_owned};
use anyhow::{Context, Result};
use serde_json::Value;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::os::unix::fs::DirBuilderExt;
use std::path::{Path, PathBuf};

pub fn materialize_kicad_board_request(
    board_file: &Path,
    project_root: Option<PathBuf>,
) -> Result<LiveReviewRequest> {
    let parent = std::env::temp_dir().join("datum-eda/gui-imports");
    materialize_with_parent(board_file, project_root, &parent, &cli_prefix())
}

fn materialize_with_parent(
    board_file: &Path,
    project_root: Option<PathBuf>,
    automatic_parent: &Path,
    cli: &[String],
) -> Result<LiveReviewRequest> {
    let source = board_file
        .canonicalize()
        .with_context(|| format!("failed to resolve KiCad board {}", board_file.display()))?;
    let root = match project_root {
        Some(root) => root,
        None => {
            // Only the GUI-managed container is initialized here. The destination
            // itself must remain absent for atomic engine-owned project genesis.
            initialize_automatic_parent(automatic_parent)?;
            automatic_parent.join(materialized_project_directory_name(&source))
        }
    };
    let root_display = root.display().to_string();
    let source_display = source.display().to_string();

    if !root.join("project.json").is_file() {
        let project_name = materialized_kicad_board_project_name(&source);
        run_cli_json_owned::<Value>(
            cli,
            &[
                "project".to_string(),
                "new".to_string(),
                root_display.clone(),
                "--name".to_string(),
                project_name,
            ],
        )
        .with_context(|| {
            format!(
                "failed to create native Datum project at {}",
                root.display()
            )
        })?;
    }

    run_cli_json_owned::<Value>(
        cli,
        &[
            "project".to_string(),
            "import-kicad-board".to_string(),
            root_display,
            "--source".to_string(),
            source_display,
        ],
    )
    .with_context(|| {
        format!(
            "failed to materialize KiCad board {} into native Datum project {}",
            source.display(),
            root.display()
        )
    })?;

    Ok(LiveReviewRequest {
        project_root: root,
        board_file: None,
        artifact_path: None,
        net_uuid: None,
        from_anchor_pad_uuid: None,
        to_anchor_pad_uuid: None,
        profile: None,
        // Carry the original KiCad board so pane B can draw its companion
        // `.kicad_sch`; the materialized project holds no sibling schematic.
        kicad_board_source: Some(source),
    })
}

fn initialize_automatic_parent(parent: &Path) -> Result<()> {
    // Reject redirection before recursive creation can mutate a symlink target.
    // Explicit caller-selected project roots never pass through this helper.
    for component in parent.ancestors() {
        match std::fs::symlink_metadata(component) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                anyhow::bail!(
                    "automatic GUI import parent contains a symlink: {}",
                    component.display()
                );
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(error).with_context(|| {
                    format!(
                        "inspect automatic GUI import parent {}",
                        component.display()
                    )
                });
            }
        }
    }
    std::fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(parent)
        .with_context(|| {
            format!(
                "initialize automatic GUI import parent {}",
                parent.display()
            )
        })
}

fn materialized_project_directory_name(source: &Path) -> String {
    let mut hasher = DefaultHasher::new();
    source.display().to_string().hash(&mut hasher);
    let digest = hasher.finish();
    let stem = source
        .file_stem()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty())
        .unwrap_or("board");
    format!("{stem}-{digest:016x}")
}

fn materialized_kicad_board_project_name(source: &Path) -> String {
    let stem = source
        .file_stem()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty())
        .unwrap_or("Imported Board");
    format!("{stem} Datum Workspace")
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Fixture {
        root: PathBuf,
        source: PathBuf,
    }
    impl Fixture {
        fn new() -> Self {
            let root = std::env::temp_dir()
                .join(format!("datum-gui-import-parent-{}", uuid::Uuid::new_v4()));
            std::fs::create_dir(&root).unwrap();
            let source = root.join("board with spaces.kicad_pcb");
            std::fs::write(
                &source,
                include_bytes!("../../engine/testdata/import/kicad/simple-demo.kicad_pcb"),
            )
            .unwrap();
            Self { root, source }
        }
        fn cli(&self) -> Vec<String> {
            let binary = super::super::resolve_workspace_eda_binary()
                .expect("build datum-eda-cli before the GUI materialization integration tests");
            vec![
                "env".into(),
                "-u".into(),
                "DATUM_ENGINE_SOCKET".into(),
                "-u".into(),
                "EDA_ENGINE_SOCKET".into(),
                "-u".into(),
                "DATUM_GUI_PREFERENCES_PATH".into(),
                format!("XDG_CONFIG_HOME={}", self.root.join("config").display()),
                binary,
            ]
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.root);
        }
    }

    #[test]
    fn automatic_import_creates_initially_absent_parent_and_materializes_board() {
        let fixture = Fixture::new();
        let parent = fixture.root.join("missing-container/gui-imports");
        let original = std::fs::read(&fixture.source).unwrap();
        assert!(!parent.exists());
        let request =
            materialize_with_parent(&fixture.source, None, &parent, &fixture.cli()).unwrap();
        assert_eq!(request.project_root.parent(), Some(parent.as_path()));
        assert!(request.project_root.join("project.json").is_file());
        assert!(request.project_root.join("board/board.json").is_file());
        assert_eq!(
            request.kicad_board_source,
            Some(fixture.source.canonicalize().unwrap())
        );
        assert!(request.board_file.is_none());
        assert_eq!(std::fs::read(&fixture.source).unwrap(), original);
    }

    #[test]
    fn explicit_root_does_not_initialize_missing_parent_or_automatic_cache() {
        let fixture = Fixture::new();
        let parent = fixture.root.join("explicit-missing");
        let automatic = fixture.root.join("automatic-missing");
        let result = materialize_with_parent(
            &fixture.source,
            Some(parent.join("project")),
            &automatic,
            &fixture.cli(),
        );
        assert!(result.is_err());
        assert!(!parent.exists());
        assert!(!automatic.exists());
    }

    #[test]
    fn automatic_parent_file_is_reported_without_touching_source() {
        let fixture = Fixture::new();
        let parent = fixture.root.join("not-a-directory");
        std::fs::write(&parent, b"keep this file").unwrap();
        let error = materialize_with_parent(&fixture.source, None, &parent, &[]).unwrap_err();
        assert!(format!("{error:#}").contains("initialize automatic GUI import parent"));
        assert_eq!(std::fs::read(parent).unwrap(), b"keep this file");
    }

    #[test]
    fn automatic_parent_symlink_is_refused() {
        let fixture = Fixture::new();
        let parent = fixture.root.join("redirected");
        let target = fixture.root.join("elsewhere");
        std::fs::create_dir(&target).unwrap();
        std::os::unix::fs::symlink(&target, &parent).unwrap();
        let error = materialize_with_parent(&fixture.source, None, &parent, &[]).unwrap_err();
        assert!(error.to_string().contains("symlink"));
        assert_eq!(std::fs::read_dir(target).unwrap().count(), 0);
    }

    #[test]
    fn automatic_parent_ancestor_symlink_is_refused_before_creating_children() {
        let fixture = Fixture::new();
        let ancestor = fixture.root.join("redirected-ancestor");
        let target = fixture.root.join("untouched-target");
        std::fs::create_dir(&target).unwrap();
        std::os::unix::fs::symlink(&target, &ancestor).unwrap();
        let error =
            materialize_with_parent(&fixture.source, None, &ancestor.join("gui-imports"), &[])
                .unwrap_err();
        assert!(error.to_string().contains("symlink"));
        assert_eq!(std::fs::read_dir(target).unwrap().count(), 0);
    }

    #[test]
    fn materialized_kicad_board_defaults_to_stable_native_workspace_root() {
        let source = PathBuf::from("/tmp/example boards/DOA2526.kicad_pcb");
        let first = materialized_project_directory_name(&source);
        let second = materialized_project_directory_name(&source);

        assert_eq!(first, second);
        assert!(first.starts_with("DOA2526-"));
        assert_eq!(
            materialized_kicad_board_project_name(&source),
            "DOA2526 Datum Workspace"
        );
    }
}
