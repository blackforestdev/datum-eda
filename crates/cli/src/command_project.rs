use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{Context, Result, bail};
use eda_engine::ir::serialization::to_json_deterministic;
use eda_engine::ir::geometry::Point;
use eda_engine::schematic::{
    HierarchicalPort, Junction, LabelInfo, LabelKind, NetLabel, PortDirection, PortInfo,
    SchematicWire,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{
    NativeProjectBoardSummaryView, NativeProjectCreateReportView, NativeProjectInspectReportView,
    NativeProjectJunctionMutationReportView,
    NativeProjectLabelMutationReportView,
    NativeProjectPortMutationReportView,
    NativeProjectWireMutationReportView,
    NativeProjectRulesSummaryView, NativeProjectRulesView, NativeProjectSchematicSummaryView,
    NativeProjectSummaryView,
};

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
    let project = load_native_project(root)?;

    Ok(NativeProjectInspectReportView {
        project_root: project.root.display().to_string(),
        project_name: project.manifest.name.clone(),
        schema_version: project.manifest.schema_version,
        project_uuid: project.manifest.uuid.to_string(),
        schematic_uuid: project.schematic.uuid.to_string(),
        board_uuid: project.board.uuid.to_string(),
        pools: project.manifest.pools.len(),
        schematic_path: project.schematic_path.display().to_string(),
        board_path: project.board_path.display().to_string(),
        rules_path: project.rules_path.display().to_string(),
        sheet_count: project.schematic.sheets.len(),
        sheet_definition_count: project.schematic.definitions.len(),
        sheet_instance_count: project.schematic.instances.len(),
        variant_count: project.schematic.variants.len(),
        board_package_count: project.board.packages.len(),
        board_net_count: project.board.nets.len(),
        board_track_count: project.board.tracks.len(),
        board_via_count: project.board.vias.len(),
        board_zone_count: project.board.zones.len(),
        rule_count: project.rules.rules.len(),
    })
}

pub(crate) fn query_native_project_summary(root: &Path) -> Result<NativeProjectSummaryView> {
    let project = load_native_project(root)?;
    let schematic_counts = collect_schematic_counts(&project.root, &project.schematic)?;
    Ok(NativeProjectSummaryView {
        domain: "native_project",
        project_name: project.manifest.name,
        schema_version: project.manifest.schema_version,
        pools: project.manifest.pools.len(),
        schematic: NativeProjectSchematicSummaryView {
            sheets: project.schematic.sheets.len(),
            sheet_definitions: project.schematic.definitions.len(),
            sheet_instances: project.schematic.instances.len(),
            variants: project.schematic.variants.len(),
            symbols: schematic_counts.symbols,
            wires: schematic_counts.wires,
            junctions: schematic_counts.junctions,
            labels: schematic_counts.labels,
            ports: schematic_counts.ports,
            buses: schematic_counts.buses,
            bus_entries: schematic_counts.bus_entries,
            noconnects: schematic_counts.noconnects,
        },
        board: NativeProjectBoardSummaryView {
            name: project.board.name,
            layers: project.board.stackup.layers.len(),
            components: project.board.packages.len(),
            nets: project.board.nets.len(),
            tracks: project.board.tracks.len(),
            vias: project.board.vias.len(),
            zones: project.board.zones.len(),
            keepouts: project.board.keepouts.len(),
            texts: project.board.texts.len(),
        },
        rules: NativeProjectRulesSummaryView {
            count: project.rules.rules.len(),
        },
    })
}

pub(crate) fn query_native_project_rules(root: &Path) -> Result<NativeProjectRulesView> {
    let project = load_native_project(root)?;
    Ok(NativeProjectRulesView {
        domain: "native_project",
        count: project.rules.rules.len(),
        rules: project.rules.rules,
    })
}

