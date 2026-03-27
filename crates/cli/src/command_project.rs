use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{Context, Result, bail};
use eda_engine::ir::serialization::to_json_deterministic;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{NativeProjectCreateReportView, NativeProjectInspectReportView};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NativeProjectManifest {
    schema_version: u32,
    uuid: Uuid,
    name: String,
    pools: Vec<NativeProjectPoolRef>,
    schematic: String,
    board: String,
    rules: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NativeProjectPoolRef {
    path: String,
    priority: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NativeSchematicRoot {
    schema_version: u32,
    uuid: Uuid,
    sheets: BTreeMap<String, String>,
    definitions: BTreeMap<String, String>,
    instances: Vec<NativeSchematicInstance>,
    variants: BTreeMap<String, NativeVariant>,
    waivers: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NativeSchematicInstance {
    uuid: Uuid,
    definition: Uuid,
    parent_sheet: Option<Uuid>,
    position: NativePoint,
    name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NativeVariant {
    name: String,
    fitted_components: BTreeMap<String, bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NativeBoardRoot {
    schema_version: u32,
    uuid: Uuid,
    name: String,
    stackup: NativeStackup,
    outline: NativeOutline,
    packages: BTreeMap<String, serde_json::Value>,
    tracks: BTreeMap<String, serde_json::Value>,
    vias: BTreeMap<String, serde_json::Value>,
    zones: BTreeMap<String, serde_json::Value>,
    nets: BTreeMap<String, serde_json::Value>,
    net_classes: BTreeMap<String, serde_json::Value>,
    keepouts: Vec<serde_json::Value>,
    dimensions: Vec<serde_json::Value>,
    texts: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NativeStackup {
    layers: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NativeOutline {
    vertices: Vec<NativePoint>,
    closed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NativePoint {
    x: i64,
    y: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NativeRulesRoot {
    schema_version: u32,
    rules: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ExistingProjectIds {
    project_uuid: Uuid,
    schematic_uuid: Uuid,
    board_uuid: Uuid,
}

pub(crate) fn create_native_project(
    root: &Path,
    name_override: Option<String>,
) -> Result<NativeProjectCreateReportView> {
    let root = root.to_path_buf();
    ensure_project_root(&root)?;

    let default_name = root
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .map(str::to_string)
        .ok_or_else(|| anyhow::anyhow!("project root must have a terminal directory name"))?;
    let project_name = name_override.unwrap_or(default_name);

    let ids = load_existing_ids(&root)?.unwrap_or_else(|| ExistingProjectIds {
        project_uuid: Uuid::new_v4(),
        schematic_uuid: Uuid::new_v4(),
        board_uuid: Uuid::new_v4(),
    });

    let manifest = NativeProjectManifest {
        schema_version: 1,
        uuid: ids.project_uuid,
        name: project_name.clone(),
        pools: Vec::new(),
        schematic: "schematic/schematic.json".to_string(),
        board: "board/board.json".to_string(),
        rules: "rules/rules.json".to_string(),
    };
    let schematic = NativeSchematicRoot {
        schema_version: 1,
        uuid: ids.schematic_uuid,
        sheets: BTreeMap::new(),
        definitions: BTreeMap::new(),
        instances: Vec::new(),
        variants: BTreeMap::new(),
        waivers: Vec::new(),
    };
    let board = NativeBoardRoot {
        schema_version: 1,
        uuid: ids.board_uuid,
        name: format!("{project_name} Board"),
        stackup: NativeStackup { layers: Vec::new() },
        outline: NativeOutline {
            vertices: Vec::new(),
            closed: true,
        },
        packages: BTreeMap::new(),
        tracks: BTreeMap::new(),
        vias: BTreeMap::new(),
        zones: BTreeMap::new(),
        nets: BTreeMap::new(),
        net_classes: BTreeMap::new(),
        keepouts: Vec::new(),
        dimensions: Vec::new(),
        texts: Vec::new(),
    };
    let rules = NativeRulesRoot {
        schema_version: 1,
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

    std::fs::create_dir_all(&sheets_dir)
        .with_context(|| format!("failed to create {}", sheets_dir.display()))?;
    std::fs::create_dir_all(&definitions_dir)
        .with_context(|| format!("failed to create {}", definitions_dir.display()))?;
    std::fs::create_dir_all(&board_dir)
        .with_context(|| format!("failed to create {}", board_dir.display()))?;
    std::fs::create_dir_all(&rules_dir)
        .with_context(|| format!("failed to create {}", rules_dir.display()))?;

    write_canonical_json(&project_json, &manifest)?;
    write_canonical_json(&schematic_json, &schematic)?;
    write_canonical_json(&board_json, &board)?;
    write_canonical_json(&rules_json, &rules)?;

    Ok(NativeProjectCreateReportView {
        project_root: root.display().to_string(),
        project_name,
        project_uuid: ids.project_uuid.to_string(),
        schematic_uuid: ids.schematic_uuid.to_string(),
        board_uuid: ids.board_uuid.to_string(),
        files_written: vec![
            project_json.display().to_string(),
            schematic_json.display().to_string(),
            board_json.display().to_string(),
            rules_json.display().to_string(),
        ],
    })
}

pub(crate) fn inspect_native_project(root: &Path) -> Result<NativeProjectInspectReportView> {
    let root = root.to_path_buf();
    if !root.is_dir() {
        bail!("project root does not exist or is not a directory: {}", root.display());
    }

    let manifest_path = root.join("project.json");
    let manifest_text = std::fs::read_to_string(&manifest_path)
        .with_context(|| format!("failed to read {}", manifest_path.display()))?;
    let manifest: NativeProjectManifest = serde_json::from_str(&manifest_text)
        .with_context(|| format!("failed to parse {}", manifest_path.display()))?;

    let schematic_path = root.join(&manifest.schematic);
    let board_path = root.join(&manifest.board);
    let rules_path = root.join(&manifest.rules);
    let schematic_text = std::fs::read_to_string(&schematic_path)
        .with_context(|| format!("failed to read {}", schematic_path.display()))?;
    let board_text = std::fs::read_to_string(&board_path)
        .with_context(|| format!("failed to read {}", board_path.display()))?;
    let rules_text = std::fs::read_to_string(&rules_path)
        .with_context(|| format!("failed to read {}", rules_path.display()))?;
    let schematic: NativeSchematicRoot = serde_json::from_str(&schematic_text)
        .with_context(|| format!("failed to parse {}", schematic_path.display()))?;
    let board: NativeBoardRoot = serde_json::from_str(&board_text)
        .with_context(|| format!("failed to parse {}", board_path.display()))?;
    let rules: NativeRulesRoot = serde_json::from_str(&rules_text)
        .with_context(|| format!("failed to parse {}", rules_path.display()))?;

    Ok(NativeProjectInspectReportView {
        project_root: root.display().to_string(),
        project_name: manifest.name,
        schema_version: manifest.schema_version,
        project_uuid: manifest.uuid.to_string(),
        schematic_uuid: schematic.uuid.to_string(),
        board_uuid: board.uuid.to_string(),
        pools: manifest.pools.len(),
        schematic_path: schematic_path.display().to_string(),
        board_path: board_path.display().to_string(),
        rules_path: rules_path.display().to_string(),
        sheet_count: schematic.sheets.len(),
        sheet_definition_count: schematic.definitions.len(),
        sheet_instance_count: schematic.instances.len(),
        variant_count: schematic.variants.len(),
        board_package_count: board.packages.len(),
        board_net_count: board.nets.len(),
        board_track_count: board.tracks.len(),
        board_via_count: board.vias.len(),
        board_zone_count: board.zones.len(),
        rule_count: rules.rules.len(),
    })
}

fn ensure_project_root(root: &Path) -> Result<()> {
    if root.exists() {
        if !root.is_dir() {
            bail!("project root exists but is not a directory: {}", root.display());
        }
    } else {
        std::fs::create_dir_all(root)
            .with_context(|| format!("failed to create project root {}", root.display()))?;
    }
    Ok(())
}

fn load_existing_ids(root: &Path) -> Result<Option<ExistingProjectIds>> {
    let project_path = root.join("project.json");
    if !project_path.exists() {
        return Ok(None);
    }

    let project_text = std::fs::read_to_string(&project_path)
        .with_context(|| format!("failed to read {}", project_path.display()))?;
    let manifest: NativeProjectManifest = serde_json::from_str(&project_text)
        .with_context(|| format!("failed to parse {}", project_path.display()))?;

    let schematic_path = root.join(&manifest.schematic);
    let board_path = root.join(&manifest.board);
    let schematic_text = std::fs::read_to_string(&schematic_path)
        .with_context(|| format!("failed to read {}", schematic_path.display()))?;
    let board_text = std::fs::read_to_string(&board_path)
        .with_context(|| format!("failed to read {}", board_path.display()))?;
    let schematic: NativeSchematicRoot = serde_json::from_str(&schematic_text)
        .with_context(|| format!("failed to parse {}", schematic_path.display()))?;
    let board: NativeBoardRoot = serde_json::from_str(&board_text)
        .with_context(|| format!("failed to parse {}", board_path.display()))?;

    Ok(Some(ExistingProjectIds {
        project_uuid: manifest.uuid,
        schematic_uuid: schematic.uuid,
        board_uuid: board.uuid,
    }))
}

fn write_canonical_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let json = to_json_deterministic(value).context("failed to serialize canonical JSON")?;
    std::fs::write(path, format!("{json}\n"))
        .with_context(|| format!("failed to write {}", path.display()))
}
