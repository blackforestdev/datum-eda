use super::*;

pub(super) fn render_symbol_display_mode(mode: &SymbolDisplayMode) -> String {
    match mode {
        SymbolDisplayMode::LibraryDefault => "LibraryDefault",
        SymbolDisplayMode::ShowHiddenPins => "ShowHiddenPins",
        SymbolDisplayMode::HideOptionalPins => "HideOptionalPins",
    }
    .to_string()
}

pub(super) fn render_hidden_power_behavior(mode: &HiddenPowerBehavior) -> String {
    match mode {
        HiddenPowerBehavior::SourceDefinedImplicit => "SourceDefinedImplicit",
        HiddenPowerBehavior::ExplicitPowerObject => "ExplicitPowerObject",
        HiddenPowerBehavior::PreservedAsImportedMetadata => "PreservedAsImportedMetadata",
    }
    .to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct NativeProjectManifest {
    pub(super) schema_version: u32,
    pub(super) uuid: Uuid,
    pub(super) name: String,
    pub(super) pools: Vec<NativeProjectPoolRef>,
    pub(super) schematic: String,
    pub(super) board: String,
    pub(super) rules: String,
    #[serde(default)]
    pub(super) forward_annotation_review: BTreeMap<String, NativeForwardAnnotationReviewRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct NativeProjectPoolRef {
    pub(super) path: String,
    pub(super) priority: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct NativeForwardAnnotationReviewRecord {
    pub(super) action_id: String,
    pub(super) decision: String,
    pub(super) proposal_action: String,
    pub(super) reference: String,
    pub(super) reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct NativeSchematicRoot {
    pub(super) schema_version: u32,
    pub(super) uuid: Uuid,
    pub(super) sheets: BTreeMap<String, String>,
    pub(super) definitions: BTreeMap<String, String>,
    pub(super) instances: Vec<NativeSchematicInstance>,
    pub(super) variants: BTreeMap<String, NativeVariant>,
    pub(super) waivers: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct NativeSchematicInstance {
    pub(super) uuid: Uuid,
    pub(super) definition: Uuid,
    pub(super) parent_sheet: Option<Uuid>,
    pub(super) position: NativePoint,
    pub(super) name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct NativeVariant {
    pub(super) name: String,
    pub(super) fitted_components: BTreeMap<String, bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct NativeRulesRoot {
    pub(super) schema_version: u32,
    pub(super) rules: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct NativeSheetRoot {
    pub(super) schema_version: u32,
    pub(super) uuid: Uuid,
    pub(super) name: String,
    pub(super) frame: Option<SheetFrame>,
    pub(super) symbols: BTreeMap<String, PlacedSymbol>,
    pub(super) wires: BTreeMap<String, SchematicWire>,
    pub(super) junctions: BTreeMap<String, Junction>,
    pub(super) labels: BTreeMap<String, NetLabel>,
    pub(super) buses: BTreeMap<String, Bus>,
    pub(super) bus_entries: BTreeMap<String, BusEntry>,
    pub(super) ports: BTreeMap<String, HierarchicalPort>,
    pub(super) noconnects: BTreeMap<String, NoConnectMarker>,
    pub(super) texts: BTreeMap<String, SchematicText>,
    pub(super) drawings: BTreeMap<String, SchematicPrimitive>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct ExistingProjectIds {
    pub(super) project_uuid: Uuid,
    pub(super) schematic_uuid: Uuid,
    pub(super) board_uuid: Uuid,
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
        forward_annotation_review: BTreeMap::new(),
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
        stackup: NativeStackup {
            layers: default_native_project_stackup_layers(),
        },
        outline: NativeOutline {
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

pub(crate) fn query_native_project_rules(root: &Path) -> Result<NativeProjectRulesView> {
    let project = load_native_project(root)?;
    Ok(NativeProjectRulesView {
        domain: "native_project",
        count: project.rules.rules.len(),
        rules: project.rules.rules,
    })
}