pub(crate) fn query_native_project_labels(root: &Path) -> Result<Vec<LabelInfo>> {
    let project = load_native_project(root)?;
    let mut labels = Vec::new();
    for (sheet_uuid, relative_path) in &project.schematic.sheets {
        let sheet_uuid = Uuid::parse_str(sheet_uuid)
            .with_context(|| format!("invalid sheet UUID key `{sheet_uuid}` in schematic root"))?;
        let path = project.root.join("schematic").join(relative_path);
        let sheet_text = std::fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let sheet_value: serde_json::Value = serde_json::from_str(&sheet_text)
            .with_context(|| format!("failed to parse {}", path.display()))?;
        if let Some(entries) = sheet_value.get("labels").and_then(serde_json::Value::as_object) {
            for value in entries.values() {
                let label: NetLabel = serde_json::from_value(value.clone())
                    .with_context(|| format!("failed to parse label in {}", path.display()))?;
                labels.push(LabelInfo {
                    uuid: label.uuid,
                    sheet: sheet_uuid,
                    kind: label.kind,
                    name: label.name,
                    position: label.position,
                });
            }
        }
    }
    labels.sort_by(|a, b| a.name.cmp(&b.name).then_with(|| a.uuid.cmp(&b.uuid)));
    Ok(labels)
}

pub(crate) fn query_native_project_wires(root: &Path) -> Result<Vec<serde_json::Value>> {
    let project = load_native_project(root)?;
    let mut wires = Vec::new();
    for (sheet_uuid, relative_path) in &project.schematic.sheets {
        let sheet_uuid = Uuid::parse_str(sheet_uuid)
            .with_context(|| format!("invalid sheet UUID key `{sheet_uuid}` in schematic root"))?;
        let path = project.root.join("schematic").join(relative_path);
        let sheet_text = std::fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let sheet_value: serde_json::Value = serde_json::from_str(&sheet_text)
            .with_context(|| format!("failed to parse {}", path.display()))?;
        if let Some(entries) = sheet_value.get("wires").and_then(serde_json::Value::as_object) {
            for value in entries.values() {
                let wire: SchematicWire = serde_json::from_value(value.clone())
                    .with_context(|| format!("failed to parse wire in {}", path.display()))?;
                wires.push(serde_json::json!({
                    "uuid": wire.uuid,
                    "sheet": sheet_uuid,
                    "from": wire.from,
                    "to": wire.to,
                }));
            }
        }
    }
    wires.sort_by(|a, b| {
        a.get("uuid")
            .and_then(serde_json::Value::as_str)
            .cmp(&b.get("uuid").and_then(serde_json::Value::as_str))
    });
    Ok(wires)
}

pub(crate) fn query_native_project_junctions(root: &Path) -> Result<Vec<serde_json::Value>> {
    let project = load_native_project(root)?;
    let mut junctions = Vec::new();
    for (sheet_uuid, relative_path) in &project.schematic.sheets {
        let sheet_uuid = Uuid::parse_str(sheet_uuid)
            .with_context(|| format!("invalid sheet UUID key `{sheet_uuid}` in schematic root"))?;
        let path = project.root.join("schematic").join(relative_path);
        let sheet_text = std::fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let sheet_value: serde_json::Value = serde_json::from_str(&sheet_text)
            .with_context(|| format!("failed to parse {}", path.display()))?;
        if let Some(entries) = sheet_value
            .get("junctions")
            .and_then(serde_json::Value::as_object)
        {
            for value in entries.values() {
                let junction: Junction = serde_json::from_value(value.clone())
                    .with_context(|| format!("failed to parse junction in {}", path.display()))?;
                junctions.push(serde_json::json!({
                    "uuid": junction.uuid,
                    "sheet": sheet_uuid,
                    "position": junction.position,
                }));
            }
        }
    }
    junctions.sort_by(|a, b| {
        a.get("uuid")
            .and_then(serde_json::Value::as_str)
            .cmp(&b.get("uuid").and_then(serde_json::Value::as_str))
    });
    Ok(junctions)
}

