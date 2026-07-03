//! Engine-owned project genesis: the four canonical root shards of a native
//! project (`project.json`, `schematic/schematic.json`, `board/board.json`,
//! `rules/rules.json`) are born here, not in CLI-private writes.
//!
//! Family J of the native-write migration. The CLI's
//! `crates/cli/src/command_project_roots.rs` `create_native_project` is a thin
//! caller: it resolves the project name and any pre-existing root ids, then
//! delegates to [`bootstrap_native_project`], which writes the shards
//! atomically (temp file + rename + fsync) and immediately re-resolves the
//! fresh project through [`ProjectResolver`] so writer/resolver drift is
//! caught at genesis time, not at first use.
//!
//! The genesis shard schemas ([`GenesisProjectManifest`],
//! [`GenesisSchematicRoot`], [`GenesisBoardRoot`], [`GenesisRulesRoot`]) are
//! the engine-owned writer definitions of the root shapes. The resolver
//! deserializes roots through `serde_json::Value` paths (plus the private
//! `NativeProjectManifestShape` in `substrate/project_resolver.rs`) rather
//! than shared structs, so parity is locked by tests here: a byte-identity
//! fixture transcribed from the historical CLI bootstrap, a repeat-bootstrap
//! idempotence check, and a resolver roundtrip.
//!
//! Genesis is deliberately **not journaled**. A provenance record at t=0 was
//! evaluated and rejected for now: `commit()` refuses empty operation
//! batches, `validate_transaction_links` and the undo/redo surfaces match
//! exhaustively on `TransactionKind`, and — decisively — the CLI contract
//! (locked by untouchable CLI tests) counts journal transactions from zero
//! after `project new` and requires repeat bootstraps to be byte-identical
//! on disk. Recording genesis therefore needs a ratified substrate decision
//! (e.g. a `TransactionKind::Genesis` zero-operation record excluded from
//! the mutation count, or a dedicated genesis evidence sidecar), not a
//! side-effect of this migration. Until then provenance starts at the first
//! committed mutation, exactly as before.

use std::collections::BTreeMap;
use std::io::Write as _;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::board::{PadExpansionSetup, StackupLayer, StackupLayerType};
use crate::error::EngineError;
use crate::ir::serialization::to_json_deterministic;
use crate::substrate::ProjectResolver;

/// Root object ids to reuse when re-running genesis over an existing
/// scaffold (`project new` is idempotent for an already-created project).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenesisRootIds {
    pub project: Uuid,
    pub schematic: Uuid,
    pub board: Uuid,
    /// Historical scaffolds may predate rules-root uuids; a fresh id is
    /// minted when absent (matching the historical CLI bootstrap).
    pub rules: Option<Uuid>,
}

/// What to bootstrap: the resolved project name and, for idempotent
/// re-initialisation, the existing root ids to preserve.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenesisSpec {
    pub project_name: String,
    /// `None` mints fresh v4 root ids (new project); `Some` re-uses the ids
    /// of an existing scaffold so repeat genesis is byte-identical.
    pub existing_ids: Option<GenesisRootIds>,
}

/// What genesis produced, for caller-side reporting.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenesisReport {
    pub project_name: String,
    pub project_uuid: Uuid,
    pub schematic_uuid: Uuid,
    pub board_uuid: Uuid,
    pub rules_uuid: Uuid,
    /// The four canonical shards, in write order: project, schematic, board,
    /// rules.
    pub files_written: Vec<PathBuf>,
}

/// Engine-owned writer schema for the genesis `project.json` manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisProjectManifest {
    pub schema_version: u32,
    pub uuid: Uuid,
    pub name: String,
    pub pools: Vec<serde_json::Value>,
    pub schematic: String,
    pub board: String,
    pub rules: String,
    #[serde(default)]
    pub forward_annotation_review: BTreeMap<String, serde_json::Value>,
}

/// Engine-owned writer schema for the genesis `schematic/schematic.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisSchematicRoot {
    pub schema_version: u32,
    pub uuid: Uuid,
    pub sheets: BTreeMap<String, String>,
    pub definitions: BTreeMap<String, String>,
    pub instances: Vec<serde_json::Value>,
    pub variants: BTreeMap<String, serde_json::Value>,
    #[serde(default)]
    pub waivers: Vec<serde_json::Value>,
    #[serde(default)]
    pub deviations: Vec<serde_json::Value>,
}

