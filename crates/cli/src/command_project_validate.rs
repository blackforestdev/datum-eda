use anyhow::Result;
use serde::de::DeserializeOwned;
use uuid::Uuid;

use super::{
    BoardText, Dimension, Keepout, NativeBoardRoot, NativeProjectManifest,
    NativeProjectValidationIssueView, NativeProjectValidationView, NativeRulesRoot,
    NativeSchematicInstance, NativeSchematicRoot, NativeSheetRoot, PlacedPackage, PlacedPad, Rule,
    Track, Via, Zone, drawing_uuid, load_native_sheet,
};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

const SUPPORTED_NATIVE_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, serde::Deserialize)]
struct NativeSheetDefinitionRoot {
    schema_version: u32,
    uuid: Uuid,
    root_sheet: Uuid,
}

pub(crate) fn validate_native_project(root: &Path) -> Result<NativeProjectValidationView> {
    let mut issues = Vec::new();
    let mut required_files_validated = 0usize;
    let mut checked_sheet_files = 0usize;
    let mut checked_definition_files = 0usize;

    if !root.is_dir() {
        push_issue(
            &mut issues,
            "error",
            "missing_project_root",
            root.display().to_string(),
            "project root does not exist or is not a directory",
        );
        return Ok(finalize_validation_report(
            root,
            false,
            required_files_validated,
            checked_sheet_files,
            checked_definition_files,
            issues,
        ));
    }

    let manifest_path = root.join("project.json");
    let manifest: Option<NativeProjectManifest> =
        load_json_document(root, &manifest_path, "missing_required_file", &mut issues);
    if let Some(manifest) = manifest {
        required_files_validated += 1;
        validate_schema_version(
            &mut issues,
            relative_subject(root, &manifest_path),
            manifest.schema_version,
        );

        let schematic_path = root.join(&manifest.schematic);
        let board_path = root.join(&manifest.board);
        let rules_path = root.join(&manifest.rules);

        let schematic: Option<NativeSchematicRoot> =
            load_json_document(root, &schematic_path, "missing_required_file", &mut issues);
        let board: Option<NativeBoardRoot> =
            load_json_document(root, &board_path, "missing_required_file", &mut issues);
        let rules: Option<NativeRulesRoot> =
            load_json_document(root, &rules_path, "missing_required_file", &mut issues);

        if let Some(schematic) = schematic {
            required_files_validated += 1;
            validate_schema_version(
                &mut issues,
                relative_subject(root, &schematic_path),
                schematic.schema_version,
            );
            let (sheet_uuids, definition_uuids, sheets_checked, defs_checked) =
                validate_schematic_root(root, &schematic, &mut issues)?;
            checked_sheet_files += sheets_checked;
            checked_definition_files += defs_checked;
            validate_schematic_instances(
                &schematic.instances,
                &sheet_uuids,
                &definition_uuids,
                &mut issues,
            );
        }

        if let Some(board) = board {
            required_files_validated += 1;
            validate_schema_version(
                &mut issues,
                relative_subject(root, &board_path),
                board.schema_version,
            );
            validate_board_root(&board, &mut issues)?;
        }

        if let Some(rules) = rules {
            required_files_validated += 1;
            validate_schema_version(
                &mut issues,
                relative_subject(root, &rules_path),
                rules.schema_version,
            );
            validate_rules_root(&rules, &mut issues)?;
        }
    }

    Ok(finalize_validation_report(
        root,
        !issues
            .iter()
            .any(|issue| issue.code == "invalid_schema_version"),
        required_files_validated,
        checked_sheet_files,
        checked_definition_files,
        issues,
    ))
}

fn finalize_validation_report(
    root: &Path,
    schema_compatible: bool,
    required_files_validated: usize,
    checked_sheet_files: usize,
    checked_definition_files: usize,
    mut issues: Vec<NativeProjectValidationIssueView>,
) -> NativeProjectValidationView {
    issues.sort_by(|a, b| {
        a.subject
            .cmp(&b.subject)
            .then_with(|| a.code.cmp(&b.code))
            .then_with(|| a.message.cmp(&b.message))
    });
    let issue_count = issues.len();
    NativeProjectValidationView {
        action: "validate_project".to_string(),
        project_root: root.display().to_string(),
        valid: issue_count == 0,
        schema_compatible,
        required_files_expected: 4,
        required_files_validated,
        checked_sheet_files,
        checked_definition_files,
        issue_count,
        issues,
    }
}

