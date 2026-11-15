//! Product genesis starts with an authorable sheet before staged publication.
//! Bare scaffold helpers retain their existing empty-container semantics.
use super::*;

pub(super) fn complete(
    root: &Path,
    mut report: GenesisReport,
) -> Result<GenesisReport, EngineError> {
    let index_path = root.join("schematic/schematic.json");
    let mut index: GenesisSchematicRoot =
        serde_json::from_slice(&std::fs::read(&index_path).map_err(EngineError::Io)?)?;
    let sheet = Uuid::new_v5(&report.schematic_uuid, b"initial-root-sheet");
    let relative = format!("sheets/{sheet}.json");
    let path = root.join("schematic").join(&relative);
    // Same native sheet shape as datum.schematic.create_sheet. This is initial
    // content, not an undoable edit or a migration of an existing project.
    let payload = serde_json::json!({
        "schema_version": 1, "uuid": sheet, "name": "Main", "frame": null,
        "symbols": {}, "wires": {}, "junctions": {}, "labels": {}, "buses": {},
        "bus_entries": {}, "ports": {}, "noconnects": {}, "texts": {}, "drawings": {}
    });
    write_genesis_shard(&path, &payload)?;
    index.sheets.insert(sheet.to_string(), relative);
    write_genesis_shard(&index_path, &index)?;
    ProjectResolver::new(root).resolve()?;
    report.files_written.push(path);
    Ok(report)
}