/// Engine-owned writer schema for the genesis `board/board.json`.
///
/// Field set and serde attributes mirror the CLI's `NativeBoardRoot`
/// byte-for-byte under deterministic (key-sorted) serialization; collection
/// payloads are `serde_json::Value` because genesis only ever writes them
/// empty.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisBoardRoot {
    pub schema_version: u32,
    pub uuid: Uuid,
    pub name: String,
    pub stackup: GenesisStackup,
    #[serde(default)]
    pub pad_expansion_setup: PadExpansionSetup,
    pub outline: GenesisOutline,
    #[serde(default)]
    pub packages: BTreeMap<String, serde_json::Value>,
    #[serde(default)]
    pub component_silkscreen: BTreeMap<String, serde_json::Value>,
    #[serde(default)]
    pub component_silkscreen_texts: BTreeMap<String, serde_json::Value>,
    #[serde(default)]
    pub component_silkscreen_arcs: BTreeMap<String, serde_json::Value>,
    #[serde(default)]
    pub component_silkscreen_circles: BTreeMap<String, serde_json::Value>,
    #[serde(default)]
    pub component_silkscreen_polygons: BTreeMap<String, serde_json::Value>,
    #[serde(default)]
    pub component_silkscreen_polylines: BTreeMap<String, serde_json::Value>,
    #[serde(default)]
    pub component_mechanical_lines: BTreeMap<String, serde_json::Value>,
    #[serde(default)]
    pub component_mechanical_texts: BTreeMap<String, serde_json::Value>,
    #[serde(default)]
    pub component_mechanical_polygons: BTreeMap<String, serde_json::Value>,
    #[serde(default)]
    pub component_mechanical_polylines: BTreeMap<String, serde_json::Value>,
    #[serde(default)]
    pub component_mechanical_circles: BTreeMap<String, serde_json::Value>,
    #[serde(default)]
    pub component_mechanical_arcs: BTreeMap<String, serde_json::Value>,
    #[serde(default)]
    pub component_pads: BTreeMap<String, serde_json::Value>,
    #[serde(default)]
    pub component_models_3d: BTreeMap<String, serde_json::Value>,
    #[serde(default)]
    pub pads: BTreeMap<String, serde_json::Value>,
    #[serde(default)]
    pub tracks: BTreeMap<String, serde_json::Value>,
    #[serde(default)]
    pub vias: BTreeMap<String, serde_json::Value>,
    #[serde(default)]
    pub zones: BTreeMap<String, serde_json::Value>,
    #[serde(default)]
    pub nets: BTreeMap<String, serde_json::Value>,
    #[serde(default)]
    pub net_classes: BTreeMap<String, serde_json::Value>,
    #[serde(default)]
    pub keepouts: Vec<serde_json::Value>,
    #[serde(default)]
    pub dimensions: Vec<serde_json::Value>,
    #[serde(default)]
    pub texts: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisStackup {
    pub layers: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisOutline {
    pub vertices: Vec<serde_json::Value>,
    pub closed: bool,
}

/// Engine-owned writer schema for the genesis `rules/rules.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisRulesRoot {
    pub schema_version: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uuid: Option<Uuid>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub object_revision: Option<u64>,
    pub rules: Vec<serde_json::Value>,
}

/// The default board stackup seeded at genesis (single-sided top stack plus
/// one mechanical layer). Byte-identical to the historical CLI default in
/// `crates/cli/src/command_project_default_stackup.rs`; the CLI asserts
/// parity against this function.
pub fn default_genesis_stackup_layers() -> Vec<serde_json::Value> {
    [
        StackupLayer::new(1, "Top Copper", StackupLayerType::Copper, 35_000),
        StackupLayer::new(2, "Top Mask", StackupLayerType::SolderMask, 10_000),
        StackupLayer::new(3, "Top Silk", StackupLayerType::Silkscreen, 10_000),
        StackupLayer::new(4, "Top Paste", StackupLayerType::Paste, 10_000),
        StackupLayer::new(41, "Mechanical 41", StackupLayerType::Mechanical, 0),
    ]
    .into_iter()
    .map(|layer| {
        serde_json::to_value(layer).expect("genesis stackup layer serialization must succeed")
    })
    .collect()
}

