//! Shared test fixtures for the native write facade.
//!
//! Mirrors the minimal-project fixture writers used by the substrate tests
//! (`crates/engine/src/substrate/tests.rs`): a real on-disk project resolved
//! through `ProjectResolver`, never a fabricated in-memory model.

use std::path::{Path, PathBuf};

use uuid::Uuid;

use crate::substrate::{DesignModel, ProjectResolver};

fn write_json(path: &Path, value: serde_json::Value) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("parent dir should create");
    }
    std::fs::write(
        path,
        serde_json::to_string_pretty(&value).expect("json should serialize"),
    )
    .expect("json should write");
}

pub(super) fn temp_project_root(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("datum_native_write_{name}_{}", Uuid::new_v4()));
    std::fs::create_dir_all(&root).expect("temp project root should create");
    root
}

pub(super) fn write_minimal_project(root: &Path, project_id: Uuid, board_id: Uuid) {
    write_json(
        &root.join("project.json"),
        serde_json::json!({
            "schema_version": 1,
            "uuid": project_id,
            "name": "native-write-test",
            "pools": [],
            "schematic": "schematic/schematic.json",
            "board": "board/board.json",
            "rules": "rules/rules.json"
        }),
    );
    write_json(
        &root.join("schematic/schematic.json"),
        serde_json::json!({
            "schema_version": 1,
            "uuid": Uuid::new_v5(&project_id, b"schematic"),
            "sheets": {
                Uuid::new_v5(&project_id, b"sheet").to_string(): "sheets/main.json"
            },
            "definitions": {},
            "instances": [],
            "variants": {},
            "waivers": []
        }),
    );
    write_json(
        &root.join("schematic/sheets/main.json"),
        serde_json::json!({
            "schema_version": 1,
            "uuid": Uuid::new_v5(&project_id, b"sheet"),
            "name": "Main",
            "symbols": {},
            "wires": {},
            "junctions": {},
            "labels": {},
            "buses": {},
            "bus_entries": {},
            "ports": {},
            "noconnects": {},
            "texts": {},
            "drawings": {}
        }),
    );
    write_json(
        &root.join("board/board.json"),
        serde_json::json!({
            "schema_version": 1,
            "uuid": board_id,
            "name": "Board",
            "packages": {},
            "tracks": {},
            "vias": {},
            "zones": {},
            "nets": {},
            "net_classes": {}
        }),
    );
    write_json(
        &root.join("rules/rules.json"),
        serde_json::json!({
            "schema_version": 1,
            "rules": []
        }),
    );
}

fn write_project_with_board_package(
    root: &Path,
    project_id: Uuid,
    board_id: Uuid,
    package_id: Uuid,
) {
    write_minimal_project(root, project_id, board_id);
    write_json(
        &root.join("board/board.json"),
        serde_json::json!({
            "schema_version": 1,
            "uuid": board_id,
            "name": "Board",
            "packages": {
                package_id.to_string(): {
                    "uuid": package_id,
                    "part": Uuid::new_v5(&project_id, b"part"),
                    "package": Uuid::new_v5(&project_id, b"package"),
                    "reference": "U1",
                    "value": "OLD",
                    "position": { "x": 0, "y": 0 },
                    "rotation": 0,
                    "layer": 0,
                    "locked": false
                }
            },
            "tracks": {},
            "vias": {},
            "zones": {},
            "nets": {},
            "net_classes": {}
        }),
    );
}

/// Write and resolve a minimal project containing one placed board package.
/// Returns `(project_root, model, board_id, package_id)`.
pub(super) fn resolved_model_with_board_package(name: &str) -> (PathBuf, DesignModel, Uuid, Uuid) {
    let root = temp_project_root(name);
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let package_id = Uuid::new_v4();
    write_project_with_board_package(&root, project_id, board_id, package_id);
    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("fixture project should resolve");
    (root, model, board_id, package_id)
}