fn validate_schematic_root(
    root: &Path,
    schematic: &NativeSchematicRoot,
    issues: &mut Vec<NativeProjectValidationIssueView>,
) -> Result<(BTreeSet<Uuid>, BTreeSet<Uuid>, usize, usize)> {
    let mut sheet_uuids = BTreeSet::new();
    let mut definition_uuids = BTreeSet::new();
    let mut seen_sheets = BTreeMap::new();
    let mut seen_definitions = BTreeMap::new();
    let mut seen_symbols = BTreeMap::new();
    let mut seen_wires = BTreeMap::new();
    let mut seen_junctions = BTreeMap::new();
    let mut seen_labels = BTreeMap::new();
    let mut seen_buses = BTreeMap::new();
    let mut seen_bus_entries = BTreeMap::new();
    let mut seen_ports = BTreeMap::new();
    let mut seen_noconnects = BTreeMap::new();
    let mut seen_texts = BTreeMap::new();
    let mut seen_drawings = BTreeMap::new();
    let mut checked_sheet_files = 0usize;
    let mut checked_definition_files = 0usize;

    for (sheet_key, relative_path) in &schematic.sheets {
        let subject = format!("schematic/schematic.json#sheets/{sheet_key}");
        let expected_uuid = parse_uuid_key(sheet_key, &subject, issues);
        let path = root.join("schematic").join(relative_path);
        if let Some(sheet) = load_sheet_document(root, &path, issues)? {
            checked_sheet_files += 1;
            validate_schema_version(issues, relative_subject(root, &path), sheet.schema_version);
            if let Some(expected_uuid) = expected_uuid {
                if sheet.uuid != expected_uuid {
                    push_issue(
                        issues,
                        "error",
                        "uuid_key_mismatch",
                        subject.clone(),
                        format!(
                            "schematic sheet key {expected_uuid} does not match file UUID {}",
                            sheet.uuid
                        ),
                    );
                }
            }
            record_uuid(
                "schematic_sheet",
                sheet.uuid,
                relative_subject(root, &path),
                &mut seen_sheets,
                issues,
            );
            sheet_uuids.insert(sheet.uuid);
            validate_sheet_contents(
                &path,
                &sheet,
                &mut seen_symbols,
                &mut seen_wires,
                &mut seen_junctions,
                &mut seen_labels,
                &mut seen_buses,
                &mut seen_bus_entries,
                &mut seen_ports,
                &mut seen_noconnects,
                &mut seen_texts,
                &mut seen_drawings,
                issues,
            );
        }
    }

    for (definition_key, relative_path) in &schematic.definitions {
        let subject = format!("schematic/schematic.json#definitions/{definition_key}");
        let expected_uuid = parse_uuid_key(definition_key, &subject, issues);
        let path = root.join("schematic").join(relative_path);
        let definition: Option<NativeSheetDefinitionRoot> =
            load_json_document(root, &path, "missing_file", issues);
        if let Some(definition) = definition {
            checked_definition_files += 1;
            validate_schema_version(
                issues,
                relative_subject(root, &path),
                definition.schema_version,
            );
            if let Some(expected_uuid) = expected_uuid {
                if definition.uuid != expected_uuid {
                    push_issue(
                        issues,
                        "error",
                        "uuid_key_mismatch",
                        subject,
                        format!(
                            "sheet definition key {expected_uuid} does not match file UUID {}",
                            definition.uuid
                        ),
                    );
                }
            }
            record_uuid(
                "sheet_definition",
                definition.uuid,
                relative_subject(root, &path),
                &mut seen_definitions,
                issues,
            );
            definition_uuids.insert(definition.uuid);
            if !sheet_uuids.contains(&definition.root_sheet) {
                push_issue(
                    issues,
                    "error",
                    "dangling_reference",
                    relative_subject(root, &path),
                    format!(
                        "sheet definition {} references missing root sheet {}",
                        definition.uuid, definition.root_sheet
                    ),
                );
            }
        }
    }

    Ok((
        sheet_uuids,
        definition_uuids,
        checked_sheet_files,
        checked_definition_files,
    ))
}