/// Bootstrap the four canonical shards of a native project under `root`,
/// then resolve the fresh project as a writer/resolver parity check.
///
/// Output is byte-identical to the historical CLI bootstrap (deterministic
/// key-sorted JSON with a trailing newline); repeat bootstraps with the same
/// [`GenesisRootIds`] are byte-idempotent. Writes are atomic per shard
/// (same-directory temp file, fsync, rename, directory fsync). See the
/// module docs for why genesis does not append a journal record yet.
pub fn bootstrap_native_project(
    root: &Path,
    spec: GenesisSpec,
) -> Result<GenesisReport, EngineError> {
    let ids = spec.existing_ids.unwrap_or_else(|| GenesisRootIds {
        project: Uuid::new_v4(),
        schematic: Uuid::new_v4(),
        board: Uuid::new_v4(),
        rules: None,
    });
    let rules_uuid = ids.rules.unwrap_or_else(Uuid::new_v4);
    let project_name = spec.project_name;

    let manifest = GenesisProjectManifest {
        schema_version: 1,
        uuid: ids.project,
        name: project_name.clone(),
        pools: Vec::new(),
        schematic: "schematic/schematic.json".to_string(),
        board: "board/board.json".to_string(),
        rules: "rules/rules.json".to_string(),
        forward_annotation_review: BTreeMap::new(),
    };
    let schematic = GenesisSchematicRoot {
        schema_version: 1,
        uuid: ids.schematic,
        sheets: BTreeMap::new(),
        definitions: BTreeMap::new(),
        instances: Vec::new(),
        variants: BTreeMap::new(),
        waivers: Vec::new(),
        deviations: Vec::new(),
    };
    let board = GenesisBoardRoot {
        schema_version: 1,
        uuid: ids.board,
        name: format!("{project_name} Board"),
        stackup: GenesisStackup {
            layers: default_genesis_stackup_layers(),
        },
        pad_expansion_setup: PadExpansionSetup::default(),
        outline: GenesisOutline {
            vertices: Vec::new(),
            closed: true,
        },
        packages: BTreeMap::new(),
        component_silkscreen: BTreeMap::new(),
        component_silkscreen_texts: BTreeMap::new(),
        component_silkscreen_arcs: BTreeMap::new(),
        component_silkscreen_circles: BTreeMap::new(),
        component_silkscreen_polygons: BTreeMap::new(),
        component_silkscreen_polylines: BTreeMap::new(),
        component_mechanical_lines: BTreeMap::new(),
        component_mechanical_texts: BTreeMap::new(),
        component_mechanical_polygons: BTreeMap::new(),
        component_mechanical_polylines: BTreeMap::new(),
        component_mechanical_circles: BTreeMap::new(),
        component_mechanical_arcs: BTreeMap::new(),
        component_pads: BTreeMap::new(),
        component_models_3d: BTreeMap::new(),
        pads: BTreeMap::new(),
        tracks: BTreeMap::new(),
        vias: BTreeMap::new(),
        zones: BTreeMap::new(),
        nets: BTreeMap::new(),
        net_classes: BTreeMap::new(),
        keepouts: Vec::new(),
        dimensions: Vec::new(),
        texts: Vec::new(),
    };
    let rules = GenesisRulesRoot {
        schema_version: 1,
        uuid: Some(rules_uuid),
        object_revision: Some(0),
        rules: Vec::new(),
    };

    let project_json = root.join("project.json");
    let schematic_dir = root.join("schematic");
    let sheets_dir = schematic_dir.join("sheets");
    let definitions_dir = schematic_dir.join("definitions");
    let board_dir = root.join("board");
    let rules_dir = root.join("rules");
    let schematic_json = schematic_dir.join("schematic.json");
    let board_json = board_dir.join("board.json");
    let rules_json = rules_dir.join("rules.json");

    for dir in [&sheets_dir, &definitions_dir, &board_dir, &rules_dir] {
        std::fs::create_dir_all(dir).map_err(|error| {
            EngineError::Operation(format!("failed to create {}: {error}", dir.display()))
        })?;
    }

    write_genesis_shard(&project_json, &manifest)?;
    write_genesis_shard(&schematic_json, &schematic)?;
    write_genesis_shard(&board_json, &board)?;
    write_genesis_shard(&rules_json, &rules)?;

    // Writer/resolver parity tripwire: the fresh scaffold must resolve
    // through the one canonical resolver before genesis reports success.
    ProjectResolver::new(root).resolve()?;

    Ok(GenesisReport {
        project_name,
        project_uuid: ids.project,
        schematic_uuid: ids.schematic,
        board_uuid: ids.board,
        rules_uuid,
        files_written: vec![project_json, schematic_json, board_json, rules_json],
    })
}

