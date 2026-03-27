use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{Context, Result, bail};
use eda_engine::ir::serialization::to_json_deterministic;
use eda_engine::ir::geometry::{Arc, Point};
use eda_engine::schematic::{
    Bus, BusEntry, BusEntryInfo, BusInfo, HiddenPowerBehavior, HierarchicalPort, Junction,
    LabelInfo, LabelKind, NetLabel, NoConnectInfo, NoConnectMarker, PinDisplayOverride,
    PlacedSymbol, PortDirection, PortInfo, SchematicWire, SymbolDisplayMode, SymbolField,
    SymbolFieldInfo, SymbolInfo, SymbolPin, SchematicPrimitive, SchematicText,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{
    NativeProjectBoardSummaryView, NativeProjectCreateReportView, NativeProjectInspectReportView,
    NativeProjectBusEntryMutationReportView, NativeProjectBusMutationReportView,
    NativeProjectJunctionMutationReportView,
    NativeProjectLabelMutationReportView,
    NativeProjectNoConnectMutationReportView,
    NativeProjectPortMutationReportView,
    NativeProjectDrawingMutationReportView,
    NativeProjectPinOverrideMutationReportView,
    NativeProjectSymbolFieldMutationReportView,
    NativeProjectSymbolPinInfoView,
    NativeProjectSymbolSemanticsView,
    NativeProjectSymbolMutationReportView,
    NativeProjectTextMutationReportView,
    NativeProjectWireMutationReportView,
    NativeProjectRulesSummaryView, NativeProjectRulesView, NativeProjectSchematicSummaryView,
    NativeProjectSummaryView,
};

fn render_symbol_display_mode(mode: &SymbolDisplayMode) -> String {
    match mode {
        SymbolDisplayMode::LibraryDefault => "LibraryDefault",
        SymbolDisplayMode::ShowHiddenPins => "ShowHiddenPins",
        SymbolDisplayMode::HideOptionalPins => "HideOptionalPins",
    }
    .to_string()
}

fn render_hidden_power_behavior(mode: &HiddenPowerBehavior) -> String {
    match mode {
        HiddenPowerBehavior::SourceDefinedImplicit => "SourceDefinedImplicit",
        HiddenPowerBehavior::ExplicitPowerObject => "ExplicitPowerObject",
        HiddenPowerBehavior::PreservedAsImportedMetadata => "PreservedAsImportedMetadata",
    }
    .to_string()
}

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
            texts: schematic_counts.texts,
            drawings: schematic_counts.drawings,
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

pub(crate) fn query_native_project_symbols(root: &Path) -> Result<Vec<SymbolInfo>> {
    let project = load_native_project(root)?;
    let mut symbols = Vec::new();
    for (sheet_uuid, relative_path) in &project.schematic.sheets {
        let sheet_uuid = Uuid::parse_str(sheet_uuid)
            .with_context(|| format!("invalid sheet UUID key `{sheet_uuid}` in schematic root"))?;
        let path = project.root.join("schematic").join(relative_path);
        let sheet_text = std::fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let sheet_value: serde_json::Value = serde_json::from_str(&sheet_text)
            .with_context(|| format!("failed to parse {}", path.display()))?;
        if let Some(entries) = sheet_value.get("symbols").and_then(serde_json::Value::as_object) {
            for value in entries.values() {
                let symbol: PlacedSymbol = serde_json::from_value(value.clone())
                    .with_context(|| format!("failed to parse symbol in {}", path.display()))?;
                symbols.push(SymbolInfo {
                    uuid: symbol.uuid,
                    sheet: sheet_uuid,
                    reference: symbol.reference,
                    value: symbol.value,
                    position: symbol.position,
                    rotation: symbol.rotation,
                    mirrored: symbol.mirrored,
                    part_uuid: symbol.part,
                    entity_uuid: symbol.entity,
                    gate_uuid: symbol.gate,
                });
            }
        }
    }
    symbols.sort_by(|a, b| a.reference.cmp(&b.reference).then_with(|| a.uuid.cmp(&b.uuid)));
    Ok(symbols)
}

pub(crate) fn query_native_project_symbol_fields(
    root: &Path,
    symbol_uuid: Uuid,
) -> Result<Vec<SymbolFieldInfo>> {
    let project = load_native_project(root)?;
    let (_, _, _, symbol) = load_native_symbol_mutation_target(&project, symbol_uuid)?;
    let mut fields = symbol
        .fields
        .into_iter()
        .map(|field| SymbolFieldInfo {
            uuid: field.uuid,
            symbol: symbol_uuid,
            key: field.key,
            value: field.value,
            visible: field.visible,
            position: field.position,
        })
        .collect::<Vec<_>>();
    fields.sort_by(|a, b| a.key.cmp(&b.key).then_with(|| a.uuid.cmp(&b.uuid)));
    Ok(fields)
}

pub(crate) fn query_native_project_symbol_semantics(
    root: &Path,
    symbol_uuid: Uuid,
) -> Result<NativeProjectSymbolSemanticsView> {
    let project = load_native_project(root)?;
    let (_, _, _, symbol) = load_native_symbol_mutation_target(&project, symbol_uuid)?;
    Ok(NativeProjectSymbolSemanticsView {
        symbol_uuid: symbol.uuid.to_string(),
        gate_uuid: symbol.gate.map(|uuid| uuid.to_string()),
        unit_selection: symbol.unit_selection,
        hidden_power_behavior: render_hidden_power_behavior(&symbol.hidden_power_behavior),
    })
}

pub(crate) fn query_native_project_symbol_pins(
    root: &Path,
    symbol_uuid: Uuid,
) -> Result<Vec<NativeProjectSymbolPinInfoView>> {
    let project = load_native_project(root)?;
    let (_, _, _, symbol) = load_native_symbol_mutation_target(&project, symbol_uuid)?;
    let mut pins = symbol
        .pins
        .into_iter()
        .map(|pin| {
            let pin_override = symbol.pin_overrides.iter().find(|entry| entry.pin == pin.uuid);
            NativeProjectSymbolPinInfoView {
                symbol_uuid: symbol_uuid.to_string(),
                pin_uuid: pin.uuid.to_string(),
                number: pin.number,
                name: pin.name,
                electrical_type: format!("{:?}", pin.electrical_type),
                x_nm: pin.position.x,
                y_nm: pin.position.y,
                visible_override: pin_override.map(|entry| entry.visible),
                override_x_nm: pin_override.and_then(|entry| entry.position.map(|p| p.x)),
                override_y_nm: pin_override.and_then(|entry| entry.position.map(|p| p.y)),
            }
        })
        .collect::<Vec<_>>();
    pins.sort_by(|a, b| a.number.cmp(&b.number).then_with(|| a.pin_uuid.cmp(&b.pin_uuid)));
    Ok(pins)
}

pub(crate) fn query_native_project_texts(root: &Path) -> Result<Vec<serde_json::Value>> {
    let project = load_native_project(root)?;
    let mut texts = Vec::new();
    for (sheet_uuid, relative_path) in &project.schematic.sheets {
        let sheet_uuid = Uuid::parse_str(sheet_uuid)
            .with_context(|| format!("invalid sheet UUID key `{sheet_uuid}` in schematic root"))?;
        let path = project.root.join("schematic").join(relative_path);
        let sheet_text = std::fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let sheet_value: serde_json::Value = serde_json::from_str(&sheet_text)
            .with_context(|| format!("failed to parse {}", path.display()))?;
        if let Some(entries) = sheet_value.get("texts").and_then(serde_json::Value::as_object) {
            for value in entries.values() {
                let text: SchematicText = serde_json::from_value(value.clone())
                    .with_context(|| format!("failed to parse text in {}", path.display()))?;
                texts.push(serde_json::json!({
                    "uuid": text.uuid,
                    "sheet": sheet_uuid,
                    "text": text.text,
                    "position": text.position,
                    "rotation": text.rotation,
                }));
            }
        }
    }
    texts.sort_by(|a, b| {
        a.get("uuid")
            .and_then(serde_json::Value::as_str)
            .cmp(&b.get("uuid").and_then(serde_json::Value::as_str))
    });
    Ok(texts)
}

pub(crate) fn query_native_project_drawings(root: &Path) -> Result<Vec<serde_json::Value>> {
    let project = load_native_project(root)?;
    let mut drawings = Vec::new();
    for (sheet_uuid, relative_path) in &project.schematic.sheets {
        let sheet_uuid = Uuid::parse_str(sheet_uuid)
            .with_context(|| format!("invalid sheet UUID key `{sheet_uuid}` in schematic root"))?;
        let path = project.root.join("schematic").join(relative_path);
        let sheet_text = std::fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let sheet_value: serde_json::Value = serde_json::from_str(&sheet_text)
            .with_context(|| format!("failed to parse {}", path.display()))?;
        if let Some(entries) = sheet_value
            .get("drawings")
            .and_then(serde_json::Value::as_object)
        {
            for value in entries.values() {
                let primitive: SchematicPrimitive = serde_json::from_value(value.clone())
                    .with_context(|| format!("failed to parse drawing in {}", path.display()))?;
                if let Some(view) = render_drawing_query_view(sheet_uuid, primitive) {
                    drawings.push(view);
                }
            }
        }
    }
    drawings.sort_by(|a, b| {
        a.get("uuid")
            .and_then(serde_json::Value::as_str)
            .cmp(&b.get("uuid").and_then(serde_json::Value::as_str))
    });
    Ok(drawings)
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

pub(crate) fn query_native_project_buses(root: &Path) -> Result<Vec<BusInfo>> {
    let project = load_native_project(root)?;
    let mut buses = Vec::new();
    for (sheet_uuid, relative_path) in &project.schematic.sheets {
        let sheet_uuid = Uuid::parse_str(sheet_uuid)
            .with_context(|| format!("invalid sheet UUID key `{sheet_uuid}` in schematic root"))?;
        let path = project.root.join("schematic").join(relative_path);
        let sheet_text = std::fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let sheet_value: serde_json::Value = serde_json::from_str(&sheet_text)
            .with_context(|| format!("failed to parse {}", path.display()))?;
        if let Some(entries) = sheet_value.get("buses").and_then(serde_json::Value::as_object) {
            for value in entries.values() {
                let bus: Bus = serde_json::from_value(value.clone())
                    .with_context(|| format!("failed to parse bus in {}", path.display()))?;
                buses.push(BusInfo {
                    uuid: bus.uuid,
                    sheet: sheet_uuid,
                    name: bus.name,
                    members: bus.members,
                });
            }
        }
    }
    buses.sort_by(|a, b| a.name.cmp(&b.name).then_with(|| a.uuid.cmp(&b.uuid)));
    Ok(buses)
}

pub(crate) fn query_native_project_bus_entries(root: &Path) -> Result<Vec<BusEntryInfo>> {
    let project = load_native_project(root)?;
    let mut entries = Vec::new();
    for (sheet_uuid, relative_path) in &project.schematic.sheets {
        let sheet_uuid = Uuid::parse_str(sheet_uuid)
            .with_context(|| format!("invalid sheet UUID key `{sheet_uuid}` in schematic root"))?;
        let path = project.root.join("schematic").join(relative_path);
        let sheet_text = std::fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let sheet_value: serde_json::Value = serde_json::from_str(&sheet_text)
            .with_context(|| format!("failed to parse {}", path.display()))?;
        if let Some(values) = sheet_value.get("bus_entries").and_then(serde_json::Value::as_object)
        {
            for value in values.values() {
                let entry: BusEntry = serde_json::from_value(value.clone())
                    .with_context(|| format!("failed to parse bus entry in {}", path.display()))?;
                entries.push(BusEntryInfo {
                    uuid: entry.uuid,
                    sheet: sheet_uuid,
                    bus: entry.bus,
                    wire: entry.wire,
                    position: entry.position,
                });
            }
        }
    }
    entries.sort_by(|a, b| a.uuid.cmp(&b.uuid));
    Ok(entries)
}

pub(crate) fn query_native_project_noconnects(root: &Path) -> Result<Vec<NoConnectInfo>> {
    let project = load_native_project(root)?;
    let mut noconnects = Vec::new();
    for (sheet_uuid, relative_path) in &project.schematic.sheets {
        let sheet_uuid = Uuid::parse_str(sheet_uuid)
            .with_context(|| format!("invalid sheet UUID key `{sheet_uuid}` in schematic root"))?;
        let path = project.root.join("schematic").join(relative_path);
        let sheet_text = std::fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let sheet_value: serde_json::Value = serde_json::from_str(&sheet_text)
            .with_context(|| format!("failed to parse {}", path.display()))?;
        if let Some(values) = sheet_value
            .get("noconnects")
            .and_then(serde_json::Value::as_object)
        {
            for value in values.values() {
                let marker: NoConnectMarker = serde_json::from_value(value.clone())
                    .with_context(|| format!("failed to parse no-connect in {}", path.display()))?;
                noconnects.push(NoConnectInfo {
                    uuid: marker.uuid,
                    sheet: sheet_uuid,
                    symbol: marker.symbol,
                    pin: marker.pin,
                    position: marker.position,
                });
            }
        }
    }
    noconnects.sort_by(|a, b| a.uuid.cmp(&b.uuid));
    Ok(noconnects)
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

pub(crate) fn place_native_project_symbol(
    root: &Path,
    sheet_uuid: Uuid,
    reference: String,
    value: String,
    lib_id: Option<String>,
    position: Point,
    rotation_deg: i32,
    mirrored: bool,
) -> Result<NativeProjectSymbolMutationReportView> {
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
    let symbols = sheet_value
        .as_object_mut()
        .and_then(|object| object.get_mut("symbols"))
        .and_then(serde_json::Value::as_object_mut)
        .ok_or_else(|| anyhow::anyhow!("sheet symbols object missing in {}", sheet_path.display()))?;

    let symbol_uuid = Uuid::new_v4();
    symbols.insert(
        symbol_uuid.to_string(),
        serde_json::to_value(PlacedSymbol {
            uuid: symbol_uuid,
            part: None,
            entity: None,
            gate: None,
            lib_id: lib_id.clone(),
            reference: reference.clone(),
            value: value.clone(),
            fields: Vec::<SymbolField>::new(),
            pins: Vec::<SymbolPin>::new(),
            position,
            rotation: rotation_deg,
            mirrored,
            unit_selection: None,
            display_mode: SymbolDisplayMode::LibraryDefault,
            pin_overrides: Vec::<PinDisplayOverride>::new(),
            hidden_power_behavior: HiddenPowerBehavior::SourceDefinedImplicit,
        })
        .expect("native symbol serialization must succeed"),
    );

    write_canonical_json(&sheet_path, &sheet_value)?;

    Ok(NativeProjectSymbolMutationReportView {
        action: "place_symbol".to_string(),
        project_root: project.root.display().to_string(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: sheet_path.display().to_string(),
        symbol_uuid: symbol_uuid.to_string(),
        reference,
        value,
        lib_id,
        x_nm: position.x,
        y_nm: position.y,
        rotation_deg,
        mirrored,
        gate_uuid: None,
        unit_selection: None,
        display_mode: render_symbol_display_mode(&SymbolDisplayMode::LibraryDefault),
        hidden_power_behavior: render_hidden_power_behavior(&HiddenPowerBehavior::SourceDefinedImplicit),
    })
}

pub(crate) fn move_native_project_symbol(
    root: &Path,
    symbol_uuid: Uuid,
    position: Point,
) -> Result<NativeProjectSymbolMutationReportView> {
    let project = load_native_project(root)?;
    let (sheet_uuid, sheet_path, mut sheet_value, mut symbol) =
        load_native_symbol_mutation_target(&project, symbol_uuid)?;
    symbol.position = position;
    write_symbol_into_sheet(&mut sheet_value, &symbol)?;
    write_canonical_json(&sheet_path, &sheet_value)?;

    Ok(NativeProjectSymbolMutationReportView {
        action: "move_symbol".to_string(),
        project_root: project.root.display().to_string(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: sheet_path.display().to_string(),
        symbol_uuid: symbol.uuid.to_string(),
        reference: symbol.reference,
        value: symbol.value,
        lib_id: symbol.lib_id,
        x_nm: position.x,
        y_nm: position.y,
        rotation_deg: symbol.rotation,
        mirrored: symbol.mirrored,
        gate_uuid: symbol.gate.map(|uuid| uuid.to_string()),
        unit_selection: symbol.unit_selection,
        display_mode: render_symbol_display_mode(&symbol.display_mode),
        hidden_power_behavior: render_hidden_power_behavior(&symbol.hidden_power_behavior),
    })
}

pub(crate) fn rotate_native_project_symbol(
    root: &Path,
    symbol_uuid: Uuid,
    rotation_deg: i32,
) -> Result<NativeProjectSymbolMutationReportView> {
    let project = load_native_project(root)?;
    let (sheet_uuid, sheet_path, mut sheet_value, mut symbol) =
        load_native_symbol_mutation_target(&project, symbol_uuid)?;
    symbol.rotation = rotation_deg;
    write_symbol_into_sheet(&mut sheet_value, &symbol)?;
    write_canonical_json(&sheet_path, &sheet_value)?;

    Ok(NativeProjectSymbolMutationReportView {
        action: "rotate_symbol".to_string(),
        project_root: project.root.display().to_string(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: sheet_path.display().to_string(),
        symbol_uuid: symbol.uuid.to_string(),
        reference: symbol.reference,
        value: symbol.value,
        lib_id: symbol.lib_id,
        x_nm: symbol.position.x,
        y_nm: symbol.position.y,
        rotation_deg,
        mirrored: symbol.mirrored,
        gate_uuid: symbol.gate.map(|uuid| uuid.to_string()),
        unit_selection: symbol.unit_selection,
        display_mode: render_symbol_display_mode(&symbol.display_mode),
        hidden_power_behavior: render_hidden_power_behavior(&symbol.hidden_power_behavior),
    })
}

pub(crate) fn mirror_native_project_symbol(
    root: &Path,
    symbol_uuid: Uuid,
) -> Result<NativeProjectSymbolMutationReportView> {
    let project = load_native_project(root)?;
    let (sheet_uuid, sheet_path, mut sheet_value, mut symbol) =
        load_native_symbol_mutation_target(&project, symbol_uuid)?;
    symbol.mirrored = !symbol.mirrored;
    write_symbol_into_sheet(&mut sheet_value, &symbol)?;
    write_canonical_json(&sheet_path, &sheet_value)?;

    Ok(NativeProjectSymbolMutationReportView {
        action: "mirror_symbol".to_string(),
        project_root: project.root.display().to_string(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: sheet_path.display().to_string(),
        symbol_uuid: symbol.uuid.to_string(),
        reference: symbol.reference,
        value: symbol.value,
        lib_id: symbol.lib_id,
        x_nm: symbol.position.x,
        y_nm: symbol.position.y,
        rotation_deg: symbol.rotation,
        mirrored: symbol.mirrored,
        gate_uuid: symbol.gate.map(|uuid| uuid.to_string()),
        unit_selection: symbol.unit_selection,
        display_mode: render_symbol_display_mode(&symbol.display_mode),
        hidden_power_behavior: render_hidden_power_behavior(&symbol.hidden_power_behavior),
    })
}

pub(crate) fn delete_native_project_symbol(
    root: &Path,
    symbol_uuid: Uuid,
) -> Result<NativeProjectSymbolMutationReportView> {
    let project = load_native_project(root)?;
    let (sheet_uuid, sheet_path, mut sheet_value, symbol) =
        load_native_symbol_mutation_target(&project, symbol_uuid)?;
    let symbols = sheet_value
        .as_object_mut()
        .and_then(|object| object.get_mut("symbols"))
        .and_then(serde_json::Value::as_object_mut)
        .ok_or_else(|| anyhow::anyhow!("sheet symbols object missing in {}", sheet_path.display()))?;
    symbols.remove(&symbol_uuid.to_string());
    write_canonical_json(&sheet_path, &sheet_value)?;

    Ok(NativeProjectSymbolMutationReportView {
        action: "delete_symbol".to_string(),
        project_root: project.root.display().to_string(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: sheet_path.display().to_string(),
        symbol_uuid: symbol.uuid.to_string(),
        reference: symbol.reference,
        value: symbol.value,
        lib_id: symbol.lib_id,
        x_nm: symbol.position.x,
        y_nm: symbol.position.y,
        rotation_deg: symbol.rotation,
        mirrored: symbol.mirrored,
        gate_uuid: symbol.gate.map(|uuid| uuid.to_string()),
        unit_selection: symbol.unit_selection,
        display_mode: render_symbol_display_mode(&symbol.display_mode),
        hidden_power_behavior: render_hidden_power_behavior(&symbol.hidden_power_behavior),
    })
}

pub(crate) fn set_native_project_symbol_reference(
    root: &Path,
    symbol_uuid: Uuid,
    reference: String,
) -> Result<NativeProjectSymbolMutationReportView> {
    let project = load_native_project(root)?;
    let (sheet_uuid, sheet_path, mut sheet_value, mut symbol) =
        load_native_symbol_mutation_target(&project, symbol_uuid)?;
    symbol.reference = reference;
    write_symbol_into_sheet(&mut sheet_value, &symbol)?;
    write_canonical_json(&sheet_path, &sheet_value)?;

    Ok(NativeProjectSymbolMutationReportView {
        action: "set_symbol_reference".to_string(),
        project_root: project.root.display().to_string(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: sheet_path.display().to_string(),
        symbol_uuid: symbol.uuid.to_string(),
        reference: symbol.reference,
        value: symbol.value,
        lib_id: symbol.lib_id,
        x_nm: symbol.position.x,
        y_nm: symbol.position.y,
        rotation_deg: symbol.rotation,
        mirrored: symbol.mirrored,
        gate_uuid: symbol.gate.map(|uuid| uuid.to_string()),
        unit_selection: symbol.unit_selection,
        display_mode: render_symbol_display_mode(&symbol.display_mode),
        hidden_power_behavior: render_hidden_power_behavior(&symbol.hidden_power_behavior),
    })
}

pub(crate) fn set_native_project_symbol_value(
    root: &Path,
    symbol_uuid: Uuid,
    value: String,
) -> Result<NativeProjectSymbolMutationReportView> {
    let project = load_native_project(root)?;
    let (sheet_uuid, sheet_path, mut sheet_value, mut symbol) =
        load_native_symbol_mutation_target(&project, symbol_uuid)?;
    symbol.value = value;
    write_symbol_into_sheet(&mut sheet_value, &symbol)?;
    write_canonical_json(&sheet_path, &sheet_value)?;

    Ok(NativeProjectSymbolMutationReportView {
        action: "set_symbol_value".to_string(),
        project_root: project.root.display().to_string(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: sheet_path.display().to_string(),
        symbol_uuid: symbol.uuid.to_string(),
        reference: symbol.reference,
        value: symbol.value,
        lib_id: symbol.lib_id,
        x_nm: symbol.position.x,
        y_nm: symbol.position.y,
        rotation_deg: symbol.rotation,
        mirrored: symbol.mirrored,
        gate_uuid: symbol.gate.map(|uuid| uuid.to_string()),
        unit_selection: symbol.unit_selection,
        display_mode: render_symbol_display_mode(&symbol.display_mode),
        hidden_power_behavior: render_hidden_power_behavior(&symbol.hidden_power_behavior),
    })
}

pub(crate) fn set_native_project_symbol_unit(
    root: &Path,
    symbol_uuid: Uuid,
    unit_selection: String,
) -> Result<NativeProjectSymbolMutationReportView> {
    let project = load_native_project(root)?;
    let (sheet_uuid, sheet_path, mut sheet_value, mut symbol) =
        load_native_symbol_mutation_target(&project, symbol_uuid)?;
    symbol.unit_selection = Some(unit_selection);
    write_symbol_into_sheet(&mut sheet_value, &symbol)?;
    write_canonical_json(&sheet_path, &sheet_value)?;

    Ok(NativeProjectSymbolMutationReportView {
        action: "set_symbol_unit".to_string(),
        project_root: project.root.display().to_string(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: sheet_path.display().to_string(),
        symbol_uuid: symbol.uuid.to_string(),
        reference: symbol.reference,
        value: symbol.value,
        lib_id: symbol.lib_id,
        x_nm: symbol.position.x,
        y_nm: symbol.position.y,
        rotation_deg: symbol.rotation,
        mirrored: symbol.mirrored,
        gate_uuid: symbol.gate.map(|uuid| uuid.to_string()),
        unit_selection: symbol.unit_selection,
        display_mode: render_symbol_display_mode(&symbol.display_mode),
        hidden_power_behavior: render_hidden_power_behavior(&symbol.hidden_power_behavior),
    })
}

pub(crate) fn clear_native_project_symbol_unit(
    root: &Path,
    symbol_uuid: Uuid,
) -> Result<NativeProjectSymbolMutationReportView> {
    let project = load_native_project(root)?;
    let (sheet_uuid, sheet_path, mut sheet_value, mut symbol) =
        load_native_symbol_mutation_target(&project, symbol_uuid)?;
    symbol.unit_selection = None;
    write_symbol_into_sheet(&mut sheet_value, &symbol)?;
    write_canonical_json(&sheet_path, &sheet_value)?;

    Ok(NativeProjectSymbolMutationReportView {
        action: "clear_symbol_unit".to_string(),
        project_root: project.root.display().to_string(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: sheet_path.display().to_string(),
        symbol_uuid: symbol.uuid.to_string(),
        reference: symbol.reference,
        value: symbol.value,
        lib_id: symbol.lib_id,
        x_nm: symbol.position.x,
        y_nm: symbol.position.y,
        rotation_deg: symbol.rotation,
        mirrored: symbol.mirrored,
        gate_uuid: symbol.gate.map(|uuid| uuid.to_string()),
        unit_selection: symbol.unit_selection,
        display_mode: render_symbol_display_mode(&symbol.display_mode),
        hidden_power_behavior: render_hidden_power_behavior(&symbol.hidden_power_behavior),
    })
}

pub(crate) fn set_native_project_symbol_gate(
    root: &Path,
    symbol_uuid: Uuid,
    gate_uuid: Uuid,
) -> Result<NativeProjectSymbolMutationReportView> {
    let project = load_native_project(root)?;
    let (sheet_uuid, sheet_path, mut sheet_value, mut symbol) =
        load_native_symbol_mutation_target(&project, symbol_uuid)?;
    symbol.gate = Some(gate_uuid);
    write_symbol_into_sheet(&mut sheet_value, &symbol)?;
    write_canonical_json(&sheet_path, &sheet_value)?;

    Ok(NativeProjectSymbolMutationReportView {
        action: "set_symbol_gate".to_string(),
        project_root: project.root.display().to_string(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: sheet_path.display().to_string(),
        symbol_uuid: symbol.uuid.to_string(),
        reference: symbol.reference,
        value: symbol.value,
        lib_id: symbol.lib_id,
        x_nm: symbol.position.x,
        y_nm: symbol.position.y,
        rotation_deg: symbol.rotation,
        mirrored: symbol.mirrored,
        gate_uuid: symbol.gate.map(|uuid| uuid.to_string()),
        unit_selection: symbol.unit_selection,
        display_mode: render_symbol_display_mode(&symbol.display_mode),
        hidden_power_behavior: render_hidden_power_behavior(&symbol.hidden_power_behavior),
    })
}

pub(crate) fn clear_native_project_symbol_gate(
    root: &Path,
    symbol_uuid: Uuid,
) -> Result<NativeProjectSymbolMutationReportView> {
    let project = load_native_project(root)?;
    let (sheet_uuid, sheet_path, mut sheet_value, mut symbol) =
        load_native_symbol_mutation_target(&project, symbol_uuid)?;
    symbol.gate = None;
    write_symbol_into_sheet(&mut sheet_value, &symbol)?;
    write_canonical_json(&sheet_path, &sheet_value)?;

    Ok(NativeProjectSymbolMutationReportView {
        action: "clear_symbol_gate".to_string(),
        project_root: project.root.display().to_string(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: sheet_path.display().to_string(),
        symbol_uuid: symbol.uuid.to_string(),
        reference: symbol.reference,
        value: symbol.value,
        lib_id: symbol.lib_id,
        x_nm: symbol.position.x,
        y_nm: symbol.position.y,
        rotation_deg: symbol.rotation,
        mirrored: symbol.mirrored,
        gate_uuid: symbol.gate.map(|uuid| uuid.to_string()),
        unit_selection: symbol.unit_selection,
        display_mode: render_symbol_display_mode(&symbol.display_mode),
        hidden_power_behavior: render_hidden_power_behavior(&symbol.hidden_power_behavior),
    })
}

pub(crate) fn set_native_project_symbol_display_mode(
    root: &Path,
    symbol_uuid: Uuid,
    display_mode: SymbolDisplayMode,
) -> Result<NativeProjectSymbolMutationReportView> {
    let project = load_native_project(root)?;
    let (sheet_uuid, sheet_path, mut sheet_value, mut symbol) =
        load_native_symbol_mutation_target(&project, symbol_uuid)?;
    symbol.display_mode = display_mode;
    write_symbol_into_sheet(&mut sheet_value, &symbol)?;
    write_canonical_json(&sheet_path, &sheet_value)?;

    Ok(NativeProjectSymbolMutationReportView {
        action: "set_symbol_display_mode".to_string(),
        project_root: project.root.display().to_string(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: sheet_path.display().to_string(),
        symbol_uuid: symbol.uuid.to_string(),
        reference: symbol.reference,
        value: symbol.value,
        lib_id: symbol.lib_id,
        x_nm: symbol.position.x,
        y_nm: symbol.position.y,
        rotation_deg: symbol.rotation,
        mirrored: symbol.mirrored,
        gate_uuid: symbol.gate.map(|uuid| uuid.to_string()),
        unit_selection: symbol.unit_selection,
        display_mode: render_symbol_display_mode(&symbol.display_mode),
        hidden_power_behavior: render_hidden_power_behavior(&symbol.hidden_power_behavior),
    })
}

pub(crate) fn set_native_project_symbol_hidden_power_behavior(
    root: &Path,
    symbol_uuid: Uuid,
    hidden_power_behavior: HiddenPowerBehavior,
) -> Result<NativeProjectSymbolMutationReportView> {
    let project = load_native_project(root)?;
    let (sheet_uuid, sheet_path, mut sheet_value, mut symbol) =
        load_native_symbol_mutation_target(&project, symbol_uuid)?;
    symbol.hidden_power_behavior = hidden_power_behavior;
    write_symbol_into_sheet(&mut sheet_value, &symbol)?;
    write_canonical_json(&sheet_path, &sheet_value)?;

    Ok(NativeProjectSymbolMutationReportView {
        action: "set_symbol_hidden_power_behavior".to_string(),
        project_root: project.root.display().to_string(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: sheet_path.display().to_string(),
        symbol_uuid: symbol.uuid.to_string(),
        reference: symbol.reference,
        value: symbol.value,
        lib_id: symbol.lib_id,
        x_nm: symbol.position.x,
        y_nm: symbol.position.y,
        rotation_deg: symbol.rotation,
        mirrored: symbol.mirrored,
        gate_uuid: symbol.gate.map(|uuid| uuid.to_string()),
        unit_selection: symbol.unit_selection,
        display_mode: render_symbol_display_mode(&symbol.display_mode),
        hidden_power_behavior: render_hidden_power_behavior(&symbol.hidden_power_behavior),
    })
}

pub(crate) fn set_native_project_symbol_pin_override(
    root: &Path,
    symbol_uuid: Uuid,
    pin_uuid: Uuid,
    visible: bool,
    position: Option<Point>,
) -> Result<NativeProjectPinOverrideMutationReportView> {
    let project = load_native_project(root)?;
    let (sheet_uuid, sheet_path, mut sheet_value, mut symbol) =
        load_native_symbol_mutation_target(&project, symbol_uuid)?;
    if !symbol.pins.iter().any(|pin| pin.uuid == pin_uuid) {
        bail!("pin not found on native symbol: {pin_uuid}");
    }
    if let Some(entry) = symbol.pin_overrides.iter_mut().find(|entry| entry.pin == pin_uuid) {
        entry.visible = visible;
        entry.position = position;
    } else {
        symbol.pin_overrides.push(PinDisplayOverride {
            pin: pin_uuid,
            visible,
            position,
        });
    }
    write_symbol_into_sheet(&mut sheet_value, &symbol)?;
    write_canonical_json(&sheet_path, &sheet_value)?;

    Ok(NativeProjectPinOverrideMutationReportView {
        action: "set_pin_override".to_string(),
        project_root: project.root.display().to_string(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: sheet_path.display().to_string(),
        symbol_uuid: symbol.uuid.to_string(),
        pin_uuid: pin_uuid.to_string(),
        visible: Some(visible),
        x_nm: position.map(|point| point.x),
        y_nm: position.map(|point| point.y),
    })
}

pub(crate) fn clear_native_project_symbol_pin_override(
    root: &Path,
    symbol_uuid: Uuid,
    pin_uuid: Uuid,
) -> Result<NativeProjectPinOverrideMutationReportView> {
    let project = load_native_project(root)?;
    let (sheet_uuid, sheet_path, mut sheet_value, mut symbol) =
        load_native_symbol_mutation_target(&project, symbol_uuid)?;
    let before = symbol.pin_overrides.len();
    symbol.pin_overrides.retain(|entry| entry.pin != pin_uuid);
    if symbol.pin_overrides.len() == before {
        bail!("pin override not found on native symbol: {pin_uuid}");
    }
    write_symbol_into_sheet(&mut sheet_value, &symbol)?;
    write_canonical_json(&sheet_path, &sheet_value)?;

    Ok(NativeProjectPinOverrideMutationReportView {
        action: "clear_pin_override".to_string(),
        project_root: project.root.display().to_string(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: sheet_path.display().to_string(),
        symbol_uuid: symbol.uuid.to_string(),
        pin_uuid: pin_uuid.to_string(),
        visible: None,
        x_nm: None,
        y_nm: None,
    })
}

pub(crate) fn add_native_project_symbol_field(
    root: &Path,
    symbol_uuid: Uuid,
    key: String,
    value: String,
    visible: bool,
    position: Option<Point>,
) -> Result<NativeProjectSymbolFieldMutationReportView> {
    let project = load_native_project(root)?;
    let (sheet_uuid, sheet_path, mut sheet_value, mut symbol) =
        load_native_symbol_mutation_target(&project, symbol_uuid)?;
    let field_uuid = Uuid::new_v4();
    symbol.fields.push(SymbolField {
        uuid: field_uuid,
        key: key.clone(),
        value: value.clone(),
        position,
        visible,
    });
    write_symbol_into_sheet(&mut sheet_value, &symbol)?;
    write_canonical_json(&sheet_path, &sheet_value)?;

    Ok(NativeProjectSymbolFieldMutationReportView {
        action: "add_symbol_field".to_string(),
        project_root: project.root.display().to_string(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: sheet_path.display().to_string(),
        symbol_uuid: symbol.uuid.to_string(),
        field_uuid: field_uuid.to_string(),
        key,
        value,
        visible,
        x_nm: position.map(|point| point.x),
        y_nm: position.map(|point| point.y),
    })
}

pub(crate) fn edit_native_project_symbol_field(
    root: &Path,
    field_uuid: Uuid,
    key: Option<String>,
    value: Option<String>,
    visible: Option<bool>,
    position: Option<Point>,
) -> Result<NativeProjectSymbolFieldMutationReportView> {
    let project = load_native_project(root)?;
    let (sheet_uuid, sheet_path, mut sheet_value, symbol_uuid, mut symbol, mut field) =
        load_native_field_mutation_target(&project, field_uuid)?;
    if let Some(key) = key {
        field.key = key;
    }
    if let Some(value) = value {
        field.value = value;
    }
    if let Some(visible) = visible {
        field.visible = visible;
    }
    if let Some(position) = position {
        field.position = Some(position);
    }
    for existing in &mut symbol.fields {
        if existing.uuid == field_uuid {
            *existing = field.clone();
            break;
        }
    }
    write_symbol_into_sheet(&mut sheet_value, &symbol)?;
    write_canonical_json(&sheet_path, &sheet_value)?;

    Ok(NativeProjectSymbolFieldMutationReportView {
        action: "edit_symbol_field".to_string(),
        project_root: project.root.display().to_string(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: sheet_path.display().to_string(),
        symbol_uuid: symbol_uuid.to_string(),
        field_uuid: field.uuid.to_string(),
        key: field.key,
        value: field.value,
        visible: field.visible,
        x_nm: field.position.map(|point| point.x),
        y_nm: field.position.map(|point| point.y),
    })
}

pub(crate) fn delete_native_project_symbol_field(
    root: &Path,
    field_uuid: Uuid,
) -> Result<NativeProjectSymbolFieldMutationReportView> {
    let project = load_native_project(root)?;
    let (sheet_uuid, sheet_path, mut sheet_value, symbol_uuid, mut symbol, field) =
        load_native_field_mutation_target(&project, field_uuid)?;
    symbol.fields.retain(|existing| existing.uuid != field_uuid);
    write_symbol_into_sheet(&mut sheet_value, &symbol)?;
    write_canonical_json(&sheet_path, &sheet_value)?;

    Ok(NativeProjectSymbolFieldMutationReportView {
        action: "delete_symbol_field".to_string(),
        project_root: project.root.display().to_string(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: sheet_path.display().to_string(),
        symbol_uuid: symbol_uuid.to_string(),
        field_uuid: field.uuid.to_string(),
        key: field.key,
        value: field.value,
        visible: field.visible,
        x_nm: field.position.map(|point| point.x),
        y_nm: field.position.map(|point| point.y),
    })
}

pub(crate) fn place_native_project_text(
    root: &Path,
    sheet_uuid: Uuid,
    text: String,
    position: Point,
    rotation_deg: i32,
) -> Result<NativeProjectTextMutationReportView> {
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
    let texts = sheet_value
        .as_object_mut()
        .and_then(|object| object.get_mut("texts"))
        .and_then(serde_json::Value::as_object_mut)
        .ok_or_else(|| anyhow::anyhow!("sheet texts object missing in {}", sheet_path.display()))?;

    let text_uuid = Uuid::new_v4();
    texts.insert(
        text_uuid.to_string(),
        serde_json::to_value(SchematicText {
            uuid: text_uuid,
            text: text.clone(),
            position,
            rotation: rotation_deg,
        })
        .expect("native text serialization must succeed"),
    );
    write_canonical_json(&sheet_path, &sheet_value)?;

    Ok(NativeProjectTextMutationReportView {
        action: "place_text".to_string(),
        project_root: project.root.display().to_string(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: sheet_path.display().to_string(),
        text_uuid: text_uuid.to_string(),
        text,
        x_nm: position.x,
        y_nm: position.y,
        rotation_deg,
    })
}

pub(crate) fn edit_native_project_text(
    root: &Path,
    text_uuid: Uuid,
    text: Option<String>,
    position: Option<Point>,
    rotation_deg: Option<i32>,
) -> Result<NativeProjectTextMutationReportView> {
    let project = load_native_project(root)?;
    let (sheet_uuid, sheet_path, mut sheet_value, mut text_object) =
        load_native_text_mutation_target(&project, text_uuid)?;
    if let Some(text) = text {
        text_object.text = text;
    }
    if let Some(position) = position {
        text_object.position = position;
    }
    if let Some(rotation_deg) = rotation_deg {
        text_object.rotation = rotation_deg;
    }
    write_text_into_sheet(&mut sheet_value, &text_object)?;
    write_canonical_json(&sheet_path, &sheet_value)?;

    Ok(NativeProjectTextMutationReportView {
        action: "edit_text".to_string(),
        project_root: project.root.display().to_string(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: sheet_path.display().to_string(),
        text_uuid: text_object.uuid.to_string(),
        text: text_object.text,
        x_nm: text_object.position.x,
        y_nm: text_object.position.y,
        rotation_deg: text_object.rotation,
    })
}

pub(crate) fn delete_native_project_text(
    root: &Path,
    text_uuid: Uuid,
) -> Result<NativeProjectTextMutationReportView> {
    let project = load_native_project(root)?;
    let (sheet_uuid, sheet_path, mut sheet_value, text_object) =
        load_native_text_mutation_target(&project, text_uuid)?;
    let texts = sheet_value
        .as_object_mut()
        .and_then(|object| object.get_mut("texts"))
        .and_then(serde_json::Value::as_object_mut)
        .ok_or_else(|| anyhow::anyhow!("sheet texts object missing in {}", sheet_path.display()))?;
    texts.remove(&text_uuid.to_string());
    write_canonical_json(&sheet_path, &sheet_value)?;

    Ok(NativeProjectTextMutationReportView {
        action: "delete_text".to_string(),
        project_root: project.root.display().to_string(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: sheet_path.display().to_string(),
        text_uuid: text_object.uuid.to_string(),
        text: text_object.text,
        x_nm: text_object.position.x,
        y_nm: text_object.position.y,
        rotation_deg: text_object.rotation,
    })
}

pub(crate) fn place_native_project_drawing_line(
    root: &Path,
    sheet_uuid: Uuid,
    from: Point,
    to: Point,
) -> Result<NativeProjectDrawingMutationReportView> {
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
    let drawings = sheet_value
        .as_object_mut()
        .and_then(|object| object.get_mut("drawings"))
        .and_then(serde_json::Value::as_object_mut)
        .ok_or_else(|| anyhow::anyhow!("sheet drawings object missing in {}", sheet_path.display()))?;

    let drawing_uuid = Uuid::new_v4();
    drawings.insert(
        drawing_uuid.to_string(),
        serde_json::to_value(SchematicPrimitive::Line {
            uuid: drawing_uuid,
            from,
            to,
        })
        .expect("native drawing serialization must succeed"),
    );
    write_canonical_json(&sheet_path, &sheet_value)?;

    Ok(NativeProjectDrawingMutationReportView {
        action: "place_drawing_line".to_string(),
        project_root: project.root.display().to_string(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: sheet_path.display().to_string(),
        drawing_uuid: drawing_uuid.to_string(),
        kind: "line".to_string(),
        from_x_nm: from.x,
        from_y_nm: from.y,
        to_x_nm: to.x,
        to_y_nm: to.y,
    })
}

pub(crate) fn place_native_project_drawing_rect(
    root: &Path,
    sheet_uuid: Uuid,
    min: Point,
    max: Point,
) -> Result<NativeProjectDrawingMutationReportView> {
    let project = load_native_project(root)?;
    let (sheet_path, mut sheet_value) = load_native_sheet_for_insert(&project, sheet_uuid)?;
    let drawing_uuid = Uuid::new_v4();
    write_drawing_into_sheet(
        &mut sheet_value,
        &SchematicPrimitive::Rect {
            uuid: drawing_uuid,
            min,
            max,
        },
    )?;
    write_canonical_json(&sheet_path, &sheet_value)?;
    Ok(NativeProjectDrawingMutationReportView {
        action: "place_drawing_rect".to_string(),
        project_root: project.root.display().to_string(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: sheet_path.display().to_string(),
        drawing_uuid: drawing_uuid.to_string(),
        kind: "rect".to_string(),
        from_x_nm: min.x,
        from_y_nm: min.y,
        to_x_nm: max.x,
        to_y_nm: max.y,
    })
}

pub(crate) fn place_native_project_drawing_circle(
    root: &Path,
    sheet_uuid: Uuid,
    center: Point,
    radius: i64,
) -> Result<NativeProjectDrawingMutationReportView> {
    let project = load_native_project(root)?;
    let (sheet_path, mut sheet_value) = load_native_sheet_for_insert(&project, sheet_uuid)?;
    let drawing_uuid = Uuid::new_v4();
    write_drawing_into_sheet(
        &mut sheet_value,
        &SchematicPrimitive::Circle {
            uuid: drawing_uuid,
            center,
            radius,
        },
    )?;
    write_canonical_json(&sheet_path, &sheet_value)?;
    Ok(NativeProjectDrawingMutationReportView {
        action: "place_drawing_circle".to_string(),
        project_root: project.root.display().to_string(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: sheet_path.display().to_string(),
        drawing_uuid: drawing_uuid.to_string(),
        kind: "circle".to_string(),
        from_x_nm: center.x,
        from_y_nm: center.y,
        to_x_nm: center.x + radius,
        to_y_nm: center.y,
    })
}

pub(crate) fn place_native_project_drawing_arc(
    root: &Path,
    sheet_uuid: Uuid,
    arc: Arc,
) -> Result<NativeProjectDrawingMutationReportView> {
    let project = load_native_project(root)?;
    let (sheet_path, mut sheet_value) = load_native_sheet_for_insert(&project, sheet_uuid)?;
    let drawing_uuid = Uuid::new_v4();
    write_drawing_into_sheet(
        &mut sheet_value,
        &SchematicPrimitive::Arc {
            uuid: drawing_uuid,
            arc,
        },
    )?;
    write_canonical_json(&sheet_path, &sheet_value)?;
    Ok(NativeProjectDrawingMutationReportView {
        action: "place_drawing_arc".to_string(),
        project_root: project.root.display().to_string(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: sheet_path.display().to_string(),
        drawing_uuid: drawing_uuid.to_string(),
        kind: "arc".to_string(),
        from_x_nm: arc.center.x,
        from_y_nm: arc.center.y,
        to_x_nm: arc.radius,
        to_y_nm: i64::from(arc.end_angle),
    })
}

pub(crate) fn edit_native_project_drawing_line(
    root: &Path,
    drawing_uuid: Uuid,
    from: Option<Point>,
    to: Option<Point>,
) -> Result<NativeProjectDrawingMutationReportView> {
    let project = load_native_project(root)?;
    let (sheet_uuid, sheet_path, mut sheet_value, drawing) =
        load_native_drawing_mutation_target(&project, drawing_uuid)?;
    let (current_from, current_to) = match drawing {
        SchematicPrimitive::Line { from, to, .. } => (from, to),
        _ => bail!("drawing is not a line: {drawing_uuid}"),
    };
    let from = from.unwrap_or(current_from);
    let to = to.unwrap_or(current_to);
    write_drawing_into_sheet(
        &mut sheet_value,
        &SchematicPrimitive::Line {
            uuid: drawing_uuid,
            from,
            to,
        },
    )?;
    write_canonical_json(&sheet_path, &sheet_value)?;

    Ok(NativeProjectDrawingMutationReportView {
        action: "edit_drawing_line".to_string(),
        project_root: project.root.display().to_string(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: sheet_path.display().to_string(),
        drawing_uuid: drawing_uuid.to_string(),
        kind: "line".to_string(),
        from_x_nm: from.x,
        from_y_nm: from.y,
        to_x_nm: to.x,
        to_y_nm: to.y,
    })
}

pub(crate) fn edit_native_project_drawing_rect(
    root: &Path,
    drawing_uuid: Uuid,
    min: Option<Point>,
    max: Option<Point>,
) -> Result<NativeProjectDrawingMutationReportView> {
    let project = load_native_project(root)?;
    let (sheet_uuid, sheet_path, mut sheet_value, drawing) =
        load_native_drawing_mutation_target(&project, drawing_uuid)?;
    let (current_min, current_max) = match drawing {
        SchematicPrimitive::Rect { min, max, .. } => (min, max),
        _ => bail!("drawing is not a rect: {drawing_uuid}"),
    };
    let min = min.unwrap_or(current_min);
    let max = max.unwrap_or(current_max);
    write_drawing_into_sheet(
        &mut sheet_value,
        &SchematicPrimitive::Rect {
            uuid: drawing_uuid,
            min,
            max,
        },
    )?;
    write_canonical_json(&sheet_path, &sheet_value)?;
    Ok(NativeProjectDrawingMutationReportView {
        action: "edit_drawing_rect".to_string(),
        project_root: project.root.display().to_string(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: sheet_path.display().to_string(),
        drawing_uuid: drawing_uuid.to_string(),
        kind: "rect".to_string(),
        from_x_nm: min.x,
        from_y_nm: min.y,
        to_x_nm: max.x,
        to_y_nm: max.y,
    })
}

pub(crate) fn edit_native_project_drawing_circle(
    root: &Path,
    drawing_uuid: Uuid,
    center: Option<Point>,
    radius: Option<i64>,
) -> Result<NativeProjectDrawingMutationReportView> {
    let project = load_native_project(root)?;
    let (sheet_uuid, sheet_path, mut sheet_value, drawing) =
        load_native_drawing_mutation_target(&project, drawing_uuid)?;
    let (current_center, current_radius) = match drawing {
        SchematicPrimitive::Circle { center, radius, .. } => (center, radius),
        _ => bail!("drawing is not a circle: {drawing_uuid}"),
    };
    let center = center.unwrap_or(current_center);
    let radius = radius.unwrap_or(current_radius);
    write_drawing_into_sheet(
        &mut sheet_value,
        &SchematicPrimitive::Circle {
            uuid: drawing_uuid,
            center,
            radius,
        },
    )?;
    write_canonical_json(&sheet_path, &sheet_value)?;
    Ok(NativeProjectDrawingMutationReportView {
        action: "edit_drawing_circle".to_string(),
        project_root: project.root.display().to_string(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: sheet_path.display().to_string(),
        drawing_uuid: drawing_uuid.to_string(),
        kind: "circle".to_string(),
        from_x_nm: center.x,
        from_y_nm: center.y,
        to_x_nm: center.x + radius,
        to_y_nm: center.y,
    })
}

pub(crate) fn edit_native_project_drawing_arc(
    root: &Path,
    drawing_uuid: Uuid,
    center: Option<Point>,
    radius: Option<i64>,
    start_angle: Option<i32>,
    end_angle: Option<i32>,
) -> Result<NativeProjectDrawingMutationReportView> {
    let project = load_native_project(root)?;
    let (sheet_uuid, sheet_path, mut sheet_value, drawing) =
        load_native_drawing_mutation_target(&project, drawing_uuid)?;
    let current_arc = match drawing {
        SchematicPrimitive::Arc { arc, .. } => arc,
        _ => bail!("drawing is not an arc: {drawing_uuid}"),
    };
    let arc = Arc {
        center: center.unwrap_or(current_arc.center),
        radius: radius.unwrap_or(current_arc.radius),
        start_angle: start_angle.unwrap_or(current_arc.start_angle),
        end_angle: end_angle.unwrap_or(current_arc.end_angle),
    };
    write_drawing_into_sheet(
        &mut sheet_value,
        &SchematicPrimitive::Arc {
            uuid: drawing_uuid,
            arc,
        },
    )?;
    write_canonical_json(&sheet_path, &sheet_value)?;
    Ok(NativeProjectDrawingMutationReportView {
        action: "edit_drawing_arc".to_string(),
        project_root: project.root.display().to_string(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: sheet_path.display().to_string(),
        drawing_uuid: drawing_uuid.to_string(),
        kind: "arc".to_string(),
        from_x_nm: arc.center.x,
        from_y_nm: arc.center.y,
        to_x_nm: arc.radius,
        to_y_nm: i64::from(arc.end_angle),
    })
}

pub(crate) fn delete_native_project_drawing(
    root: &Path,
    drawing_uuid: Uuid,
) -> Result<NativeProjectDrawingMutationReportView> {
    let project = load_native_project(root)?;
    let (sheet_uuid, sheet_path, mut sheet_value, drawing) =
        load_native_drawing_mutation_target(&project, drawing_uuid)?;
    let drawings = sheet_value
        .as_object_mut()
        .and_then(|object| object.get_mut("drawings"))
        .and_then(serde_json::Value::as_object_mut)
        .ok_or_else(|| anyhow::anyhow!("sheet drawings object missing in {}", sheet_path.display()))?;
    drawings.remove(&drawing_uuid.to_string());
    write_canonical_json(&sheet_path, &sheet_value)?;

    let (kind, from, to) = match drawing {
        SchematicPrimitive::Line { from, to, .. } => ("line".to_string(), from, to),
        SchematicPrimitive::Rect { min, max, .. } => ("rect".to_string(), min, max),
        SchematicPrimitive::Circle { center, radius, .. } => (
            "circle".to_string(),
            center,
            Point {
                x: center.x + radius,
                y: center.y,
            },
        ),
        SchematicPrimitive::Arc { arc, .. } => ("arc".to_string(), arc.center, arc.center),
    };

    Ok(NativeProjectDrawingMutationReportView {
        action: "delete_drawing".to_string(),
        project_root: project.root.display().to_string(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: sheet_path.display().to_string(),
        drawing_uuid: drawing_uuid.to_string(),
        kind,
        from_x_nm: from.x,
        from_y_nm: from.y,
        to_x_nm: to.x,
        to_y_nm: to.y,
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

pub(crate) fn create_native_project_bus(
    root: &Path,
    sheet_uuid: Uuid,
    name: String,
    members: Vec<String>,
) -> Result<NativeProjectBusMutationReportView> {
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
    let buses = sheet_value
        .as_object_mut()
        .and_then(|object| object.get_mut("buses"))
        .and_then(serde_json::Value::as_object_mut)
        .ok_or_else(|| anyhow::anyhow!("sheet buses object missing in {}", sheet_path.display()))?;

    let bus_uuid = Uuid::new_v4();
    buses.insert(
        bus_uuid.to_string(),
        serde_json::to_value(Bus {
            uuid: bus_uuid,
            name: name.clone(),
            members: members.clone(),
        })
        .expect("native bus serialization must succeed"),
    );
    write_canonical_json(&sheet_path, &sheet_value)?;

    Ok(NativeProjectBusMutationReportView {
        action: "create_bus".to_string(),
        project_root: project.root.display().to_string(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: sheet_path.display().to_string(),
        bus_uuid: bus_uuid.to_string(),
        name,
        members,
    })
}

pub(crate) fn edit_native_project_bus_members(
    root: &Path,
    bus_uuid: Uuid,
    members: Vec<String>,
) -> Result<NativeProjectBusMutationReportView> {
    let project = load_native_project(root)?;
    let (sheet_uuid, sheet_path, mut sheet_value, mut bus) =
        load_native_bus_mutation_target(&project, bus_uuid)?;
    bus.members = members.clone();
    write_bus_into_sheet(&mut sheet_value, &bus)?;
    write_canonical_json(&sheet_path, &sheet_value)?;

    Ok(NativeProjectBusMutationReportView {
        action: "edit_bus_members".to_string(),
        project_root: project.root.display().to_string(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: sheet_path.display().to_string(),
        bus_uuid: bus.uuid.to_string(),
        name: bus.name,
        members,
    })
}

pub(crate) fn place_native_project_bus_entry(
    root: &Path,
    sheet_uuid: Uuid,
    bus_uuid: Uuid,
    wire_uuid: Option<Uuid>,
    position: Point,
) -> Result<NativeProjectBusEntryMutationReportView> {
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
    let bus_entries = sheet_value
        .as_object_mut()
        .and_then(|object| object.get_mut("bus_entries"))
        .and_then(serde_json::Value::as_object_mut)
        .ok_or_else(|| anyhow::anyhow!("sheet bus_entries object missing in {}", sheet_path.display()))?;

    let bus_entry_uuid = Uuid::new_v4();
    bus_entries.insert(
        bus_entry_uuid.to_string(),
        serde_json::to_value(BusEntry {
            uuid: bus_entry_uuid,
            bus: bus_uuid,
            wire: wire_uuid,
            position,
        })
        .expect("native bus entry serialization must succeed"),
    );
    write_canonical_json(&sheet_path, &sheet_value)?;

    Ok(NativeProjectBusEntryMutationReportView {
        action: "place_bus_entry".to_string(),
        project_root: project.root.display().to_string(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: sheet_path.display().to_string(),
        bus_entry_uuid: bus_entry_uuid.to_string(),
        bus_uuid: bus_uuid.to_string(),
        wire_uuid: wire_uuid.map(|uuid| uuid.to_string()),
        x_nm: position.x,
        y_nm: position.y,
    })
}

pub(crate) fn delete_native_project_bus_entry(
    root: &Path,
    bus_entry_uuid: Uuid,
) -> Result<NativeProjectBusEntryMutationReportView> {
    let project = load_native_project(root)?;
    let (sheet_uuid, sheet_path, mut sheet_value, bus_entry) =
        load_native_bus_entry_mutation_target(&project, bus_entry_uuid)?;
    let bus_entries = sheet_value
        .as_object_mut()
        .and_then(|object| object.get_mut("bus_entries"))
        .and_then(serde_json::Value::as_object_mut)
        .ok_or_else(|| anyhow::anyhow!("sheet bus_entries object missing in {}", sheet_path.display()))?;
    bus_entries.remove(&bus_entry_uuid.to_string());
    write_canonical_json(&sheet_path, &sheet_value)?;

    Ok(NativeProjectBusEntryMutationReportView {
        action: "delete_bus_entry".to_string(),
        project_root: project.root.display().to_string(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: sheet_path.display().to_string(),
        bus_entry_uuid: bus_entry.uuid.to_string(),
        bus_uuid: bus_entry.bus.to_string(),
        wire_uuid: bus_entry.wire.map(|uuid| uuid.to_string()),
        x_nm: bus_entry.position.x,
        y_nm: bus_entry.position.y,
    })
}

pub(crate) fn place_native_project_noconnect(
    root: &Path,
    sheet_uuid: Uuid,
    symbol_uuid: Uuid,
    pin_uuid: Uuid,
    position: Point,
) -> Result<NativeProjectNoConnectMutationReportView> {
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
    let noconnects = sheet_value
        .as_object_mut()
        .and_then(|object| object.get_mut("noconnects"))
        .and_then(serde_json::Value::as_object_mut)
        .ok_or_else(|| anyhow::anyhow!("sheet noconnects object missing in {}", sheet_path.display()))?;

    let noconnect_uuid = Uuid::new_v4();
    noconnects.insert(
        noconnect_uuid.to_string(),
        serde_json::to_value(NoConnectMarker {
            uuid: noconnect_uuid,
            symbol: symbol_uuid,
            pin: pin_uuid,
            position,
        })
        .expect("native no-connect serialization must succeed"),
    );
    write_canonical_json(&sheet_path, &sheet_value)?;

    Ok(NativeProjectNoConnectMutationReportView {
        action: "place_noconnect".to_string(),
        project_root: project.root.display().to_string(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: sheet_path.display().to_string(),
        noconnect_uuid: noconnect_uuid.to_string(),
        symbol_uuid: symbol_uuid.to_string(),
        pin_uuid: pin_uuid.to_string(),
        x_nm: position.x,
        y_nm: position.y,
    })
}

pub(crate) fn delete_native_project_noconnect(
    root: &Path,
    noconnect_uuid: Uuid,
) -> Result<NativeProjectNoConnectMutationReportView> {
    let project = load_native_project(root)?;
    let (sheet_uuid, sheet_path, mut sheet_value, marker) =
        load_native_noconnect_mutation_target(&project, noconnect_uuid)?;
    let noconnects = sheet_value
        .as_object_mut()
        .and_then(|object| object.get_mut("noconnects"))
        .and_then(serde_json::Value::as_object_mut)
        .ok_or_else(|| anyhow::anyhow!("sheet noconnects object missing in {}", sheet_path.display()))?;
    noconnects.remove(&noconnect_uuid.to_string());
    write_canonical_json(&sheet_path, &sheet_value)?;

    Ok(NativeProjectNoConnectMutationReportView {
        action: "delete_noconnect".to_string(),
        project_root: project.root.display().to_string(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: sheet_path.display().to_string(),
        noconnect_uuid: marker.uuid.to_string(),
        symbol_uuid: marker.symbol.to_string(),
        pin_uuid: marker.pin.to_string(),
        x_nm: marker.position.x,
        y_nm: marker.position.y,
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
    texts: usize,
    drawings: usize,
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
    let mut texts = 0usize;
    let mut drawings = 0usize;

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
        texts += json_object_len(&sheet_value, "texts");
        drawings += json_object_len(&sheet_value, "drawings");
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
        texts,
        drawings,
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

fn load_native_symbol_mutation_target(
    project: &LoadedNativeProject,
    symbol_uuid: Uuid,
) -> Result<(Uuid, std::path::PathBuf, serde_json::Value, PlacedSymbol)> {
    for (sheet_uuid, relative_path) in &project.schematic.sheets {
        let parsed_sheet_uuid = Uuid::parse_str(sheet_uuid)
            .with_context(|| format!("invalid sheet UUID key `{sheet_uuid}` in schematic root"))?;
        let sheet_path = project.root.join("schematic").join(relative_path);
        let sheet_text = std::fs::read_to_string(&sheet_path)
            .with_context(|| format!("failed to read {}", sheet_path.display()))?;
        let sheet_value: serde_json::Value = serde_json::from_str(&sheet_text)
            .with_context(|| format!("failed to parse {}", sheet_path.display()))?;
        if let Some(entry) = sheet_value
            .get("symbols")
            .and_then(serde_json::Value::as_object)
            .and_then(|symbols| symbols.get(&symbol_uuid.to_string()))
        {
            let symbol: PlacedSymbol = serde_json::from_value(entry.clone())
                .with_context(|| format!("failed to parse symbol in {}", sheet_path.display()))?;
            return Ok((parsed_sheet_uuid, sheet_path, sheet_value, symbol));
        }
    }

    bail!("symbol not found in native project: {symbol_uuid}");
}

fn load_native_field_mutation_target(
    project: &LoadedNativeProject,
    field_uuid: Uuid,
) -> Result<(Uuid, std::path::PathBuf, serde_json::Value, Uuid, PlacedSymbol, SymbolField)> {
    for (sheet_uuid, relative_path) in &project.schematic.sheets {
        let parsed_sheet_uuid = Uuid::parse_str(sheet_uuid)
            .with_context(|| format!("invalid sheet UUID key `{sheet_uuid}` in schematic root"))?;
        let sheet_path = project.root.join("schematic").join(relative_path);
        let sheet_text = std::fs::read_to_string(&sheet_path)
            .with_context(|| format!("failed to read {}", sheet_path.display()))?;
        let sheet_value: serde_json::Value = serde_json::from_str(&sheet_text)
            .with_context(|| format!("failed to parse {}", sheet_path.display()))?;
        if let Some(entries) = sheet_value.get("symbols").and_then(serde_json::Value::as_object) {
            for entry in entries.values() {
                let symbol: PlacedSymbol = serde_json::from_value(entry.clone()).with_context(|| {
                    format!("failed to parse symbol in {}", sheet_path.display())
                })?;
                if let Some(field) = symbol.fields.iter().find(|field| field.uuid == field_uuid) {
                    return Ok((
                        parsed_sheet_uuid,
                        sheet_path,
                        sheet_value,
                        symbol.uuid,
                        symbol.clone(),
                        field.clone(),
                    ));
                }
            }
        }
    }

    bail!("symbol field not found in native project: {field_uuid}");
}

fn load_native_text_mutation_target(
    project: &LoadedNativeProject,
    text_uuid: Uuid,
) -> Result<(Uuid, std::path::PathBuf, serde_json::Value, SchematicText)> {
    for (sheet_uuid, relative_path) in &project.schematic.sheets {
        let parsed_sheet_uuid = Uuid::parse_str(sheet_uuid)
            .with_context(|| format!("invalid sheet UUID key `{sheet_uuid}` in schematic root"))?;
        let sheet_path = project.root.join("schematic").join(relative_path);
        let sheet_text = std::fs::read_to_string(&sheet_path)
            .with_context(|| format!("failed to read {}", sheet_path.display()))?;
        let sheet_value: serde_json::Value = serde_json::from_str(&sheet_text)
            .with_context(|| format!("failed to parse {}", sheet_path.display()))?;
        if let Some(entry) = sheet_value
            .get("texts")
            .and_then(serde_json::Value::as_object)
            .and_then(|texts| texts.get(&text_uuid.to_string()))
        {
            let text: SchematicText = serde_json::from_value(entry.clone())
                .with_context(|| format!("failed to parse text in {}", sheet_path.display()))?;
            return Ok((parsed_sheet_uuid, sheet_path, sheet_value, text));
        }
    }

    bail!("text not found in native project: {text_uuid}");
}

fn load_native_sheet_for_insert(
    project: &LoadedNativeProject,
    sheet_uuid: Uuid,
) -> Result<(std::path::PathBuf, serde_json::Value)> {
    let sheet_key = sheet_uuid.to_string();
    let relative_path = project
        .schematic
        .sheets
        .get(&sheet_key)
        .ok_or_else(|| anyhow::anyhow!("sheet not found in native schematic root: {sheet_uuid}"))?;
    let sheet_path = project.root.join("schematic").join(relative_path);
    let sheet_text = std::fs::read_to_string(&sheet_path)
        .with_context(|| format!("failed to read {}", sheet_path.display()))?;
    let sheet_value: serde_json::Value = serde_json::from_str(&sheet_text)
        .with_context(|| format!("failed to parse {}", sheet_path.display()))?;
    Ok((sheet_path, sheet_value))
}

fn load_native_drawing_mutation_target(
    project: &LoadedNativeProject,
    drawing_uuid: Uuid,
) -> Result<(Uuid, std::path::PathBuf, serde_json::Value, SchematicPrimitive)> {
    for (sheet_uuid, relative_path) in &project.schematic.sheets {
        let parsed_sheet_uuid = Uuid::parse_str(sheet_uuid)
            .with_context(|| format!("invalid sheet UUID key `{sheet_uuid}` in schematic root"))?;
        let sheet_path = project.root.join("schematic").join(relative_path);
        let sheet_text = std::fs::read_to_string(&sheet_path)
            .with_context(|| format!("failed to read {}", sheet_path.display()))?;
        let sheet_value: serde_json::Value = serde_json::from_str(&sheet_text)
            .with_context(|| format!("failed to parse {}", sheet_path.display()))?;
        if let Some(entry) = sheet_value
            .get("drawings")
            .and_then(serde_json::Value::as_object)
            .and_then(|drawings| drawings.get(&drawing_uuid.to_string()))
        {
            let drawing: SchematicPrimitive = serde_json::from_value(entry.clone())
                .with_context(|| format!("failed to parse drawing in {}", sheet_path.display()))?;
            return Ok((parsed_sheet_uuid, sheet_path, sheet_value, drawing));
        }
    }

    bail!("drawing not found in native project: {drawing_uuid}");
}

pub(crate) fn parse_native_field_position(x_nm: Option<i64>, y_nm: Option<i64>) -> Result<Option<Point>> {
    match (x_nm, y_nm) {
        (None, None) => Ok(None),
        (Some(x), Some(y)) => Ok(Some(Point { x, y })),
        _ => bail!("field position requires both --x-nm and --y-nm"),
    }
}

fn write_symbol_into_sheet(sheet_value: &mut serde_json::Value, symbol: &PlacedSymbol) -> Result<()> {
    let symbols = sheet_value
        .as_object_mut()
        .and_then(|object| object.get_mut("symbols"))
        .and_then(serde_json::Value::as_object_mut)
        .ok_or_else(|| anyhow::anyhow!("sheet symbols object missing during symbol mutation"))?;
    symbols.insert(
        symbol.uuid.to_string(),
        serde_json::to_value(symbol).expect("native symbol serialization must succeed"),
    );
    Ok(())
}

fn write_text_into_sheet(sheet_value: &mut serde_json::Value, text: &SchematicText) -> Result<()> {
    let texts = sheet_value
        .as_object_mut()
        .and_then(|object| object.get_mut("texts"))
        .and_then(serde_json::Value::as_object_mut)
        .ok_or_else(|| anyhow::anyhow!("sheet texts object missing during text mutation"))?;
    texts.insert(
        text.uuid.to_string(),
        serde_json::to_value(text).expect("native text serialization must succeed"),
    );
    Ok(())
}

fn write_drawing_into_sheet(
    sheet_value: &mut serde_json::Value,
    drawing: &SchematicPrimitive,
) -> Result<()> {
    let drawings = sheet_value
        .as_object_mut()
        .and_then(|object| object.get_mut("drawings"))
        .and_then(serde_json::Value::as_object_mut)
        .ok_or_else(|| anyhow::anyhow!("sheet drawings object missing during drawing mutation"))?;
    let uuid = drawing_uuid(drawing);
    drawings.insert(
        uuid.to_string(),
        serde_json::to_value(drawing).expect("native drawing serialization must succeed"),
    );
    Ok(())
}

fn drawing_uuid(drawing: &SchematicPrimitive) -> Uuid {
    match drawing {
        SchematicPrimitive::Line { uuid, .. }
        | SchematicPrimitive::Rect { uuid, .. }
        | SchematicPrimitive::Circle { uuid, .. }
        | SchematicPrimitive::Arc { uuid, .. } => *uuid,
    }
}

fn render_drawing_query_view(sheet_uuid: Uuid, drawing: SchematicPrimitive) -> Option<serde_json::Value> {
    match drawing {
        SchematicPrimitive::Line { uuid, from, to } => Some(serde_json::json!({
            "uuid": uuid,
            "sheet": sheet_uuid,
            "kind": "line",
            "from": from,
            "to": to,
        })),
        SchematicPrimitive::Rect { uuid, min, max } => Some(serde_json::json!({
            "uuid": uuid,
            "sheet": sheet_uuid,
            "kind": "rect",
            "min": min,
            "max": max,
        })),
        SchematicPrimitive::Circle {
            uuid,
            center,
            radius,
        } => Some(serde_json::json!({
            "uuid": uuid,
            "sheet": sheet_uuid,
            "kind": "circle",
            "center": center,
            "radius": radius,
        })),
        SchematicPrimitive::Arc { uuid, arc } => Some(serde_json::json!({
            "uuid": uuid,
            "sheet": sheet_uuid,
            "kind": "arc",
            "arc": arc,
        })),
    }
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

fn load_native_bus_mutation_target(
    project: &LoadedNativeProject,
    bus_uuid: Uuid,
) -> Result<(Uuid, std::path::PathBuf, serde_json::Value, Bus)> {
    for (sheet_uuid, relative_path) in &project.schematic.sheets {
        let parsed_sheet_uuid = Uuid::parse_str(sheet_uuid)
            .with_context(|| format!("invalid sheet UUID key `{sheet_uuid}` in schematic root"))?;
        let sheet_path = project.root.join("schematic").join(relative_path);
        let sheet_text = std::fs::read_to_string(&sheet_path)
            .with_context(|| format!("failed to read {}", sheet_path.display()))?;
        let sheet_value: serde_json::Value = serde_json::from_str(&sheet_text)
            .with_context(|| format!("failed to parse {}", sheet_path.display()))?;
        if let Some(entry) = sheet_value
            .get("buses")
            .and_then(serde_json::Value::as_object)
            .and_then(|buses| buses.get(&bus_uuid.to_string()))
        {
            let bus: Bus = serde_json::from_value(entry.clone())
                .with_context(|| format!("failed to parse bus in {}", sheet_path.display()))?;
            return Ok((parsed_sheet_uuid, sheet_path, sheet_value, bus));
        }
    }

    bail!("bus not found in native project: {bus_uuid}");
}

fn write_bus_into_sheet(sheet_value: &mut serde_json::Value, bus: &Bus) -> Result<()> {
    let buses = sheet_value
        .as_object_mut()
        .and_then(|object| object.get_mut("buses"))
        .and_then(serde_json::Value::as_object_mut)
        .ok_or_else(|| anyhow::anyhow!("sheet buses object missing during bus mutation"))?;
    buses.insert(
        bus.uuid.to_string(),
        serde_json::to_value(bus).expect("native bus serialization must succeed"),
    );
    Ok(())
}

fn load_native_bus_entry_mutation_target(
    project: &LoadedNativeProject,
    bus_entry_uuid: Uuid,
) -> Result<(Uuid, std::path::PathBuf, serde_json::Value, BusEntry)> {
    for (sheet_uuid, relative_path) in &project.schematic.sheets {
        let parsed_sheet_uuid = Uuid::parse_str(sheet_uuid)
            .with_context(|| format!("invalid sheet UUID key `{sheet_uuid}` in schematic root"))?;
        let sheet_path = project.root.join("schematic").join(relative_path);
        let sheet_text = std::fs::read_to_string(&sheet_path)
            .with_context(|| format!("failed to read {}", sheet_path.display()))?;
        let sheet_value: serde_json::Value = serde_json::from_str(&sheet_text)
            .with_context(|| format!("failed to parse {}", sheet_path.display()))?;
        if let Some(entry) = sheet_value
            .get("bus_entries")
            .and_then(serde_json::Value::as_object)
            .and_then(|entries| entries.get(&bus_entry_uuid.to_string()))
        {
            let bus_entry: BusEntry = serde_json::from_value(entry.clone()).with_context(|| {
                format!("failed to parse bus entry in {}", sheet_path.display())
            })?;
            return Ok((parsed_sheet_uuid, sheet_path, sheet_value, bus_entry));
        }
    }

    bail!("bus entry not found in native project: {bus_entry_uuid}");
}

fn load_native_noconnect_mutation_target(
    project: &LoadedNativeProject,
    noconnect_uuid: Uuid,
) -> Result<(Uuid, std::path::PathBuf, serde_json::Value, NoConnectMarker)> {
    for (sheet_uuid, relative_path) in &project.schematic.sheets {
        let parsed_sheet_uuid = Uuid::parse_str(sheet_uuid)
            .with_context(|| format!("invalid sheet UUID key `{sheet_uuid}` in schematic root"))?;
        let sheet_path = project.root.join("schematic").join(relative_path);
        let sheet_text = std::fs::read_to_string(&sheet_path)
            .with_context(|| format!("failed to read {}", sheet_path.display()))?;
        let sheet_value: serde_json::Value = serde_json::from_str(&sheet_text)
            .with_context(|| format!("failed to parse {}", sheet_path.display()))?;
        if let Some(entry) = sheet_value
            .get("noconnects")
            .and_then(serde_json::Value::as_object)
            .and_then(|markers| markers.get(&noconnect_uuid.to_string()))
        {
            let marker: NoConnectMarker = serde_json::from_value(entry.clone()).with_context(|| {
                format!("failed to parse no-connect in {}", sheet_path.display())
            })?;
            return Ok((parsed_sheet_uuid, sheet_path, sheet_value, marker));
        }
    }

    bail!("no-connect not found in native project: {noconnect_uuid}");
}

fn write_canonical_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let json = to_json_deterministic(value).context("failed to serialize canonical JSON")?;
    std::fs::write(path, format!("{json}\n"))
        .with_context(|| format!("failed to write {}", path.display()))
}