#[allow(clippy::too_many_arguments)]
fn validate_sheet_contents(
    path: &Path,
    sheet: &NativeSheetRoot,
    seen_symbols: &mut BTreeMap<Uuid, String>,
    seen_wires: &mut BTreeMap<Uuid, String>,
    seen_junctions: &mut BTreeMap<Uuid, String>,
    seen_labels: &mut BTreeMap<Uuid, String>,
    seen_buses: &mut BTreeMap<Uuid, String>,
    seen_bus_entries: &mut BTreeMap<Uuid, String>,
    seen_ports: &mut BTreeMap<Uuid, String>,
    seen_noconnects: &mut BTreeMap<Uuid, String>,
    seen_texts: &mut BTreeMap<Uuid, String>,
    seen_drawings: &mut BTreeMap<Uuid, String>,
    issues: &mut Vec<NativeProjectValidationIssueView>,
) {
    let subject_prefix = format!("{}#", path.display());
    validate_uuid_keyed_typed_map(
        &sheet.symbols,
        &format!("{subject_prefix}symbols"),
        "symbol",
        seen_symbols,
        |value| value.uuid,
        issues,
    );
    validate_uuid_keyed_typed_map(
        &sheet.wires,
        &format!("{subject_prefix}wires"),
        "wire",
        seen_wires,
        |value| value.uuid,
        issues,
    );
    validate_uuid_keyed_typed_map(
        &sheet.junctions,
        &format!("{subject_prefix}junctions"),
        "junction",
        seen_junctions,
        |value| value.uuid,
        issues,
    );
    validate_uuid_keyed_typed_map(
        &sheet.labels,
        &format!("{subject_prefix}labels"),
        "label",
        seen_labels,
        |value| value.uuid,
        issues,
    );
    validate_uuid_keyed_typed_map(
        &sheet.buses,
        &format!("{subject_prefix}buses"),
        "bus",
        seen_buses,
        |value| value.uuid,
        issues,
    );
    validate_uuid_keyed_typed_map(
        &sheet.bus_entries,
        &format!("{subject_prefix}bus_entries"),
        "bus_entry",
        seen_bus_entries,
        |value| value.uuid,
        issues,
    );
    validate_uuid_keyed_typed_map(
        &sheet.ports,
        &format!("{subject_prefix}ports"),
        "port",
        seen_ports,
        |value| value.uuid,
        issues,
    );
    validate_uuid_keyed_typed_map(
        &sheet.noconnects,
        &format!("{subject_prefix}noconnects"),
        "noconnect",
        seen_noconnects,
        |value| value.uuid,
        issues,
    );
    validate_uuid_keyed_typed_map(
        &sheet.texts,
        &format!("{subject_prefix}texts"),
        "schematic_text",
        seen_texts,
        |value| value.uuid,
        issues,
    );
    validate_uuid_keyed_typed_map(
        &sheet.drawings,
        &format!("{subject_prefix}drawings"),
        "drawing",
        seen_drawings,
        drawing_uuid,
        issues,
    );
}

fn validate_schematic_instances(
    instances: &[NativeSchematicInstance],
    sheet_uuids: &BTreeSet<Uuid>,
    definition_uuids: &BTreeSet<Uuid>,
    issues: &mut Vec<NativeProjectValidationIssueView>,
) {
    let mut seen_instances = BTreeMap::new();
    for (index, instance) in instances.iter().enumerate() {
        let subject = format!("schematic/schematic.json#instances/{index}");
        record_uuid(
            "sheet_instance",
            instance.uuid,
            subject.clone(),
            &mut seen_instances,
            issues,
        );
        if !definition_uuids.contains(&instance.definition) {
            push_issue(
                issues,
                "error",
                "dangling_reference",
                subject.clone(),
                format!(
                    "sheet instance {} references missing definition {}",
                    instance.uuid, instance.definition
                ),
            );
        }
        if let Some(parent_sheet) = instance.parent_sheet {
            if !sheet_uuids.contains(&parent_sheet) {
                push_issue(
                    issues,
                    "error",
                    "dangling_reference",
                    subject,
                    format!(
                        "sheet instance {} references missing parent sheet {}",
                        instance.uuid, parent_sheet
                    ),
                );
            }
        }
    }
}