/// Atomically write one genesis shard as canonical (deterministic,
/// key-sorted) JSON with a trailing newline — byte-identical to the
/// historical CLI `write_canonical_json` output.
fn write_genesis_shard<T: Serialize>(path: &Path, value: &T) -> Result<(), EngineError> {
    let json = to_json_deterministic(value)?;
    let bytes = format!("{json}\n").into_bytes();
    let temp_path = genesis_temp_path(path)?;
    let write_result = (|| -> std::io::Result<()> {
        let mut file = std::fs::File::create(&temp_path)?;
        file.write_all(&bytes)?;
        file.sync_all()?;
        std::fs::rename(&temp_path, path)?;
        if let Some(parent) = path.parent() {
            std::fs::File::open(parent)?.sync_all()?;
        }
        Ok(())
    })();
    if write_result.is_err() {
        let _ = std::fs::remove_file(&temp_path);
    }
    write_result
        .map_err(|error| EngineError::Operation(format!("failed to write {}: {error}", path.display())))
}

fn genesis_temp_path(path: &Path) -> Result<PathBuf, EngineError> {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| {
            EngineError::Operation(format!(
                "genesis shard path has no file name: {}",
                path.display()
            ))
        })?;
    Ok(path.with_file_name(format!(".{file_name}.genesis-tmp")))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::super::test_support::temp_project_root;
    use super::*;
    use crate::substrate::SourceShardKind;

    fn fixture_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("testdata/native_write/genesis")
    }

    fn read_fixture(name: &str) -> String {
        let path = fixture_dir().join(name);
        std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("fixture {} should read: {error}", path.display()))
    }

    fn fixture_ids() -> (String, GenesisRootIds) {
        let manifest: serde_json::Value =
            serde_json::from_str(&read_fixture("project.json")).expect("fixture manifest parses");
        let schematic: serde_json::Value = serde_json::from_str(&read_fixture("schematic.json"))
            .expect("fixture schematic parses");
        let board: serde_json::Value =
            serde_json::from_str(&read_fixture("board.json")).expect("fixture board parses");
        let rules: serde_json::Value =
            serde_json::from_str(&read_fixture("rules.json")).expect("fixture rules parses");
        let uuid = |value: &serde_json::Value| {
            Uuid::parse_str(value["uuid"].as_str().expect("fixture uuid is a string"))
                .expect("fixture uuid parses")
        };
        (
            manifest["name"].as_str().expect("fixture name").to_string(),
            GenesisRootIds {
                project: uuid(&manifest),
                schematic: uuid(&schematic),
                board: uuid(&board),
                rules: Some(uuid(&rules)),
            },
        )
    }

    fn bootstrap_fixture_project(label: &str) -> (PathBuf, GenesisReport) {
        let root = temp_project_root(label);
        let (name, ids) = fixture_ids();
        let report = bootstrap_native_project(
            &root,
            GenesisSpec {
                project_name: name,
                existing_ids: Some(ids),
            },
        )
        .expect("genesis should succeed");
        (root, report)
    }

    #[test]
    fn genesis_is_byte_identical_to_transcribed_cli_bootstrap_fixture() {
        // The fixture under testdata/native_write/genesis was transcribed
        // from the historical CLI `project new` bootstrap output before this
        // migration; engine genesis must reproduce it byte-for-byte.
        let (root, _report) = bootstrap_fixture_project("genesis_bytes");
        for (written, fixture) in [
            ("project.json", "project.json"),
            ("schematic/schematic.json", "schematic.json"),
            ("board/board.json", "board.json"),
            ("rules/rules.json", "rules.json"),
        ] {
            let actual = std::fs::read_to_string(root.join(written))
                .expect("genesis shard should read");
            assert_eq!(
                actual,
                read_fixture(fixture),
                "genesis output for {written} drifted from the transcribed CLI bootstrap"
            );
        }
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn genesis_is_byte_idempotent_for_existing_ids() {
        let (root, report) = bootstrap_fixture_project("genesis_idempotent");
        let before = report
            .files_written
            .iter()
            .map(|path| std::fs::read(path).expect("genesis shard should read"))
            .collect::<Vec<_>>();
        let (name, ids) = fixture_ids();
        let second = bootstrap_native_project(
            &root,
            GenesisSpec {
                project_name: name,
                existing_ids: Some(ids),
            },
        )
        .expect("repeat genesis should succeed");
        let after = second
            .files_written
            .iter()
            .map(|path| std::fs::read(path).expect("genesis shard should read"))
            .collect::<Vec<_>>();
        assert_eq!(before, after);
        assert_eq!(report.project_uuid, second.project_uuid);
        assert_eq!(report.schematic_uuid, second.schematic_uuid);
        assert_eq!(report.board_uuid, second.board_uuid);
        assert_eq!(report.rules_uuid, second.rules_uuid);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn genesis_resolves_cleanly_through_the_project_resolver() {
        // Resolver-roundtrip parity: the resolver reads roots through
        // serde_json::Value paths, so the writer schemas are locked against
        // it here rather than by shared struct definitions.
        let (root, report) = bootstrap_fixture_project("genesis_resolves");
        let model = ProjectResolver::new(&root)
            .resolve()
            .expect("fresh genesis project should resolve");
        assert_eq!(model.project.project_id, report.project_uuid);
        assert_eq!(model.project.name, report.project_name);
        assert!(
            model.diagnostics.is_empty(),
            "fresh genesis project should resolve without diagnostics: {:?}",
            model.diagnostics
        );
        for kind in [
            SourceShardKind::ProjectManifest,
            SourceShardKind::SchematicRoot,
            SourceShardKind::BoardRoot,
            SourceShardKind::RulesRoot,
        ] {
            assert!(
                model.source_shards.iter().any(|shard| shard.kind == kind),
                "genesis project should expose a {kind:?} shard"
            );
        }
        for (label, object_id) in [
            ("project", report.project_uuid),
            ("schematic", report.schematic_uuid),
            ("board", report.board_uuid),
            ("rules", report.rules_uuid),
        ] {
            assert!(
                model.objects.contains_key(&object_id),
                "genesis {label} root object should be resolved as a domain object"
            );
        }
        assert!(
            model.journal.is_empty(),
            "genesis does not append journal records (see module docs)"
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn genesis_writer_schemas_roundtrip_their_own_output() {
        let (root, _report) = bootstrap_fixture_project("genesis_roundtrip");
        let manifest_text = std::fs::read_to_string(root.join("project.json"))
            .expect("manifest should read");
        let manifest: GenesisProjectManifest =
            serde_json::from_str(&manifest_text).expect("manifest should roundtrip");
        assert_eq!(
            to_json_deterministic(&manifest).expect("manifest should serialize") + "\n",
            manifest_text
        );
        let schematic_text = std::fs::read_to_string(root.join("schematic/schematic.json"))
            .expect("schematic should read");
        let schematic: GenesisSchematicRoot =
            serde_json::from_str(&schematic_text).expect("schematic should roundtrip");
        assert_eq!(
            to_json_deterministic(&schematic).expect("schematic should serialize") + "\n",
            schematic_text
        );
        let board_text =
            std::fs::read_to_string(root.join("board/board.json")).expect("board should read");
        let board: GenesisBoardRoot =
            serde_json::from_str(&board_text).expect("board should roundtrip");
        assert_eq!(
            to_json_deterministic(&board).expect("board should serialize") + "\n",
            board_text
        );
        let rules_text =
            std::fs::read_to_string(root.join("rules/rules.json")).expect("rules should read");
        let rules: GenesisRulesRoot =
            serde_json::from_str(&rules_text).expect("rules should roundtrip");
        assert_eq!(
            to_json_deterministic(&rules).expect("rules should serialize") + "\n",
            rules_text
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn genesis_mints_fresh_ids_when_none_are_supplied() {
        let root = temp_project_root("genesis_fresh_ids");
        let report = bootstrap_native_project(
            &root,
            GenesisSpec {
                project_name: "Fresh Genesis".to_string(),
                existing_ids: None,
            },
        )
        .expect("genesis should succeed");
        assert_eq!(report.project_name, "Fresh Genesis");
        let ids = [
            report.project_uuid,
            report.schematic_uuid,
            report.board_uuid,
            report.rules_uuid,
        ];
        for (index, id) in ids.iter().enumerate() {
            assert!(!id.is_nil());
            for other in &ids[index + 1..] {
                assert_ne!(id, other, "genesis root ids must be distinct");
            }
        }
        assert_eq!(report.files_written.len(), 4);
        let _ = std::fs::remove_dir_all(&root);
    }
}