pub(crate) fn query_native_project_ports(root: &Path) -> Result<Vec<PortInfo>> {
    let project = load_native_project(root)?;
    let mut ports = Vec::new();
    for (sheet_uuid, relative_path) in &project.schematic.sheets {
        let sheet_uuid = Uuid::parse_str(sheet_uuid)
            .with_context(|| format!("invalid sheet UUID key `{sheet_uuid}` in schematic root"))?;
        let path = project.root.join("schematic").join(relative_path);
        let sheet_text = std::fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let sheet_value: serde_json::Value = serde_json::from_str(&sheet_text)
            .with_context(|| format!("failed to parse {}", path.display()))?;
        if let Some(entries) = sheet_value.get("ports").and_then(serde_json::Value::as_object) {
            for value in entries.values() {
                let port: HierarchicalPort = serde_json::from_value(value.clone())
                    .with_context(|| format!("failed to parse port in {}", path.display()))?;
                ports.push(PortInfo {
                    uuid: port.uuid,
                    sheet: sheet_uuid,
                    name: port.name,
                    direction: port.direction,
                    position: port.position,
                });
            }
        }
    }
    ports.sort_by(|a, b| a.name.cmp(&b.name).then_with(|| a.uuid.cmp(&b.uuid)));
    Ok(ports)
}

