use std::path::Path;

use crate::error::EngineError;
use crate::import::{ImportKind, ImportObjectCounts, ImportReport};
use crate::ir::geometry::Point;
use crate::schematic::PinElectricalType;

mod parser_helpers;
mod skeleton;
mod symbol_helpers;

use parser_helpers::*;
use skeleton::{parse_board_skeleton, parse_schematic_skeleton};

// KiCad importer — see specs/IMPORT_SPEC.md §3

pub fn import_board_file(path: &Path) -> Result<ImportReport, EngineError> {
    let (_board, report) = import_board_document(path)?;
    Ok(report)
}

pub fn import_board_document(
    path: &Path,
) -> Result<(crate::board::Board, ImportReport), EngineError> {
    let contents = std::fs::read_to_string(path)?;
    let board = parse_board_skeleton(path, &contents)?;
    let mut report =
        ImportReport::new(ImportKind::KiCadBoard, path, ImportObjectCounts::default()).with_warning(
            "parsed KiCad board skeleton into canonical nets, packages, tracks, vias, zones, and stackup; full geometry and rule import is not implemented yet",
        );

    if let Some(version) = extract_kicad_board_version(&contents) {
        report = report.with_metadata("kicad_version", version);
    }

    report = report
        .with_metadata(
            "footprint_count",
            count_top_level_form_lines(&contents, "footprint").to_string(),
        )
        .with_metadata(
            "segment_count",
            count_top_level_form_lines(&contents, "segment").to_string(),
        )
        .with_metadata(
            "via_count",
            count_top_level_form_lines(&contents, "via").to_string(),
        )
        .with_metadata(
            "zone_count",
            count_top_level_form_lines(&contents, "zone").to_string(),
        )
        .with_metadata(
            "net_count",
            count_top_level_form_lines(&contents, "net").to_string(),
        )
        .with_metadata(
            "gr_line_count",
            count_top_level_form_lines(&contents, "gr_line").to_string(),
        )
        .with_metadata(
            "gr_arc_count",
            count_top_level_form_lines(&contents, "gr_arc").to_string(),
        )
        .with_metadata(
            "dimension_count",
            count_top_level_form_lines(&contents, "dimension").to_string(),
        )
        .with_metadata(
            "gr_text_count",
            count_top_level_form_lines(&contents, "gr_text").to_string(),
        );

    Ok((board, report))
}

pub fn import_schematic_file(path: &Path) -> Result<ImportReport, EngineError> {
    let (_schematic, report) = import_schematic_document(path)?;
    Ok(report)
}

pub fn import_schematic_document(
    path: &Path,
) -> Result<(crate::schematic::Schematic, ImportReport), EngineError> {
    let contents = std::fs::read_to_string(path)?;
    let schematic_uuid = find_top_level_uuid(&contents).unwrap_or_else(uuid::Uuid::new_v4);
    let mut schematic = crate::schematic::Schematic {
        uuid: schematic_uuid,
        sheets: std::collections::HashMap::new(),
        sheet_definitions: std::collections::HashMap::new(),
        sheet_instances: std::collections::HashMap::new(),
        variants: std::collections::HashMap::new(),
        waivers: Vec::new(),
    };
    import_schematic_sheet_recursive(path, "Root", &mut schematic)?;
    let mut report = ImportReport::new(
        ImportKind::KiCadSchematic,
        path,
        ImportObjectCounts::default(),
    )
    .with_warning(
        "parsed KiCad schematic header and skeleton forms only; full symbol/connectivity import is not implemented yet",
    );

    if let Some(version) = extract_kicad_schematic_version(&contents) {
        report = report.with_metadata("kicad_version", version);
    }

    report = report
        .with_metadata(
            "symbol_count",
            count_top_level_form_lines(&contents, "symbol").to_string(),
        )
        .with_metadata(
            "wire_count",
            count_top_level_form_lines(&contents, "wire").to_string(),
        )
        .with_metadata(
            "junction_count",
            count_top_level_form_lines(&contents, "junction").to_string(),
        )
        .with_metadata(
            "label_count",
            count_top_level_form_lines(&contents, "label").to_string(),
        )
        .with_metadata(
            "global_label_count",
            count_top_level_form_lines(&contents, "global_label").to_string(),
        )
        .with_metadata(
            "hierarchical_label_count",
            count_top_level_form_lines(&contents, "hierarchical_label").to_string(),
        )
        .with_metadata(
            "bus_count",
            count_top_level_form_lines(&contents, "bus").to_string(),
        )
        .with_metadata(
            "sheet_count",
            count_top_level_form_lines(&contents, "sheet").to_string(),
        )
        .with_metadata(
            "no_connect_count",
            count_top_level_form_lines(&contents, "no_connect").to_string(),
        );

    Ok((schematic, report))
}