fn validate_board_root(
    board: &NativeBoardRoot,
    issues: &mut Vec<NativeProjectValidationIssueView>,
) -> Result<()> {
    let mut seen_packages = BTreeMap::new();
    let mut seen_pads = BTreeMap::new();
    let mut seen_tracks = BTreeMap::new();
    let mut seen_vias = BTreeMap::new();
    let mut seen_zones = BTreeMap::new();
    let mut seen_nets = BTreeMap::new();
    let mut seen_net_classes = BTreeMap::new();
    let mut seen_component_pads = BTreeMap::new();
    let mut seen_keepouts = BTreeMap::new();
    let mut seen_dimensions = BTreeMap::new();
    let mut seen_texts = BTreeMap::new();

    let packages = validate_uuid_keyed_json_map(
        &board.packages,
        "board/board.json#packages",
        "board_package",
        &mut seen_packages,
        |value: &PlacedPackage| value.uuid,
        issues,
    )?;
    let package_uuids: BTreeSet<_> = packages.iter().map(|value| value.uuid).collect();

    for component_key in board.component_silkscreen.keys() {
        validate_component_attachment_key(
            component_key,
            "board/board.json#component_silkscreen",
            &package_uuids,
            issues,
        );
    }
    for component_key in board.component_silkscreen_texts.keys() {
        validate_component_attachment_key(
            component_key,
            "board/board.json#component_silkscreen_texts",
            &package_uuids,
            issues,
        );
    }
    for component_key in board.component_silkscreen_arcs.keys() {
        validate_component_attachment_key(
            component_key,
            "board/board.json#component_silkscreen_arcs",
            &package_uuids,
            issues,
        );
    }
    for component_key in board.component_silkscreen_circles.keys() {
        validate_component_attachment_key(
            component_key,
            "board/board.json#component_silkscreen_circles",
            &package_uuids,
            issues,
        );
    }
    for component_key in board.component_silkscreen_polygons.keys() {
        validate_component_attachment_key(
            component_key,
            "board/board.json#component_silkscreen_polygons",
            &package_uuids,
            issues,
        );
    }
    for component_key in board.component_silkscreen_polylines.keys() {
        validate_component_attachment_key(
            component_key,
            "board/board.json#component_silkscreen_polylines",
            &package_uuids,
            issues,
        );
    }
    for component_key in board.component_mechanical_lines.keys() {
        validate_component_attachment_key(
            component_key,
            "board/board.json#component_mechanical_lines",
            &package_uuids,
            issues,
        );
    }
    for component_key in board.component_mechanical_texts.keys() {
        validate_component_attachment_key(
            component_key,
            "board/board.json#component_mechanical_texts",
            &package_uuids,
            issues,
        );
    }
    for component_key in board.component_mechanical_polygons.keys() {
        validate_component_attachment_key(
            component_key,
            "board/board.json#component_mechanical_polygons",
            &package_uuids,
            issues,
        );
    }
    for component_key in board.component_mechanical_polylines.keys() {
        validate_component_attachment_key(
            component_key,
            "board/board.json#component_mechanical_polylines",
            &package_uuids,
            issues,
        );
    }
    for component_key in board.component_mechanical_circles.keys() {
        validate_component_attachment_key(
            component_key,
            "board/board.json#component_mechanical_circles",
            &package_uuids,
            issues,
        );
    }
    for component_key in board.component_mechanical_arcs.keys() {
        validate_component_attachment_key(
            component_key,
            "board/board.json#component_mechanical_arcs",
            &package_uuids,
            issues,
        );
    }
    for component_key in board.component_models_3d.keys() {
        validate_component_attachment_key(
            component_key,
            "board/board.json#component_models_3d",
            &package_uuids,
            issues,
        );
    }
    for (component_key, pads) in &board.component_pads {
        validate_component_attachment_key(
            component_key,
            "board/board.json#component_pads",
            &package_uuids,
            issues,
        );
        for pad in pads {
            let subject = format!(
                "board/board.json#component_pads/{component_key}/{}",
                pad.uuid
            );
            record_uuid(
                "component_pad",
                pad.uuid,
                subject,
                &mut seen_component_pads,
                issues,
            );
        }
    }

    let net_classes = validate_uuid_keyed_json_map(
        &board.net_classes,
        "board/board.json#net_classes",
        "net_class",
        &mut seen_net_classes,
        |value: &super::NetClass| value.uuid,
        issues,
    )?;
    let net_class_uuids: BTreeSet<_> = net_classes.iter().map(|value| value.uuid).collect();

    let nets = validate_uuid_keyed_json_map(
        &board.nets,
        "board/board.json#nets",
        "net",
        &mut seen_nets,
        |value: &super::Net| value.uuid,
        issues,
    )?;
    let net_uuids: BTreeSet<_> = nets.iter().map(|value| value.uuid).collect();
    for net in &nets {
        if !net_class_uuids.contains(&net.class) {
            push_issue(
                issues,
                "error",
                "dangling_reference",
                format!("board/board.json#nets/{}", net.uuid),
                format!(
                    "board net {} references missing net class {}",
                    net.uuid, net.class
                ),
            );
        }
    }

    let pads = validate_uuid_keyed_json_map(
        &board.pads,
        "board/board.json#pads",
        "pad",
        &mut seen_pads,
        |value: &PlacedPad| value.uuid,
        issues,
    )?;
    for pad in &pads {
        if !package_uuids.contains(&pad.package) {
            push_issue(
                issues,
                "error",
                "dangling_reference",
                format!("board/board.json#pads/{}", pad.uuid),
                format!(
                    "board pad {} references missing package {}",
                    pad.uuid, pad.package
                ),
            );
        }
        if let Some(net) = pad.net {
            if !net_uuids.contains(&net) {
                push_issue(
                    issues,
                    "error",
                    "dangling_reference",
                    format!("board/board.json#pads/{}", pad.uuid),
                    format!("board pad {} references missing net {}", pad.uuid, net),
                );
            }
        }
    }

    let tracks = validate_uuid_keyed_json_map(
        &board.tracks,
        "board/board.json#tracks",
        "track",
        &mut seen_tracks,
        |value: &Track| value.uuid,
        issues,
    )?;
    for track in &tracks {
        if !net_uuids.contains(&track.net) {
            push_issue(
                issues,
                "error",
                "dangling_reference",
                format!("board/board.json#tracks/{}", track.uuid),
                format!(
                    "board track {} references missing net {}",
                    track.uuid, track.net
                ),
            );
        }
    }

    let vias = validate_uuid_keyed_json_map(
        &board.vias,
        "board/board.json#vias",
        "via",
        &mut seen_vias,
        |value: &Via| value.uuid,
        issues,
    )?;
    for via in &vias {
        if !net_uuids.contains(&via.net) {
            push_issue(
                issues,
                "error",
                "dangling_reference",
                format!("board/board.json#vias/{}", via.uuid),
                format!("board via {} references missing net {}", via.uuid, via.net),
            );
        }
    }

    let zones = validate_uuid_keyed_json_map(
        &board.zones,
        "board/board.json#zones",
        "zone",
        &mut seen_zones,
        |value: &Zone| value.uuid,
        issues,
    )?;
    for zone in &zones {
        if !net_uuids.contains(&zone.net) {
            push_issue(
                issues,
                "error",
                "dangling_reference",
                format!("board/board.json#zones/{}", zone.uuid),
                format!(
                    "board zone {} references missing net {}",
                    zone.uuid, zone.net
                ),
            );
        }
    }

    validate_uuid_value_list(
        &board.keepouts,
        "board/board.json#keepouts",
        "keepout",
        &mut seen_keepouts,
        |value: &Keepout| value.uuid,
        issues,
    )?;
    validate_uuid_value_list(
        &board.dimensions,
        "board/board.json#dimensions",
        "dimension",
        &mut seen_dimensions,
        |value: &Dimension| value.uuid,
        issues,
    )?;
    validate_uuid_value_list(
        &board.texts,
        "board/board.json#texts",
        "board_text",
        &mut seen_texts,
        |value: &BoardText| value.uuid,
        issues,
    )?;
    Ok(())
}