pub(crate) fn place_native_project_label(
    root: &Path,
    sheet_uuid: Uuid,
    name: String,
    kind: LabelKind,
    position: Point,
) -> Result<NativeProjectLabelMutationReportView> {
    let project = load_native_project(root)?;
    let sheet_key = sheet_uuid.to_string();
    let relative_path = project
        .schematic
        .sheets
        .get(&sheet_key)
        .ok_or_else(|| anyhow::anyhow!("sheet not found in native schematic root: {sheet_uuid}"))?;
    let sheet_path = project.root.join("schematic").join(relative_path);
    let sheet_text = std::fs::read_to_string(&sheet_path)
        .with_context(|| format!("failed to read {}", sheet_path.display()))?;
    let mut sheet_value: serde_json::Value = serde_json::from_str(&sheet_text)
        .with_context(|| format!("failed to parse {}", sheet_path.display()))?;
    let labels = sheet_value
        .as_object_mut()
        .and_then(|object| object.get_mut("labels"))
        .and_then(serde_json::Value::as_object_mut)
        .ok_or_else(|| anyhow::anyhow!("sheet labels object missing in {}", sheet_path.display()))?;

    let label_uuid = Uuid::new_v4();
    labels.insert(
        label_uuid.to_string(),
        serde_json::to_value(NetLabel {
            uuid: label_uuid,
            kind: kind.clone(),
            name: name.clone(),
            position,
        })
        .expect("native label serialization must succeed"),
    );

    write_canonical_json(&sheet_path, &sheet_value)?;

    Ok(NativeProjectLabelMutationReportView {
        action: "place_label".to_string(),
        project_root: project.root.display().to_string(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: sheet_path.display().to_string(),
        label_uuid: label_uuid.to_string(),
        name,
        kind: render_label_kind(&kind).to_string(),
        x_nm: position.x,
        y_nm: position.y,
    })
}

pub(crate) fn rename_native_project_label(
    root: &Path,
    label_uuid: Uuid,
    name: String,
) -> Result<NativeProjectLabelMutationReportView> {
    let project = load_native_project(root)?;
    let (sheet_uuid, sheet_path, mut sheet_value, mut label) =
        load_native_label_mutation_target(&project, label_uuid)?;
    label.name = name.clone();
    write_label_into_sheet(&mut sheet_value, &label)?;
    write_canonical_json(&sheet_path, &sheet_value)?;

    Ok(NativeProjectLabelMutationReportView {
        action: "rename_label".to_string(),
        project_root: project.root.display().to_string(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: sheet_path.display().to_string(),
        label_uuid: label.uuid.to_string(),
        name,
        kind: render_label_kind(&label.kind).to_string(),
        x_nm: label.position.x,
        y_nm: label.position.y,
    })
}

pub(crate) fn delete_native_project_label(
    root: &Path,
    label_uuid: Uuid,
) -> Result<NativeProjectLabelMutationReportView> {
    let project = load_native_project(root)?;
    let (sheet_uuid, sheet_path, mut sheet_value, label) =
        load_native_label_mutation_target(&project, label_uuid)?;
    let labels = sheet_value
        .as_object_mut()
        .and_then(|object| object.get_mut("labels"))
        .and_then(serde_json::Value::as_object_mut)
        .ok_or_else(|| anyhow::anyhow!("sheet labels object missing in {}", sheet_path.display()))?;
    labels.remove(&label_uuid.to_string());
    write_canonical_json(&sheet_path, &sheet_value)?;

    Ok(NativeProjectLabelMutationReportView {
        action: "delete_label".to_string(),
        project_root: project.root.display().to_string(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: sheet_path.display().to_string(),
        label_uuid: label.uuid.to_string(),
        name: label.name,
        kind: render_label_kind(&label.kind).to_string(),
        x_nm: label.position.x,
        y_nm: label.position.y,
    })
}

pub(crate) fn draw_native_project_wire(
    root: &Path,
    sheet_uuid: Uuid,
    from: Point,
    to: Point,
) -> Result<NativeProjectWireMutationReportView> {
    let project = load_native_project(root)?;
    let sheet_key = sheet_uuid.to_string();
    let relative_path = project
        .schematic
        .sheets
        .get(&sheet_key)
        .ok_or_else(|| anyhow::anyhow!("sheet not found in native schematic root: {sheet_uuid}"))?;
    let sheet_path = project.root.join("schematic").join(relative_path);
    let sheet_text = std::fs::read_to_string(&sheet_path)
        .with_context(|| format!("failed to read {}", sheet_path.display()))?;
    let mut sheet_value: serde_json::Value = serde_json::from_str(&sheet_text)
        .with_context(|| format!("failed to parse {}", sheet_path.display()))?;
    let wires = sheet_value
        .as_object_mut()
        .and_then(|object| object.get_mut("wires"))
        .and_then(serde_json::Value::as_object_mut)
        .ok_or_else(|| anyhow::anyhow!("sheet wires object missing in {}", sheet_path.display()))?;

    let wire_uuid = Uuid::new_v4();
    wires.insert(
        wire_uuid.to_string(),
        serde_json::to_value(SchematicWire { uuid: wire_uuid, from, to })
            .expect("native wire serialization must succeed"),
    );

    write_canonical_json(&sheet_path, &sheet_value)?;

    Ok(NativeProjectWireMutationReportView {
        action: "draw_wire".to_string(),
        project_root: project.root.display().to_string(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: sheet_path.display().to_string(),
        wire_uuid: wire_uuid.to_string(),
        from_x_nm: from.x,
        from_y_nm: from.y,
        to_x_nm: to.x,
        to_y_nm: to.y,
    })
}

pub(crate) fn delete_native_project_wire(
    root: &Path,
    wire_uuid: Uuid,
) -> Result<NativeProjectWireMutationReportView> {
    let project = load_native_project(root)?;
    let (sheet_uuid, sheet_path, mut sheet_value, wire) =
        load_native_wire_mutation_target(&project, wire_uuid)?;
    let wires = sheet_value
        .as_object_mut()
        .and_then(|object| object.get_mut("wires"))
        .and_then(serde_json::Value::as_object_mut)
        .ok_or_else(|| anyhow::anyhow!("sheet wires object missing in {}", sheet_path.display()))?;
    wires.remove(&wire_uuid.to_string());
    write_canonical_json(&sheet_path, &sheet_value)?;

    Ok(NativeProjectWireMutationReportView {
        action: "delete_wire".to_string(),
        project_root: project.root.display().to_string(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: sheet_path.display().to_string(),
        wire_uuid: wire.uuid.to_string(),
        from_x_nm: wire.from.x,
        from_y_nm: wire.from.y,
        to_x_nm: wire.to.x,
        to_y_nm: wire.to.y,
    })
}

pub(crate) fn place_native_project_junction(
    root: &Path,
    sheet_uuid: Uuid,
    position: Point,
) -> Result<NativeProjectJunctionMutationReportView> {
    let project = load_native_project(root)?;
    let sheet_key = sheet_uuid.to_string();
    let relative_path = project
        .schematic
        .sheets
        .get(&sheet_key)
        .ok_or_else(|| anyhow::anyhow!("sheet not found in native schematic root: {sheet_uuid}"))?;
    let sheet_path = project.root.join("schematic").join(relative_path);
    let sheet_text = std::fs::read_to_string(&sheet_path)
        .with_context(|| format!("failed to read {}", sheet_path.display()))?;
    let mut sheet_value: serde_json::Value = serde_json::from_str(&sheet_text)
        .with_context(|| format!("failed to parse {}", sheet_path.display()))?;
    let junctions = sheet_value
        .as_object_mut()
        .and_then(|object| object.get_mut("junctions"))
        .and_then(serde_json::Value::as_object_mut)
        .ok_or_else(|| anyhow::anyhow!("sheet junctions object missing in {}", sheet_path.display()))?;

    let junction_uuid = Uuid::new_v4();
    junctions.insert(
        junction_uuid.to_string(),
        serde_json::to_value(Junction {
            uuid: junction_uuid,
            position,
        })
        .expect("native junction serialization must succeed"),
    );

    write_canonical_json(&sheet_path, &sheet_value)?;

    Ok(NativeProjectJunctionMutationReportView {
        action: "place_junction".to_string(),
        project_root: project.root.display().to_string(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: sheet_path.display().to_string(),
        junction_uuid: junction_uuid.to_string(),
        x_nm: position.x,
        y_nm: position.y,
    })
}

pub(crate) fn delete_native_project_junction(
    root: &Path,
    junction_uuid: Uuid,
) -> Result<NativeProjectJunctionMutationReportView> {
    let project = load_native_project(root)?;
    let (sheet_uuid, sheet_path, mut sheet_value, junction) =
        load_native_junction_mutation_target(&project, junction_uuid)?;
    let junctions = sheet_value
        .as_object_mut()
        .and_then(|object| object.get_mut("junctions"))
        .and_then(serde_json::Value::as_object_mut)
        .ok_or_else(|| anyhow::anyhow!("sheet junctions object missing in {}", sheet_path.display()))?;
    junctions.remove(&junction_uuid.to_string());
    write_canonical_json(&sheet_path, &sheet_value)?;

    Ok(NativeProjectJunctionMutationReportView {
        action: "delete_junction".to_string(),
        project_root: project.root.display().to_string(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: sheet_path.display().to_string(),
        junction_uuid: junction.uuid.to_string(),
        x_nm: junction.position.x,
        y_nm: junction.position.y,
    })
}

pub(crate) fn place_native_project_port(
    root: &Path,
    sheet_uuid: Uuid,
    name: String,
    direction: PortDirection,
    position: Point,
) -> Result<NativeProjectPortMutationReportView> {
    let project = load_native_project(root)?;
    let sheet_key = sheet_uuid.to_string();
    let relative_path = project
        .schematic
        .sheets
        .get(&sheet_key)
        .ok_or_else(|| anyhow::anyhow!("sheet not found in native schematic root: {sheet_uuid}"))?;
    let sheet_path = project.root.join("schematic").join(relative_path);
    let sheet_text = std::fs::read_to_string(&sheet_path)
        .with_context(|| format!("failed to read {}", sheet_path.display()))?;
    let mut sheet_value: serde_json::Value = serde_json::from_str(&sheet_text)
        .with_context(|| format!("failed to parse {}", sheet_path.display()))?;
    let ports = sheet_value
        .as_object_mut()
        .and_then(|object| object.get_mut("ports"))
        .and_then(serde_json::Value::as_object_mut)
        .ok_or_else(|| anyhow::anyhow!("sheet ports object missing in {}", sheet_path.display()))?;

    let port_uuid = Uuid::new_v4();
    ports.insert(
        port_uuid.to_string(),
        serde_json::to_value(HierarchicalPort {
            uuid: port_uuid,
            name: name.clone(),
            direction: direction.clone(),
            position,
        })
        .expect("native port serialization must succeed"),
    );

    write_canonical_json(&sheet_path, &sheet_value)?;

    Ok(NativeProjectPortMutationReportView {
        action: "place_port".to_string(),
        project_root: project.root.display().to_string(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: sheet_path.display().to_string(),
        port_uuid: port_uuid.to_string(),
        name,
        direction: render_port_direction(&direction).to_string(),
        x_nm: position.x,
        y_nm: position.y,
    })
}

pub(crate) fn edit_native_project_port(
    root: &Path,
    port_uuid: Uuid,
    name: Option<String>,
    direction: Option<PortDirection>,
    x_nm: Option<i64>,
    y_nm: Option<i64>,
) -> Result<NativeProjectPortMutationReportView> {
    let project = load_native_project(root)?;
    let (sheet_uuid, sheet_path, mut sheet_value, mut port) =
        load_native_port_mutation_target(&project, port_uuid)?;
    if let Some(name) = name {
        port.name = name;
    }
    if let Some(direction) = direction {
        port.direction = direction;
    }
    if x_nm.is_some() || y_nm.is_some() {
        port.position = Point {
            x: x_nm.unwrap_or(port.position.x),
            y: y_nm.unwrap_or(port.position.y),
        };
    }
    write_port_into_sheet(&mut sheet_value, &port)?;
    write_canonical_json(&sheet_path, &sheet_value)?;

    Ok(NativeProjectPortMutationReportView {
        action: "edit_port".to_string(),
        project_root: project.root.display().to_string(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: sheet_path.display().to_string(),
        port_uuid: port.uuid.to_string(),
        name: port.name,
        direction: render_port_direction(&port.direction).to_string(),
        x_nm: port.position.x,
        y_nm: port.position.y,
    })
}

pub(crate) fn delete_native_project_port(
    root: &Path,
    port_uuid: Uuid,
) -> Result<NativeProjectPortMutationReportView> {
    let project = load_native_project(root)?;
    let (sheet_uuid, sheet_path, mut sheet_value, port) =
        load_native_port_mutation_target(&project, port_uuid)?;
    let ports = sheet_value
        .as_object_mut()
        .and_then(|object| object.get_mut("ports"))
        .and_then(serde_json::Value::as_object_mut)
        .ok_or_else(|| anyhow::anyhow!("sheet ports object missing in {}", sheet_path.display()))?;
    ports.remove(&port_uuid.to_string());
    write_canonical_json(&sheet_path, &sheet_value)?;

    Ok(NativeProjectPortMutationReportView {
        action: "delete_port".to_string(),
        project_root: project.root.display().to_string(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: sheet_path.display().to_string(),
        port_uuid: port.uuid.to_string(),
        name: port.name,
        direction: render_port_direction(&port.direction).to_string(),
        x_nm: port.position.x,
        y_nm: port.position.y,
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

struct LoadedNativeProject {
    root: std::path::PathBuf,
    manifest: NativeProjectManifest,
    schematic: NativeSchematicRoot,
    board: NativeBoardRoot,
    rules: NativeRulesRoot,
    schematic_path: std::path::PathBuf,
    board_path: std::path::PathBuf,
    rules_path: std::path::PathBuf,
}

struct NativeSchematicCounts {
    symbols: usize,
    wires: usize,
    junctions: usize,
    labels: usize,
    ports: usize,
    buses: usize,
    bus_entries: usize,
    noconnects: usize,
}

fn load_native_project(root: &Path) -> Result<LoadedNativeProject> {
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

    Ok(LoadedNativeProject {
        root,
        manifest,
        schematic,
        board,
        rules,
        schematic_path,
        board_path,
        rules_path,
    })
}

fn collect_schematic_counts(root: &Path, schematic: &NativeSchematicRoot) -> Result<NativeSchematicCounts> {
    let mut symbols = 0usize;
    let mut wires = 0usize;
    let mut junctions = 0usize;
    let mut labels = 0usize;
    let mut ports = 0usize;
    let mut buses = 0usize;
    let mut bus_entries = 0usize;
    let mut noconnects = 0usize;

    for sheet_path in schematic.sheets.values() {
        let path = root.join("schematic").join(sheet_path);
        let sheet_text = std::fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let sheet_value: serde_json::Value = serde_json::from_str(&sheet_text)
            .with_context(|| format!("failed to parse {}", path.display()))?;
        symbols += json_object_len(&sheet_value, "symbols");
        wires += json_object_len(&sheet_value, "wires");
        junctions += json_object_len(&sheet_value, "junctions");
        labels += json_object_len(&sheet_value, "labels");
        ports += json_object_len(&sheet_value, "ports");
        buses += json_object_len(&sheet_value, "buses");
        bus_entries += json_object_len(&sheet_value, "bus_entries");
        noconnects += json_object_len(&sheet_value, "noconnects");
    }

    Ok(NativeSchematicCounts {
        symbols,
        wires,
        junctions,
        labels,
        ports,
        buses,
        bus_entries,
        noconnects,
    })
}

fn json_object_len(value: &serde_json::Value, key: &str) -> usize {
    value
        .get(key)
        .and_then(serde_json::Value::as_object)
        .map(|items| items.len())
        .unwrap_or(0)
}

fn render_label_kind(kind: &LabelKind) -> &'static str {
    match kind {
        LabelKind::Local => "local",
        LabelKind::Global => "global",
        LabelKind::Hierarchical => "hierarchical",
        LabelKind::Power => "power",
    }
}

fn render_port_direction(direction: &PortDirection) -> &'static str {
    match direction {
        PortDirection::Input => "input",
        PortDirection::Output => "output",
        PortDirection::Bidirectional => "bidirectional",
        PortDirection::Passive => "passive",
    }
}

fn load_native_label_mutation_target(
    project: &LoadedNativeProject,
    label_uuid: Uuid,
) -> Result<(Uuid, std::path::PathBuf, serde_json::Value, NetLabel)> {
    for (sheet_uuid, relative_path) in &project.schematic.sheets {
        let parsed_sheet_uuid = Uuid::parse_str(sheet_uuid)
            .with_context(|| format!("invalid sheet UUID key `{sheet_uuid}` in schematic root"))?;
        let sheet_path = project.root.join("schematic").join(relative_path);
        let sheet_text = std::fs::read_to_string(&sheet_path)
            .with_context(|| format!("failed to read {}", sheet_path.display()))?;
        let sheet_value: serde_json::Value = serde_json::from_str(&sheet_text)
            .with_context(|| format!("failed to parse {}", sheet_path.display()))?;
        if let Some(entry) = sheet_value
            .get("labels")
            .and_then(serde_json::Value::as_object)
            .and_then(|labels| labels.get(&label_uuid.to_string()))
        {
            let label: NetLabel = serde_json::from_value(entry.clone())
                .with_context(|| format!("failed to parse label in {}", sheet_path.display()))?;
            return Ok((parsed_sheet_uuid, sheet_path, sheet_value, label));
        }
    }

    bail!("label not found in native project: {label_uuid}");
}

fn write_label_into_sheet(sheet_value: &mut serde_json::Value, label: &NetLabel) -> Result<()> {
    let labels = sheet_value
        .as_object_mut()
        .and_then(|object| object.get_mut("labels"))
        .and_then(serde_json::Value::as_object_mut)
        .ok_or_else(|| anyhow::anyhow!("sheet labels object missing during label mutation"))?;
    labels.insert(
        label.uuid.to_string(),
        serde_json::to_value(label).expect("native label serialization must succeed"),
    );
    Ok(())
}

fn load_native_wire_mutation_target(
    project: &LoadedNativeProject,
    wire_uuid: Uuid,
) -> Result<(Uuid, std::path::PathBuf, serde_json::Value, SchematicWire)> {
    for (sheet_uuid, relative_path) in &project.schematic.sheets {
        let parsed_sheet_uuid = Uuid::parse_str(sheet_uuid)
            .with_context(|| format!("invalid sheet UUID key `{sheet_uuid}` in schematic root"))?;
        let sheet_path = project.root.join("schematic").join(relative_path);
        let sheet_text = std::fs::read_to_string(&sheet_path)
            .with_context(|| format!("failed to read {}", sheet_path.display()))?;
        let sheet_value: serde_json::Value = serde_json::from_str(&sheet_text)
            .with_context(|| format!("failed to parse {}", sheet_path.display()))?;
        if let Some(entry) = sheet_value
            .get("wires")
            .and_then(serde_json::Value::as_object)
            .and_then(|wires| wires.get(&wire_uuid.to_string()))
        {
            let wire: SchematicWire = serde_json::from_value(entry.clone())
                .with_context(|| format!("failed to parse wire in {}", sheet_path.display()))?;
            return Ok((parsed_sheet_uuid, sheet_path, sheet_value, wire));
        }
    }

    bail!("wire not found in native project: {wire_uuid}");
}

fn load_native_junction_mutation_target(
    project: &LoadedNativeProject,
    junction_uuid: Uuid,
) -> Result<(Uuid, std::path::PathBuf, serde_json::Value, Junction)> {
    for (sheet_uuid, relative_path) in &project.schematic.sheets {
        let parsed_sheet_uuid = Uuid::parse_str(sheet_uuid)
            .with_context(|| format!("invalid sheet UUID key `{sheet_uuid}` in schematic root"))?;
        let sheet_path = project.root.join("schematic").join(relative_path);
        let sheet_text = std::fs::read_to_string(&sheet_path)
            .with_context(|| format!("failed to read {}", sheet_path.display()))?;
        let sheet_value: serde_json::Value = serde_json::from_str(&sheet_text)
            .with_context(|| format!("failed to parse {}", sheet_path.display()))?;
        if let Some(entry) = sheet_value
            .get("junctions")
            .and_then(serde_json::Value::as_object)
            .and_then(|junctions| junctions.get(&junction_uuid.to_string()))
        {
            let junction: Junction = serde_json::from_value(entry.clone()).with_context(|| {
                format!("failed to parse junction in {}", sheet_path.display())
            })?;
            return Ok((parsed_sheet_uuid, sheet_path, sheet_value, junction));
        }
    }

    bail!("junction not found in native project: {junction_uuid}");
}

fn load_native_port_mutation_target(
    project: &LoadedNativeProject,
    port_uuid: Uuid,
) -> Result<(Uuid, std::path::PathBuf, serde_json::Value, HierarchicalPort)> {
    for (sheet_uuid, relative_path) in &project.schematic.sheets {
        let parsed_sheet_uuid = Uuid::parse_str(sheet_uuid)
            .with_context(|| format!("invalid sheet UUID key `{sheet_uuid}` in schematic root"))?;
        let sheet_path = project.root.join("schematic").join(relative_path);
        let sheet_text = std::fs::read_to_string(&sheet_path)
            .with_context(|| format!("failed to read {}", sheet_path.display()))?;
        let sheet_value: serde_json::Value = serde_json::from_str(&sheet_text)
            .with_context(|| format!("failed to parse {}", sheet_path.display()))?;
        if let Some(entry) = sheet_value
            .get("ports")
            .and_then(serde_json::Value::as_object)
            .and_then(|ports| ports.get(&port_uuid.to_string()))
        {
            let port: HierarchicalPort = serde_json::from_value(entry.clone())
                .with_context(|| format!("failed to parse port in {}", sheet_path.display()))?;
            return Ok((parsed_sheet_uuid, sheet_path, sheet_value, port));
        }
    }

    bail!("port not found in native project: {port_uuid}");
}

fn write_port_into_sheet(
    sheet_value: &mut serde_json::Value,
    port: &HierarchicalPort,
) -> Result<()> {
    let ports = sheet_value
        .as_object_mut()
        .and_then(|object| object.get_mut("ports"))
        .and_then(serde_json::Value::as_object_mut)
        .ok_or_else(|| anyhow::anyhow!("sheet ports object missing during port mutation"))?;
    ports.insert(
        port.uuid.to_string(),
        serde_json::to_value(port).expect("native port serialization must succeed"),
    );
    Ok(())
}

fn write_canonical_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let json = to_json_deterministic(value).context("failed to serialize canonical JSON")?;
    std::fs::write(path, format!("{json}\n"))
        .with_context(|| format!("failed to write {}", path.display()))
}