fn import_schematic_sheet_recursive(
    path: &Path,
    sheet_name: &str,
    schematic: &mut crate::schematic::Schematic,
) -> Result<uuid::Uuid, EngineError> {
    let contents = std::fs::read_to_string(path)?;
    let parsed = parse_schematic_skeleton(&contents, sheet_name)?;
    let sheet_uuid = parsed.root_sheet.uuid;
    schematic.sheets.insert(sheet_uuid, parsed.root_sheet);

    for child in parsed.child_sheets {
        let child_sheet_uuid = child
            .sheetfile
            .as_ref()
            .map(|sheetfile| {
                path.parent()
                    .unwrap_or_else(|| Path::new(""))
                    .join(sheetfile)
            })
            .filter(|candidate| candidate.exists())
            .map(|child_path| import_schematic_sheet_recursive(&child_path, &child.name, schematic))
            .transpose()?
            .unwrap_or(uuid::Uuid::nil());
        let definition_uuid = crate::ir::ids::import_uuid(
            &crate::ir::ids::namespace_kicad(),
            &format!(
                "schematic-sheet-definition/{}/{}/{}",
                path.display(),
                child.instance_uuid,
                child.name
            ),
        );
        schematic.sheet_definitions.insert(
            definition_uuid,
            crate::schematic::SheetDefinition {
                uuid: definition_uuid,
                root_sheet: child_sheet_uuid,
                name: child.name.clone(),
            },
        );
        schematic.sheet_instances.insert(
            child.instance_uuid,
            crate::schematic::SheetInstance {
                uuid: child.instance_uuid,
                definition: definition_uuid,
                parent_sheet: Some(sheet_uuid),
                position: child.position,
                name: child.name,
                ports: child.ports,
            },
        );
    }

    Ok(sheet_uuid)
}

pub fn import_project_file(path: &Path) -> Result<ImportReport, EngineError> {
    let contents = std::fs::read_to_string(path)?;
    let value: serde_json::Value = serde_json::from_str(&contents).map_err(|err| {
        EngineError::Import(format!(
            "failed to parse KiCad project JSON {}: {err}",
            path.display()
        ))
    })?;

    let mut report = ImportReport::new(
        ImportKind::KiCadProject,
        path,
        ImportObjectCounts::default(),
    )
    .with_warning(
        "parsed KiCad project metadata only; board and schematic import are not implemented yet",
    );

    if let Some(meta) = value.get("meta").and_then(|v| v.as_object()) {
        if let Some(filename) = meta.get("filename").and_then(|v| v.as_str()) {
            report = report.with_metadata("project_name", filename);
        }
        if let Some(version) = meta.get("version") {
            report = report.with_metadata("project_version", version.to_string());
        }
    }

    if !report.metadata.contains_key("project_name")
        && let Some(stem) = path.file_stem().and_then(|s| s.to_str())
    {
        report = report.with_metadata("project_name", stem);
    }

    Ok(report)
}

fn extract_kicad_schematic_version(contents: &str) -> Option<String> {
    let marker = "(version ";
    let start = contents.find(marker)? + marker.len();
    let rest = &contents[start..];
    let end = rest.find(')')?;
    Some(rest[..end].trim().to_string())
}

fn extract_kicad_board_version(contents: &str) -> Option<String> {
    let marker = "(version ";
    let start = contents.find(marker)? + marker.len();
    let rest = &contents[start..];
    let end = rest.find(')')?;
    Some(rest[..end].trim().to_string())
}

#[derive(Debug, Clone)]
pub(super) struct LibraryPinTemplate {
    pub(super) number: String,
    pub(super) name: String,
    pub(super) electrical_type: PinElectricalType,
    pub(super) position: Point,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_path(name: &str) -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("testdata/import/kicad")
            .join(name)
    }

    fn optional_doa2526_board_path() -> Option<std::path::PathBuf> {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../..")
            .join("kicad_projects/DOA2526/hardware/DOA2526/DOA2526.kicad_pcb");
        path.exists().then_some(path)
    }

    fn optional_doa2526_schematic_path() -> Option<std::path::PathBuf> {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../..")
            .join("kicad_projects/DOA2526/hardware/DOA2526/DOA2526.kicad_sch");
        path.exists().then_some(path)
    }

    #[path = "mod_tests_import_kicad_basics.rs"]
    mod import_kicad_basics;

    #[path = "mod_tests_import_kicad_doa2526.rs"]
    mod import_kicad_doa2526;

    #[path = "mod_tests_import_kicad_schematic.rs"]
    mod import_kicad_schematic;
}