fn validate_rules_root(
    rules: &NativeRulesRoot,
    issues: &mut Vec<NativeProjectValidationIssueView>,
) -> Result<()> {
    let mut seen_rules = BTreeMap::new();
    for (index, value) in rules.rules.iter().enumerate() {
        let subject = format!("rules/rules.json#rules/{index}");
        match serde_json::from_value::<Rule>(value.clone()) {
            Ok(rule) => record_uuid("rule", rule.uuid, subject, &mut seen_rules, issues),
            Err(err) => push_issue(
                issues,
                "error",
                "invalid_json",
                subject,
                format!("failed to parse rule: {err}"),
            ),
        }
    }
    Ok(())
}

fn validate_component_attachment_key(
    component_key: &str,
    section_subject: &str,
    package_uuids: &BTreeSet<Uuid>,
    issues: &mut Vec<NativeProjectValidationIssueView>,
) {
    let subject = format!("{section_subject}/{component_key}");
    let Some(component_uuid) = parse_uuid_key(component_key, &subject, issues) else {
        return;
    };
    if !package_uuids.contains(&component_uuid) {
        push_issue(
            issues,
            "error",
            "dangling_reference",
            subject,
            format!(
                "component attachment references missing package {}",
                component_uuid
            ),
        );
    }
}

