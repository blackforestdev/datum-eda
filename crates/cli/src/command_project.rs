use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};

#[path = "command_project_gerber_inspect.rs"]
mod command_project_gerber_inspect;
#[path = "command_project_gerber_mechanical.rs"]
mod command_project_gerber_mechanical;
#[path = "command_project_gerber_silkscreen.rs"]
mod command_project_gerber_silkscreen;
#[path = "command_project_pool_materialization.rs"]
mod command_project_pool_materialization;
#[path = "command_project_native_types.rs"]
mod command_project_native_types;

pub(crate) use self::command_project_gerber_inspect::inspect_gerber;
pub(crate) use self::command_project_gerber_mechanical::{
    compare_native_project_gerber_mechanical_layer, export_native_project_gerber_mechanical_layer,
    validate_native_project_gerber_mechanical_layer,
};
use self::command_project_gerber_silkscreen::{
    count_native_component_silkscreen_arcs, count_native_component_silkscreen_circles,
    count_native_component_silkscreen_lines, count_native_component_silkscreen_polygons,
    count_native_component_silkscreen_polylines, count_native_component_silkscreen_texts,
    resolve_native_project_silkscreen_context,
};
use self::command_project_pool_materialization::{
    materialize_supported_pool_package_graphics, resolve_native_project_pool_path,
};
pub(crate) use self::command_project_native_types::{
    NativeBoardRoot, NativeOutline, NativePoint, NativeStackup,
};
use anyhow::{Context, Result, bail};
use eda_engine::api::{CheckCodeCount, CheckReport, CheckStatus, CheckSummary};
use eda_engine::board::{
    Board, BoardText, Dimension, Keepout, Net, NetClass, PadAperture, PadShape, PlacedPackage,
    PlacedPad, Stackup, StackupLayer, StackupLayerType, Track, Via, Zone,
};
use eda_engine::connectivity::{schematic_diagnostics, schematic_net_info};
use eda_engine::erc::{ErcFinding, run_prechecks};
use eda_engine::export::{
    render_excellon_drill, render_rs274x_copper_layer, render_rs274x_outline_default,
    render_rs274x_paste_layer, render_rs274x_silkscreen_layer, render_rs274x_soldermask_layer,
};
use eda_engine::import::ids_sidecar::compute_source_hash_bytes;
use eda_engine::ir::geometry::Polygon;
use eda_engine::ir::geometry::{Arc, Point};
use eda_engine::ir::serialization::to_json_deterministic;
use eda_engine::rules::ast::Rule;
use eda_engine::schematic::{
    Bus, BusEntry, BusEntryInfo, BusInfo, CheckWaiver, ConnectivityDiagnosticInfo,
    HiddenPowerBehavior, HierarchicalPort, Junction, LabelInfo, LabelKind, NetLabel, NoConnectInfo,
    NoConnectMarker, PinDisplayOverride, PlacedSymbol, PortDirection, PortInfo, Schematic,
    SchematicNetInfo, SchematicPrimitive, SchematicText, SchematicWire, Sheet, SheetFrame,
    SymbolDisplayMode, SymbolField, SymbolFieldInfo, SymbolInfo, SymbolPin,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{
    DiagnosticsView, NativeProjectBoardComponentMutationReportView,
    NativeProjectBoardDimensionMutationReportView, NativeProjectBoardKeepoutMutationReportView,
    NativeProjectBoardNetClassMutationReportView, NativeProjectBoardNetMutationReportView,
    NativeProjectBoardOutlineMutationReportView, NativeProjectBoardPadMutationReportView,
    NativeProjectBoardStackupMutationReportView, NativeProjectBoardSummaryView,
    NativeProjectBoardTextMutationReportView, NativeProjectBoardTrackMutationReportView,
    NativeProjectBoardViaMutationReportView, NativeProjectBoardZoneMutationReportView,
    NativeProjectBomComparisonView, NativeProjectBomDriftView, NativeProjectBomExportView,
    NativeProjectBusEntryMutationReportView, NativeProjectBusMutationReportView,
    NativeProjectCreateReportView, NativeProjectDrawingMutationReportView,
    NativeProjectDrillExportView, NativeProjectDrillHoleClassBucketView,
    NativeProjectDrillHoleClassReportView, NativeProjectExcellonDrillComparisonView,
    NativeProjectExcellonDrillExportView, NativeProjectExcellonDrillHitDriftView,
    NativeProjectExcellonDrillInspectionView, NativeProjectExcellonDrillToolView,
    NativeProjectExcellonDrillValidationView, NativeProjectForwardAnnotationApplyReportView,
    NativeProjectForwardAnnotationArtifactApplyPlanActionView,
    NativeProjectForwardAnnotationArtifactApplyPlanView,
    NativeProjectForwardAnnotationArtifactApplyView,
    NativeProjectForwardAnnotationArtifactComparisonActionView,
    NativeProjectForwardAnnotationArtifactComparisonView,
    NativeProjectForwardAnnotationArtifactFilterView,
    NativeProjectForwardAnnotationArtifactInspectionView,
    NativeProjectForwardAnnotationArtifactReviewImportView,
    NativeProjectForwardAnnotationArtifactReviewReplaceView,
    NativeProjectForwardAnnotationAuditView, NativeProjectForwardAnnotationBatchApplyReportView,
    NativeProjectForwardAnnotationBatchApplySkippedActionView,
    NativeProjectForwardAnnotationExportReportView, NativeProjectForwardAnnotationMissingView,
    NativeProjectForwardAnnotationOrphanView, NativeProjectForwardAnnotationPartMismatchView,
    NativeProjectForwardAnnotationProposalActionView, NativeProjectForwardAnnotationProposalView,
    NativeProjectForwardAnnotationReviewActionView, NativeProjectForwardAnnotationReviewReportView,
    NativeProjectForwardAnnotationReviewView, NativeProjectForwardAnnotationValueMismatchView,
    NativeProjectGerberCopperComparisonView, NativeProjectGerberCopperExportView,
    NativeProjectGerberCopperValidationView, NativeProjectGerberGeometryEntryView,
    NativeProjectGerberInspectionView, NativeProjectGerberMechanicalComparisonView,
    NativeProjectGerberMechanicalExportView, NativeProjectGerberMechanicalValidationView,
    NativeProjectGerberOutlineComparisonView, NativeProjectGerberOutlineExportView,
    NativeProjectGerberOutlineValidationView, NativeProjectGerberPasteComparisonView,
    NativeProjectGerberPasteExportView, NativeProjectGerberPasteValidationView,
    NativeProjectGerberPlanArtifactView, NativeProjectGerberPlanComparisonView,
    NativeProjectGerberPlanView, NativeProjectGerberSilkscreenComparisonView,
    NativeProjectGerberSilkscreenExportView, NativeProjectGerberSilkscreenValidationView,
    NativeProjectGerberSoldermaskComparisonView, NativeProjectGerberSoldermaskExportView,
    NativeProjectGerberSoldermaskValidationView, NativeProjectInspectPoolRefView,
    NativeProjectInspectReportView, NativeProjectJunctionMutationReportView,
    NativeProjectLabelMutationReportView,
    NativeProjectNoConnectMutationReportView, NativeProjectPinOverrideMutationReportView,
    NativeProjectPnpComparisonView, NativeProjectPnpDriftView, NativeProjectPnpExportView,
    NativeProjectPortMutationReportView, NativeProjectRulesSummaryView, NativeProjectRulesView,
    NativeProjectSchematicSummaryView, NativeProjectSummaryView,
    NativeProjectSymbolFieldMutationReportView, NativeProjectSymbolMutationReportView,
    NativeProjectSymbolPinInfoView, NativeProjectSymbolSemanticsView,
    NativeProjectTextMutationReportView, NativeProjectWireMutationReportView, UnroutedView,
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
    #[serde(default)]
    forward_annotation_review: BTreeMap<String, NativeForwardAnnotationReviewRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NativeProjectPoolRef {
    path: String,
    priority: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NativeForwardAnnotationReviewRecord {
    action_id: String,
    decision: String,
    proposal_action: String,
    reference: String,
    reason: String,
}

const FORWARD_ANNOTATION_ARTIFACT_KIND: &str = "native_forward_annotation_proposal_artifact";
const FORWARD_ANNOTATION_ARTIFACT_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ForwardAnnotationProposalArtifact {
    kind: String,
    version: u32,
    project_uuid: Uuid,
    project_name: String,
    actions: Vec<NativeProjectForwardAnnotationProposalActionView>,
    reviews: Vec<NativeProjectForwardAnnotationReviewActionView>,
}

struct LoadedForwardAnnotationProposalArtifact {
    artifact_path: PathBuf,
    source_version: u32,
    artifact: ForwardAnnotationProposalArtifact,
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
struct NativeRulesRoot {
    schema_version: u32,
    rules: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NativeSheetRoot {
    schema_version: u32,
    uuid: Uuid,
    name: String,
    frame: Option<SheetFrame>,
    symbols: BTreeMap<String, PlacedSymbol>,
    wires: BTreeMap<String, SchematicWire>,
    junctions: BTreeMap<String, Junction>,
    labels: BTreeMap<String, NetLabel>,
    buses: BTreeMap<String, Bus>,
    bus_entries: BTreeMap<String, BusEntry>,
    ports: BTreeMap<String, HierarchicalPort>,
    noconnects: BTreeMap<String, NoConnectMarker>,
    texts: BTreeMap<String, SchematicText>,
    drawings: BTreeMap<String, SchematicPrimitive>,
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
        stackup: NativeStackup { layers: Vec::new() },
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

pub(crate) fn inspect_native_project(root: &Path) -> Result<NativeProjectInspectReportView> {
    let project = load_native_project(root)?;
    let pool_refs = project
        .manifest
        .pools
        .iter()
        .map(|pool_ref| {
            let resolved_path = resolve_native_project_pool_path(&project.root, &pool_ref.path);
            NativeProjectInspectPoolRefView {
                manifest_path: pool_ref.path.clone(),
                priority: pool_ref.priority,
                resolved_path: resolved_path.display().to_string(),
                exists: resolved_path.exists(),
            }
        })
        .collect();

    Ok(NativeProjectInspectReportView {
        project_root: project.root.display().to_string(),
        project_name: project.manifest.name.clone(),
        schema_version: project.manifest.schema_version,
        project_uuid: project.manifest.uuid.to_string(),
        schematic_uuid: project.schematic.uuid.to_string(),
        board_uuid: project.board.uuid.to_string(),
        pools: project.manifest.pools.len(),
        pool_refs,
        schematic_path: project.schematic_path.display().to_string(),
        board_path: project.board_path.display().to_string(),
        rules_path: project.rules_path.display().to_string(),
        sheet_count: project.schematic.sheets.len(),
        sheet_definition_count: project.schematic.definitions.len(),
        sheet_instance_count: project.schematic.instances.len(),
        variant_count: project.schematic.variants.len(),
        board_package_count: project.board.packages.len(),
        board_pad_count: project.board.pads.len(),
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
    let pool_refs = project
        .manifest
        .pools
        .iter()
        .map(|pool_ref| {
            let resolved_path = resolve_native_project_pool_path(&project.root, &pool_ref.path);
            NativeProjectInspectPoolRefView {
                manifest_path: pool_ref.path.clone(),
                priority: pool_ref.priority,
                resolved_path: resolved_path.display().to_string(),
                exists: resolved_path.exists(),
            }
        })
        .collect();
    Ok(NativeProjectSummaryView {
        domain: "native_project",
        project_name: project.manifest.name,
        schema_version: project.manifest.schema_version,
        pools: project.manifest.pools.len(),
        pool_refs,
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
            pads: project.board.pads.len(),
            nets: project.board.nets.len(),
            net_classes: project.board.net_classes.len(),
            tracks: project.board.tracks.len(),
            vias: project.board.vias.len(),
            zones: project.board.zones.len(),
            keepouts: project.board.keepouts.len(),
            dimensions: project.board.dimensions.len(),
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

pub(crate) fn query_native_project_forward_annotation_audit(
    root: &Path,
) -> Result<NativeProjectForwardAnnotationAuditView> {
    let symbols = query_native_project_symbols(root)?;
    let components = query_native_project_board_components(root)?;

    let mut symbols_by_reference = BTreeMap::new();
    for symbol in symbols {
        symbols_by_reference.insert(symbol.reference.clone(), symbol);
    }
    let mut components_by_reference = BTreeMap::new();
    for component in components {
        components_by_reference.insert(component.reference.clone(), component);
    }

    let mut missing_on_board = Vec::new();
    let mut orphaned_on_board = Vec::new();
    let mut value_mismatches = Vec::new();
    let mut part_mismatches = Vec::new();
    let mut unresolved_symbol_count = 0usize;
    let mut matched_count = 0usize;

    for (reference, symbol) in &symbols_by_reference {
        if symbol.part_uuid.is_none() {
            unresolved_symbol_count += 1;
        }

        if let Some(component) = components_by_reference.get(reference) {
            matched_count += 1;
            if symbol.value != component.value {
                value_mismatches.push(NativeProjectForwardAnnotationValueMismatchView {
                    reference: reference.clone(),
                    symbol_uuid: symbol.uuid.to_string(),
                    component_uuid: component.uuid.to_string(),
                    schematic_value: symbol.value.clone(),
                    board_value: component.value.clone(),
                });
            }
            if let Some(part_uuid) = symbol.part_uuid {
                if part_uuid != component.part {
                    part_mismatches.push(NativeProjectForwardAnnotationPartMismatchView {
                        reference: reference.clone(),
                        symbol_uuid: symbol.uuid.to_string(),
                        component_uuid: component.uuid.to_string(),
                        schematic_part_uuid: part_uuid.to_string(),
                        board_part_uuid: component.part.to_string(),
                    });
                }
            }
        } else {
            missing_on_board.push(NativeProjectForwardAnnotationMissingView {
                symbol_uuid: symbol.uuid.to_string(),
                sheet_uuid: symbol.sheet.to_string(),
                reference: reference.clone(),
                value: symbol.value.clone(),
                part_uuid: symbol.part_uuid.map(|uuid| uuid.to_string()),
            });
        }
    }

    for (reference, component) in &components_by_reference {
        if !symbols_by_reference.contains_key(reference) {
            orphaned_on_board.push(NativeProjectForwardAnnotationOrphanView {
                component_uuid: component.uuid.to_string(),
                reference: reference.clone(),
                value: component.value.clone(),
                part_uuid: component.part.to_string(),
            });
        }
    }

    Ok(NativeProjectForwardAnnotationAuditView {
        domain: "native_project",
        schematic_symbol_count: symbols_by_reference.len(),
        board_component_count: components_by_reference.len(),
        matched_count,
        unresolved_symbol_count,
        missing_on_board,
        orphaned_on_board,
        value_mismatches,
        part_mismatches,
    })
}

pub(crate) fn query_native_project_forward_annotation_proposal(
    root: &Path,
) -> Result<NativeProjectForwardAnnotationProposalView> {
    let audit = query_native_project_forward_annotation_audit(root)?;
    let mut actions = Vec::new();

    for entry in &audit.missing_on_board {
        actions.push(NativeProjectForwardAnnotationProposalActionView {
            action_id: forward_annotation_action_id(
                "add_component",
                &entry.reference,
                Some(&entry.symbol_uuid),
                None,
                if entry.part_uuid.is_some() {
                    "symbol_missing_on_board"
                } else {
                    "symbol_missing_on_board_unresolved_part"
                },
            ),
            action: "add_component".to_string(),
            reference: entry.reference.clone(),
            symbol_uuid: Some(entry.symbol_uuid.clone()),
            component_uuid: None,
            reason: if entry.part_uuid.is_some() {
                "symbol_missing_on_board".to_string()
            } else {
                "symbol_missing_on_board_unresolved_part".to_string()
            },
            schematic_value: Some(entry.value.clone()),
            board_value: None,
            schematic_part_uuid: entry.part_uuid.clone(),
            board_part_uuid: None,
        });
    }

    for entry in &audit.orphaned_on_board {
        actions.push(NativeProjectForwardAnnotationProposalActionView {
            action_id: forward_annotation_action_id(
                "remove_component",
                &entry.reference,
                None,
                Some(&entry.component_uuid),
                "board_component_missing_in_schematic",
            ),
            action: "remove_component".to_string(),
            reference: entry.reference.clone(),
            symbol_uuid: None,
            component_uuid: Some(entry.component_uuid.clone()),
            reason: "board_component_missing_in_schematic".to_string(),
            schematic_value: None,
            board_value: Some(entry.value.clone()),
            schematic_part_uuid: None,
            board_part_uuid: Some(entry.part_uuid.clone()),
        });
    }

    for entry in &audit.value_mismatches {
        actions.push(NativeProjectForwardAnnotationProposalActionView {
            action_id: forward_annotation_action_id(
                "update_component",
                &entry.reference,
                Some(&entry.symbol_uuid),
                Some(&entry.component_uuid),
                "value_mismatch",
            ),
            action: "update_component".to_string(),
            reference: entry.reference.clone(),
            symbol_uuid: Some(entry.symbol_uuid.clone()),
            component_uuid: Some(entry.component_uuid.clone()),
            reason: "value_mismatch".to_string(),
            schematic_value: Some(entry.schematic_value.clone()),
            board_value: Some(entry.board_value.clone()),
            schematic_part_uuid: None,
            board_part_uuid: None,
        });
    }

    for entry in &audit.part_mismatches {
        actions.push(NativeProjectForwardAnnotationProposalActionView {
            action_id: forward_annotation_action_id(
                "update_component",
                &entry.reference,
                Some(&entry.symbol_uuid),
                Some(&entry.component_uuid),
                "part_mismatch",
            ),
            action: "update_component".to_string(),
            reference: entry.reference.clone(),
            symbol_uuid: Some(entry.symbol_uuid.clone()),
            component_uuid: Some(entry.component_uuid.clone()),
            reason: "part_mismatch".to_string(),
            schematic_value: None,
            board_value: None,
            schematic_part_uuid: Some(entry.schematic_part_uuid.clone()),
            board_part_uuid: Some(entry.board_part_uuid.clone()),
        });
    }

    actions.sort_by(|a, b| {
        a.reference
            .cmp(&b.reference)
            .then_with(|| a.action.cmp(&b.action))
            .then_with(|| a.reason.cmp(&b.reason))
    });

    let add_component_actions = actions
        .iter()
        .filter(|action| action.action == "add_component")
        .count();
    let remove_component_actions = actions
        .iter()
        .filter(|action| action.action == "remove_component")
        .count();
    let update_component_actions = actions
        .iter()
        .filter(|action| action.action == "update_component")
        .count();
    let add_component_group = actions
        .iter()
        .filter(|action| action.action == "add_component")
        .cloned()
        .collect::<Vec<_>>();
    let remove_component_group = actions
        .iter()
        .filter(|action| action.action == "remove_component")
        .cloned()
        .collect::<Vec<_>>();
    let update_component_group = actions
        .iter()
        .filter(|action| action.action == "update_component")
        .cloned()
        .collect::<Vec<_>>();

    Ok(NativeProjectForwardAnnotationProposalView {
        domain: "native_project",
        total_actions: actions.len(),
        add_component_actions,
        remove_component_actions,
        update_component_actions,
        add_component_group,
        remove_component_group,
        update_component_group,
        actions,
    })
}

pub(crate) fn apply_native_project_forward_annotation_action(
    root: &Path,
    action_id: &str,
    package_uuid: Option<Uuid>,
    part_uuid: Option<Uuid>,
    x_nm: Option<i64>,
    y_nm: Option<i64>,
    layer: Option<i32>,
) -> Result<NativeProjectForwardAnnotationApplyReportView> {
    let proposal = query_native_project_forward_annotation_proposal(root)?;
    let action = proposal
        .actions
        .into_iter()
        .find(|action| action.action_id == action_id)
        .ok_or_else(|| {
            anyhow::anyhow!("forward-annotation proposal action not found: {action_id}")
        })?;

    execute_native_project_forward_annotation_action(
        root,
        action,
        package_uuid,
        part_uuid,
        x_nm,
        y_nm,
        layer,
    )
}

fn execute_native_project_forward_annotation_action(
    root: &Path,
    action: NativeProjectForwardAnnotationProposalActionView,
    package_uuid: Option<Uuid>,
    part_uuid: Option<Uuid>,
    x_nm: Option<i64>,
    y_nm: Option<i64>,
    layer: Option<i32>,
) -> Result<NativeProjectForwardAnnotationApplyReportView> {
    let component_report = match (action.action.as_str(), action.reason.as_str()) {
        ("remove_component", "board_component_missing_in_schematic") => {
            let component_uuid = Uuid::parse_str(
                action
                    .component_uuid
                    .as_deref()
                    .ok_or_else(|| anyhow::anyhow!("proposal action missing component UUID"))?,
            )
            .context("invalid component UUID in forward-annotation proposal")?;
            delete_native_project_board_component(root, component_uuid)?
        }
        ("update_component", "value_mismatch") => {
            let component_uuid = Uuid::parse_str(
                action
                    .component_uuid
                    .as_deref()
                    .ok_or_else(|| anyhow::anyhow!("proposal action missing component UUID"))?,
            )
            .context("invalid component UUID in forward-annotation proposal")?;
            set_native_project_board_component_value(
                root,
                component_uuid,
                action
                    .schematic_value
                    .clone()
                    .ok_or_else(|| anyhow::anyhow!("proposal action missing schematic value"))?,
            )?
        }
        ("add_component", _) => {
            let package_uuid = package_uuid.ok_or_else(|| {
                anyhow::anyhow!("forward-annotation add_component apply requires --package <uuid>")
            })?;
            let x_nm = x_nm.ok_or_else(|| {
                anyhow::anyhow!("forward-annotation add_component apply requires --x-nm <i64>")
            })?;
            let y_nm = y_nm.ok_or_else(|| {
                anyhow::anyhow!("forward-annotation add_component apply requires --y-nm <i64>")
            })?;
            let layer = layer.ok_or_else(|| {
                anyhow::anyhow!("forward-annotation add_component apply requires --layer <i32>")
            })?;
            let resolved_part_uuid = match (part_uuid, action.schematic_part_uuid.as_deref()) {
                (Some(part_uuid), _) => part_uuid,
                (None, Some(part_uuid)) => Uuid::parse_str(part_uuid)
                    .context("invalid schematic part UUID in forward-annotation proposal")?,
                (None, None) => {
                    bail!(
                        "forward-annotation add_component apply requires --part <uuid> when the proposal does not carry a resolved schematic part"
                    )
                }
            };
            place_native_project_board_component(
                root,
                resolved_part_uuid,
                package_uuid,
                action.reference.clone(),
                action
                    .schematic_value
                    .clone()
                    .ok_or_else(|| anyhow::anyhow!("proposal action missing schematic value"))?,
                Point::new(x_nm, y_nm),
                layer,
            )?
        }
        ("update_component", "part_mismatch") => {
            let component_uuid = Uuid::parse_str(
                action
                    .component_uuid
                    .as_deref()
                    .ok_or_else(|| anyhow::anyhow!("proposal action missing component UUID"))?,
            )
            .context("invalid component UUID in forward-annotation proposal")?;
            let package_uuid = package_uuid.ok_or_else(|| {
                anyhow::anyhow!("forward-annotation part_mismatch apply requires --package <uuid>")
            })?;
            let resolved_part_uuid = match (part_uuid, action.schematic_part_uuid.as_deref()) {
                (Some(part_uuid), _) => part_uuid,
                (None, Some(part_uuid)) => Uuid::parse_str(part_uuid)
                    .context("invalid schematic part UUID in forward-annotation proposal")?,
                (None, None) => {
                    bail!(
                        "forward-annotation part_mismatch apply requires --part <uuid> when the proposal does not carry a resolved schematic part"
                    )
                }
            };
            let _ = set_native_project_board_component_package(root, component_uuid, package_uuid)?;
            set_native_project_board_component_part(root, component_uuid, resolved_part_uuid)?
        }
        _ => bail!(
            "forward-annotation apply is not supported for {} reason={}",
            action.action,
            action.reason
        ),
    };

    Ok(NativeProjectForwardAnnotationApplyReportView {
        action: "apply_forward_annotation_action".to_string(),
        action_id: action.action_id,
        proposal_action: action.action,
        reason: action.reason,
        component_report,
    })
}

pub(crate) fn apply_native_project_forward_annotation_reviewed(
    root: &Path,
) -> Result<NativeProjectForwardAnnotationBatchApplyReportView> {
    let proposal = query_native_project_forward_annotation_proposal(root)?;
    let project = load_native_project(root)?;
    let review = project.manifest.forward_annotation_review;
    let mut applied = Vec::new();
    let mut skipped = Vec::new();

    for action in proposal.actions {
        if let Some(review_record) = review.get(&action.action_id) {
            let skip_reason = match review_record.decision.as_str() {
                "deferred" => Some("deferred_by_review"),
                "rejected" => Some("rejected_by_review"),
                _ => None,
            };
            if let Some(skip_reason) = skip_reason {
                skipped.push(NativeProjectForwardAnnotationBatchApplySkippedActionView {
                    action_id: action.action_id.clone(),
                    proposal_action: action.action.clone(),
                    reference: action.reference.clone(),
                    reason: action.reason.clone(),
                    skip_reason: skip_reason.to_string(),
                });
                continue;
            }
        }

        match (action.action.as_str(), action.reason.as_str()) {
            ("remove_component", "board_component_missing_in_schematic")
            | ("update_component", "value_mismatch") => {
                applied.push(execute_native_project_forward_annotation_action(
                    root, action, None, None, None, None, None,
                )?);
            }
            ("add_component", _) | ("update_component", "part_mismatch") => {
                skipped.push(NativeProjectForwardAnnotationBatchApplySkippedActionView {
                    action_id: action.action_id.clone(),
                    proposal_action: action.action.clone(),
                    reference: action.reference.clone(),
                    reason: action.reason.clone(),
                    skip_reason: "requires_explicit_input".to_string(),
                });
            }
            _ => {
                skipped.push(NativeProjectForwardAnnotationBatchApplySkippedActionView {
                    action_id: action.action_id.clone(),
                    proposal_action: action.action.clone(),
                    reference: action.reference.clone(),
                    reason: action.reason.clone(),
                    skip_reason: "unsupported_action".to_string(),
                });
            }
        }
    }

    let skipped_deferred_actions = skipped
        .iter()
        .filter(|entry| entry.skip_reason == "deferred_by_review")
        .count();
    let skipped_rejected_actions = skipped
        .iter()
        .filter(|entry| entry.skip_reason == "rejected_by_review")
        .count();
    let skipped_requires_input_actions = skipped
        .iter()
        .filter(|entry| entry.skip_reason == "requires_explicit_input")
        .count();

    Ok(NativeProjectForwardAnnotationBatchApplyReportView {
        action: "apply_forward_annotation_reviewed".to_string(),
        domain: "native_project",
        proposal_actions: applied.len() + skipped.len(),
        applied_actions: applied.len(),
        skipped_deferred_actions,
        skipped_rejected_actions,
        skipped_requires_input_actions,
        applied,
        skipped,
    })
}

pub(crate) fn query_native_project_forward_annotation_review(
    root: &Path,
) -> Result<NativeProjectForwardAnnotationReviewView> {
    let project = load_native_project(root)?;
    let mut actions = project
        .manifest
        .forward_annotation_review
        .values()
        .map(|record| NativeProjectForwardAnnotationReviewActionView {
            action_id: record.action_id.clone(),
            decision: record.decision.clone(),
            proposal_action: record.proposal_action.clone(),
            reference: record.reference.clone(),
            reason: record.reason.clone(),
        })
        .collect::<Vec<_>>();
    actions.sort_by(|a, b| {
        a.reference
            .cmp(&b.reference)
            .then_with(|| a.proposal_action.cmp(&b.proposal_action))
            .then_with(|| a.reason.cmp(&b.reason))
            .then_with(|| a.action_id.cmp(&b.action_id))
    });
    let deferred_actions = actions
        .iter()
        .filter(|action| action.decision == "deferred")
        .count();
    let rejected_actions = actions
        .iter()
        .filter(|action| action.decision == "rejected")
        .count();
    Ok(NativeProjectForwardAnnotationReviewView {
        domain: "native_project",
        total_reviews: actions.len(),
        deferred_actions,
        rejected_actions,
        actions,
    })
}

pub(crate) fn record_native_project_forward_annotation_review(
    root: &Path,
    action_id: &str,
    decision: &str,
) -> Result<NativeProjectForwardAnnotationReviewReportView> {
    if decision != "deferred" && decision != "rejected" {
        bail!("unsupported forward-annotation review decision: {decision}");
    }

    let proposal = query_native_project_forward_annotation_proposal(root)?;
    let action = proposal
        .actions
        .into_iter()
        .find(|action| action.action_id == action_id)
        .ok_or_else(|| {
            anyhow::anyhow!("forward-annotation proposal action not found: {action_id}")
        })?;

    let mut project = load_native_project(root)?;
    project.manifest.forward_annotation_review.insert(
        action.action_id.clone(),
        NativeForwardAnnotationReviewRecord {
            action_id: action.action_id.clone(),
            decision: decision.to_string(),
            proposal_action: action.action.clone(),
            reference: action.reference.clone(),
            reason: action.reason.clone(),
        },
    );
    write_canonical_json(&project.root.join("project.json"), &project.manifest)?;

    Ok(NativeProjectForwardAnnotationReviewReportView {
        action: format!("{decision}_forward_annotation_action"),
        action_id: action.action_id,
        decision: decision.to_string(),
        proposal_action: action.action,
        reference: action.reference,
        reason: action.reason,
    })
}

pub(crate) fn clear_native_project_forward_annotation_review(
    root: &Path,
    action_id: &str,
) -> Result<NativeProjectForwardAnnotationReviewReportView> {
    let mut project = load_native_project(root)?;
    let cleared = project
        .manifest
        .forward_annotation_review
        .remove(action_id)
        .ok_or_else(|| {
            anyhow::anyhow!("forward-annotation review action not found: {action_id}")
        })?;
    write_canonical_json(&project.root.join("project.json"), &project.manifest)?;
    Ok(NativeProjectForwardAnnotationReviewReportView {
        action: "clear_forward_annotation_action_review".to_string(),
        action_id: cleared.action_id,
        decision: cleared.decision,
        proposal_action: cleared.proposal_action,
        reference: cleared.reference,
        reason: cleared.reason,
    })
}

pub(crate) fn export_native_project_forward_annotation_proposal(
    root: &Path,
    output_path: &Path,
) -> Result<NativeProjectForwardAnnotationExportReportView> {
    let project = load_native_project(root)?;
    let proposal = query_native_project_forward_annotation_proposal(root)?;
    let review = query_native_project_forward_annotation_review(root)?;
    let artifact = ForwardAnnotationProposalArtifact {
        kind: FORWARD_ANNOTATION_ARTIFACT_KIND.to_string(),
        version: FORWARD_ANNOTATION_ARTIFACT_VERSION,
        project_uuid: project.manifest.uuid,
        project_name: project.manifest.name.clone(),
        actions: proposal.actions,
        reviews: review.actions,
    };
    write_canonical_json(output_path, &artifact)?;
    Ok(NativeProjectForwardAnnotationExportReportView {
        action: "export_forward_annotation_proposal".to_string(),
        artifact_path: output_path.display().to_string(),
        kind: artifact.kind,
        version: artifact.version,
        project_uuid: artifact.project_uuid.to_string(),
        actions: artifact.actions.len(),
        reviews: artifact.reviews.len(),
    })
}

pub(crate) fn export_native_project_forward_annotation_proposal_selection(
    root: &Path,
    action_ids: &[String],
    output_path: &Path,
) -> Result<NativeProjectForwardAnnotationExportReportView> {
    if action_ids.is_empty() {
        bail!("forward-annotation proposal selection export requires at least one --action-id");
    }

    let project = load_native_project(root)?;
    let proposal = query_native_project_forward_annotation_proposal(root)?;
    let review = query_native_project_forward_annotation_review(root)?;
    let selected_action_ids = action_ids.iter().cloned().collect::<BTreeSet<_>>();
    let actions = proposal
        .actions
        .into_iter()
        .filter(|action| selected_action_ids.contains(&action.action_id))
        .collect::<Vec<_>>();
    if actions.len() != selected_action_ids.len() {
        let exported_action_ids = actions
            .iter()
            .map(|action| action.action_id.as_str())
            .collect::<BTreeSet<_>>();
        let missing = selected_action_ids
            .iter()
            .filter(|action_id| !exported_action_ids.contains(action_id.as_str()))
            .cloned()
            .collect::<Vec<_>>();
        bail!(
            "forward-annotation proposal action not found for selection export: {}",
            missing.join(", ")
        );
    }

    let reviews = review
        .actions
        .into_iter()
        .filter(|entry| selected_action_ids.contains(&entry.action_id))
        .collect::<Vec<_>>();
    let artifact = ForwardAnnotationProposalArtifact {
        kind: FORWARD_ANNOTATION_ARTIFACT_KIND.to_string(),
        version: FORWARD_ANNOTATION_ARTIFACT_VERSION,
        project_uuid: project.manifest.uuid,
        project_name: project.manifest.name.clone(),
        actions,
        reviews,
    };
    write_canonical_json(output_path, &artifact)?;
    Ok(NativeProjectForwardAnnotationExportReportView {
        action: "export_forward_annotation_proposal_selection".to_string(),
        artifact_path: output_path.display().to_string(),
        kind: artifact.kind,
        version: artifact.version,
        project_uuid: artifact.project_uuid.to_string(),
        actions: artifact.actions.len(),
        reviews: artifact.reviews.len(),
    })
}

pub(crate) fn select_forward_annotation_proposal_artifact(
    artifact_path: &Path,
    action_ids: &[String],
    output_path: &Path,
) -> Result<NativeProjectForwardAnnotationExportReportView> {
    if action_ids.is_empty() {
        bail!("forward-annotation artifact selection requires at least one --action-id");
    }

    let loaded = load_forward_annotation_proposal_artifact(artifact_path)?;
    let selected_action_ids = action_ids.iter().cloned().collect::<BTreeSet<_>>();
    let actions = loaded
        .artifact
        .actions
        .into_iter()
        .filter(|action| selected_action_ids.contains(&action.action_id))
        .collect::<Vec<_>>();
    if actions.len() != selected_action_ids.len() {
        let exported_action_ids = actions
            .iter()
            .map(|action| action.action_id.as_str())
            .collect::<BTreeSet<_>>();
        let missing = selected_action_ids
            .iter()
            .filter(|action_id| !exported_action_ids.contains(action_id.as_str()))
            .cloned()
            .collect::<Vec<_>>();
        bail!(
            "forward-annotation artifact action not found for selection: {}",
            missing.join(", ")
        );
    }

    let reviews = loaded
        .artifact
        .reviews
        .into_iter()
        .filter(|entry| selected_action_ids.contains(&entry.action_id))
        .collect::<Vec<_>>();
    let artifact = ForwardAnnotationProposalArtifact {
        kind: FORWARD_ANNOTATION_ARTIFACT_KIND.to_string(),
        version: FORWARD_ANNOTATION_ARTIFACT_VERSION,
        project_uuid: loaded.artifact.project_uuid,
        project_name: loaded.artifact.project_name,
        actions,
        reviews,
    };
    write_canonical_json(output_path, &artifact)?;
    Ok(NativeProjectForwardAnnotationExportReportView {
        action: "select_forward_annotation_proposal_artifact".to_string(),
        artifact_path: output_path.display().to_string(),
        kind: artifact.kind,
        version: artifact.version,
        project_uuid: artifact.project_uuid.to_string(),
        actions: artifact.actions.len(),
        reviews: artifact.reviews.len(),
    })
}

fn load_forward_annotation_proposal_artifact(
    artifact_path: &Path,
) -> Result<LoadedForwardAnnotationProposalArtifact> {
    let contents = std::fs::read_to_string(artifact_path).with_context(|| {
        format!(
            "failed to read forward-annotation artifact {}",
            artifact_path.display()
        )
    })?;
    let value = serde_json::from_str::<serde_json::Value>(&contents).with_context(|| {
        format!(
            "failed to parse forward-annotation artifact {}",
            artifact_path.display()
        )
    })?;

    let kind = value.get("kind").and_then(serde_json::Value::as_str);
    if let Some(kind) = kind
        && kind != FORWARD_ANNOTATION_ARTIFACT_KIND
    {
        bail!(
            "unsupported forward-annotation artifact kind '{}' in {}",
            kind,
            artifact_path.display()
        );
    }

    let version = match value.get("version") {
        Some(version) => {
            let raw = version.as_u64().ok_or_else(|| {
                anyhow::Error::msg(format!(
                    "invalid forward-annotation artifact version in {}",
                    artifact_path.display()
                ))
            })?;
            u32::try_from(raw).map_err(|_| {
                anyhow::Error::msg(format!(
                    "invalid forward-annotation artifact version in {}",
                    artifact_path.display()
                ))
            })?
        }
        None => 0,
    };

    match version {
        FORWARD_ANNOTATION_ARTIFACT_VERSION => {
            let artifact = serde_json::from_value::<ForwardAnnotationProposalArtifact>(value)
                .with_context(|| {
                    format!(
                        "failed to parse forward-annotation artifact {}",
                        artifact_path.display()
                    )
                })?;
            if artifact.kind != FORWARD_ANNOTATION_ARTIFACT_KIND {
                bail!(
                    "unsupported forward-annotation artifact kind '{}' in {}",
                    artifact.kind,
                    artifact_path.display()
                );
            }
            Ok(LoadedForwardAnnotationProposalArtifact {
                artifact_path: artifact_path.to_path_buf(),
                source_version: FORWARD_ANNOTATION_ARTIFACT_VERSION,
                artifact,
            })
        }
        _ => {
            bail!(
                "unsupported forward-annotation artifact version {} in {}",
                version,
                artifact_path.display()
            );
        }
    }
}

pub(crate) fn inspect_forward_annotation_proposal_artifact(
    artifact_path: &Path,
) -> Result<NativeProjectForwardAnnotationArtifactInspectionView> {
    let loaded = load_forward_annotation_proposal_artifact(artifact_path)?;
    let add_component_actions = loaded
        .artifact
        .actions
        .iter()
        .filter(|action| action.action == "add_component")
        .count();
    let remove_component_actions = loaded
        .artifact
        .actions
        .iter()
        .filter(|action| action.action == "remove_component")
        .count();
    let update_component_actions = loaded
        .artifact
        .actions
        .iter()
        .filter(|action| action.action == "update_component")
        .count();
    let deferred_reviews = loaded
        .artifact
        .reviews
        .iter()
        .filter(|review| review.decision == "deferred")
        .count();
    let rejected_reviews = loaded
        .artifact
        .reviews
        .iter()
        .filter(|review| review.decision == "rejected")
        .count();

    Ok(NativeProjectForwardAnnotationArtifactInspectionView {
        artifact_path: loaded.artifact_path.display().to_string(),
        kind: loaded.artifact.kind,
        source_version: loaded.source_version,
        version: loaded.artifact.version,
        migration_applied: false,
        project_uuid: loaded.artifact.project_uuid.to_string(),
        project_name: loaded.artifact.project_name,
        actions: loaded.artifact.actions.len(),
        reviews: loaded.artifact.reviews.len(),
        add_component_actions,
        remove_component_actions,
        update_component_actions,
        deferred_reviews,
        rejected_reviews,
    })
}

pub(crate) fn compare_forward_annotation_proposal_artifact(
    root: &Path,
    artifact_path: &Path,
) -> Result<NativeProjectForwardAnnotationArtifactComparisonView> {
    let project = load_native_project(root)?;
    let loaded = load_forward_annotation_proposal_artifact(artifact_path)?;
    let current_proposal = query_native_project_forward_annotation_proposal(root)?;

    let mut current_by_id = BTreeMap::new();
    let mut current_by_reference_and_action = BTreeMap::new();
    for action in current_proposal.actions {
        current_by_reference_and_action.insert(
            (action.reference.clone(), action.action.clone()),
            action.action_id.clone(),
        );
        current_by_id.insert(action.action_id.clone(), action);
    }

    let review_by_id = loaded
        .artifact
        .reviews
        .iter()
        .map(|review| (review.action_id.clone(), review.decision.clone()))
        .collect::<BTreeMap<_, _>>();

    let mut actions = Vec::new();
    for action in &loaded.artifact.actions {
        let status = if current_by_id.contains_key(&action.action_id) {
            "applicable"
        } else if current_by_reference_and_action
            .contains_key(&(action.reference.clone(), action.action.clone()))
        {
            "drifted"
        } else {
            "stale"
        };
        actions.push(NativeProjectForwardAnnotationArtifactComparisonActionView {
            action_id: action.action_id.clone(),
            proposal_action: action.action.clone(),
            reference: action.reference.clone(),
            reason: action.reason.clone(),
            status: status.to_string(),
            review_decision: review_by_id.get(&action.action_id).cloned(),
        });
    }
    actions.sort_by(|a, b| {
        a.reference
            .cmp(&b.reference)
            .then_with(|| a.proposal_action.cmp(&b.proposal_action))
            .then_with(|| a.reason.cmp(&b.reason))
            .then_with(|| a.action_id.cmp(&b.action_id))
    });

    let applicable_actions = actions
        .iter()
        .filter(|action| action.status == "applicable")
        .count();
    let drifted_actions = actions
        .iter()
        .filter(|action| action.status == "drifted")
        .count();
    let stale_actions = actions
        .iter()
        .filter(|action| action.status == "stale")
        .count();

    Ok(NativeProjectForwardAnnotationArtifactComparisonView {
        artifact_path: loaded.artifact_path.display().to_string(),
        project_root: project.root.display().to_string(),
        kind: loaded.artifact.kind,
        artifact_version: loaded.artifact.version,
        current_project_uuid: project.manifest.uuid.to_string(),
        artifact_project_uuid: loaded.artifact.project_uuid.to_string(),
        artifact_actions: actions.len(),
        applicable_actions,
        drifted_actions,
        stale_actions,
        actions,
    })
}

pub(crate) fn filter_forward_annotation_proposal_artifact(
    root: &Path,
    artifact_path: &Path,
    output_path: &Path,
) -> Result<NativeProjectForwardAnnotationArtifactFilterView> {
    let project = load_native_project(root)?;
    let loaded = load_forward_annotation_proposal_artifact(artifact_path)?;
    let comparison = compare_forward_annotation_proposal_artifact(root, artifact_path)?;
    let applicable_action_ids = comparison
        .actions
        .iter()
        .filter(|action| action.status == "applicable")
        .map(|action| action.action_id.as_str())
        .collect::<BTreeSet<_>>();

    let filtered_artifact = ForwardAnnotationProposalArtifact {
        kind: loaded.artifact.kind,
        version: loaded.artifact.version,
        project_uuid: loaded.artifact.project_uuid,
        project_name: loaded.artifact.project_name,
        actions: loaded
            .artifact
            .actions
            .into_iter()
            .filter(|action| applicable_action_ids.contains(action.action_id.as_str()))
            .collect(),
        reviews: loaded
            .artifact
            .reviews
            .into_iter()
            .filter(|review| applicable_action_ids.contains(review.action_id.as_str()))
            .collect(),
    };
    write_canonical_json(output_path, &filtered_artifact)?;

    Ok(NativeProjectForwardAnnotationArtifactFilterView {
        action: "filter_forward_annotation_proposal_artifact".to_string(),
        input_artifact_path: loaded.artifact_path.display().to_string(),
        output_artifact_path: output_path.display().to_string(),
        project_root: project.root.display().to_string(),
        kind: filtered_artifact.kind,
        version: filtered_artifact.version,
        artifact_actions: comparison.artifact_actions,
        applicable_actions: filtered_artifact.actions.len(),
        filtered_reviews: filtered_artifact.reviews.len(),
    })
}

fn forward_annotation_apply_required_inputs(
    action: &NativeProjectForwardAnnotationProposalActionView,
) -> (&'static str, Vec<String>) {
    match (action.action.as_str(), action.reason.as_str()) {
        ("remove_component", "board_component_missing_in_schematic") => {
            ("self_sufficient", Vec::new())
        }
        ("update_component", "value_mismatch") => ("self_sufficient", Vec::new()),
        ("add_component", _) => {
            let mut required = vec![
                "package_uuid".to_string(),
                "x_nm".to_string(),
                "y_nm".to_string(),
                "layer".to_string(),
            ];
            if action.schematic_part_uuid.is_none() {
                required.push("part_uuid".to_string());
            }
            ("requires_explicit_input", required)
        }
        ("update_component", "part_mismatch") => {
            let mut required = vec!["package_uuid".to_string()];
            if action.schematic_part_uuid.is_none() {
                required.push("part_uuid".to_string());
            }
            ("requires_explicit_input", required)
        }
        _ => ("unsupported", Vec::new()),
    }
}

pub(crate) fn plan_forward_annotation_proposal_artifact_apply(
    root: &Path,
    artifact_path: &Path,
) -> Result<NativeProjectForwardAnnotationArtifactApplyPlanView> {
    let comparison = compare_forward_annotation_proposal_artifact(root, artifact_path)?;
    let loaded = load_forward_annotation_proposal_artifact(artifact_path)?;
    let review_by_id = loaded
        .artifact
        .reviews
        .iter()
        .map(|review| (review.action_id.clone(), review.decision.clone()))
        .collect::<BTreeMap<_, _>>();
    let actions_by_id = loaded
        .artifact
        .actions
        .iter()
        .map(|action| (action.action_id.clone(), action.clone()))
        .collect::<BTreeMap<_, _>>();

    let mut actions = Vec::new();
    for comparison_action in comparison.actions {
        let artifact_action = actions_by_id
            .get(&comparison_action.action_id)
            .ok_or_else(|| anyhow::anyhow!("artifact action missing during apply planning"))?;
        let (execution, required_inputs) = if comparison_action.status == "applicable" {
            let (execution, required_inputs) =
                forward_annotation_apply_required_inputs(artifact_action);
            (execution.to_string(), required_inputs)
        } else {
            ("not_applicable".to_string(), Vec::new())
        };
        actions.push(NativeProjectForwardAnnotationArtifactApplyPlanActionView {
            action_id: comparison_action.action_id,
            proposal_action: comparison_action.proposal_action,
            reference: comparison_action.reference,
            reason: comparison_action.reason,
            applicability: comparison_action.status,
            execution,
            review_decision: review_by_id.get(&artifact_action.action_id).cloned(),
            required_inputs,
        });
    }

    let self_sufficient_actions = actions
        .iter()
        .filter(|action| action.execution == "self_sufficient")
        .count();
    let requires_input_actions = actions
        .iter()
        .filter(|action| action.execution == "requires_explicit_input")
        .count();
    let not_applicable_actions = actions
        .iter()
        .filter(|action| action.execution == "not_applicable")
        .count();

    Ok(NativeProjectForwardAnnotationArtifactApplyPlanView {
        action: "plan_forward_annotation_proposal_artifact_apply".to_string(),
        artifact_path: loaded.artifact_path.display().to_string(),
        project_root: root.display().to_string(),
        kind: loaded.artifact.kind,
        artifact_version: loaded.artifact.version,
        artifact_actions: actions.len(),
        self_sufficient_actions,
        requires_input_actions,
        not_applicable_actions,
        actions,
    })
}

pub(crate) fn apply_forward_annotation_proposal_artifact(
    root: &Path,
    artifact_path: &Path,
) -> Result<NativeProjectForwardAnnotationArtifactApplyView> {
    let project = load_native_project(root)?;
    let loaded = load_forward_annotation_proposal_artifact(artifact_path)?;
    if loaded.artifact.project_uuid != project.manifest.uuid {
        bail!(
            "forward-annotation artifact project UUID {} does not match current project UUID {}",
            loaded.artifact.project_uuid,
            project.manifest.uuid
        );
    }

    let plan = plan_forward_annotation_proposal_artifact_apply(root, artifact_path)?;
    let non_applicable = plan
        .actions
        .iter()
        .find(|action| action.applicability != "applicable");
    if let Some(action) = non_applicable {
        bail!(
            "forward-annotation artifact apply requires only applicable actions; action {} is {}",
            action.action_id,
            action.applicability
        );
    }
    let input_bound = plan
        .actions
        .iter()
        .find(|action| action.execution != "self_sufficient");
    if let Some(action) = input_bound {
        bail!(
            "forward-annotation artifact apply requires only self-sufficient actions; action {} is {}",
            action.action_id,
            action.execution
        );
    }

    let review_by_id = loaded
        .artifact
        .reviews
        .iter()
        .map(|review| (review.action_id.clone(), review.decision.clone()))
        .collect::<BTreeMap<_, _>>();

    let mut applied = Vec::new();
    let mut skipped = Vec::new();
    for action in loaded.artifact.actions {
        if let Some(review_decision) = review_by_id.get(&action.action_id) {
            let skip_reason = match review_decision.as_str() {
                "deferred" => Some("deferred_by_review"),
                "rejected" => Some("rejected_by_review"),
                _ => None,
            };
            if let Some(skip_reason) = skip_reason {
                skipped.push(NativeProjectForwardAnnotationBatchApplySkippedActionView {
                    action_id: action.action_id.clone(),
                    proposal_action: action.action.clone(),
                    reference: action.reference.clone(),
                    reason: action.reason.clone(),
                    skip_reason: skip_reason.to_string(),
                });
                continue;
            }
        }

        applied.push(execute_native_project_forward_annotation_action(
            root, action, None, None, None, None, None,
        )?);
    }

    let skipped_deferred_actions = skipped
        .iter()
        .filter(|entry| entry.skip_reason == "deferred_by_review")
        .count();
    let skipped_rejected_actions = skipped
        .iter()
        .filter(|entry| entry.skip_reason == "rejected_by_review")
        .count();

    Ok(NativeProjectForwardAnnotationArtifactApplyView {
        action: "apply_forward_annotation_proposal_artifact".to_string(),
        artifact_path: loaded.artifact_path.display().to_string(),
        project_root: project.root.display().to_string(),
        artifact_actions: plan.artifact_actions,
        applied_actions: applied.len(),
        skipped_deferred_actions,
        skipped_rejected_actions,
        applied,
        skipped,
    })
}

pub(crate) fn import_forward_annotation_artifact_review(
    root: &Path,
    artifact_path: &Path,
) -> Result<NativeProjectForwardAnnotationArtifactReviewImportView> {
    let mut project = load_native_project(root)?;
    let loaded = load_forward_annotation_proposal_artifact(artifact_path)?;
    if loaded.artifact.project_uuid != project.manifest.uuid {
        bail!(
            "forward-annotation artifact project UUID {} does not match current project UUID {}",
            loaded.artifact.project_uuid,
            project.manifest.uuid
        );
    }

    let live_proposal = query_native_project_forward_annotation_proposal(root)?;
    let live_actions = live_proposal
        .actions
        .into_iter()
        .map(|action| (action.action_id.clone(), action))
        .collect::<BTreeMap<_, _>>();

    let total_artifact_reviews = loaded.artifact.reviews.len();
    let mut imported_reviews = 0usize;
    let mut skipped_missing_live_actions = 0usize;
    for review in loaded.artifact.reviews {
        if let Some(live_action) = live_actions.get(&review.action_id) {
            project.manifest.forward_annotation_review.insert(
                review.action_id.clone(),
                NativeForwardAnnotationReviewRecord {
                    action_id: review.action_id,
                    decision: review.decision,
                    proposal_action: live_action.action.clone(),
                    reference: live_action.reference.clone(),
                    reason: live_action.reason.clone(),
                },
            );
            imported_reviews += 1;
        } else {
            skipped_missing_live_actions += 1;
        }
    }

    write_canonical_json(&project.root.join("project.json"), &project.manifest)?;
    Ok(NativeProjectForwardAnnotationArtifactReviewImportView {
        action: "import_forward_annotation_artifact_review".to_string(),
        artifact_path: loaded.artifact_path.display().to_string(),
        project_root: project.root.display().to_string(),
        imported_reviews,
        skipped_missing_live_actions,
        total_artifact_reviews,
    })
}

pub(crate) fn replace_forward_annotation_artifact_review(
    root: &Path,
    artifact_path: &Path,
) -> Result<NativeProjectForwardAnnotationArtifactReviewReplaceView> {
    let mut project = load_native_project(root)?;
    let loaded = load_forward_annotation_proposal_artifact(artifact_path)?;
    if loaded.artifact.project_uuid != project.manifest.uuid {
        bail!(
            "forward-annotation artifact project UUID {} does not match current project UUID {}",
            loaded.artifact.project_uuid,
            project.manifest.uuid
        );
    }

    let live_proposal = query_native_project_forward_annotation_proposal(root)?;
    let live_actions = live_proposal
        .actions
        .into_iter()
        .map(|action| (action.action_id.clone(), action))
        .collect::<BTreeMap<_, _>>();

    let total_artifact_reviews = loaded.artifact.reviews.len();
    let removed_existing_reviews = project.manifest.forward_annotation_review.len();
    let mut replacement_reviews = BTreeMap::new();
    let mut replaced_reviews = 0usize;
    let mut skipped_missing_live_actions = 0usize;
    for review in loaded.artifact.reviews {
        if let Some(live_action) = live_actions.get(&review.action_id) {
            replacement_reviews.insert(
                review.action_id.clone(),
                NativeForwardAnnotationReviewRecord {
                    action_id: review.action_id,
                    decision: review.decision,
                    proposal_action: live_action.action.clone(),
                    reference: live_action.reference.clone(),
                    reason: live_action.reason.clone(),
                },
            );
            replaced_reviews += 1;
        } else {
            skipped_missing_live_actions += 1;
        }
    }

    project.manifest.forward_annotation_review = replacement_reviews;
    write_canonical_json(&project.root.join("project.json"), &project.manifest)?;
    Ok(NativeProjectForwardAnnotationArtifactReviewReplaceView {
        action: "replace_forward_annotation_artifact_review".to_string(),
        artifact_path: loaded.artifact_path.display().to_string(),
        project_root: project.root.display().to_string(),
        replaced_reviews,
        removed_existing_reviews,
        skipped_missing_live_actions,
        total_artifact_reviews,
    })
}

fn forward_annotation_action_id(
    action: &str,
    reference: &str,
    symbol_uuid: Option<&str>,
    component_uuid: Option<&str>,
    reason: &str,
) -> String {
    let stable_key = format!(
        "{action}|{reference}|{}|{}|{reason}",
        symbol_uuid.unwrap_or(""),
        component_uuid.unwrap_or("")
    );
    compute_source_hash_bytes(stable_key.as_bytes())
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
        if let Some(entries) = sheet_value
            .get("symbols")
            .and_then(serde_json::Value::as_object)
        {
            for value in entries.values() {
                let symbol: PlacedSymbol = serde_json::from_value(value.clone())
                    .with_context(|| format!("failed to parse symbol in {}", path.display()))?;
                symbols.push(SymbolInfo {
                    uuid: symbol.uuid,
                    sheet: sheet_uuid,
                    reference: symbol.reference,
                    value: symbol.value,
                    lib_id: symbol.lib_id,
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
    symbols.sort_by(|a, b| {
        a.reference
            .cmp(&b.reference)
            .then_with(|| a.uuid.cmp(&b.uuid))
    });
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
            let pin_override = symbol
                .pin_overrides
                .iter()
                .find(|entry| entry.pin == pin.uuid);
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
    pins.sort_by(|a, b| {
        a.number
            .cmp(&b.number)
            .then_with(|| a.pin_uuid.cmp(&b.pin_uuid))
    });
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
        if let Some(entries) = sheet_value
            .get("texts")
            .and_then(serde_json::Value::as_object)
        {
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
        if let Some(entries) = sheet_value
            .get("labels")
            .and_then(serde_json::Value::as_object)
        {
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
        if let Some(entries) = sheet_value
            .get("wires")
            .and_then(serde_json::Value::as_object)
        {
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
        if let Some(entries) = sheet_value
            .get("ports")
            .and_then(serde_json::Value::as_object)
        {
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
        if let Some(entries) = sheet_value
            .get("buses")
            .and_then(serde_json::Value::as_object)
        {
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
        if let Some(values) = sheet_value
            .get("bus_entries")
            .and_then(serde_json::Value::as_object)
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

pub(crate) fn query_native_project_nets(root: &Path) -> Result<Vec<SchematicNetInfo>> {
    let project = load_native_project(root)?;
    Ok(schematic_net_info(&build_native_project_schematic(
        &project,
    )?))
}

pub(crate) fn query_native_project_diagnostics(
    root: &Path,
) -> Result<Vec<ConnectivityDiagnosticInfo>> {
    let project = load_native_project(root)?;
    Ok(schematic_diagnostics(&build_native_project_schematic(
        &project,
    )?))
}

pub(crate) fn query_native_project_erc(root: &Path) -> Result<Vec<ErcFinding>> {
    let project = load_native_project(root)?;
    Ok(run_prechecks(&build_native_project_schematic(&project)?))
}

pub(crate) fn query_native_project_check(root: &Path) -> Result<CheckReport> {
    let project = load_native_project(root)?;
    let schematic = build_native_project_schematic(&project)?;
    let diagnostics = schematic_diagnostics(&schematic);
    let erc = run_prechecks(&schematic);
    Ok(CheckReport::Schematic {
        summary: summarize_native_schematic_checks(&diagnostics, &erc),
        diagnostics,
        erc,
    })
}

pub(crate) fn query_native_project_board_texts(root: &Path) -> Result<Vec<BoardText>> {
    let project = load_native_project(root)?;
    let mut texts = project
        .board
        .texts
        .into_iter()
        .map(|value| serde_json::from_value(value).context("failed to parse board text"))
        .collect::<Result<Vec<BoardText>>>()?;
    texts.sort_by(|a, b| a.text.cmp(&b.text).then_with(|| a.uuid.cmp(&b.uuid)));
    Ok(texts)
}

pub(crate) fn query_native_project_board_keepouts(root: &Path) -> Result<Vec<Keepout>> {
    let project = load_native_project(root)?;
    let mut keepouts = project
        .board
        .keepouts
        .into_iter()
        .map(|value| serde_json::from_value(value).context("failed to parse board keepout"))
        .collect::<Result<Vec<Keepout>>>()?;
    keepouts.sort_by(|a, b| a.kind.cmp(&b.kind).then_with(|| a.uuid.cmp(&b.uuid)));
    Ok(keepouts)
}

pub(crate) fn query_native_project_board_outline(root: &Path) -> Result<Polygon> {
    let project = load_native_project(root)?;
    Ok(Polygon {
        vertices: project
            .board
            .outline
            .vertices
            .into_iter()
            .map(|point| Point {
                x: point.x,
                y: point.y,
            })
            .collect(),
        closed: project.board.outline.closed,
    })
}

pub(crate) fn query_native_project_board_stackup(root: &Path) -> Result<Vec<StackupLayer>> {
    let project = load_native_project(root)?;
    project
        .board
        .stackup
        .layers
        .into_iter()
        .map(|value| serde_json::from_value(value).context("failed to parse board stackup layer"))
        .collect::<Result<Vec<StackupLayer>>>()
}

pub(crate) fn query_native_project_board_components(root: &Path) -> Result<Vec<PlacedPackage>> {
    let project = load_native_project(root)?;
    let mut packages = project
        .board
        .packages
        .into_values()
        .map(|value| serde_json::from_value(value).context("failed to parse board component"))
        .collect::<Result<Vec<PlacedPackage>>>()?;
    packages.sort_by(|a, b| {
        a.reference
            .cmp(&b.reference)
            .then_with(|| a.uuid.cmp(&b.uuid))
    });
    Ok(packages)
}

pub(crate) fn export_native_project_bom(
    root: &Path,
    output_path: &Path,
) -> Result<NativeProjectBomExportView> {
    let project = load_native_project(root)?;
    let components = query_native_project_board_components(root)?;
    let mut csv = String::from(
        "reference,value,part_uuid,package_uuid,layer,x_nm,y_nm,rotation_deg,locked\n",
    );
    for component in &components {
        let row = [
            csv_escape(&component.reference),
            csv_escape(&component.value),
            csv_escape(&component.part.to_string()),
            csv_escape(&component.package.to_string()),
            component.layer.to_string(),
            component.position.x.to_string(),
            component.position.y.to_string(),
            component.rotation.to_string(),
            component.locked.to_string(),
        ]
        .join(",");
        csv.push_str(&row);
        csv.push('\n');
    }
    std::fs::write(output_path, csv)
        .with_context(|| format!("failed to write {}", output_path.display()))?;
    Ok(NativeProjectBomExportView {
        action: "export_bom".to_string(),
        project_root: project.root.display().to_string(),
        bom_path: output_path.display().to_string(),
        rows: components.len(),
    })
}

pub(crate) fn compare_native_project_bom(
    root: &Path,
    bom_path: &Path,
) -> Result<NativeProjectBomComparisonView> {
    let project = load_native_project(root)?;
    let expected = query_native_project_board_components(root)?
        .into_iter()
        .map(component_to_bom_row)
        .collect::<Vec<_>>();
    let actual = parse_bom_csv(bom_path)?;

    let expected_by_reference = expected
        .iter()
        .map(|row| (row.reference.clone(), row.clone()))
        .collect::<BTreeMap<_, _>>();
    let actual_by_reference = actual
        .iter()
        .map(|row| (row.reference.clone(), row.clone()))
        .collect::<BTreeMap<_, _>>();

    let matched = expected_by_reference
        .iter()
        .filter_map(|(reference, expected)| {
            actual_by_reference
                .get(reference)
                .filter(|actual| *actual == expected)
                .map(|_| reference.clone())
        })
        .collect::<Vec<_>>();
    let missing = expected_by_reference
        .keys()
        .filter(|reference| !actual_by_reference.contains_key(*reference))
        .cloned()
        .collect::<Vec<_>>();
    let extra = actual_by_reference
        .keys()
        .filter(|reference| !expected_by_reference.contains_key(*reference))
        .cloned()
        .collect::<Vec<_>>();
    let drift = expected_by_reference
        .iter()
        .filter_map(|(reference, expected)| {
            actual_by_reference.get(reference).and_then(|actual| {
                let fields = expected.diff_fields(actual);
                if fields.is_empty() {
                    None
                } else {
                    Some(NativeProjectBomDriftView {
                        reference: reference.clone(),
                        fields,
                    })
                }
            })
        })
        .collect::<Vec<_>>();

    Ok(NativeProjectBomComparisonView {
        action: "compare_bom".to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        bom_path: bom_path.display().to_string(),
        expected_count: expected.len(),
        actual_count: actual.len(),
        matched_count: matched.len(),
        missing_count: missing.len(),
        extra_count: extra.len(),
        drift_count: drift.len(),
        matched,
        missing,
        extra,
        drift,
    })
}

pub(crate) fn export_native_project_pnp(
    root: &Path,
    output_path: &Path,
) -> Result<NativeProjectPnpExportView> {
    let project = load_native_project(root)?;
    let components = query_native_project_board_components(root)?;
    let mut csv = String::from(
        "reference,x_nm,y_nm,rotation_deg,layer,side,package_uuid,part_uuid,value,locked\n",
    );
    for component in &components {
        let side = if component.layer <= 16 {
            "top"
        } else {
            "bottom"
        };
        let row = [
            csv_escape(&component.reference),
            component.position.x.to_string(),
            component.position.y.to_string(),
            component.rotation.to_string(),
            component.layer.to_string(),
            side.to_string(),
            csv_escape(&component.package.to_string()),
            csv_escape(&component.part.to_string()),
            csv_escape(&component.value),
            component.locked.to_string(),
        ]
        .join(",");
        csv.push_str(&row);
        csv.push('\n');
    }
    std::fs::write(output_path, csv)
        .with_context(|| format!("failed to write {}", output_path.display()))?;
    Ok(NativeProjectPnpExportView {
        action: "export_pnp".to_string(),
        project_root: project.root.display().to_string(),
        pnp_path: output_path.display().to_string(),
        rows: components.len(),
    })
}

pub(crate) fn compare_native_project_pnp(
    root: &Path,
    pnp_path: &Path,
) -> Result<NativeProjectPnpComparisonView> {
    let project = load_native_project(root)?;
    let expected = query_native_project_board_components(root)?
        .into_iter()
        .map(component_to_pnp_row)
        .collect::<Vec<_>>();
    let actual = parse_pnp_csv(pnp_path)?;

    let expected_by_reference = expected
        .iter()
        .map(|row| (row.reference.clone(), row.clone()))
        .collect::<BTreeMap<_, _>>();
    let actual_by_reference = actual
        .iter()
        .map(|row| (row.reference.clone(), row.clone()))
        .collect::<BTreeMap<_, _>>();

    let matched = expected_by_reference
        .iter()
        .filter_map(|(reference, expected)| {
            actual_by_reference
                .get(reference)
                .filter(|actual| *actual == expected)
                .map(|_| reference.clone())
        })
        .collect::<Vec<_>>();
    let missing = expected_by_reference
        .keys()
        .filter(|reference| !actual_by_reference.contains_key(*reference))
        .cloned()
        .collect::<Vec<_>>();
    let extra = actual_by_reference
        .keys()
        .filter(|reference| !expected_by_reference.contains_key(*reference))
        .cloned()
        .collect::<Vec<_>>();
    let drift = expected_by_reference
        .iter()
        .filter_map(|(reference, expected)| {
            actual_by_reference.get(reference).and_then(|actual| {
                let fields = expected.diff_fields(actual);
                if fields.is_empty() {
                    None
                } else {
                    Some(NativeProjectPnpDriftView {
                        reference: reference.clone(),
                        fields,
                    })
                }
            })
        })
        .collect::<Vec<_>>();

    Ok(NativeProjectPnpComparisonView {
        action: "compare_pnp".to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        pnp_path: pnp_path.display().to_string(),
        expected_count: expected.len(),
        actual_count: actual.len(),
        matched_count: matched.len(),
        missing_count: missing.len(),
        extra_count: extra.len(),
        drift_count: drift.len(),
        matched,
        missing,
        extra,
        drift,
    })
}

pub(crate) fn export_native_project_drill(
    root: &Path,
    output_path: &Path,
) -> Result<NativeProjectDrillExportView> {
    let project = load_native_project(root)?;
    let mut vias = query_native_project_board_vias(root)?;
    vias.sort_by(|a, b| {
        a.position
            .x
            .cmp(&b.position.x)
            .then_with(|| a.position.y.cmp(&b.position.y))
            .then_with(|| a.uuid.cmp(&b.uuid))
    });
    let mut csv =
        String::from("via_uuid,net_uuid,x_nm,y_nm,drill_nm,diameter_nm,from_layer,to_layer\n");
    for via in &vias {
        let row = [
            csv_escape(&via.uuid.to_string()),
            csv_escape(&via.net.to_string()),
            via.position.x.to_string(),
            via.position.y.to_string(),
            via.drill.to_string(),
            via.diameter.to_string(),
            via.from_layer.to_string(),
            via.to_layer.to_string(),
        ]
        .join(",");
        csv.push_str(&row);
        csv.push('\n');
    }
    std::fs::write(output_path, csv)
        .with_context(|| format!("failed to write {}", output_path.display()))?;
    Ok(NativeProjectDrillExportView {
        action: "export_drill".to_string(),
        project_root: project.root.display().to_string(),
        drill_path: output_path.display().to_string(),
        rows: vias.len(),
    })
}

pub(crate) fn export_native_project_excellon_drill(
    root: &Path,
    output_path: &Path,
) -> Result<NativeProjectExcellonDrillExportView> {
    let project = load_native_project(root)?;
    let vias = query_native_project_board_vias(root)?;
    let tools = build_excellon_tool_views(&vias);
    let tool_count = tools.len();
    let excellon = render_excellon_drill(&vias)
        .context("failed to render native board vias as Excellon drill")?;
    std::fs::write(output_path, excellon)
        .with_context(|| format!("failed to write {}", output_path.display()))?;
    Ok(NativeProjectExcellonDrillExportView {
        action: "export_excellon_drill".to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        drill_path: output_path.display().to_string(),
        via_count: vias.len(),
        tool_count,
        tools,
    })
}

pub(crate) fn validate_native_project_excellon_drill(
    root: &Path,
    drill_path: &Path,
) -> Result<NativeProjectExcellonDrillValidationView> {
    let project = load_native_project(root)?;
    let vias = query_native_project_board_vias(root)?;
    let tools = build_excellon_tool_views(&vias);
    let tool_count = tools.len();
    let expected = render_excellon_drill(&vias)
        .context("failed to render expected native board vias as Excellon drill")?;
    let actual = std::fs::read_to_string(drill_path)
        .with_context(|| format!("failed to read {}", drill_path.display()))?;

    Ok(NativeProjectExcellonDrillValidationView {
        action: "validate_excellon_drill".to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        drill_path: drill_path.display().to_string(),
        matches_expected: actual == expected,
        expected_bytes: expected.len(),
        actual_bytes: actual.len(),
        via_count: vias.len(),
        tool_count,
        tools,
    })
}

pub(crate) fn inspect_excellon_drill(
    drill_path: &Path,
) -> Result<NativeProjectExcellonDrillInspectionView> {
    let contents = std::fs::read_to_string(drill_path)
        .with_context(|| format!("failed to read {}", drill_path.display()))?;

    let mut metric = false;
    let mut tools = BTreeMap::<String, NativeProjectExcellonDrillToolView>::new();
    let mut current_tool = None::<String>;
    let mut hit_count = 0usize;

    for raw_line in contents.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line == "M48" || line == "%" || line == "M30" {
            continue;
        }
        if line == "METRIC,TZ" {
            metric = true;
            continue;
        }
        if let Some(rest) = line.strip_prefix('T') {
            if let Some((tool_digits, diameter)) = rest.split_once('C') {
                let tool = format!("T{tool_digits}");
                tools.insert(
                    tool.clone(),
                    NativeProjectExcellonDrillToolView {
                        tool,
                        diameter_mm: diameter.to_string(),
                        hits: 0,
                    },
                );
                continue;
            }
            current_tool = Some(format!("T{rest}"));
            continue;
        }
        if line.starts_with('X') {
            let tool = current_tool.clone().with_context(|| {
                format!("drill hit without active tool in {}", drill_path.display())
            })?;
            let entry = tools.get_mut(&tool).with_context(|| {
                format!(
                    "drill hit references unknown tool `{tool}` in {}",
                    drill_path.display()
                )
            })?;
            entry.hits += 1;
            hit_count += 1;
        }
    }

    Ok(NativeProjectExcellonDrillInspectionView {
        action: "inspect_excellon_drill".to_string(),
        drill_path: drill_path.display().to_string(),
        metric,
        tool_count: tools.len(),
        hit_count,
        tools: tools.into_values().collect(),
    })
}

pub(crate) fn compare_native_project_excellon_drill(
    root: &Path,
    drill_path: &Path,
) -> Result<NativeProjectExcellonDrillComparisonView> {
    let project = load_native_project(root)?;
    let vias = query_native_project_board_vias(root)?;
    let expected_tools = build_excellon_tool_views(&vias);
    let actual = inspect_excellon_drill(drill_path)?;

    let expected_by_diameter = expected_tools
        .iter()
        .map(|tool| (tool.diameter_mm.clone(), tool.hits))
        .collect::<BTreeMap<_, _>>();
    let actual_by_diameter = actual
        .tools
        .iter()
        .map(|tool| (tool.diameter_mm.clone(), tool.hits))
        .collect::<BTreeMap<_, _>>();

    let matched = expected_by_diameter
        .iter()
        .filter_map(|(diameter, expected_hits)| {
            actual_by_diameter
                .get(diameter)
                .filter(|actual_hits| **actual_hits == *expected_hits)
                .map(|_| diameter.clone())
        })
        .collect::<Vec<_>>();
    let missing = expected_by_diameter
        .keys()
        .filter(|diameter| !actual_by_diameter.contains_key(*diameter))
        .cloned()
        .collect::<Vec<_>>();
    let extra = actual_by_diameter
        .keys()
        .filter(|diameter| !expected_by_diameter.contains_key(*diameter))
        .cloned()
        .collect::<Vec<_>>();
    let hit_drift = expected_by_diameter
        .iter()
        .filter_map(|(diameter, expected_hits)| {
            actual_by_diameter.get(diameter).and_then(|actual_hits| {
                if actual_hits == expected_hits {
                    None
                } else {
                    Some(NativeProjectExcellonDrillHitDriftView {
                        diameter_mm: diameter.clone(),
                        expected_hits: *expected_hits,
                        actual_hits: *actual_hits,
                    })
                }
            })
        })
        .collect::<Vec<_>>();

    Ok(NativeProjectExcellonDrillComparisonView {
        action: "compare_excellon_drill".to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        drill_path: drill_path.display().to_string(),
        expected_tool_count: expected_tools.len(),
        actual_tool_count: actual.tools.len(),
        expected_hit_count: vias.len(),
        actual_hit_count: actual.hit_count,
        matched_count: matched.len(),
        missing_count: missing.len(),
        extra_count: extra.len(),
        hit_drift_count: hit_drift.len(),
        matched,
        missing,
        extra,
        hit_drift,
    })
}

pub(crate) fn report_native_project_drill_hole_classes(
    root: &Path,
) -> Result<NativeProjectDrillHoleClassReportView> {
    let project = load_native_project(root)?;
    let vias = query_native_project_board_vias(root)?;
    let copper_layers = query_native_project_board_stackup(root)?
        .into_iter()
        .filter(|layer| matches!(layer.layer_type, StackupLayerType::Copper))
        .map(|layer| layer.id)
        .collect::<Vec<_>>();
    let top_copper = copper_layers.iter().min().copied();
    let bottom_copper = copper_layers.iter().max().copied();

    let mut grouped = BTreeMap::<(String, i32, i32), Vec<Via>>::new();
    for via in vias.iter().cloned() {
        let start = via.from_layer.min(via.to_layer);
        let end = via.from_layer.max(via.to_layer);
        let class = classify_via_hole_class(start, end, top_copper, bottom_copper);
        grouped.entry((class, start, end)).or_default().push(via);
    }

    let classes = grouped
        .into_iter()
        .map(|((class, from_layer, to_layer), vias)| {
            let tools = build_excellon_tool_views(&vias);
            NativeProjectDrillHoleClassBucketView {
                class,
                from_layer,
                to_layer,
                via_count: vias.len(),
                tool_count: tools.len(),
                tools,
            }
        })
        .collect::<Vec<_>>();

    Ok(NativeProjectDrillHoleClassReportView {
        action: "report_drill_hole_classes".to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        copper_layer_count: copper_layers.len(),
        via_count: vias.len(),
        class_count: classes.len(),
        classes,
    })
}

fn build_excellon_tool_views(vias: &[Via]) -> Vec<NativeProjectExcellonDrillToolView> {
    let mut grouped = BTreeMap::<i64, usize>::new();
    for via in vias {
        *grouped.entry(via.drill).or_default() += 1;
    }
    grouped
        .into_iter()
        .enumerate()
        .map(
            |(idx, (drill_nm, hits))| NativeProjectExcellonDrillToolView {
                tool: format!("T{:02}", idx + 1),
                diameter_mm: render_mm_6(drill_nm),
                hits,
            },
        )
        .collect()
}

fn render_mm_6(nm: i64) -> String {
    let sign = if nm < 0 { "-" } else { "" };
    let abs = nm.abs();
    let whole = abs / 1_000_000;
    let frac = abs % 1_000_000;
    format!("{sign}{whole}.{frac:06}")
}

fn classify_via_hole_class(
    start: i32,
    end: i32,
    top_copper: Option<i32>,
    bottom_copper: Option<i32>,
) -> String {
    match (top_copper, bottom_copper) {
        (Some(top), Some(bottom)) if start == top && end == bottom => "through".to_string(),
        (Some(top), Some(bottom)) if start == top || end == bottom => "blind".to_string(),
        (Some(_), Some(_)) => "buried".to_string(),
        _ => "unclassified".to_string(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ParsedGerberAperture {
    Circle { diameter_nm: i64 },
    Rect { width_nm: i64, height_nm: i64 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ParsedGerberGeometry {
    Stroke {
        aperture_diameter_nm: i64,
        points: Vec<Point>,
    },
    Flash {
        aperture: ParsedGerberAperture,
        position: Point,
    },
    Region {
        points: Vec<Point>,
    },
}

#[derive(Debug)]
struct ParsedGerber {
    geometries: Vec<ParsedGerberGeometry>,
}

#[derive(Debug, Default)]
struct PendingStroke {
    aperture_diameter_nm: i64,
    points: Vec<Point>,
}

fn compare_entry_views(
    expected: BTreeMap<(String, String), usize>,
    actual: BTreeMap<(String, String), usize>,
) -> (
    usize,
    usize,
    usize,
    Vec<NativeProjectGerberGeometryEntryView>,
    Vec<NativeProjectGerberGeometryEntryView>,
    Vec<NativeProjectGerberGeometryEntryView>,
) {
    let keys = expected
        .keys()
        .chain(actual.keys())
        .cloned()
        .collect::<BTreeSet<_>>();
    let mut matched_count = 0;
    let mut missing_count = 0;
    let mut extra_count = 0;
    let mut matched = Vec::new();
    let mut missing = Vec::new();
    let mut extra = Vec::new();

    for (kind, geometry) in keys {
        let key = (kind.clone(), geometry.clone());
        let expected_count = *expected.get(&key).unwrap_or(&0);
        let actual_count = *actual.get(&key).unwrap_or(&0);
        let matched_instances = expected_count.min(actual_count);
        let missing_instances = expected_count.saturating_sub(actual_count);
        let extra_instances = actual_count.saturating_sub(expected_count);

        if matched_instances > 0 {
            matched_count += matched_instances;
            matched.push(NativeProjectGerberGeometryEntryView {
                kind: kind.clone(),
                geometry: geometry.clone(),
                count: matched_instances,
            });
        }
        if missing_instances > 0 {
            missing_count += missing_instances;
            missing.push(NativeProjectGerberGeometryEntryView {
                kind: kind.clone(),
                geometry: geometry.clone(),
                count: missing_instances,
            });
        }
        if extra_instances > 0 {
            extra_count += extra_instances;
            extra.push(NativeProjectGerberGeometryEntryView {
                kind,
                geometry,
                count: extra_instances,
            });
        }
    }

    (
        matched_count,
        missing_count,
        extra_count,
        matched,
        missing,
        extra,
    )
}

fn gerber_outline_expected_entries(outline: &Polygon) -> BTreeMap<(String, String), usize> {
    let mut entries = BTreeMap::new();
    let outline_points = if outline.closed {
        canonicalize_closed_loop(&outline.vertices)
    } else {
        canonicalize_open_path(&outline.vertices)
    };
    entries.insert(
        (
            "outline".to_string(),
            render_stroke_geometry(DEFAULT_GERBER_OUTLINE_APERTURE_NM, &outline_points),
        ),
        1,
    );
    entries
}

fn gerber_outline_actual_entries(gerber: &ParsedGerber) -> BTreeMap<(String, String), usize> {
    let mut entries = BTreeMap::new();
    for geometry in &gerber.geometries {
        let (kind, signature) = match geometry {
            ParsedGerberGeometry::Stroke {
                aperture_diameter_nm,
                points,
            } => (
                "outline".to_string(),
                render_stroke_geometry(*aperture_diameter_nm, points),
            ),
            ParsedGerberGeometry::Flash { aperture, position } => (
                "flash".to_string(),
                render_parsed_flash_geometry(aperture, position),
            ),
            ParsedGerberGeometry::Region { points } => {
                ("region".to_string(), render_region_geometry(points))
            }
        };
        *entries.entry((kind, signature)).or_insert(0) += 1;
    }
    entries
}

fn gerber_copper_expected_entries(
    pads: &[PlacedPad],
    tracks: &[Track],
    zones: &[Zone],
    vias: &[Via],
) -> BTreeMap<(String, String), usize> {
    let mut entries = BTreeMap::new();
    for pad in pads {
        *entries
            .entry(("pad".to_string(), render_pad_flash_geometry(pad)))
            .or_insert(0) += 1;
    }
    for track in tracks {
        *entries
            .entry((
                "track".to_string(),
                render_stroke_geometry(track.width, &[track.from, track.to]),
            ))
            .or_insert(0) += 1;
    }
    for zone in zones {
        *entries
            .entry((
                "zone".to_string(),
                render_region_geometry(&zone.polygon.vertices),
            ))
            .or_insert(0) += 1;
    }
    for via in vias {
        *entries
            .entry((
                "via".to_string(),
                render_circular_flash_geometry(via.diameter, &via.position),
            ))
            .or_insert(0) += 1;
    }
    entries
}

fn gerber_copper_actual_entries(
    gerber: &ParsedGerber,
    expected_pads: &BTreeSet<String>,
    expected_vias: &BTreeSet<String>,
) -> BTreeMap<(String, String), usize> {
    let mut entries = BTreeMap::new();
    for geometry in &gerber.geometries {
        let (kind, signature) = match geometry {
            ParsedGerberGeometry::Stroke {
                aperture_diameter_nm,
                points,
            } => (
                "track".to_string(),
                render_stroke_geometry(*aperture_diameter_nm, points),
            ),
            ParsedGerberGeometry::Flash { aperture, position } => {
                let signature = render_parsed_flash_geometry(aperture, position);
                let kind = if expected_pads.contains(&signature) {
                    "pad"
                } else if expected_vias.contains(&signature) {
                    "via"
                } else {
                    "flash"
                };
                (kind.to_string(), signature)
            }
            ParsedGerberGeometry::Region { points } => {
                ("zone".to_string(), render_region_geometry(points))
            }
        };
        *entries.entry((kind, signature)).or_insert(0) += 1;
    }
    entries
}

fn gerber_soldermask_expected_entries(pads: &[PlacedPad]) -> BTreeMap<(String, String), usize> {
    let mut entries = BTreeMap::new();
    for pad in pads {
        *entries
            .entry(("pad".to_string(), render_pad_flash_geometry(pad)))
            .or_insert(0) += 1;
    }
    entries
}

fn gerber_soldermask_actual_entries(
    gerber: &ParsedGerber,
    expected_pads: &BTreeSet<String>,
) -> BTreeMap<(String, String), usize> {
    let mut entries = BTreeMap::new();
    for geometry in &gerber.geometries {
        let (kind, signature) = match geometry {
            ParsedGerberGeometry::Stroke {
                aperture_diameter_nm,
                points,
            } => (
                "track".to_string(),
                render_stroke_geometry(*aperture_diameter_nm, points),
            ),
            ParsedGerberGeometry::Flash { aperture, position } => {
                let signature = render_parsed_flash_geometry(aperture, position);
                let kind = if expected_pads.contains(&signature) {
                    "pad"
                } else {
                    "flash"
                };
                (kind.to_string(), signature)
            }
            ParsedGerberGeometry::Region { points } => {
                ("region".to_string(), render_region_geometry(points))
            }
        };
        *entries.entry((kind, signature)).or_insert(0) += 1;
    }
    entries
}

fn gerber_silkscreen_expected_entries(gerber: &ParsedGerber) -> BTreeMap<(String, String), usize> {
    let mut entries = BTreeMap::new();
    for geometry in &gerber.geometries {
        let (kind, signature) = match geometry {
            ParsedGerberGeometry::Stroke {
                aperture_diameter_nm,
                points,
            } => (
                "stroke".to_string(),
                render_stroke_geometry(*aperture_diameter_nm, points),
            ),
            ParsedGerberGeometry::Flash { aperture, position } => (
                "flash".to_string(),
                render_parsed_flash_geometry(aperture, position),
            ),
            ParsedGerberGeometry::Region { points } => {
                ("region".to_string(), render_region_geometry(points))
            }
        };
        *entries.entry((kind, signature)).or_insert(0) += 1;
    }
    entries
}

fn render_stroke_geometry(aperture_diameter_nm: i64, points: &[Point]) -> String {
    let points = canonicalize_path_points(points);
    format!(
        "aperture_mm={} points={}",
        render_mm_6(aperture_diameter_nm),
        render_point_path(&points)
    )
}

fn render_circular_flash_geometry(aperture_diameter_nm: i64, position: &Point) -> String {
    format!(
        "shape=circle diameter_mm={} at=({}, {})",
        render_mm_6(aperture_diameter_nm),
        position.x,
        position.y
    )
}

fn render_rect_flash_geometry(width_nm: i64, height_nm: i64, position: &Point) -> String {
    format!(
        "shape=rect width_mm={} height_mm={} at=({}, {})",
        render_mm_6(width_nm),
        render_mm_6(height_nm),
        position.x,
        position.y
    )
}

fn render_pad_flash_geometry(pad: &PlacedPad) -> String {
    match pad.aperture() {
        PadAperture::Circle { diameter_nm } => {
            render_circular_flash_geometry(diameter_nm, &pad.position)
        }
        PadAperture::Rect {
            width_nm,
            height_nm,
        } => render_rect_flash_geometry(width_nm, height_nm, &pad.position),
    }
}

fn render_parsed_flash_geometry(aperture: &ParsedGerberAperture, position: &Point) -> String {
    match aperture {
        ParsedGerberAperture::Circle { diameter_nm } => {
            render_circular_flash_geometry(*diameter_nm, position)
        }
        ParsedGerberAperture::Rect {
            width_nm,
            height_nm,
        } => render_rect_flash_geometry(*width_nm, *height_nm, position),
    }
}

fn render_region_geometry(points: &[Point]) -> String {
    let points = canonicalize_path_points(points);
    format!("points={}", render_point_path(&points))
}

fn render_point_path(points: &[Point]) -> String {
    points
        .iter()
        .map(|point| format!("({}, {})", point.x, point.y))
        .collect::<Vec<_>>()
        .join(" -> ")
}

const DEFAULT_GERBER_OUTLINE_APERTURE_NM: i64 = 100_000;

fn parse_rs274x_subset(gerber: &str) -> Result<ParsedGerber> {
    let mut aperture_map = BTreeMap::<usize, ParsedGerberAperture>::new();
    let mut current_aperture = None;
    let mut current_position = None;
    let mut pending_stroke = None::<PendingStroke>;
    let mut in_region = false;
    let mut region_points = Vec::<Point>::new();
    let mut geometries = Vec::new();

    for raw_line in gerber.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with("G04") {
            continue;
        }
        if line == "%FSLAX46Y46*%" || line == "%MOMM*%" || line == "%LPD*%" {
            continue;
        }
        if let Some(rest) = line.strip_prefix("%ADD") {
            let split_idx = rest
                .find(|ch: char| ch.is_ascii_alphabetic())
                .context("unsupported Gerber aperture definition in comparison input")?;
            let (code_str, definition) = rest.split_at(split_idx);
            let (kind, params) = definition.split_at(1);
            let params = params
                .strip_prefix(',')
                .context("unsupported Gerber aperture definition in comparison input")?
                .strip_suffix("*%")
                .context("unterminated Gerber aperture definition in comparison input")?;
            let code = code_str
                .parse::<usize>()
                .context("invalid Gerber aperture code in comparison input")?;
            let aperture = match kind {
                "C" => ParsedGerberAperture::Circle {
                    diameter_nm: parse_mm_6_to_nm(params)
                        .context("invalid Gerber circular aperture diameter in comparison input")?,
                },
                "R" => {
                    let (width_str, height_str) = params.split_once('X').context(
                        "invalid Gerber rectangular aperture definition in comparison input",
                    )?;
                    ParsedGerberAperture::Rect {
                        width_nm: parse_mm_6_to_nm(width_str).context(
                            "invalid Gerber rectangular aperture width in comparison input",
                        )?,
                        height_nm: parse_mm_6_to_nm(height_str).context(
                            "invalid Gerber rectangular aperture height in comparison input",
                        )?,
                    }
                }
                _ => bail!("unsupported Gerber aperture definition in comparison input: {line}"),
            };
            aperture_map.insert(code, aperture);
            continue;
        }
        if line == "G36*" {
            finalize_pending_stroke(&mut pending_stroke, &mut geometries);
            if in_region {
                bail!("nested Gerber regions are not supported by semantic comparison");
            }
            in_region = true;
            region_points.clear();
            continue;
        }
        if line == "G37*" {
            if !in_region {
                bail!("Gerber region end encountered without region start");
            }
            in_region = false;
            if region_points.len() >= 2 {
                geometries.push(ParsedGerberGeometry::Region {
                    points: canonicalize_path_points(&region_points),
                });
            }
            region_points.clear();
            continue;
        }
        if let Some(code) = parse_aperture_select(line)? {
            finalize_pending_stroke(&mut pending_stroke, &mut geometries);
            current_aperture = Some(code);
            continue;
        }
        if line == "M02*" {
            break;
        }

        let Some((point, operation)) = parse_gerber_coordinate_operation(line)? else {
            bail!("unsupported Gerber command in semantic comparison input: {line}");
        };
        let previous_position = current_position;
        current_position = Some(point);

        if in_region {
            match operation {
                2 => {
                    region_points.clear();
                    region_points.push(point);
                }
                1 => {
                    if region_points.is_empty() {
                        region_points.push(point);
                    } else {
                        region_points.push(point);
                    }
                }
                3 => bail!("Gerber flashes inside regions are not supported"),
                _ => bail!("unsupported Gerber interpolation operation D0{operation}"),
            }
            continue;
        }

        let aperture_code = current_aperture.context(
            "Gerber semantic comparison requires an active circular aperture before geometry",
        )?;
        let aperture = aperture_map.get(&aperture_code).with_context(|| {
            format!("unknown Gerber aperture D{aperture_code} in comparison input")
        })?;

        match operation {
            2 => {
                let aperture_diameter_nm = match aperture {
                    ParsedGerberAperture::Circle { diameter_nm } => *diameter_nm,
                    ParsedGerberAperture::Rect { .. } => bail!(
                        "Gerber semantic comparison only supports circular apertures for strokes"
                    ),
                };
                finalize_pending_stroke(&mut pending_stroke, &mut geometries);
                pending_stroke = Some(PendingStroke {
                    aperture_diameter_nm,
                    points: vec![point],
                });
            }
            1 => {
                let aperture_diameter_nm = match aperture {
                    ParsedGerberAperture::Circle { diameter_nm } => *diameter_nm,
                    ParsedGerberAperture::Rect { .. } => bail!(
                        "Gerber semantic comparison only supports circular apertures for strokes"
                    ),
                };
                let stroke = pending_stroke.get_or_insert_with(|| PendingStroke {
                    aperture_diameter_nm,
                    points: previous_position.into_iter().collect::<Vec<_>>(),
                });
                if stroke.aperture_diameter_nm != aperture_diameter_nm {
                    finalize_pending_stroke(&mut pending_stroke, &mut geometries);
                    pending_stroke = Some(PendingStroke {
                        aperture_diameter_nm,
                        points: previous_position
                            .into_iter()
                            .chain(std::iter::once(point))
                            .collect::<Vec<_>>(),
                    });
                } else {
                    stroke.points.push(point);
                }
            }
            3 => {
                finalize_pending_stroke(&mut pending_stroke, &mut geometries);
                geometries.push(ParsedGerberGeometry::Flash {
                    aperture: aperture.clone(),
                    position: point,
                });
            }
            _ => bail!("unsupported Gerber interpolation operation D0{operation}"),
        }
    }

    if in_region {
        bail!("unterminated Gerber region in comparison input");
    }
    finalize_pending_stroke(&mut pending_stroke, &mut geometries);
    Ok(ParsedGerber { geometries })
}

fn finalize_pending_stroke(
    pending_stroke: &mut Option<PendingStroke>,
    geometries: &mut Vec<ParsedGerberGeometry>,
) {
    if let Some(stroke) = pending_stroke.take() {
        if stroke.points.len() >= 2 {
            geometries.push(ParsedGerberGeometry::Stroke {
                aperture_diameter_nm: stroke.aperture_diameter_nm,
                points: canonicalize_path_points(&stroke.points),
            });
        }
    }
}

fn canonicalize_path_points(points: &[Point]) -> Vec<Point> {
    let mut normalized = points.to_vec();
    if normalized.len() >= 2 && normalized.first() == normalized.last() {
        normalized.pop();
        return canonicalize_closed_loop(&normalized);
    }
    canonicalize_open_path(&normalized)
}

fn canonicalize_open_path(points: &[Point]) -> Vec<Point> {
    let reversed = points.iter().rev().copied().collect::<Vec<_>>();
    if point_path_cmp(points, &reversed).is_gt() {
        reversed
    } else {
        points.to_vec()
    }
}

fn canonicalize_closed_loop(points: &[Point]) -> Vec<Point> {
    if points.is_empty() {
        return Vec::new();
    }

    let mut best = rotate_points(points, 0);
    for start in 1..points.len() {
        let candidate = rotate_points(points, start);
        if point_path_cmp(&candidate, &best).is_lt() {
            best = candidate;
        }
    }

    let reversed_points = points.iter().rev().copied().collect::<Vec<_>>();
    for start in 0..reversed_points.len() {
        let candidate = rotate_points(&reversed_points, start);
        if point_path_cmp(&candidate, &best).is_lt() {
            best = candidate;
        }
    }

    best
}

fn rotate_points(points: &[Point], start: usize) -> Vec<Point> {
    points[start..]
        .iter()
        .chain(points[..start].iter())
        .copied()
        .collect()
}

fn point_path_cmp(a: &[Point], b: &[Point]) -> std::cmp::Ordering {
    for (lhs, rhs) in a.iter().zip(b.iter()) {
        let ordering = point_cmp(lhs, rhs);
        if !ordering.is_eq() {
            return ordering;
        }
    }
    a.len().cmp(&b.len())
}

fn point_cmp(a: &Point, b: &Point) -> std::cmp::Ordering {
    a.x.cmp(&b.x).then_with(|| a.y.cmp(&b.y))
}

fn parse_aperture_select(line: &str) -> Result<Option<usize>> {
    if !line.starts_with('D') || !line.ends_with('*') {
        return Ok(None);
    }
    let code = line[1..line.len() - 1]
        .parse::<usize>()
        .context("invalid Gerber aperture selection in comparison input")?;
    if code < 10 {
        return Ok(None);
    }
    Ok(Some(code))
}

fn parse_gerber_coordinate_operation(line: &str) -> Result<Option<(Point, u8)>> {
    let Some(rest) = line.strip_prefix('X') else {
        return Ok(None);
    };
    let (x_str, rest) = rest
        .split_once('Y')
        .context("invalid Gerber coordinate command in comparison input")?;
    let (y_str, d_str) = rest
        .split_once('D')
        .context("invalid Gerber coordinate command in comparison input")?;
    let operation = d_str
        .strip_suffix('*')
        .context("unterminated Gerber coordinate command in comparison input")?
        .parse::<u8>()
        .context("invalid Gerber coordinate operation in comparison input")?;
    Ok(Some((
        Point {
            x: x_str
                .parse::<i64>()
                .context("invalid Gerber X coordinate in comparison input")?,
            y: y_str
                .parse::<i64>()
                .context("invalid Gerber Y coordinate in comparison input")?,
        },
        operation,
    )))
}

fn parse_mm_6_to_nm(value: &str) -> Option<i64> {
    let mut parts = value.split('.');
    let whole = parts.next()?.parse::<i64>().ok()?;
    let frac_str = parts.next().unwrap_or("0");
    if parts.next().is_some() {
        return None;
    }
    let mut frac = frac_str.to_string();
    if frac.len() > 6 {
        return None;
    }
    while frac.len() < 6 {
        frac.push('0');
    }
    let frac = frac.parse::<i64>().ok()?;
    let whole_nm = whole.checked_mul(1_000_000)?;
    if whole >= 0 {
        whole_nm.checked_add(frac)
    } else {
        whole_nm.checked_sub(frac)
    }
}

fn resolve_native_project_soldermask_context(
    root: &Path,
    layer: i32,
) -> Result<(StackupLayer, i32, Vec<PlacedPad>)> {
    let stackup = query_native_project_board_stackup(root)?;
    let mask_layer = stackup
        .iter()
        .find(|entry| entry.id == layer)
        .cloned()
        .ok_or_else(|| {
            anyhow::anyhow!("board stackup layer not found in native project: {layer}")
        })?;
    if !matches!(mask_layer.layer_type, StackupLayerType::SolderMask) {
        bail!("board stackup layer is not a soldermask layer: {layer}");
    }

    let associated_copper_layer = stackup
        .iter()
        .filter(|entry| matches!(entry.layer_type, StackupLayerType::Copper))
        .min_by(|a, b| {
            (i64::from((a.id - layer).abs()), a.id).cmp(&(i64::from((b.id - layer).abs()), b.id))
        })
        .map(|entry| entry.id)
        .ok_or_else(|| {
            anyhow::anyhow!("no copper layer available to derive soldermask openings")
        })?;

    let pads = query_native_project_board_pads(root)?
        .into_iter()
        .filter(|pad| pad.layer == associated_copper_layer)
        .collect::<Vec<_>>();

    Ok((mask_layer, associated_copper_layer, pads))
}

fn resolve_native_project_paste_context(
    root: &Path,
    layer: i32,
) -> Result<(StackupLayer, i32, Vec<PlacedPad>)> {
    let stackup = query_native_project_board_stackup(root)?;
    let paste_layer = stackup
        .iter()
        .find(|entry| entry.id == layer)
        .cloned()
        .ok_or_else(|| {
            anyhow::anyhow!("board stackup layer not found in native project: {layer}")
        })?;
    if !matches!(paste_layer.layer_type, StackupLayerType::Paste) {
        bail!("board stackup layer is not a paste layer: {layer}");
    }

    let associated_copper_layer = stackup
        .iter()
        .filter(|entry| matches!(entry.layer_type, StackupLayerType::Copper))
        .min_by(|a, b| {
            (i64::from((a.id - layer).abs()), a.id).cmp(&(i64::from((b.id - layer).abs()), b.id))
        })
        .map(|entry| entry.id)
        .ok_or_else(|| anyhow::anyhow!("no copper layer available to derive paste openings"))?;

    let pads = query_native_project_board_pads(root)?
        .into_iter()
        .filter(|pad| pad.layer == associated_copper_layer)
        .collect::<Vec<_>>();

    Ok((paste_layer, associated_copper_layer, pads))
}

pub(crate) fn export_native_project_gerber_outline(
    root: &Path,
    output_path: &Path,
) -> Result<NativeProjectGerberOutlineExportView> {
    let project = load_native_project(root)?;
    let outline = native_outline_to_polygon(&project.board.outline);
    let gerber = render_rs274x_outline_default(&outline)
        .context("failed to render native board outline as RS-274X")?;
    std::fs::write(output_path, gerber)
        .with_context(|| format!("failed to write {}", output_path.display()))?;
    Ok(NativeProjectGerberOutlineExportView {
        action: "export_gerber_outline".to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        gerber_path: output_path.display().to_string(),
        outline_vertex_count: project.board.outline.vertices.len(),
        outline_closed: project.board.outline.closed,
    })
}

pub(crate) fn export_native_project_gerber_copper_layer(
    root: &Path,
    layer: i32,
    output_path: &Path,
) -> Result<NativeProjectGerberCopperExportView> {
    let project = load_native_project(root)?;
    let pads = query_native_project_board_pads(root)?
        .into_iter()
        .filter(|pad| pad.layer == layer)
        .collect::<Vec<_>>();
    let tracks = query_native_project_board_tracks(root)?
        .into_iter()
        .filter(|track| track.layer == layer)
        .collect::<Vec<_>>();
    let zones = query_native_project_board_zones(root)?
        .into_iter()
        .filter(|zone| zone.layer == layer)
        .collect::<Vec<_>>();
    let vias = query_native_project_board_vias(root)?
        .into_iter()
        .filter(|via| {
            let min_layer = via.from_layer.min(via.to_layer);
            let max_layer = via.from_layer.max(via.to_layer);
            layer >= min_layer && layer <= max_layer
        })
        .collect::<Vec<_>>();
    let gerber = render_rs274x_copper_layer(layer, &pads, &tracks, &zones, &vias)
        .context("failed to render native board copper layer as RS-274X")?;
    std::fs::write(output_path, gerber)
        .with_context(|| format!("failed to write {}", output_path.display()))?;
    Ok(NativeProjectGerberCopperExportView {
        action: "export_gerber_copper_layer".to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        gerber_path: output_path.display().to_string(),
        layer,
        pad_count: pads.len(),
        track_count: tracks.len(),
        zone_count: zones.len(),
        via_count: vias.len(),
    })
}

pub(crate) fn export_native_project_gerber_soldermask_layer(
    root: &Path,
    layer: i32,
    output_path: &Path,
) -> Result<NativeProjectGerberSoldermaskExportView> {
    let project = load_native_project(root)?;
    let (_mask_layer, source_copper_layer, pads) =
        resolve_native_project_soldermask_context(root, layer)?;
    let gerber = render_rs274x_soldermask_layer(layer, &pads)
        .context("failed to render native board soldermask layer as RS-274X")?;
    std::fs::write(output_path, gerber)
        .with_context(|| format!("failed to write {}", output_path.display()))?;
    Ok(NativeProjectGerberSoldermaskExportView {
        action: "export_gerber_soldermask_layer".to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        gerber_path: output_path.display().to_string(),
        layer,
        source_copper_layer,
        pad_count: pads.len(),
    })
}

pub(crate) fn export_native_project_gerber_silkscreen_layer(
    root: &Path,
    layer: i32,
    output_path: &Path,
) -> Result<NativeProjectGerberSilkscreenExportView> {
    let project = load_native_project(root)?;
    let component_text_count = count_native_component_silkscreen_texts(&project.board, layer);
    let component_line_count = count_native_component_silkscreen_lines(&project.board, layer);
    let component_arc_count = count_native_component_silkscreen_arcs(&project.board, layer);
    let component_circle_count = count_native_component_silkscreen_circles(&project.board, layer);
    let component_polygon_count = count_native_component_silkscreen_polygons(&project.board, layer);
    let component_polyline_count =
        count_native_component_silkscreen_polylines(&project.board, layer);
    let (_silk_layer, texts, component_strokes) =
        resolve_native_project_silkscreen_context(root, layer)?;
    let gerber = render_rs274x_silkscreen_layer(layer, &texts, &component_strokes)
        .context("failed to render native board silkscreen layer as RS-274X")?;
    std::fs::write(output_path, gerber)
        .with_context(|| format!("failed to write {}", output_path.display()))?;
    Ok(NativeProjectGerberSilkscreenExportView {
        action: "export_gerber_silkscreen_layer".to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        gerber_path: output_path.display().to_string(),
        layer,
        text_count: texts.len().saturating_sub(component_text_count),
        component_text_count,
        component_stroke_count: component_line_count,
        component_arc_count,
        component_circle_count,
        component_polygon_count,
        component_polyline_count,
    })
}

pub(crate) fn export_native_project_gerber_paste_layer(
    root: &Path,
    layer: i32,
    output_path: &Path,
) -> Result<NativeProjectGerberPasteExportView> {
    let project = load_native_project(root)?;
    let (_paste_layer, source_copper_layer, pads) =
        resolve_native_project_paste_context(root, layer)?;
    let gerber = render_rs274x_paste_layer(layer, &pads)
        .context("failed to render native board paste layer as RS-274X")?;
    std::fs::write(output_path, gerber)
        .with_context(|| format!("failed to write {}", output_path.display()))?;
    Ok(NativeProjectGerberPasteExportView {
        action: "export_gerber_paste_layer".to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        gerber_path: output_path.display().to_string(),
        layer,
        source_copper_layer,
        pad_count: pads.len(),
    })
}

pub(crate) fn validate_native_project_gerber_outline(
    root: &Path,
    gerber_path: &Path,
) -> Result<NativeProjectGerberOutlineValidationView> {
    let project = load_native_project(root)?;
    let outline = native_outline_to_polygon(&project.board.outline);
    let expected = render_rs274x_outline_default(&outline)
        .context("failed to render expected native board outline as RS-274X")?;
    let actual = std::fs::read_to_string(gerber_path)
        .with_context(|| format!("failed to read {}", gerber_path.display()))?;

    Ok(NativeProjectGerberOutlineValidationView {
        action: "validate_gerber_outline".to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        gerber_path: gerber_path.display().to_string(),
        matches_expected: actual == expected,
        expected_bytes: expected.len(),
        actual_bytes: actual.len(),
        outline_vertex_count: project.board.outline.vertices.len(),
        outline_closed: project.board.outline.closed,
    })
}

pub(crate) fn validate_native_project_gerber_copper_layer(
    root: &Path,
    layer: i32,
    gerber_path: &Path,
) -> Result<NativeProjectGerberCopperValidationView> {
    let project = load_native_project(root)?;
    let pads = query_native_project_board_pads(root)?
        .into_iter()
        .filter(|pad| pad.layer == layer)
        .collect::<Vec<_>>();
    let tracks = query_native_project_board_tracks(root)?
        .into_iter()
        .filter(|track| track.layer == layer)
        .collect::<Vec<_>>();
    let zones = query_native_project_board_zones(root)?
        .into_iter()
        .filter(|zone| zone.layer == layer)
        .collect::<Vec<_>>();
    let vias = query_native_project_board_vias(root)?
        .into_iter()
        .filter(|via| {
            let min_layer = via.from_layer.min(via.to_layer);
            let max_layer = via.from_layer.max(via.to_layer);
            layer >= min_layer && layer <= max_layer
        })
        .collect::<Vec<_>>();
    let expected = render_rs274x_copper_layer(layer, &pads, &tracks, &zones, &vias)
        .context("failed to render expected native board copper layer as RS-274X")?;
    let actual = std::fs::read_to_string(gerber_path)
        .with_context(|| format!("failed to read {}", gerber_path.display()))?;

    Ok(NativeProjectGerberCopperValidationView {
        action: "validate_gerber_copper_layer".to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        gerber_path: gerber_path.display().to_string(),
        layer,
        matches_expected: actual == expected,
        expected_bytes: expected.len(),
        actual_bytes: actual.len(),
        pad_count: pads.len(),
        track_count: tracks.len(),
        zone_count: zones.len(),
        via_count: vias.len(),
    })
}

pub(crate) fn validate_native_project_gerber_soldermask_layer(
    root: &Path,
    layer: i32,
    gerber_path: &Path,
) -> Result<NativeProjectGerberSoldermaskValidationView> {
    let project = load_native_project(root)?;
    let (_mask_layer, source_copper_layer, pads) =
        resolve_native_project_soldermask_context(root, layer)?;
    let expected = render_rs274x_soldermask_layer(layer, &pads)
        .context("failed to render expected native board soldermask layer as RS-274X")?;
    let actual = std::fs::read_to_string(gerber_path)
        .with_context(|| format!("failed to read {}", gerber_path.display()))?;

    Ok(NativeProjectGerberSoldermaskValidationView {
        action: "validate_gerber_soldermask_layer".to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        gerber_path: gerber_path.display().to_string(),
        layer,
        source_copper_layer,
        matches_expected: actual == expected,
        expected_bytes: expected.len(),
        actual_bytes: actual.len(),
        pad_count: pads.len(),
    })
}

pub(crate) fn validate_native_project_gerber_silkscreen_layer(
    root: &Path,
    layer: i32,
    gerber_path: &Path,
) -> Result<NativeProjectGerberSilkscreenValidationView> {
    let project = load_native_project(root)?;
    let component_text_count = count_native_component_silkscreen_texts(&project.board, layer);
    let component_line_count = count_native_component_silkscreen_lines(&project.board, layer);
    let component_arc_count = count_native_component_silkscreen_arcs(&project.board, layer);
    let component_circle_count = count_native_component_silkscreen_circles(&project.board, layer);
    let component_polygon_count = count_native_component_silkscreen_polygons(&project.board, layer);
    let component_polyline_count =
        count_native_component_silkscreen_polylines(&project.board, layer);
    let (_silk_layer, texts, component_strokes) =
        resolve_native_project_silkscreen_context(root, layer)?;
    let expected = render_rs274x_silkscreen_layer(layer, &texts, &component_strokes)
        .context("failed to render expected native board silkscreen layer as RS-274X")?;
    let actual = std::fs::read_to_string(gerber_path)
        .with_context(|| format!("failed to read {}", gerber_path.display()))?;

    Ok(NativeProjectGerberSilkscreenValidationView {
        action: "validate_gerber_silkscreen_layer".to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        gerber_path: gerber_path.display().to_string(),
        layer,
        matches_expected: actual == expected,
        expected_bytes: expected.len(),
        actual_bytes: actual.len(),
        text_count: texts.len().saturating_sub(component_text_count),
        component_text_count,
        component_stroke_count: component_line_count,
        component_arc_count,
        component_circle_count,
        component_polygon_count,
        component_polyline_count,
    })
}

pub(crate) fn validate_native_project_gerber_paste_layer(
    root: &Path,
    layer: i32,
    gerber_path: &Path,
) -> Result<NativeProjectGerberPasteValidationView> {
    let project = load_native_project(root)?;
    let (_paste_layer, source_copper_layer, pads) =
        resolve_native_project_paste_context(root, layer)?;
    let expected = render_rs274x_paste_layer(layer, &pads)
        .context("failed to render expected native board paste layer as RS-274X")?;
    let actual = std::fs::read_to_string(gerber_path)
        .with_context(|| format!("failed to read {}", gerber_path.display()))?;

    Ok(NativeProjectGerberPasteValidationView {
        action: "validate_gerber_paste_layer".to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        gerber_path: gerber_path.display().to_string(),
        layer,
        source_copper_layer,
        matches_expected: actual == expected,
        expected_bytes: expected.len(),
        actual_bytes: actual.len(),
        pad_count: pads.len(),
    })
}

pub(crate) fn compare_native_project_gerber_outline(
    root: &Path,
    gerber_path: &Path,
) -> Result<NativeProjectGerberOutlineComparisonView> {
    let project = load_native_project(root)?;
    let outline = native_outline_to_polygon(&project.board.outline);
    let actual_gerber = std::fs::read_to_string(gerber_path)
        .with_context(|| format!("failed to read {}", gerber_path.display()))?;
    let parsed = parse_rs274x_subset(&actual_gerber)
        .context("failed to parse Gerber outline for semantic comparison")?;

    let expected_entries = gerber_outline_expected_entries(&outline);
    let actual_entries = gerber_outline_actual_entries(&parsed);
    let (matched_count, missing_count, extra_count, matched, missing, extra) =
        compare_entry_views(expected_entries, actual_entries);

    Ok(NativeProjectGerberOutlineComparisonView {
        action: "compare_gerber_outline".to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        gerber_path: gerber_path.display().to_string(),
        expected_outline_count: 1,
        actual_geometry_count: parsed.geometries.len(),
        matched_count,
        missing_count,
        extra_count,
        matched,
        missing,
        extra,
    })
}

pub(crate) fn compare_native_project_gerber_copper_layer(
    root: &Path,
    layer: i32,
    gerber_path: &Path,
) -> Result<NativeProjectGerberCopperComparisonView> {
    let project = load_native_project(root)?;
    let pads = query_native_project_board_pads(root)?
        .into_iter()
        .filter(|pad| pad.layer == layer)
        .collect::<Vec<_>>();
    let tracks = query_native_project_board_tracks(root)?
        .into_iter()
        .filter(|track| track.layer == layer)
        .collect::<Vec<_>>();
    let zones = query_native_project_board_zones(root)?
        .into_iter()
        .filter(|zone| zone.layer == layer)
        .collect::<Vec<_>>();
    let vias = query_native_project_board_vias(root)?
        .into_iter()
        .filter(|via| {
            let min_layer = via.from_layer.min(via.to_layer);
            let max_layer = via.from_layer.max(via.to_layer);
            layer >= min_layer && layer <= max_layer
        })
        .collect::<Vec<_>>();
    let actual_gerber = std::fs::read_to_string(gerber_path)
        .with_context(|| format!("failed to read {}", gerber_path.display()))?;
    let parsed = parse_rs274x_subset(&actual_gerber)
        .context("failed to parse Gerber copper layer for semantic comparison")?;

    let expected_pad_signatures = pads
        .iter()
        .map(render_pad_flash_geometry)
        .collect::<BTreeSet<_>>();
    let expected_via_signatures = vias
        .iter()
        .map(|via| render_circular_flash_geometry(via.diameter, &via.position))
        .collect::<BTreeSet<_>>();
    let expected_entries = gerber_copper_expected_entries(&pads, &tracks, &zones, &vias);
    let actual_entries =
        gerber_copper_actual_entries(&parsed, &expected_pad_signatures, &expected_via_signatures);
    let (matched_count, missing_count, extra_count, matched, missing, extra) =
        compare_entry_views(expected_entries, actual_entries);

    let actual_pad_count = parsed
        .geometries
        .iter()
        .filter(|geometry| match geometry {
            ParsedGerberGeometry::Flash { aperture, position } => {
                expected_pad_signatures.contains(&render_parsed_flash_geometry(aperture, position))
            }
            _ => false,
        })
        .count();
    let actual_track_count = parsed
        .geometries
        .iter()
        .filter(|geometry| matches!(geometry, ParsedGerberGeometry::Stroke { .. }))
        .count();
    let actual_zone_count = parsed
        .geometries
        .iter()
        .filter(|geometry| matches!(geometry, ParsedGerberGeometry::Region { .. }))
        .count();
    let actual_via_count = parsed
        .geometries
        .iter()
        .filter(|geometry| match geometry {
            ParsedGerberGeometry::Flash { aperture, position } => {
                expected_via_signatures.contains(&render_parsed_flash_geometry(aperture, position))
            }
            _ => false,
        })
        .count();

    Ok(NativeProjectGerberCopperComparisonView {
        action: "compare_gerber_copper_layer".to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        gerber_path: gerber_path.display().to_string(),
        layer,
        expected_pad_count: pads.len(),
        actual_pad_count,
        expected_track_count: tracks.len(),
        actual_track_count,
        expected_zone_count: zones.len(),
        actual_zone_count,
        expected_via_count: vias.len(),
        actual_via_count,
        matched_count,
        missing_count,
        extra_count,
        matched,
        missing,
        extra,
    })
}

pub(crate) fn compare_native_project_gerber_soldermask_layer(
    root: &Path,
    layer: i32,
    gerber_path: &Path,
) -> Result<NativeProjectGerberSoldermaskComparisonView> {
    let project = load_native_project(root)?;
    let (_mask_layer, source_copper_layer, pads) =
        resolve_native_project_soldermask_context(root, layer)?;
    let actual_gerber = std::fs::read_to_string(gerber_path)
        .with_context(|| format!("failed to read {}", gerber_path.display()))?;
    let parsed = parse_rs274x_subset(&actual_gerber)
        .context("failed to parse Gerber soldermask layer for semantic comparison")?;

    let expected_pad_signatures = pads
        .iter()
        .map(render_pad_flash_geometry)
        .collect::<BTreeSet<_>>();
    let expected_entries = gerber_soldermask_expected_entries(&pads);
    let actual_entries = gerber_soldermask_actual_entries(&parsed, &expected_pad_signatures);
    let (matched_count, missing_count, extra_count, matched, missing, extra) =
        compare_entry_views(expected_entries, actual_entries);

    let actual_pad_count = parsed
        .geometries
        .iter()
        .filter(|geometry| match geometry {
            ParsedGerberGeometry::Flash { aperture, position } => {
                expected_pad_signatures.contains(&render_parsed_flash_geometry(aperture, position))
            }
            _ => false,
        })
        .count();

    Ok(NativeProjectGerberSoldermaskComparisonView {
        action: "compare_gerber_soldermask_layer".to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        gerber_path: gerber_path.display().to_string(),
        layer,
        source_copper_layer,
        expected_pad_count: pads.len(),
        actual_pad_count,
        matched_count,
        missing_count,
        extra_count,
        matched,
        missing,
        extra,
    })
}

pub(crate) fn compare_native_project_gerber_silkscreen_layer(
    root: &Path,
    layer: i32,
    gerber_path: &Path,
) -> Result<NativeProjectGerberSilkscreenComparisonView> {
    let project = load_native_project(root)?;
    let component_text_count = count_native_component_silkscreen_texts(&project.board, layer);
    let component_line_count = count_native_component_silkscreen_lines(&project.board, layer);
    let component_arc_count = count_native_component_silkscreen_arcs(&project.board, layer);
    let component_circle_count = count_native_component_silkscreen_circles(&project.board, layer);
    let component_polygon_count = count_native_component_silkscreen_polygons(&project.board, layer);
    let component_polyline_count =
        count_native_component_silkscreen_polylines(&project.board, layer);
    let (_silk_layer, texts, component_strokes) =
        resolve_native_project_silkscreen_context(root, layer)?;
    let expected_gerber = render_rs274x_silkscreen_layer(layer, &texts, &component_strokes)
        .context("failed to render expected native board silkscreen layer as RS-274X")?;
    let actual_gerber = std::fs::read_to_string(gerber_path)
        .with_context(|| format!("failed to read {}", gerber_path.display()))?;
    let expected_parsed = parse_rs274x_subset(&expected_gerber)
        .context("failed to parse expected Gerber silkscreen layer for semantic comparison")?;
    let actual_parsed = parse_rs274x_subset(&actual_gerber)
        .context("failed to parse Gerber silkscreen layer for semantic comparison")?;

    let expected_entries = gerber_silkscreen_expected_entries(&expected_parsed);
    let actual_entries = gerber_silkscreen_expected_entries(&actual_parsed);
    let (matched_count, missing_count, extra_count, matched, missing, extra) =
        compare_entry_views(expected_entries, actual_entries);

    Ok(NativeProjectGerberSilkscreenComparisonView {
        action: "compare_gerber_silkscreen_layer".to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        gerber_path: gerber_path.display().to_string(),
        layer,
        expected_text_count: texts.len().saturating_sub(component_text_count),
        expected_component_text_count: component_text_count,
        expected_component_stroke_count: component_line_count,
        expected_component_arc_count: component_arc_count,
        expected_component_circle_count: component_circle_count,
        expected_component_polygon_count: component_polygon_count,
        expected_component_polyline_count: component_polyline_count,
        actual_geometry_count: actual_parsed.geometries.len(),
        matched_count,
        missing_count,
        extra_count,
        matched,
        missing,
        extra,
    })
}

pub(crate) fn compare_native_project_gerber_paste_layer(
    root: &Path,
    layer: i32,
    gerber_path: &Path,
) -> Result<NativeProjectGerberPasteComparisonView> {
    let project = load_native_project(root)?;
    let (_paste_layer, source_copper_layer, pads) =
        resolve_native_project_paste_context(root, layer)?;
    let actual_gerber = std::fs::read_to_string(gerber_path)
        .with_context(|| format!("failed to read {}", gerber_path.display()))?;
    let parsed = parse_rs274x_subset(&actual_gerber)
        .context("failed to parse Gerber paste layer for semantic comparison")?;

    let expected_pad_signatures = pads
        .iter()
        .map(render_pad_flash_geometry)
        .collect::<BTreeSet<_>>();
    let expected_entries = gerber_soldermask_expected_entries(&pads);
    let actual_entries = gerber_soldermask_actual_entries(&parsed, &expected_pad_signatures);
    let (matched_count, missing_count, extra_count, matched, missing, extra) =
        compare_entry_views(expected_entries, actual_entries);

    let actual_pad_count = parsed
        .geometries
        .iter()
        .filter(|geometry| match geometry {
            ParsedGerberGeometry::Flash { aperture, position } => {
                expected_pad_signatures.contains(&render_parsed_flash_geometry(aperture, position))
            }
            _ => false,
        })
        .count();

    Ok(NativeProjectGerberPasteComparisonView {
        action: "compare_gerber_paste_layer".to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        gerber_path: gerber_path.display().to_string(),
        layer,
        source_copper_layer,
        expected_pad_count: pads.len(),
        actual_pad_count,
        matched_count,
        missing_count,
        extra_count,
        matched,
        missing,
        extra,
    })
}

pub(crate) fn plan_native_project_gerber_export(
    root: &Path,
    prefix_override: Option<&str>,
) -> Result<NativeProjectGerberPlanView> {
    let project = load_native_project(root)?;
    let mut layers = project
        .board
        .stackup
        .layers
        .iter()
        .map(|value| {
            serde_json::from_value::<StackupLayer>(value.clone())
                .context("failed to parse board stackup layer")
        })
        .collect::<Result<Vec<_>>>()?;
    layers.sort_by(|a, b| a.id.cmp(&b.id).then_with(|| a.name.cmp(&b.name)));

    let prefix = sanitize_export_prefix(prefix_override.unwrap_or(&project.board.name));
    let mut artifacts = vec![NativeProjectGerberPlanArtifactView {
        kind: "outline".to_string(),
        layer_id: None,
        layer_name: None,
        filename: format!("{prefix}-outline.gbr"),
    }];

    let mut copper_layers = 0;
    let mut soldermask_layers = 0;
    let mut silkscreen_layers = 0;
    let mut paste_layers = 0;
    let mut mechanical_layers = 0;

    for layer in layers {
        let (kind, suffix, count_ref) = match layer.layer_type {
            StackupLayerType::Copper => ("copper", "copper", &mut copper_layers),
            StackupLayerType::SolderMask => ("soldermask", "mask", &mut soldermask_layers),
            StackupLayerType::Silkscreen => ("silkscreen", "silk", &mut silkscreen_layers),
            StackupLayerType::Paste => ("paste", "paste", &mut paste_layers),
            StackupLayerType::Mechanical => ("mechanical", "mech", &mut mechanical_layers),
            StackupLayerType::Dielectric => continue,
        };
        *count_ref += 1;
        let layer_slug = sanitize_export_prefix(&layer.name);
        artifacts.push(NativeProjectGerberPlanArtifactView {
            kind: kind.to_string(),
            layer_id: Some(layer.id),
            layer_name: Some(layer.name.clone()),
            filename: format!("{prefix}-l{}-{layer_slug}-{suffix}.gbr", layer.id),
        });
    }

    Ok(NativeProjectGerberPlanView {
        action: "plan_gerber_export".to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        prefix,
        outline_vertex_count: project.board.outline.vertices.len(),
        outline_closed: project.board.outline.closed,
        copper_layers,
        soldermask_layers,
        silkscreen_layers,
        paste_layers,
        mechanical_layers,
        artifacts,
    })
}

pub(crate) fn compare_native_project_gerber_export_plan(
    root: &Path,
    output_dir: &Path,
    prefix_override: Option<&str>,
) -> Result<NativeProjectGerberPlanComparisonView> {
    let plan = plan_native_project_gerber_export(root, prefix_override)?;
    let expected = plan
        .artifacts
        .iter()
        .map(|artifact| artifact.filename.clone())
        .collect::<BTreeSet<_>>();

    let mut present = BTreeSet::new();
    for entry in std::fs::read_dir(output_dir)
        .with_context(|| format!("failed to read {}", output_dir.display()))?
    {
        let entry = entry.with_context(|| format!("failed to read {}", output_dir.display()))?;
        if entry
            .file_type()
            .with_context(|| format!("failed to inspect {}", entry.path().display()))?
            .is_file()
        {
            present.insert(entry.file_name().to_string_lossy().into_owned());
        }
    }

    let matched = expected.intersection(&present).cloned().collect::<Vec<_>>();
    let missing = expected.difference(&present).cloned().collect::<Vec<_>>();
    let extra = present.difference(&expected).cloned().collect::<Vec<_>>();

    Ok(NativeProjectGerberPlanComparisonView {
        action: "compare_gerber_export_plan".to_string(),
        project_root: plan.project_root,
        output_dir: output_dir.display().to_string(),
        prefix: plan.prefix,
        expected_count: expected.len(),
        present_count: present.len(),
        missing_count: missing.len(),
        extra_count: extra.len(),
        matched,
        missing,
        extra,
    })
}

pub(crate) fn query_native_project_board_pads(root: &Path) -> Result<Vec<PlacedPad>> {
    let project = load_native_project(root)?;
    let mut pads = project
        .board
        .pads
        .into_values()
        .map(|value| serde_json::from_value(value).context("failed to parse board pad"))
        .collect::<Result<Vec<PlacedPad>>>()?;
    pads.sort_by(|a, b| {
        a.package
            .cmp(&b.package)
            .then_with(|| a.name.cmp(&b.name))
            .then_with(|| a.uuid.cmp(&b.uuid))
    });
    Ok(pads)
}

pub(crate) fn query_native_project_board_tracks(root: &Path) -> Result<Vec<Track>> {
    let project = load_native_project(root)?;
    let mut tracks = project
        .board
        .tracks
        .into_values()
        .map(|value| serde_json::from_value(value).context("failed to parse board track"))
        .collect::<Result<Vec<Track>>>()?;
    tracks.sort_by(|a, b| a.uuid.cmp(&b.uuid));
    Ok(tracks)
}

pub(crate) fn query_native_project_board_vias(root: &Path) -> Result<Vec<Via>> {
    let project = load_native_project(root)?;
    let mut vias = project
        .board
        .vias
        .into_values()
        .map(|value| serde_json::from_value(value).context("failed to parse board via"))
        .collect::<Result<Vec<Via>>>()?;
    vias.sort_by(|a, b| a.uuid.cmp(&b.uuid));
    Ok(vias)
}

pub(crate) fn query_native_project_board_zones(root: &Path) -> Result<Vec<Zone>> {
    let project = load_native_project(root)?;
    let mut zones = project
        .board
        .zones
        .into_values()
        .map(|value| serde_json::from_value(value).context("failed to parse board zone"))
        .collect::<Result<Vec<Zone>>>()?;
    zones.sort_by(|a, b| a.uuid.cmp(&b.uuid));
    Ok(zones)
}

pub(crate) fn query_native_project_board_diagnostics(root: &Path) -> Result<DiagnosticsView> {
    let project = load_native_project(root)?;
    Ok(DiagnosticsView::Board {
        diagnostics: build_native_project_board(&project)?.diagnostics(),
    })
}

pub(crate) fn query_native_project_board_unrouted(root: &Path) -> Result<UnroutedView> {
    let project = load_native_project(root)?;
    Ok(UnroutedView::Board {
        airwires: build_native_project_board(&project)?.unrouted(),
    })
}

pub(crate) fn query_native_project_board_check(root: &Path) -> Result<CheckReport> {
    let project = load_native_project(root)?;
    let board = build_native_project_board(&project)?;
    let diagnostics = board.diagnostics();
    Ok(CheckReport::Board {
        summary: summarize_native_board_checks(&diagnostics),
        diagnostics,
    })
}

pub(crate) fn query_native_project_board_nets(root: &Path) -> Result<Vec<Net>> {
    let project = load_native_project(root)?;
    let mut nets = project
        .board
        .nets
        .into_values()
        .map(|value| serde_json::from_value(value).context("failed to parse board net"))
        .collect::<Result<Vec<Net>>>()?;
    nets.sort_by(|a, b| a.name.cmp(&b.name).then_with(|| a.uuid.cmp(&b.uuid)));
    Ok(nets)
}

pub(crate) fn query_native_project_board_net_classes(root: &Path) -> Result<Vec<NetClass>> {
    let project = load_native_project(root)?;
    let mut net_classes = project
        .board
        .net_classes
        .into_values()
        .map(|value| serde_json::from_value(value).context("failed to parse board net class"))
        .collect::<Result<Vec<NetClass>>>()?;
    net_classes.sort_by(|a, b| a.name.cmp(&b.name).then_with(|| a.uuid.cmp(&b.uuid)));
    Ok(net_classes)
}

pub(crate) fn query_native_project_board_dimensions(root: &Path) -> Result<Vec<Dimension>> {
    let project = load_native_project(root)?;
    let mut dimensions = project
        .board
        .dimensions
        .into_iter()
        .map(|value| serde_json::from_value(value).context("failed to parse board dimension"))
        .collect::<Result<Vec<Dimension>>>()?;
    dimensions.sort_by(|a, b| a.uuid.cmp(&b.uuid));
    Ok(dimensions)
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
    let relative_path =
        project.schematic.sheets.get(&sheet_key).ok_or_else(|| {
            anyhow::anyhow!("sheet not found in native schematic root: {sheet_uuid}")
        })?;
    let sheet_path = project.root.join("schematic").join(relative_path);
    let sheet_text = std::fs::read_to_string(&sheet_path)
        .with_context(|| format!("failed to read {}", sheet_path.display()))?;
    let mut sheet_value: serde_json::Value = serde_json::from_str(&sheet_text)
        .with_context(|| format!("failed to parse {}", sheet_path.display()))?;
    let labels = sheet_value
        .as_object_mut()
        .and_then(|object| object.get_mut("labels"))
        .and_then(serde_json::Value::as_object_mut)
        .ok_or_else(|| {
            anyhow::anyhow!("sheet labels object missing in {}", sheet_path.display())
        })?;

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
    let relative_path =
        project.schematic.sheets.get(&sheet_key).ok_or_else(|| {
            anyhow::anyhow!("sheet not found in native schematic root: {sheet_uuid}")
        })?;
    let sheet_path = project.root.join("schematic").join(relative_path);
    let sheet_text = std::fs::read_to_string(&sheet_path)
        .with_context(|| format!("failed to read {}", sheet_path.display()))?;
    let mut sheet_value: serde_json::Value = serde_json::from_str(&sheet_text)
        .with_context(|| format!("failed to parse {}", sheet_path.display()))?;
    let symbols = sheet_value
        .as_object_mut()
        .and_then(|object| object.get_mut("symbols"))
        .and_then(serde_json::Value::as_object_mut)
        .ok_or_else(|| {
            anyhow::anyhow!("sheet symbols object missing in {}", sheet_path.display())
        })?;

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
        hidden_power_behavior: render_hidden_power_behavior(
            &HiddenPowerBehavior::SourceDefinedImplicit,
        ),
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
        .ok_or_else(|| {
            anyhow::anyhow!("sheet symbols object missing in {}", sheet_path.display())
        })?;
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

pub(crate) fn set_native_project_symbol_lib_id(
    root: &Path,
    symbol_uuid: Uuid,
    lib_id: String,
) -> Result<NativeProjectSymbolMutationReportView> {
    let project = load_native_project(root)?;
    let (sheet_uuid, sheet_path, mut sheet_value, mut symbol) =
        load_native_symbol_mutation_target(&project, symbol_uuid)?;
    symbol.lib_id = Some(lib_id);
    write_symbol_into_sheet(&mut sheet_value, &symbol)?;
    write_canonical_json(&sheet_path, &sheet_value)?;

    Ok(NativeProjectSymbolMutationReportView {
        action: "set_symbol_lib_id".to_string(),
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

pub(crate) fn clear_native_project_symbol_lib_id(
    root: &Path,
    symbol_uuid: Uuid,
) -> Result<NativeProjectSymbolMutationReportView> {
    let project = load_native_project(root)?;
    let (sheet_uuid, sheet_path, mut sheet_value, mut symbol) =
        load_native_symbol_mutation_target(&project, symbol_uuid)?;
    symbol.lib_id = None;
    write_symbol_into_sheet(&mut sheet_value, &symbol)?;
    write_canonical_json(&sheet_path, &sheet_value)?;

    Ok(NativeProjectSymbolMutationReportView {
        action: "clear_symbol_lib_id".to_string(),
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

pub(crate) fn set_native_project_symbol_entity(
    root: &Path,
    symbol_uuid: Uuid,
    entity_uuid: Uuid,
) -> Result<NativeProjectSymbolMutationReportView> {
    let project = load_native_project(root)?;
    let (sheet_uuid, sheet_path, mut sheet_value, mut symbol) =
        load_native_symbol_mutation_target(&project, symbol_uuid)?;
    symbol.entity = Some(entity_uuid);
    symbol.part = None;
    write_symbol_into_sheet(&mut sheet_value, &symbol)?;
    write_canonical_json(&sheet_path, &sheet_value)?;

    Ok(NativeProjectSymbolMutationReportView {
        action: "set_symbol_entity".to_string(),
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

pub(crate) fn clear_native_project_symbol_entity(
    root: &Path,
    symbol_uuid: Uuid,
) -> Result<NativeProjectSymbolMutationReportView> {
    let project = load_native_project(root)?;
    let (sheet_uuid, sheet_path, mut sheet_value, mut symbol) =
        load_native_symbol_mutation_target(&project, symbol_uuid)?;
    symbol.entity = None;
    write_symbol_into_sheet(&mut sheet_value, &symbol)?;
    write_canonical_json(&sheet_path, &sheet_value)?;

    Ok(NativeProjectSymbolMutationReportView {
        action: "clear_symbol_entity".to_string(),
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

pub(crate) fn set_native_project_symbol_part(
    root: &Path,
    symbol_uuid: Uuid,
    part_uuid: Uuid,
) -> Result<NativeProjectSymbolMutationReportView> {
    let project = load_native_project(root)?;
    let (sheet_uuid, sheet_path, mut sheet_value, mut symbol) =
        load_native_symbol_mutation_target(&project, symbol_uuid)?;
    symbol.part = Some(part_uuid);
    symbol.entity = None;
    write_symbol_into_sheet(&mut sheet_value, &symbol)?;
    write_canonical_json(&sheet_path, &sheet_value)?;

    Ok(NativeProjectSymbolMutationReportView {
        action: "set_symbol_part".to_string(),
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

pub(crate) fn clear_native_project_symbol_part(
    root: &Path,
    symbol_uuid: Uuid,
) -> Result<NativeProjectSymbolMutationReportView> {
    let project = load_native_project(root)?;
    let (sheet_uuid, sheet_path, mut sheet_value, mut symbol) =
        load_native_symbol_mutation_target(&project, symbol_uuid)?;
    symbol.part = None;
    write_symbol_into_sheet(&mut sheet_value, &symbol)?;
    write_canonical_json(&sheet_path, &sheet_value)?;

    Ok(NativeProjectSymbolMutationReportView {
        action: "clear_symbol_part".to_string(),
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
    if let Some(entry) = symbol
        .pin_overrides
        .iter_mut()
        .find(|entry| entry.pin == pin_uuid)
    {
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
    let relative_path =
        project.schematic.sheets.get(&sheet_key).ok_or_else(|| {
            anyhow::anyhow!("sheet not found in native schematic root: {sheet_uuid}")
        })?;
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
    let relative_path =
        project.schematic.sheets.get(&sheet_key).ok_or_else(|| {
            anyhow::anyhow!("sheet not found in native schematic root: {sheet_uuid}")
        })?;
    let sheet_path = project.root.join("schematic").join(relative_path);
    let sheet_text = std::fs::read_to_string(&sheet_path)
        .with_context(|| format!("failed to read {}", sheet_path.display()))?;
    let mut sheet_value: serde_json::Value = serde_json::from_str(&sheet_text)
        .with_context(|| format!("failed to parse {}", sheet_path.display()))?;
    let drawings = sheet_value
        .as_object_mut()
        .and_then(|object| object.get_mut("drawings"))
        .and_then(serde_json::Value::as_object_mut)
        .ok_or_else(|| {
            anyhow::anyhow!("sheet drawings object missing in {}", sheet_path.display())
        })?;

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
        .ok_or_else(|| {
            anyhow::anyhow!("sheet drawings object missing in {}", sheet_path.display())
        })?;
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
        .ok_or_else(|| {
            anyhow::anyhow!("sheet labels object missing in {}", sheet_path.display())
        })?;
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
    let relative_path =
        project.schematic.sheets.get(&sheet_key).ok_or_else(|| {
            anyhow::anyhow!("sheet not found in native schematic root: {sheet_uuid}")
        })?;
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
        serde_json::to_value(SchematicWire {
            uuid: wire_uuid,
            from,
            to,
        })
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
    let relative_path =
        project.schematic.sheets.get(&sheet_key).ok_or_else(|| {
            anyhow::anyhow!("sheet not found in native schematic root: {sheet_uuid}")
        })?;
    let sheet_path = project.root.join("schematic").join(relative_path);
    let sheet_text = std::fs::read_to_string(&sheet_path)
        .with_context(|| format!("failed to read {}", sheet_path.display()))?;
    let mut sheet_value: serde_json::Value = serde_json::from_str(&sheet_text)
        .with_context(|| format!("failed to parse {}", sheet_path.display()))?;
    let junctions = sheet_value
        .as_object_mut()
        .and_then(|object| object.get_mut("junctions"))
        .and_then(serde_json::Value::as_object_mut)
        .ok_or_else(|| {
            anyhow::anyhow!("sheet junctions object missing in {}", sheet_path.display())
        })?;

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
        .ok_or_else(|| {
            anyhow::anyhow!("sheet junctions object missing in {}", sheet_path.display())
        })?;
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
    let relative_path =
        project.schematic.sheets.get(&sheet_key).ok_or_else(|| {
            anyhow::anyhow!("sheet not found in native schematic root: {sheet_uuid}")
        })?;
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
    let relative_path =
        project.schematic.sheets.get(&sheet_key).ok_or_else(|| {
            anyhow::anyhow!("sheet not found in native schematic root: {sheet_uuid}")
        })?;
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
    let relative_path =
        project.schematic.sheets.get(&sheet_key).ok_or_else(|| {
            anyhow::anyhow!("sheet not found in native schematic root: {sheet_uuid}")
        })?;
    let sheet_path = project.root.join("schematic").join(relative_path);
    let sheet_text = std::fs::read_to_string(&sheet_path)
        .with_context(|| format!("failed to read {}", sheet_path.display()))?;
    let mut sheet_value: serde_json::Value = serde_json::from_str(&sheet_text)
        .with_context(|| format!("failed to parse {}", sheet_path.display()))?;
    let bus_entries = sheet_value
        .as_object_mut()
        .and_then(|object| object.get_mut("bus_entries"))
        .and_then(serde_json::Value::as_object_mut)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "sheet bus_entries object missing in {}",
                sheet_path.display()
            )
        })?;

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
        .ok_or_else(|| {
            anyhow::anyhow!(
                "sheet bus_entries object missing in {}",
                sheet_path.display()
            )
        })?;
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
    let relative_path =
        project.schematic.sheets.get(&sheet_key).ok_or_else(|| {
            anyhow::anyhow!("sheet not found in native schematic root: {sheet_uuid}")
        })?;
    let sheet_path = project.root.join("schematic").join(relative_path);
    let sheet_text = std::fs::read_to_string(&sheet_path)
        .with_context(|| format!("failed to read {}", sheet_path.display()))?;
    let mut sheet_value: serde_json::Value = serde_json::from_str(&sheet_text)
        .with_context(|| format!("failed to parse {}", sheet_path.display()))?;
    let noconnects = sheet_value
        .as_object_mut()
        .and_then(|object| object.get_mut("noconnects"))
        .and_then(serde_json::Value::as_object_mut)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "sheet noconnects object missing in {}",
                sheet_path.display()
            )
        })?;

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
        .ok_or_else(|| {
            anyhow::anyhow!(
                "sheet noconnects object missing in {}",
                sheet_path.display()
            )
        })?;
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
            bail!(
                "project root exists but is not a directory: {}",
                root.display()
            );
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
        bail!(
            "project root does not exist or is not a directory: {}",
            root.display()
        );
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

fn collect_schematic_counts(
    root: &Path,
    schematic: &NativeSchematicRoot,
) -> Result<NativeSchematicCounts> {
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

fn build_native_project_schematic(project: &LoadedNativeProject) -> Result<Schematic> {
    let mut sheets = HashMap::new();

    for (sheet_uuid, relative_path) in &project.schematic.sheets {
        let expected_uuid = Uuid::parse_str(sheet_uuid)
            .with_context(|| format!("invalid sheet UUID key `{sheet_uuid}` in schematic root"))?;
        let path = project.root.join("schematic").join(relative_path);
        let native_sheet = load_native_sheet(&path)?;
        if native_sheet.uuid != expected_uuid {
            bail!(
                "sheet UUID mismatch: schematic root key {} does not match {} in {}",
                expected_uuid,
                native_sheet.uuid,
                path.display()
            );
        }
        sheets.insert(expected_uuid, native_sheet_into_engine_sheet(native_sheet));
    }

    Ok(Schematic {
        uuid: project.schematic.uuid,
        sheets,
        // Native connectivity queries only need the authored sheet graph for now.
        sheet_definitions: HashMap::new(),
        sheet_instances: HashMap::new(),
        variants: HashMap::new(),
        waivers: Vec::<CheckWaiver>::new(),
    })
}

fn build_native_project_board(project: &LoadedNativeProject) -> Result<Board> {
    let stackup_layers = project
        .board
        .stackup
        .layers
        .iter()
        .cloned()
        .map(|value| serde_json::from_value(value).context("failed to parse board stackup layer"))
        .collect::<Result<Vec<StackupLayer>>>()?;
    let packages = project
        .board
        .packages
        .values()
        .cloned()
        .map(|value| {
            let package: PlacedPackage =
                serde_json::from_value(value).context("failed to parse board component")?;
            Ok((package.uuid, package))
        })
        .collect::<Result<HashMap<Uuid, PlacedPackage>>>()?;
    let pads = project
        .board
        .pads
        .values()
        .cloned()
        .map(|value| {
            let pad: PlacedPad =
                serde_json::from_value(value).context("failed to parse board pad")?;
            Ok((pad.uuid, pad))
        })
        .collect::<Result<HashMap<Uuid, PlacedPad>>>()?;
    let tracks = project
        .board
        .tracks
        .values()
        .cloned()
        .map(|value| {
            let track: Track =
                serde_json::from_value(value).context("failed to parse board track")?;
            Ok((track.uuid, track))
        })
        .collect::<Result<HashMap<Uuid, Track>>>()?;
    let vias = project
        .board
        .vias
        .values()
        .cloned()
        .map(|value| {
            let via: Via = serde_json::from_value(value).context("failed to parse board via")?;
            Ok((via.uuid, via))
        })
        .collect::<Result<HashMap<Uuid, Via>>>()?;
    let zones = project
        .board
        .zones
        .values()
        .cloned()
        .map(|value| {
            let zone: Zone = serde_json::from_value(value).context("failed to parse board zone")?;
            Ok((zone.uuid, zone))
        })
        .collect::<Result<HashMap<Uuid, Zone>>>()?;
    let nets = project
        .board
        .nets
        .values()
        .cloned()
        .map(|value| {
            let net: Net = serde_json::from_value(value).context("failed to parse board net")?;
            Ok((net.uuid, net))
        })
        .collect::<Result<HashMap<Uuid, Net>>>()?;
    let net_classes = project
        .board
        .net_classes
        .values()
        .cloned()
        .map(|value| {
            let net_class: NetClass =
                serde_json::from_value(value).context("failed to parse board net class")?;
            Ok((net_class.uuid, net_class))
        })
        .collect::<Result<HashMap<Uuid, NetClass>>>()?;
    let keepouts = project
        .board
        .keepouts
        .iter()
        .cloned()
        .map(|value| serde_json::from_value(value).context("failed to parse board keepout"))
        .collect::<Result<Vec<Keepout>>>()?;
    let dimensions = project
        .board
        .dimensions
        .iter()
        .cloned()
        .map(|value| serde_json::from_value(value).context("failed to parse board dimension"))
        .collect::<Result<Vec<Dimension>>>()?;
    let texts = project
        .board
        .texts
        .iter()
        .cloned()
        .map(|value| serde_json::from_value(value).context("failed to parse board text"))
        .collect::<Result<Vec<BoardText>>>()?;
    let rules = project
        .rules
        .rules
        .iter()
        .cloned()
        .map(|value| serde_json::from_value(value).context("failed to parse board rule"))
        .collect::<Result<Vec<Rule>>>()?;

    Ok(Board {
        uuid: project.board.uuid,
        name: project.board.name.clone(),
        stackup: Stackup {
            layers: stackup_layers,
        },
        outline: Polygon {
            vertices: project
                .board
                .outline
                .vertices
                .iter()
                .map(|point| Point {
                    x: point.x,
                    y: point.y,
                })
                .collect(),
            closed: project.board.outline.closed,
        },
        packages,
        pads,
        tracks,
        vias,
        zones,
        nets,
        net_classes,
        rules,
        keepouts,
        dimensions,
        texts,
    })
}

fn summarize_native_schematic_checks(
    diagnostics: &[ConnectivityDiagnosticInfo],
    erc_findings: &[ErcFinding],
) -> CheckSummary {
    let mut by_code: BTreeMap<String, usize> = BTreeMap::new();
    let mut errors = 0usize;
    let mut warnings = 0usize;
    let mut infos = 0usize;
    let mut waived = 0usize;

    for diagnostic in diagnostics {
        *by_code.entry(diagnostic.kind.clone()).or_default() += 1;
        match diagnostic.severity.as_str() {
            "error" => errors += 1,
            "warning" => warnings += 1,
            _ => infos += 1,
        }
    }

    for finding in erc_findings {
        *by_code.entry(finding.code.to_string()).or_default() += 1;
        if finding.waived {
            waived += 1;
            continue;
        }
        match finding.severity {
            eda_engine::erc::ErcSeverity::Error => errors += 1,
            eda_engine::erc::ErcSeverity::Warning => warnings += 1,
            eda_engine::erc::ErcSeverity::Info => infos += 1,
        }
    }

    let status = if errors > 0 {
        CheckStatus::Error
    } else if warnings > 0 {
        CheckStatus::Warning
    } else if infos > 0 {
        CheckStatus::Info
    } else {
        CheckStatus::Ok
    };

    let mut by_code = by_code
        .into_iter()
        .map(|(code, count)| CheckCodeCount { code, count })
        .collect::<Vec<_>>();
    by_code.sort_by(|a, b| a.code.cmp(&b.code));

    CheckSummary {
        status,
        errors,
        warnings,
        infos,
        waived,
        by_code,
    }
}

fn summarize_native_board_checks(diagnostics: &[ConnectivityDiagnosticInfo]) -> CheckSummary {
    let mut by_code: BTreeMap<String, usize> = BTreeMap::new();
    let mut errors = 0usize;
    let mut warnings = 0usize;
    let mut infos = 0usize;

    for diagnostic in diagnostics {
        *by_code.entry(diagnostic.kind.clone()).or_default() += 1;
        match diagnostic.severity.as_str() {
            "error" => errors += 1,
            "warning" => warnings += 1,
            _ => infos += 1,
        }
    }

    let status = if errors > 0 {
        CheckStatus::Error
    } else if warnings > 0 {
        CheckStatus::Warning
    } else if infos > 0 {
        CheckStatus::Info
    } else {
        CheckStatus::Ok
    };

    let mut by_code = by_code
        .into_iter()
        .map(|(code, count)| CheckCodeCount { code, count })
        .collect::<Vec<_>>();
    by_code.sort_by(|a, b| a.code.cmp(&b.code));

    CheckSummary {
        status,
        errors,
        warnings,
        infos,
        waived: 0,
        by_code,
    }
}

pub(crate) fn place_native_project_board_text(
    root: &Path,
    text: String,
    position: Point,
    rotation_deg: i32,
    height_nm: i64,
    stroke_width_nm: i64,
    layer: i32,
) -> Result<NativeProjectBoardTextMutationReportView> {
    let mut project = load_native_project(root)?;
    let text_uuid = Uuid::new_v4();
    if height_nm <= 0 {
        bail!("board text height must be positive");
    }
    if stroke_width_nm <= 0 {
        bail!("board text stroke width must be positive");
    }
    let board_text = BoardText {
        uuid: text_uuid,
        text: text.clone(),
        position,
        rotation: rotation_deg,
        height_nm,
        stroke_width_nm,
        layer,
    };
    project.board.texts.push(
        serde_json::to_value(&board_text).expect("native board text serialization must succeed"),
    );
    write_canonical_json(&project.board_path, &project.board)?;
    Ok(NativeProjectBoardTextMutationReportView {
        action: "place_board_text".to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        text_uuid: text_uuid.to_string(),
        text,
        x_nm: position.x,
        y_nm: position.y,
        rotation_deg,
        height_nm,
        stroke_width_nm,
        layer,
    })
}

pub(crate) fn edit_native_project_board_text(
    root: &Path,
    text_uuid: Uuid,
    value: Option<String>,
    x_nm: Option<i64>,
    y_nm: Option<i64>,
    rotation_deg: Option<i32>,
    height_nm: Option<i64>,
    stroke_width_nm: Option<i64>,
    layer: Option<i32>,
) -> Result<NativeProjectBoardTextMutationReportView> {
    let mut project = load_native_project(root)?;
    let index = project
        .board
        .texts
        .iter()
        .position(|entry| {
            serde_json::from_value::<BoardText>(entry.clone())
                .map(|text| text.uuid == text_uuid)
                .unwrap_or(false)
        })
        .ok_or_else(|| anyhow::anyhow!("board text not found in native project: {text_uuid}"))?;
    let mut board_text: BoardText = serde_json::from_value(project.board.texts[index].clone())
        .with_context(|| {
            format!(
                "failed to parse board text in {}",
                project.board_path.display()
            )
        })?;
    if let Some(value) = value {
        board_text.text = value;
    }
    match (x_nm, y_nm) {
        (None, None) => {}
        (Some(x), Some(y)) => board_text.position = Point { x, y },
        _ => bail!("board text position requires both --x-nm and --y-nm"),
    }
    if let Some(rotation_deg) = rotation_deg {
        board_text.rotation = rotation_deg;
    }
    if let Some(height_nm) = height_nm {
        board_text.height_nm = height_nm;
    }
    if let Some(stroke_width_nm) = stroke_width_nm {
        board_text.stroke_width_nm = stroke_width_nm;
    }
    if let Some(layer) = layer {
        board_text.layer = layer;
    }
    if board_text.height_nm <= 0 {
        bail!("board text height must be positive");
    }
    if board_text.stroke_width_nm <= 0 {
        bail!("board text stroke width must be positive");
    }
    project.board.texts[index] =
        serde_json::to_value(&board_text).expect("native board text serialization must succeed");
    write_canonical_json(&project.board_path, &project.board)?;
    Ok(NativeProjectBoardTextMutationReportView {
        action: "edit_board_text".to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        text_uuid: board_text.uuid.to_string(),
        text: board_text.text,
        x_nm: board_text.position.x,
        y_nm: board_text.position.y,
        rotation_deg: board_text.rotation,
        height_nm: board_text.height_nm,
        stroke_width_nm: board_text.stroke_width_nm,
        layer: board_text.layer,
    })
}

pub(crate) fn delete_native_project_board_text(
    root: &Path,
    text_uuid: Uuid,
) -> Result<NativeProjectBoardTextMutationReportView> {
    let mut project = load_native_project(root)?;
    let index = project
        .board
        .texts
        .iter()
        .position(|entry| {
            serde_json::from_value::<BoardText>(entry.clone())
                .map(|text| text.uuid == text_uuid)
                .unwrap_or(false)
        })
        .ok_or_else(|| anyhow::anyhow!("board text not found in native project: {text_uuid}"))?;
    let board_text: BoardText = serde_json::from_value(project.board.texts.remove(index))
        .with_context(|| {
            format!(
                "failed to parse board text in {}",
                project.board_path.display()
            )
        })?;
    write_canonical_json(&project.board_path, &project.board)?;
    Ok(NativeProjectBoardTextMutationReportView {
        action: "delete_board_text".to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        text_uuid: board_text.uuid.to_string(),
        text: board_text.text,
        x_nm: board_text.position.x,
        y_nm: board_text.position.y,
        rotation_deg: board_text.rotation,
        height_nm: board_text.height_nm,
        stroke_width_nm: board_text.stroke_width_nm,
        layer: board_text.layer,
    })
}

pub(crate) fn place_native_project_board_keepout(
    root: &Path,
    kind: String,
    layers: Vec<i32>,
    polygon: Polygon,
) -> Result<NativeProjectBoardKeepoutMutationReportView> {
    let mut project = load_native_project(root)?;
    let keepout_uuid = Uuid::new_v4();
    let keepout = Keepout {
        uuid: keepout_uuid,
        polygon,
        layers,
        kind: kind.clone(),
    };
    project.board.keepouts.push(
        serde_json::to_value(&keepout).expect("native board keepout serialization must succeed"),
    );
    write_canonical_json(&project.board_path, &project.board)?;
    Ok(NativeProjectBoardKeepoutMutationReportView {
        action: "place_board_keepout".to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        keepout_uuid: keepout_uuid.to_string(),
        kind,
        layer_count: keepout.layers.len(),
        vertex_count: keepout.polygon.vertices.len(),
    })
}

pub(crate) fn edit_native_project_board_keepout(
    root: &Path,
    keepout_uuid: Uuid,
    kind: Option<String>,
    layers: Vec<i32>,
    polygon: Option<Polygon>,
) -> Result<NativeProjectBoardKeepoutMutationReportView> {
    let mut project = load_native_project(root)?;
    let index = project
        .board
        .keepouts
        .iter()
        .position(|entry| {
            serde_json::from_value::<Keepout>(entry.clone())
                .map(|keepout| keepout.uuid == keepout_uuid)
                .unwrap_or(false)
        })
        .ok_or_else(|| {
            anyhow::anyhow!("board keepout not found in native project: {keepout_uuid}")
        })?;
    let mut keepout: Keepout = serde_json::from_value(project.board.keepouts[index].clone())
        .with_context(|| {
            format!(
                "failed to parse board keepout in {}",
                project.board_path.display()
            )
        })?;
    if let Some(kind) = kind {
        keepout.kind = kind;
    }
    if !layers.is_empty() {
        keepout.layers = layers;
    }
    if let Some(polygon) = polygon {
        keepout.polygon = polygon;
    }
    project.board.keepouts[index] =
        serde_json::to_value(&keepout).expect("native board keepout serialization must succeed");
    write_canonical_json(&project.board_path, &project.board)?;
    Ok(NativeProjectBoardKeepoutMutationReportView {
        action: "edit_board_keepout".to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        keepout_uuid: keepout.uuid.to_string(),
        kind: keepout.kind,
        layer_count: keepout.layers.len(),
        vertex_count: keepout.polygon.vertices.len(),
    })
}

pub(crate) fn delete_native_project_board_keepout(
    root: &Path,
    keepout_uuid: Uuid,
) -> Result<NativeProjectBoardKeepoutMutationReportView> {
    let mut project = load_native_project(root)?;
    let index = project
        .board
        .keepouts
        .iter()
        .position(|entry| {
            serde_json::from_value::<Keepout>(entry.clone())
                .map(|keepout| keepout.uuid == keepout_uuid)
                .unwrap_or(false)
        })
        .ok_or_else(|| {
            anyhow::anyhow!("board keepout not found in native project: {keepout_uuid}")
        })?;
    let keepout: Keepout = serde_json::from_value(project.board.keepouts.remove(index))
        .with_context(|| {
            format!(
                "failed to parse board keepout in {}",
                project.board_path.display()
            )
        })?;
    write_canonical_json(&project.board_path, &project.board)?;
    Ok(NativeProjectBoardKeepoutMutationReportView {
        action: "delete_board_keepout".to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        keepout_uuid: keepout.uuid.to_string(),
        kind: keepout.kind,
        layer_count: keepout.layers.len(),
        vertex_count: keepout.polygon.vertices.len(),
    })
}

pub(crate) fn set_native_project_board_outline(
    root: &Path,
    polygon: Polygon,
) -> Result<NativeProjectBoardOutlineMutationReportView> {
    let mut project = load_native_project(root)?;
    project.board.outline = NativeOutline {
        vertices: polygon
            .vertices
            .iter()
            .map(|point| NativePoint {
                x: point.x,
                y: point.y,
            })
            .collect(),
        closed: polygon.closed,
    };
    write_canonical_json(&project.board_path, &project.board)?;
    Ok(NativeProjectBoardOutlineMutationReportView {
        action: "set_board_outline".to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        vertex_count: polygon.vertices.len(),
        closed: polygon.closed,
    })
}

pub(crate) fn set_native_project_board_stackup(
    root: &Path,
    layers: Vec<StackupLayer>,
) -> Result<NativeProjectBoardStackupMutationReportView> {
    let mut project = load_native_project(root)?;
    project.board.stackup = NativeStackup {
        layers: layers
            .into_iter()
            .map(|layer| {
                serde_json::to_value(layer)
                    .expect("native board stackup serialization must succeed")
            })
            .collect(),
    };
    let layer_count = project.board.stackup.layers.len();
    write_canonical_json(&project.board_path, &project.board)?;
    Ok(NativeProjectBoardStackupMutationReportView {
        action: "set_board_stackup".to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        layer_count,
    })
}

pub(crate) fn place_native_project_board_net(
    root: &Path,
    name: String,
    class_uuid: Uuid,
) -> Result<NativeProjectBoardNetMutationReportView> {
    let mut project = load_native_project(root)?;
    let net_uuid = Uuid::new_v4();
    let net = Net {
        uuid: net_uuid,
        name,
        class: class_uuid,
    };
    project.board.nets.insert(
        net_uuid.to_string(),
        serde_json::to_value(&net).expect("native board net serialization must succeed"),
    );
    write_canonical_json(&project.board_path, &project.board)?;
    Ok(native_project_board_net_report(
        "place_board_net",
        &project,
        net,
    ))
}

pub(crate) fn place_native_project_board_component(
    root: &Path,
    part_uuid: Uuid,
    package_uuid: Uuid,
    reference: String,
    value: String,
    position: Point,
    layer: i32,
) -> Result<NativeProjectBoardComponentMutationReportView> {
    let mut project = load_native_project(root)?;
    let component_uuid = Uuid::new_v4();
    let component = PlacedPackage {
        uuid: component_uuid,
        part: part_uuid,
        package: package_uuid,
        reference,
        value,
        position,
        rotation: 0,
        layer,
        locked: false,
    };
    project.board.packages.insert(
        component_uuid.to_string(),
        serde_json::to_value(&component)
            .expect("native board component serialization must succeed"),
    );
    materialize_supported_pool_package_graphics(&mut project, &component)?;
    write_canonical_json(&project.board_path, &project.board)?;
    Ok(native_project_board_component_report(
        "place_board_component",
        &project,
        component,
    ))
}

pub(crate) fn place_native_project_board_track(
    root: &Path,
    net_uuid: Uuid,
    from: Point,
    to: Point,
    width_nm: i64,
    layer: i32,
) -> Result<NativeProjectBoardTrackMutationReportView> {
    let mut project = load_native_project(root)?;
    if !project.board.nets.contains_key(&net_uuid.to_string()) {
        bail!("board net not found in native project: {net_uuid}");
    }
    let track_uuid = Uuid::new_v4();
    let track = Track {
        uuid: track_uuid,
        net: net_uuid,
        from,
        to,
        width: width_nm,
        layer,
    };
    project.board.tracks.insert(
        track_uuid.to_string(),
        serde_json::to_value(&track).expect("native board track serialization must succeed"),
    );
    write_canonical_json(&project.board_path, &project.board)?;
    Ok(native_project_board_track_report(
        "draw_board_track",
        &project,
        track,
    ))
}

pub(crate) fn place_native_project_board_via(
    root: &Path,
    net_uuid: Uuid,
    position: Point,
    drill_nm: i64,
    diameter_nm: i64,
    from_layer: i32,
    to_layer: i32,
) -> Result<NativeProjectBoardViaMutationReportView> {
    let mut project = load_native_project(root)?;
    if !project.board.nets.contains_key(&net_uuid.to_string()) {
        bail!("board net not found in native project: {net_uuid}");
    }
    let via_uuid = Uuid::new_v4();
    let via = Via {
        uuid: via_uuid,
        net: net_uuid,
        position,
        drill: drill_nm,
        diameter: diameter_nm,
        from_layer,
        to_layer,
    };
    project.board.vias.insert(
        via_uuid.to_string(),
        serde_json::to_value(&via).expect("native board via serialization must succeed"),
    );
    write_canonical_json(&project.board_path, &project.board)?;
    Ok(native_project_board_via_report(
        "place_board_via",
        &project,
        via,
    ))
}

pub(crate) fn place_native_project_board_zone(
    root: &Path,
    net_uuid: Uuid,
    polygon: Polygon,
    layer: i32,
    priority: u32,
    thermal_relief: bool,
    thermal_gap_nm: i64,
    thermal_spoke_width_nm: i64,
) -> Result<NativeProjectBoardZoneMutationReportView> {
    let mut project = load_native_project(root)?;
    if !project.board.nets.contains_key(&net_uuid.to_string()) {
        bail!("board net not found in native project: {net_uuid}");
    }
    let zone_uuid = Uuid::new_v4();
    let zone = Zone {
        uuid: zone_uuid,
        net: net_uuid,
        polygon,
        layer,
        priority,
        thermal_relief,
        thermal_gap: thermal_gap_nm,
        thermal_spoke_width: thermal_spoke_width_nm,
    };
    project.board.zones.insert(
        zone_uuid.to_string(),
        serde_json::to_value(&zone).expect("native board zone serialization must succeed"),
    );
    write_canonical_json(&project.board_path, &project.board)?;
    Ok(native_project_board_zone_report(
        "place_board_zone",
        &project,
        zone,
    ))
}

pub(crate) fn edit_native_project_board_net(
    root: &Path,
    net_uuid: Uuid,
    name: Option<String>,
    class_uuid: Option<Uuid>,
) -> Result<NativeProjectBoardNetMutationReportView> {
    let mut project = load_native_project(root)?;
    let key = net_uuid.to_string();
    let entry = project
        .board
        .nets
        .get(&key)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("board net not found in native project: {net_uuid}"))?;
    let mut net: Net = serde_json::from_value(entry).with_context(|| {
        format!(
            "failed to parse board net in {}",
            project.board_path.display()
        )
    })?;
    if let Some(name) = name {
        net.name = name;
    }
    if let Some(class_uuid) = class_uuid {
        net.class = class_uuid;
    }
    project.board.nets.insert(
        key,
        serde_json::to_value(&net).expect("native board net serialization must succeed"),
    );
    write_canonical_json(&project.board_path, &project.board)?;
    Ok(native_project_board_net_report(
        "edit_board_net",
        &project,
        net,
    ))
}

pub(crate) fn move_native_project_board_component(
    root: &Path,
    component_uuid: Uuid,
    position: Point,
) -> Result<NativeProjectBoardComponentMutationReportView> {
    let mut project = load_native_project(root)?;
    let key = component_uuid.to_string();
    let entry = project.board.packages.get(&key).cloned().ok_or_else(|| {
        anyhow::anyhow!("board component not found in native project: {component_uuid}")
    })?;
    let mut component: PlacedPackage = serde_json::from_value(entry).with_context(|| {
        format!(
            "failed to parse board component in {}",
            project.board_path.display()
        )
    })?;
    component.position = position;
    project.board.packages.insert(
        key.clone(),
        serde_json::to_value(&component)
            .expect("native board component serialization must succeed"),
    );
    write_canonical_json(&project.board_path, &project.board)?;
    Ok(native_project_board_component_report(
        "move_board_component",
        &project,
        component,
    ))
}

pub(crate) fn set_native_project_board_component_part(
    root: &Path,
    component_uuid: Uuid,
    part_uuid: Uuid,
) -> Result<NativeProjectBoardComponentMutationReportView> {
    let mut project = load_native_project(root)?;
    let key = component_uuid.to_string();
    let entry = project.board.packages.get(&key).cloned().ok_or_else(|| {
        anyhow::anyhow!("board component not found in native project: {component_uuid}")
    })?;
    let mut component: PlacedPackage = serde_json::from_value(entry).with_context(|| {
        format!(
            "failed to parse board component in {}",
            project.board_path.display()
        )
    })?;
    component.part = part_uuid;
    project.board.packages.insert(
        key.clone(),
        serde_json::to_value(&component)
            .expect("native board component serialization must succeed"),
    );
    write_canonical_json(&project.board_path, &project.board)?;
    Ok(native_project_board_component_report(
        "set_board_component_part",
        &project,
        component,
    ))
}

pub(crate) fn set_native_project_board_component_package(
    root: &Path,
    component_uuid: Uuid,
    package_uuid: Uuid,
) -> Result<NativeProjectBoardComponentMutationReportView> {
    let mut project = load_native_project(root)?;
    let key = component_uuid.to_string();
    let entry = project.board.packages.get(&key).cloned().ok_or_else(|| {
        anyhow::anyhow!("board component not found in native project: {component_uuid}")
    })?;
    let mut component: PlacedPackage = serde_json::from_value(entry).with_context(|| {
        format!(
            "failed to parse board component in {}",
            project.board_path.display()
        )
    })?;
    component.package = package_uuid;
    project.board.packages.insert(
        key.clone(),
        serde_json::to_value(&component)
            .expect("native board component serialization must succeed"),
    );
    project.board.component_silkscreen.remove(&key);
    project.board.component_silkscreen_texts.remove(&key);
    project.board.component_silkscreen_arcs.remove(&key);
    project.board.component_silkscreen_circles.remove(&key);
    project.board.component_silkscreen_polygons.remove(&key);
    project.board.component_silkscreen_polylines.remove(&key);
    project.board.component_mechanical_lines.remove(&key);
    project.board.component_mechanical_texts.remove(&key);
    project.board.component_mechanical_polygons.remove(&key);
    project.board.component_mechanical_polylines.remove(&key);
    project.board.component_mechanical_circles.remove(&key);
    project.board.component_mechanical_arcs.remove(&key);
    materialize_supported_pool_package_graphics(&mut project, &component)?;
    write_canonical_json(&project.board_path, &project.board)?;
    Ok(native_project_board_component_report(
        "set_board_component_package",
        &project,
        component,
    ))
}

pub(crate) fn set_native_project_board_component_value(
    root: &Path,
    component_uuid: Uuid,
    value: String,
) -> Result<NativeProjectBoardComponentMutationReportView> {
    let mut project = load_native_project(root)?;
    let key = component_uuid.to_string();
    let entry = project.board.packages.get(&key).cloned().ok_or_else(|| {
        anyhow::anyhow!("board component not found in native project: {component_uuid}")
    })?;
    let mut component: PlacedPackage = serde_json::from_value(entry).with_context(|| {
        format!(
            "failed to parse board component in {}",
            project.board_path.display()
        )
    })?;
    component.value = value;
    project.board.packages.insert(
        key,
        serde_json::to_value(&component)
            .expect("native board component serialization must succeed"),
    );
    write_canonical_json(&project.board_path, &project.board)?;
    Ok(native_project_board_component_report(
        "set_board_component_value",
        &project,
        component,
    ))
}

pub(crate) fn rotate_native_project_board_component(
    root: &Path,
    component_uuid: Uuid,
    rotation_deg: i32,
) -> Result<NativeProjectBoardComponentMutationReportView> {
    let mut project = load_native_project(root)?;
    let key = component_uuid.to_string();
    let entry = project.board.packages.get(&key).cloned().ok_or_else(|| {
        anyhow::anyhow!("board component not found in native project: {component_uuid}")
    })?;
    let mut component: PlacedPackage = serde_json::from_value(entry).with_context(|| {
        format!(
            "failed to parse board component in {}",
            project.board_path.display()
        )
    })?;
    component.rotation = rotation_deg;
    project.board.packages.insert(
        key,
        serde_json::to_value(&component)
            .expect("native board component serialization must succeed"),
    );
    write_canonical_json(&project.board_path, &project.board)?;
    Ok(native_project_board_component_report(
        "rotate_board_component",
        &project,
        component,
    ))
}

pub(crate) fn set_native_project_board_component_locked(
    root: &Path,
    component_uuid: Uuid,
    locked: bool,
) -> Result<NativeProjectBoardComponentMutationReportView> {
    let mut project = load_native_project(root)?;
    let key = component_uuid.to_string();
    let entry = project.board.packages.get(&key).cloned().ok_or_else(|| {
        anyhow::anyhow!("board component not found in native project: {component_uuid}")
    })?;
    let mut component: PlacedPackage = serde_json::from_value(entry).with_context(|| {
        format!(
            "failed to parse board component in {}",
            project.board_path.display()
        )
    })?;
    component.locked = locked;
    project.board.packages.insert(
        key,
        serde_json::to_value(&component)
            .expect("native board component serialization must succeed"),
    );
    write_canonical_json(&project.board_path, &project.board)?;
    Ok(native_project_board_component_report(
        if locked {
            "set_board_component_locked"
        } else {
            "clear_board_component_locked"
        },
        &project,
        component,
    ))
}

pub(crate) fn delete_native_project_board_component(
    root: &Path,
    component_uuid: Uuid,
) -> Result<NativeProjectBoardComponentMutationReportView> {
    let mut project = load_native_project(root)?;
    let value = project
        .board
        .packages
        .remove(&component_uuid.to_string())
        .ok_or_else(|| {
            anyhow::anyhow!("board component not found in native project: {component_uuid}")
        })?;
    let component: PlacedPackage = serde_json::from_value(value).with_context(|| {
        format!(
            "failed to parse board component in {}",
            project.board_path.display()
        )
    })?;
    project
        .board
        .component_silkscreen
        .remove(&component_uuid.to_string());
    project
        .board
        .component_silkscreen_texts
        .remove(&component_uuid.to_string());
    project
        .board
        .component_silkscreen_arcs
        .remove(&component_uuid.to_string());
    project
        .board
        .component_silkscreen_circles
        .remove(&component_uuid.to_string());
    project
        .board
        .component_silkscreen_polygons
        .remove(&component_uuid.to_string());
    project
        .board
        .component_silkscreen_polylines
        .remove(&component_uuid.to_string());
    project
        .board
        .component_mechanical_lines
        .remove(&component_uuid.to_string());
    project
        .board
        .component_mechanical_texts
        .remove(&component_uuid.to_string());
    project
        .board
        .component_mechanical_polygons
        .remove(&component_uuid.to_string());
    project
        .board
        .component_mechanical_polylines
        .remove(&component_uuid.to_string());
    project
        .board
        .component_mechanical_circles
        .remove(&component_uuid.to_string());
    project
        .board
        .component_mechanical_arcs
        .remove(&component_uuid.to_string());
    write_canonical_json(&project.board_path, &project.board)?;
    Ok(native_project_board_component_report(
        "delete_board_component",
        &project,
        component,
    ))
}

pub(crate) fn set_native_project_board_pad_net(
    root: &Path,
    pad_uuid: Uuid,
    net_uuid: Option<Uuid>,
) -> Result<NativeProjectBoardPadMutationReportView> {
    let mut project = load_native_project(root)?;
    if let Some(net_uuid) = net_uuid
        && !project.board.nets.contains_key(&net_uuid.to_string())
    {
        bail!("board net not found in native project: {net_uuid}");
    }
    let key = pad_uuid.to_string();
    let entry = project
        .board
        .pads
        .get(&key)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("board pad not found in native project: {pad_uuid}"))?;
    let mut pad: PlacedPad = serde_json::from_value(entry).with_context(|| {
        format!(
            "failed to parse board pad in {}",
            project.board_path.display()
        )
    })?;
    pad.net = net_uuid;
    project.board.pads.insert(
        key,
        serde_json::to_value(&pad).expect("native board pad serialization must succeed"),
    );
    write_canonical_json(&project.board_path, &project.board)?;
    Ok(native_project_board_pad_report(
        if net_uuid.is_some() {
            "set_board_pad_net"
        } else {
            "clear_board_pad_net"
        },
        &project,
        pad,
    ))
}

fn parse_native_board_pad_shape(shape: &str) -> Result<PadShape> {
    match shape {
        "circle" => Ok(PadShape::Circle),
        "rect" => Ok(PadShape::Rect),
        _ => bail!("unsupported board pad shape: {shape}"),
    }
}

fn validate_native_board_pad_geometry(pad: &PlacedPad) -> Result<()> {
    match pad.aperture() {
        PadAperture::Circle { diameter_nm } if diameter_nm <= 0 => {
            bail!("board pad circular diameter must be positive");
        }
        PadAperture::Rect { width_nm, .. } if width_nm <= 0 => {
            bail!("board pad rectangular width must be positive");
        }
        PadAperture::Rect { height_nm, .. } if height_nm <= 0 => {
            bail!("board pad rectangular height must be positive");
        }
        _ => Ok(()),
    }
}

pub(crate) fn place_native_project_board_pad(
    root: &Path,
    package_uuid: Uuid,
    name: String,
    position: Point,
    layer: i32,
    shape: Option<String>,
    diameter_nm: Option<i64>,
    width_nm: Option<i64>,
    height_nm: Option<i64>,
    net_uuid: Option<Uuid>,
) -> Result<NativeProjectBoardPadMutationReportView> {
    let mut project = load_native_project(root)?;
    if let Some(net_uuid) = net_uuid
        && !project.board.nets.contains_key(&net_uuid.to_string())
    {
        bail!("board net not found in native project: {net_uuid}");
    }
    let pad_uuid = Uuid::new_v4();
    let shape = shape
        .as_deref()
        .map(parse_native_board_pad_shape)
        .transpose()?
        .unwrap_or(PadShape::Circle);
    let pad = PlacedPad {
        uuid: pad_uuid,
        package: package_uuid,
        name,
        net: net_uuid,
        position,
        layer,
        shape,
        diameter: diameter_nm.unwrap_or(0),
        width: width_nm.unwrap_or(0),
        height: height_nm.unwrap_or(0),
    };
    validate_native_board_pad_geometry(&pad)?;
    project.board.pads.insert(
        pad_uuid.to_string(),
        serde_json::to_value(&pad).expect("native board pad serialization must succeed"),
    );
    write_canonical_json(&project.board_path, &project.board)?;
    Ok(native_project_board_pad_report(
        "place_board_pad",
        &project,
        pad,
    ))
}

pub(crate) fn edit_native_project_board_pad(
    root: &Path,
    pad_uuid: Uuid,
    position: Option<Point>,
    layer: Option<i32>,
    shape: Option<String>,
    diameter_nm: Option<i64>,
    width_nm: Option<i64>,
    height_nm: Option<i64>,
) -> Result<NativeProjectBoardPadMutationReportView> {
    let mut project = load_native_project(root)?;
    let key = pad_uuid.to_string();
    let entry = project
        .board
        .pads
        .get(&key)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("board pad not found in native project: {pad_uuid}"))?;
    let mut pad: PlacedPad = serde_json::from_value(entry).with_context(|| {
        format!(
            "failed to parse board pad in {}",
            project.board_path.display()
        )
    })?;
    if let Some(position) = position {
        pad.position = position;
    }
    if let Some(layer) = layer {
        pad.layer = layer;
    }
    if let Some(shape) = shape {
        pad.shape = parse_native_board_pad_shape(&shape)?;
    }
    if let Some(diameter_nm) = diameter_nm {
        pad.diameter = diameter_nm;
    }
    if let Some(width_nm) = width_nm {
        pad.width = width_nm;
    }
    if let Some(height_nm) = height_nm {
        pad.height = height_nm;
    }
    validate_native_board_pad_geometry(&pad)?;
    project.board.pads.insert(
        key,
        serde_json::to_value(&pad).expect("native board pad serialization must succeed"),
    );
    write_canonical_json(&project.board_path, &project.board)?;
    Ok(native_project_board_pad_report(
        "edit_board_pad",
        &project,
        pad,
    ))
}

pub(crate) fn delete_native_project_board_pad(
    root: &Path,
    pad_uuid: Uuid,
) -> Result<NativeProjectBoardPadMutationReportView> {
    let mut project = load_native_project(root)?;
    let value = project
        .board
        .pads
        .remove(&pad_uuid.to_string())
        .ok_or_else(|| anyhow::anyhow!("board pad not found in native project: {pad_uuid}"))?;
    let pad: PlacedPad = serde_json::from_value(value).with_context(|| {
        format!(
            "failed to parse board pad in {}",
            project.board_path.display()
        )
    })?;
    write_canonical_json(&project.board_path, &project.board)?;
    Ok(native_project_board_pad_report(
        "delete_board_pad",
        &project,
        pad,
    ))
}

pub(crate) fn delete_native_project_board_track(
    root: &Path,
    track_uuid: Uuid,
) -> Result<NativeProjectBoardTrackMutationReportView> {
    let mut project = load_native_project(root)?;
    let value = project
        .board
        .tracks
        .remove(&track_uuid.to_string())
        .ok_or_else(|| anyhow::anyhow!("board track not found in native project: {track_uuid}"))?;
    let track: Track = serde_json::from_value(value).with_context(|| {
        format!(
            "failed to parse board track in {}",
            project.board_path.display()
        )
    })?;
    write_canonical_json(&project.board_path, &project.board)?;
    Ok(native_project_board_track_report(
        "delete_board_track",
        &project,
        track,
    ))
}

pub(crate) fn delete_native_project_board_via(
    root: &Path,
    via_uuid: Uuid,
) -> Result<NativeProjectBoardViaMutationReportView> {
    let mut project = load_native_project(root)?;
    let value = project
        .board
        .vias
        .remove(&via_uuid.to_string())
        .ok_or_else(|| anyhow::anyhow!("board via not found in native project: {via_uuid}"))?;
    let via: Via = serde_json::from_value(value).with_context(|| {
        format!(
            "failed to parse board via in {}",
            project.board_path.display()
        )
    })?;
    write_canonical_json(&project.board_path, &project.board)?;
    Ok(native_project_board_via_report(
        "delete_board_via",
        &project,
        via,
    ))
}

pub(crate) fn delete_native_project_board_zone(
    root: &Path,
    zone_uuid: Uuid,
) -> Result<NativeProjectBoardZoneMutationReportView> {
    let mut project = load_native_project(root)?;
    let value = project
        .board
        .zones
        .remove(&zone_uuid.to_string())
        .ok_or_else(|| anyhow::anyhow!("board zone not found in native project: {zone_uuid}"))?;
    let zone: Zone = serde_json::from_value(value).with_context(|| {
        format!(
            "failed to parse board zone in {}",
            project.board_path.display()
        )
    })?;
    write_canonical_json(&project.board_path, &project.board)?;
    Ok(native_project_board_zone_report(
        "delete_board_zone",
        &project,
        zone,
    ))
}

pub(crate) fn delete_native_project_board_net(
    root: &Path,
    net_uuid: Uuid,
) -> Result<NativeProjectBoardNetMutationReportView> {
    let mut project = load_native_project(root)?;
    let value = project
        .board
        .nets
        .remove(&net_uuid.to_string())
        .ok_or_else(|| anyhow::anyhow!("board net not found in native project: {net_uuid}"))?;
    let net: Net = serde_json::from_value(value).with_context(|| {
        format!(
            "failed to parse board net in {}",
            project.board_path.display()
        )
    })?;
    write_canonical_json(&project.board_path, &project.board)?;
    Ok(native_project_board_net_report(
        "delete_board_net",
        &project,
        net,
    ))
}

pub(crate) fn place_native_project_board_net_class(
    root: &Path,
    name: String,
    clearance_nm: i64,
    track_width_nm: i64,
    via_drill_nm: i64,
    via_diameter_nm: i64,
    diffpair_width_nm: i64,
    diffpair_gap_nm: i64,
) -> Result<NativeProjectBoardNetClassMutationReportView> {
    let mut project = load_native_project(root)?;
    let net_class_uuid = Uuid::new_v4();
    let net_class = NetClass {
        uuid: net_class_uuid,
        name,
        clearance: clearance_nm,
        track_width: track_width_nm,
        via_drill: via_drill_nm,
        via_diameter: via_diameter_nm,
        diffpair_width: diffpair_width_nm,
        diffpair_gap: diffpair_gap_nm,
    };
    project.board.net_classes.insert(
        net_class_uuid.to_string(),
        serde_json::to_value(&net_class)
            .expect("native board net class serialization must succeed"),
    );
    write_canonical_json(&project.board_path, &project.board)?;
    Ok(native_project_board_net_class_report(
        "place_board_net_class",
        &project,
        net_class,
    ))
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn edit_native_project_board_net_class(
    root: &Path,
    net_class_uuid: Uuid,
    name: Option<String>,
    clearance_nm: Option<i64>,
    track_width_nm: Option<i64>,
    via_drill_nm: Option<i64>,
    via_diameter_nm: Option<i64>,
    diffpair_width_nm: Option<i64>,
    diffpair_gap_nm: Option<i64>,
) -> Result<NativeProjectBoardNetClassMutationReportView> {
    let mut project = load_native_project(root)?;
    let key = net_class_uuid.to_string();
    let entry = project
        .board
        .net_classes
        .get(&key)
        .cloned()
        .ok_or_else(|| {
            anyhow::anyhow!("board net class not found in native project: {net_class_uuid}")
        })?;
    let mut net_class: NetClass = serde_json::from_value(entry).with_context(|| {
        format!(
            "failed to parse board net class in {}",
            project.board_path.display()
        )
    })?;
    if let Some(name) = name {
        net_class.name = name;
    }
    if let Some(clearance_nm) = clearance_nm {
        net_class.clearance = clearance_nm;
    }
    if let Some(track_width_nm) = track_width_nm {
        net_class.track_width = track_width_nm;
    }
    if let Some(via_drill_nm) = via_drill_nm {
        net_class.via_drill = via_drill_nm;
    }
    if let Some(via_diameter_nm) = via_diameter_nm {
        net_class.via_diameter = via_diameter_nm;
    }
    if let Some(diffpair_width_nm) = diffpair_width_nm {
        net_class.diffpair_width = diffpair_width_nm;
    }
    if let Some(diffpair_gap_nm) = diffpair_gap_nm {
        net_class.diffpair_gap = diffpair_gap_nm;
    }
    project.board.net_classes.insert(
        key,
        serde_json::to_value(&net_class)
            .expect("native board net class serialization must succeed"),
    );
    write_canonical_json(&project.board_path, &project.board)?;
    Ok(native_project_board_net_class_report(
        "edit_board_net_class",
        &project,
        net_class,
    ))
}

pub(crate) fn delete_native_project_board_net_class(
    root: &Path,
    net_class_uuid: Uuid,
) -> Result<NativeProjectBoardNetClassMutationReportView> {
    let mut project = load_native_project(root)?;
    let value = project
        .board
        .net_classes
        .remove(&net_class_uuid.to_string())
        .ok_or_else(|| {
            anyhow::anyhow!("board net class not found in native project: {net_class_uuid}")
        })?;
    let net_class: NetClass = serde_json::from_value(value).with_context(|| {
        format!(
            "failed to parse board net class in {}",
            project.board_path.display()
        )
    })?;
    write_canonical_json(&project.board_path, &project.board)?;
    Ok(native_project_board_net_class_report(
        "delete_board_net_class",
        &project,
        net_class,
    ))
}

pub(crate) fn place_native_project_board_dimension(
    root: &Path,
    from: Point,
    to: Point,
    layer: i32,
    text: Option<String>,
) -> Result<NativeProjectBoardDimensionMutationReportView> {
    let mut project = load_native_project(root)?;
    let dimension_uuid = Uuid::new_v4();
    let dimension = Dimension {
        uuid: dimension_uuid,
        from,
        to,
        layer,
        text,
    };
    project.board.dimensions.push(
        serde_json::to_value(&dimension)
            .expect("native board dimension serialization must succeed"),
    );
    write_canonical_json(&project.board_path, &project.board)?;
    Ok(NativeProjectBoardDimensionMutationReportView {
        action: "place_board_dimension".to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        dimension_uuid: dimension.uuid.to_string(),
        from_x_nm: dimension.from.x,
        from_y_nm: dimension.from.y,
        to_x_nm: dimension.to.x,
        to_y_nm: dimension.to.y,
        layer: dimension.layer,
        text: dimension.text,
    })
}

pub(crate) fn edit_native_project_board_dimension(
    root: &Path,
    dimension_uuid: Uuid,
    from_x_nm: Option<i64>,
    from_y_nm: Option<i64>,
    to_x_nm: Option<i64>,
    to_y_nm: Option<i64>,
    layer: Option<i32>,
    text: Option<String>,
    clear_text: bool,
) -> Result<NativeProjectBoardDimensionMutationReportView> {
    let mut project = load_native_project(root)?;
    let index = project
        .board
        .dimensions
        .iter()
        .position(|entry| {
            serde_json::from_value::<Dimension>(entry.clone())
                .map(|dimension| dimension.uuid == dimension_uuid)
                .unwrap_or(false)
        })
        .ok_or_else(|| {
            anyhow::anyhow!("board dimension not found in native project: {dimension_uuid}")
        })?;
    let mut dimension: Dimension = serde_json::from_value(project.board.dimensions[index].clone())
        .with_context(|| {
            format!(
                "failed to parse board dimension in {}",
                project.board_path.display()
            )
        })?;
    match (from_x_nm, from_y_nm) {
        (None, None) => {}
        (Some(x), Some(y)) => dimension.from = Point { x, y },
        _ => bail!("board dimension start requires both --from-x-nm and --from-y-nm"),
    }
    match (to_x_nm, to_y_nm) {
        (None, None) => {}
        (Some(x), Some(y)) => dimension.to = Point { x, y },
        _ => bail!("board dimension end requires both --to-x-nm and --to-y-nm"),
    }
    if let Some(layer) = layer {
        dimension.layer = layer;
    }
    if clear_text {
        dimension.text = None;
    } else if let Some(text) = text {
        dimension.text = Some(text);
    }
    project.board.dimensions[index] = serde_json::to_value(&dimension)
        .expect("native board dimension serialization must succeed");
    write_canonical_json(&project.board_path, &project.board)?;
    Ok(NativeProjectBoardDimensionMutationReportView {
        action: "edit_board_dimension".to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        dimension_uuid: dimension.uuid.to_string(),
        from_x_nm: dimension.from.x,
        from_y_nm: dimension.from.y,
        to_x_nm: dimension.to.x,
        to_y_nm: dimension.to.y,
        layer: dimension.layer,
        text: dimension.text,
    })
}

pub(crate) fn delete_native_project_board_dimension(
    root: &Path,
    dimension_uuid: Uuid,
) -> Result<NativeProjectBoardDimensionMutationReportView> {
    let mut project = load_native_project(root)?;
    let index = project
        .board
        .dimensions
        .iter()
        .position(|entry| {
            serde_json::from_value::<Dimension>(entry.clone())
                .map(|dimension| dimension.uuid == dimension_uuid)
                .unwrap_or(false)
        })
        .ok_or_else(|| {
            anyhow::anyhow!("board dimension not found in native project: {dimension_uuid}")
        })?;
    let dimension: Dimension = serde_json::from_value(project.board.dimensions.remove(index))
        .with_context(|| {
            format!(
                "failed to parse board dimension in {}",
                project.board_path.display()
            )
        })?;
    write_canonical_json(&project.board_path, &project.board)?;
    Ok(NativeProjectBoardDimensionMutationReportView {
        action: "delete_board_dimension".to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        dimension_uuid: dimension.uuid.to_string(),
        from_x_nm: dimension.from.x,
        from_y_nm: dimension.from.y,
        to_x_nm: dimension.to.x,
        to_y_nm: dimension.to.y,
        layer: dimension.layer,
        text: dimension.text,
    })
}

pub(crate) fn parse_native_polygon_vertices(vertices: &[String]) -> Result<Polygon> {
    if vertices.len() < 3 {
        bail!("polygon requires at least three --vertex entries");
    }
    let points = vertices
        .iter()
        .map(|value| {
            let (x, y) = value
                .split_once(':')
                .ok_or_else(|| anyhow::anyhow!("vertex must be x_nm:y_nm, got `{value}`"))?;
            Ok(Point {
                x: x.parse::<i64>()?,
                y: y.parse::<i64>()?,
            })
        })
        .collect::<Result<Vec<Point>>>()?;
    Ok(Polygon {
        vertices: points,
        closed: true,
    })
}

pub(crate) fn parse_native_stackup_layers(layers: &[String]) -> Result<Vec<StackupLayer>> {
    if layers.is_empty() {
        bail!("stackup requires at least one --layer entry");
    }
    layers
        .iter()
        .map(|value| {
            let parts = value.splitn(4, ':').collect::<Vec<_>>();
            if parts.len() != 4 {
                bail!("layer must be id:name:type:thickness_nm, got `{value}`");
            }
            Ok(StackupLayer {
                id: parts[0].parse::<i32>()?,
                name: parts[1].to_string(),
                layer_type: parse_stackup_layer_type(parts[2])?,
                thickness_nm: parts[3].parse::<i64>()?,
            })
        })
        .collect()
}

fn parse_stackup_layer_type(value: &str) -> Result<StackupLayerType> {
    match value {
        "Copper" | "copper" => Ok(StackupLayerType::Copper),
        "Dielectric" | "dielectric" => Ok(StackupLayerType::Dielectric),
        "SolderMask" | "soldermask" | "solder_mask" => Ok(StackupLayerType::SolderMask),
        "Silkscreen" | "silkscreen" => Ok(StackupLayerType::Silkscreen),
        "Paste" | "paste" => Ok(StackupLayerType::Paste),
        "Mechanical" | "mechanical" => Ok(StackupLayerType::Mechanical),
        _ => bail!("unknown stackup layer type `{value}`"),
    }
}

fn native_project_board_net_class_report(
    action: &str,
    project: &LoadedNativeProject,
    net_class: NetClass,
) -> NativeProjectBoardNetClassMutationReportView {
    NativeProjectBoardNetClassMutationReportView {
        action: action.to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        net_class_uuid: net_class.uuid.to_string(),
        name: net_class.name,
        clearance_nm: net_class.clearance,
        track_width_nm: net_class.track_width,
        via_drill_nm: net_class.via_drill,
        via_diameter_nm: net_class.via_diameter,
        diffpair_width_nm: net_class.diffpair_width,
        diffpair_gap_nm: net_class.diffpair_gap,
    }
}

fn native_project_board_net_report(
    action: &str,
    project: &LoadedNativeProject,
    net: Net,
) -> NativeProjectBoardNetMutationReportView {
    NativeProjectBoardNetMutationReportView {
        action: action.to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        net_uuid: net.uuid.to_string(),
        name: net.name,
        class_uuid: net.class.to_string(),
    }
}

fn native_project_board_component_report(
    action: &str,
    project: &LoadedNativeProject,
    component: PlacedPackage,
) -> NativeProjectBoardComponentMutationReportView {
    let key = component.uuid.to_string();
    NativeProjectBoardComponentMutationReportView {
        action: action.to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        component_uuid: key.clone(),
        part_uuid: component.part.to_string(),
        package_uuid: component.package.to_string(),
        reference: component.reference,
        value: component.value,
        x_nm: component.position.x,
        y_nm: component.position.y,
        rotation_deg: component.rotation,
        layer: component.layer,
        locked: component.locked,
        persisted_component_silkscreen_text_count: component_graphic_count(
            &project.board.component_silkscreen_texts,
            &key,
        ),
        persisted_component_silkscreen_line_count: component_graphic_count(
            &project.board.component_silkscreen,
            &key,
        ),
        persisted_component_silkscreen_arc_count: component_graphic_count(
            &project.board.component_silkscreen_arcs,
            &key,
        ),
        persisted_component_silkscreen_circle_count: component_graphic_count(
            &project.board.component_silkscreen_circles,
            &key,
        ),
        persisted_component_silkscreen_polygon_count: component_graphic_count(
            &project.board.component_silkscreen_polygons,
            &key,
        ),
        persisted_component_silkscreen_polyline_count: component_graphic_count(
            &project.board.component_silkscreen_polylines,
            &key,
        ),
    }
}

fn component_graphic_count<T>(
    map: &std::collections::BTreeMap<String, Vec<T>>,
    component_key: &str,
) -> usize {
    map.get(component_key).map_or(0, Vec::len)
}

fn native_project_board_track_report(
    action: &str,
    project: &LoadedNativeProject,
    track: Track,
) -> NativeProjectBoardTrackMutationReportView {
    NativeProjectBoardTrackMutationReportView {
        action: action.to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        track_uuid: track.uuid.to_string(),
        net_uuid: track.net.to_string(),
        from_x_nm: track.from.x,
        from_y_nm: track.from.y,
        to_x_nm: track.to.x,
        to_y_nm: track.to.y,
        width_nm: track.width,
        layer: track.layer,
    }
}

fn native_project_board_via_report(
    action: &str,
    project: &LoadedNativeProject,
    via: Via,
) -> NativeProjectBoardViaMutationReportView {
    NativeProjectBoardViaMutationReportView {
        action: action.to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        via_uuid: via.uuid.to_string(),
        net_uuid: via.net.to_string(),
        x_nm: via.position.x,
        y_nm: via.position.y,
        drill_nm: via.drill,
        diameter_nm: via.diameter,
        from_layer: via.from_layer,
        to_layer: via.to_layer,
    }
}

fn native_project_board_zone_report(
    action: &str,
    project: &LoadedNativeProject,
    zone: Zone,
) -> NativeProjectBoardZoneMutationReportView {
    NativeProjectBoardZoneMutationReportView {
        action: action.to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        zone_uuid: zone.uuid.to_string(),
        net_uuid: zone.net.to_string(),
        layer: zone.layer,
        priority: zone.priority,
        thermal_relief: zone.thermal_relief,
        thermal_gap_nm: zone.thermal_gap,
        thermal_spoke_width_nm: zone.thermal_spoke_width,
        vertex_count: zone.polygon.vertices.len(),
    }
}

fn native_project_board_pad_report(
    action: &str,
    project: &LoadedNativeProject,
    pad: PlacedPad,
) -> NativeProjectBoardPadMutationReportView {
    NativeProjectBoardPadMutationReportView {
        action: action.to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        pad_uuid: pad.uuid.to_string(),
        package_uuid: pad.package.to_string(),
        name: pad.name,
        net_uuid: pad.net.map(|uuid| uuid.to_string()),
        x_nm: pad.position.x,
        y_nm: pad.position.y,
        layer: pad.layer,
        shape: match pad.shape {
            PadShape::Circle => "circle".to_string(),
            PadShape::Rect => "rect".to_string(),
        },
        diameter_nm: pad.diameter,
        width_nm: pad.width,
        height_nm: pad.height,
    }
}

fn load_native_sheet(path: &Path) -> Result<NativeSheetRoot> {
    let sheet_text = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    serde_json::from_str(&sheet_text).with_context(|| format!("failed to parse {}", path.display()))
}

fn native_sheet_into_engine_sheet(native_sheet: NativeSheetRoot) -> Sheet {
    Sheet {
        uuid: native_sheet.uuid,
        name: native_sheet.name,
        frame: native_sheet.frame,
        symbols: native_sheet
            .symbols
            .into_values()
            .map(|symbol| (symbol.uuid, symbol))
            .collect(),
        wires: native_sheet
            .wires
            .into_values()
            .map(|wire| (wire.uuid, wire))
            .collect(),
        junctions: native_sheet
            .junctions
            .into_values()
            .map(|junction| (junction.uuid, junction))
            .collect(),
        labels: native_sheet
            .labels
            .into_values()
            .map(|label| (label.uuid, label))
            .collect(),
        buses: native_sheet
            .buses
            .into_values()
            .map(|bus| (bus.uuid, bus))
            .collect(),
        bus_entries: native_sheet
            .bus_entries
            .into_values()
            .map(|entry| (entry.uuid, entry))
            .collect(),
        ports: native_sheet
            .ports
            .into_values()
            .map(|port| (port.uuid, port))
            .collect(),
        noconnects: native_sheet
            .noconnects
            .into_values()
            .map(|marker| (marker.uuid, marker))
            .collect(),
        texts: native_sheet
            .texts
            .into_values()
            .map(|text| (text.uuid, text))
            .collect(),
        drawings: native_sheet
            .drawings
            .into_values()
            .map(|drawing| (drawing_uuid(&drawing), drawing))
            .collect(),
    }
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
) -> Result<(
    Uuid,
    std::path::PathBuf,
    serde_json::Value,
    Uuid,
    PlacedSymbol,
    SymbolField,
)> {
    for (sheet_uuid, relative_path) in &project.schematic.sheets {
        let parsed_sheet_uuid = Uuid::parse_str(sheet_uuid)
            .with_context(|| format!("invalid sheet UUID key `{sheet_uuid}` in schematic root"))?;
        let sheet_path = project.root.join("schematic").join(relative_path);
        let sheet_text = std::fs::read_to_string(&sheet_path)
            .with_context(|| format!("failed to read {}", sheet_path.display()))?;
        let sheet_value: serde_json::Value = serde_json::from_str(&sheet_text)
            .with_context(|| format!("failed to parse {}", sheet_path.display()))?;
        if let Some(entries) = sheet_value
            .get("symbols")
            .and_then(serde_json::Value::as_object)
        {
            for entry in entries.values() {
                let symbol: PlacedSymbol =
                    serde_json::from_value(entry.clone()).with_context(|| {
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
    let relative_path =
        project.schematic.sheets.get(&sheet_key).ok_or_else(|| {
            anyhow::anyhow!("sheet not found in native schematic root: {sheet_uuid}")
        })?;
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
) -> Result<(
    Uuid,
    std::path::PathBuf,
    serde_json::Value,
    SchematicPrimitive,
)> {
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

pub(crate) fn parse_native_field_position(
    x_nm: Option<i64>,
    y_nm: Option<i64>,
) -> Result<Option<Point>> {
    match (x_nm, y_nm) {
        (None, None) => Ok(None),
        (Some(x), Some(y)) => Ok(Some(Point { x, y })),
        _ => bail!("field position requires both --x-nm and --y-nm"),
    }
}

fn write_symbol_into_sheet(
    sheet_value: &mut serde_json::Value,
    symbol: &PlacedSymbol,
) -> Result<()> {
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

fn render_drawing_query_view(
    sheet_uuid: Uuid,
    drawing: SchematicPrimitive,
) -> Option<serde_json::Value> {
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
            let junction: Junction = serde_json::from_value(entry.clone())
                .with_context(|| format!("failed to parse junction in {}", sheet_path.display()))?;
            return Ok((parsed_sheet_uuid, sheet_path, sheet_value, junction));
        }
    }

    bail!("junction not found in native project: {junction_uuid}");
}

fn load_native_port_mutation_target(
    project: &LoadedNativeProject,
    port_uuid: Uuid,
) -> Result<(
    Uuid,
    std::path::PathBuf,
    serde_json::Value,
    HierarchicalPort,
)> {
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
            let marker: NoConnectMarker =
                serde_json::from_value(entry.clone()).with_context(|| {
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

fn csv_escape(value: &str) -> String {
    if value.contains([',', '"', '\n']) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NativeBomRow {
    reference: String,
    value: String,
    part_uuid: String,
    package_uuid: String,
    layer: i32,
    x_nm: i64,
    y_nm: i64,
    rotation_deg: i32,
    locked: bool,
}

impl NativeBomRow {
    fn diff_fields(&self, other: &Self) -> Vec<String> {
        let mut fields = Vec::new();
        if self.value != other.value {
            fields.push("value".to_string());
        }
        if self.part_uuid != other.part_uuid {
            fields.push("part_uuid".to_string());
        }
        if self.package_uuid != other.package_uuid {
            fields.push("package_uuid".to_string());
        }
        if self.layer != other.layer {
            fields.push("layer".to_string());
        }
        if self.x_nm != other.x_nm || self.y_nm != other.y_nm {
            fields.push("position".to_string());
        }
        if self.rotation_deg != other.rotation_deg {
            fields.push("rotation_deg".to_string());
        }
        if self.locked != other.locked {
            fields.push("locked".to_string());
        }
        fields
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NativePnpRow {
    reference: String,
    x_nm: i64,
    y_nm: i64,
    rotation_deg: i32,
    layer: i32,
    side: String,
    package_uuid: String,
    part_uuid: String,
    value: String,
    locked: bool,
}

impl NativePnpRow {
    fn diff_fields(&self, other: &Self) -> Vec<String> {
        let mut fields = Vec::new();
        if self.x_nm != other.x_nm || self.y_nm != other.y_nm {
            fields.push("position".to_string());
        }
        if self.rotation_deg != other.rotation_deg {
            fields.push("rotation_deg".to_string());
        }
        if self.layer != other.layer {
            fields.push("layer".to_string());
        }
        if self.side != other.side {
            fields.push("side".to_string());
        }
        if self.package_uuid != other.package_uuid {
            fields.push("package_uuid".to_string());
        }
        if self.part_uuid != other.part_uuid {
            fields.push("part_uuid".to_string());
        }
        if self.value != other.value {
            fields.push("value".to_string());
        }
        if self.locked != other.locked {
            fields.push("locked".to_string());
        }
        fields
    }
}

fn component_to_bom_row(component: PlacedPackage) -> NativeBomRow {
    NativeBomRow {
        reference: component.reference,
        value: component.value,
        part_uuid: component.part.to_string(),
        package_uuid: component.package.to_string(),
        layer: component.layer,
        x_nm: component.position.x,
        y_nm: component.position.y,
        rotation_deg: component.rotation,
        locked: component.locked,
    }
}

fn component_to_pnp_row(component: PlacedPackage) -> NativePnpRow {
    NativePnpRow {
        reference: component.reference,
        x_nm: component.position.x,
        y_nm: component.position.y,
        rotation_deg: component.rotation,
        layer: component.layer,
        side: if component.layer <= 16 {
            "top".to_string()
        } else {
            "bottom".to_string()
        },
        package_uuid: component.package.to_string(),
        part_uuid: component.part.to_string(),
        value: component.value,
        locked: component.locked,
    }
}

fn parse_bom_csv(path: &Path) -> Result<Vec<NativeBomRow>> {
    let contents = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    let mut lines = contents.lines();
    let header = lines
        .next()
        .ok_or_else(|| anyhow::anyhow!("missing BOM CSV header in {}", path.display()))?;
    if header != "reference,value,part_uuid,package_uuid,layer,x_nm,y_nm,rotation_deg,locked" {
        bail!("unexpected BOM CSV header in {}", path.display());
    }

    let mut rows = Vec::new();
    for line in lines {
        if line.is_empty() {
            continue;
        }
        let fields = parse_csv_line(line)
            .with_context(|| format!("failed to parse BOM CSV row in {}", path.display()))?;
        if fields.len() != 9 {
            bail!("unexpected BOM CSV column count in {}", path.display());
        }
        rows.push(NativeBomRow {
            reference: fields[0].clone(),
            value: fields[1].clone(),
            part_uuid: fields[2].clone(),
            package_uuid: fields[3].clone(),
            layer: fields[4].parse().context("invalid layer")?,
            x_nm: fields[5].parse().context("invalid x_nm")?,
            y_nm: fields[6].parse().context("invalid y_nm")?,
            rotation_deg: fields[7].parse().context("invalid rotation_deg")?,
            locked: fields[8].parse().context("invalid locked")?,
        });
    }
    Ok(rows)
}

fn parse_pnp_csv(path: &Path) -> Result<Vec<NativePnpRow>> {
    let contents = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    let mut lines = contents.lines();
    let header = lines
        .next()
        .ok_or_else(|| anyhow::anyhow!("missing PnP CSV header in {}", path.display()))?;
    if header != "reference,x_nm,y_nm,rotation_deg,layer,side,package_uuid,part_uuid,value,locked" {
        bail!("unexpected PnP CSV header in {}", path.display());
    }

    let mut rows = Vec::new();
    for line in lines {
        if line.is_empty() {
            continue;
        }
        let fields = parse_csv_line(line)
            .with_context(|| format!("failed to parse PnP CSV row in {}", path.display()))?;
        if fields.len() != 10 {
            bail!("unexpected PnP CSV column count in {}", path.display());
        }
        rows.push(NativePnpRow {
            reference: fields[0].clone(),
            x_nm: fields[1].parse().context("invalid x_nm")?,
            y_nm: fields[2].parse().context("invalid y_nm")?,
            rotation_deg: fields[3].parse().context("invalid rotation_deg")?,
            layer: fields[4].parse().context("invalid layer")?,
            side: fields[5].clone(),
            package_uuid: fields[6].clone(),
            part_uuid: fields[7].clone(),
            value: fields[8].clone(),
            locked: fields[9].parse().context("invalid locked")?,
        });
    }
    Ok(rows)
}

fn parse_csv_line(line: &str) -> Result<Vec<String>> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut chars = line.chars().peekable();
    let mut in_quotes = false;
    while let Some(ch) = chars.next() {
        match ch {
            '"' => {
                if in_quotes && chars.peek() == Some(&'"') {
                    current.push('"');
                    chars.next();
                } else {
                    in_quotes = !in_quotes;
                }
            }
            ',' if !in_quotes => {
                fields.push(current);
                current = String::new();
            }
            _ => current.push(ch),
        }
    }
    if in_quotes {
        bail!("unterminated quoted field");
    }
    fields.push(current);
    Ok(fields)
}

fn sanitize_export_prefix(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut last_was_sep = false;
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            last_was_sep = false;
        } else if !last_was_sep {
            out.push('-');
            last_was_sep = true;
        }
    }
    let trimmed = out.trim_matches('-').to_string();
    if trimmed.is_empty() {
        "board".to_string()
    } else {
        trimmed
    }
}

fn native_outline_to_polygon(outline: &NativeOutline) -> Polygon {
    Polygon {
        vertices: outline
            .vertices
            .iter()
            .map(|point| Point {
                x: point.x,
                y: point.y,
            })
            .collect(),
        closed: outline.closed,
    }
}