fn validate_uuid_keyed_json_map<T, F>(
    values: &BTreeMap<String, serde_json::Value>,
    subject_prefix: &str,
    object_type: &'static str,
    seen: &mut BTreeMap<Uuid, String>,
    get_uuid: F,
    issues: &mut Vec<NativeProjectValidationIssueView>,
) -> Result<Vec<T>>
where
    T: DeserializeOwned,
    F: Fn(&T) -> Uuid,
{
    let mut parsed = Vec::new();
    for (key, value) in values {
        let subject = format!("{subject_prefix}/{key}");
        let expected_uuid = parse_uuid_key(key, &subject, issues);
        match serde_json::from_value::<T>(value.clone()) {
            Ok(item) => {
                let actual_uuid = get_uuid(&item);
                if let Some(expected_uuid) = expected_uuid {
                    if actual_uuid != expected_uuid {
                        push_issue(
                            issues,
                            "error",
                            "uuid_key_mismatch",
                            subject.clone(),
                            format!(
                                "{object_type} key {expected_uuid} does not match value UUID {}",
                                actual_uuid
                            ),
                        );
                    }
                }
                record_uuid(object_type, actual_uuid, subject, seen, issues);
                parsed.push(item);
            }
            Err(err) => push_issue(
                issues,
                "error",
                "invalid_json",
                subject,
                format!("failed to parse {object_type}: {err}"),
            ),
        }
    }
    Ok(parsed)
}

fn validate_uuid_keyed_typed_map<T, F>(
    values: &BTreeMap<String, T>,
    subject_prefix: &str,
    object_type: &'static str,
    seen: &mut BTreeMap<Uuid, String>,
    get_uuid: F,
    issues: &mut Vec<NativeProjectValidationIssueView>,
) where
    F: Fn(&T) -> Uuid,
{
    for (key, value) in values {
        let subject = format!("{subject_prefix}/{key}");
        let expected_uuid = parse_uuid_key(key, &subject, issues);
        let actual_uuid = get_uuid(value);
        if let Some(expected_uuid) = expected_uuid {
            if actual_uuid != expected_uuid {
                push_issue(
                    issues,
                    "error",
                    "uuid_key_mismatch",
                    subject.clone(),
                    format!(
                        "{object_type} key {expected_uuid} does not match value UUID {}",
                        actual_uuid
                    ),
                );
            }
        }
        record_uuid(object_type, actual_uuid, subject, seen, issues);
    }
}

fn validate_uuid_value_list<T, F>(
    values: &[serde_json::Value],
    subject_prefix: &str,
    object_type: &'static str,
    seen: &mut BTreeMap<Uuid, String>,
    get_uuid: F,
    issues: &mut Vec<NativeProjectValidationIssueView>,
) -> Result<()>
where
    T: DeserializeOwned,
    F: Fn(&T) -> Uuid,
{
    for (index, value) in values.iter().enumerate() {
        let subject = format!("{subject_prefix}/{index}");
        match serde_json::from_value::<T>(value.clone()) {
            Ok(item) => {
                record_uuid(object_type, get_uuid(&item), subject, seen, issues);
            }
            Err(err) => push_issue(
                issues,
                "error",
                "invalid_json",
                subject,
                format!("failed to parse {object_type}: {err}"),
            ),
        }
    }
    Ok(())
}

fn validate_schema_version(
    issues: &mut Vec<NativeProjectValidationIssueView>,
    subject: String,
    schema_version: u32,
) {
    if schema_version != SUPPORTED_NATIVE_SCHEMA_VERSION {
        push_issue(
            issues,
            "error",
            "invalid_schema_version",
            subject,
            format!(
                "unsupported schema_version {}; expected {}",
                schema_version, SUPPORTED_NATIVE_SCHEMA_VERSION
            ),
        );
    }
}

fn parse_uuid_key(
    key: &str,
    subject: &str,
    issues: &mut Vec<NativeProjectValidationIssueView>,
) -> Option<Uuid> {
    match Uuid::parse_str(key) {
        Ok(uuid) => Some(uuid),
        Err(err) => {
            push_issue(
                issues,
                "error",
                "invalid_uuid_key",
                subject.to_string(),
                format!("invalid UUID key `{key}`: {err}"),
            );
            None
        }
    }
}

fn record_uuid(
    object_type: &'static str,
    uuid: Uuid,
    subject: String,
    seen: &mut BTreeMap<Uuid, String>,
    issues: &mut Vec<NativeProjectValidationIssueView>,
) {
    if let Some(previous_subject) = seen.insert(uuid, subject.clone()) {
        push_issue(
            issues,
            "error",
            "duplicate_uuid_within_type",
            subject,
            format!("{object_type} UUID {uuid} already appeared at {previous_subject}"),
        );
    }
}

fn push_issue(
    issues: &mut Vec<NativeProjectValidationIssueView>,
    severity: &str,
    code: &str,
    subject: String,
    message: impl Into<String>,
) {
    issues.push(NativeProjectValidationIssueView {
        severity: severity.to_string(),
        code: code.to_string(),
        subject,
        message: message.into(),
    });
}

fn relative_subject(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .display()
        .to_string()
}

fn load_sheet_document(
    root: &Path,
    path: &Path,
    issues: &mut Vec<NativeProjectValidationIssueView>,
) -> Result<Option<NativeSheetRoot>> {
    if !path.exists() {
        push_issue(
            issues,
            "error",
            "missing_file",
            relative_subject(root, path),
            "referenced sheet file is missing",
        );
        return Ok(None);
    }
    match load_native_sheet(path) {
        Ok(sheet) => Ok(Some(sheet)),
        Err(err) => {
            push_issue(
                issues,
                "error",
                "invalid_json",
                relative_subject(root, path),
                format!("{err:#}"),
            );
            Ok(None)
        }
    }
}

fn load_json_document<T: DeserializeOwned>(
    root: &Path,
    path: &Path,
    missing_code: &str,
    issues: &mut Vec<NativeProjectValidationIssueView>,
) -> Option<T> {
    if !path.exists() {
        push_issue(
            issues,
            "error",
            missing_code,
            relative_subject(root, path),
            "required native project file is missing",
        );
        return None;
    }

    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(err) => {
            push_issue(
                issues,
                "error",
                "invalid_json",
                relative_subject(root, path),
                format!("failed to read file: {err}"),
            );
            return None;
        }
    };

    match serde_json::from_str::<T>(&text) {
        Ok(document) => Some(document),
        Err(err) => {
            push_issue(
                issues,
                "error",
                "invalid_json",
                relative_subject(root, path),
                format!("failed to parse JSON: {err}"),
            );
            None
        }
    }
}
