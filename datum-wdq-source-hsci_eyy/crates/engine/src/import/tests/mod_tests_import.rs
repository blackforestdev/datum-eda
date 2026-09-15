use std::path::{Path, PathBuf};

use super::*;
use crate::ir::serialization::to_json_deterministic;

#[test]
fn import_kind_has_stable_string_name() {
    assert_eq!(ImportKind::EagleLibrary.as_str(), "eagle_library");
    assert_eq!(ImportKind::KiCadBoard.as_str(), "kicad_board");
    assert_eq!(ImportKind::KiCadSchematic.as_str(), "kicad_schematic");
    assert_eq!(ImportKind::KiCadProject.as_str(), "kicad_project");
    assert_eq!(ImportKind::EagleBoard.as_str(), "eagle_board");
    assert_eq!(ImportKind::EagleSchematic.as_str(), "eagle_schematic");
}

#[test]
fn import_object_counts_empty_detection_is_exact() {
    assert!(ImportObjectCounts::default().is_empty());

    let counts = ImportObjectCounts {
        parts: 1,
        ..ImportObjectCounts::default()
    };
    assert!(!counts.is_empty());
}

#[test]
fn import_report_serializes_deterministically() {
    let report = ImportReport::new(
        ImportKind::EagleLibrary,
        PathBuf::from("fixtures/demo.lbr"),
        ImportObjectCounts {
            units: 1,
            symbols: 1,
            entities: 1,
            padstacks: 2,
            packages: 1,
            parts: 1,
        },
    )
    .with_metadata("zeta", "last")
    .with_metadata("alpha", "first")
    .with_warning("technology variants ignored in M0");

    let json = to_json_deterministic(&report).expect("report should serialize");
    assert_eq!(
        json,
        r#"{"counts":{"entities":1,"packages":1,"padstacks":2,"parts":1,"symbols":1,"units":1},"kind":"EagleLibrary","metadata":{"alpha":"first","zeta":"last"},"source":"fixtures/demo.lbr","warnings":["technology variants ignored in M0"]}"#
    );
}

#[test]
fn import_report_round_trips_through_serde() {
    let original = ImportReport::new(
        ImportKind::EagleLibrary,
        PathBuf::from("fixtures/demo.lbr"),
        ImportObjectCounts {
            units: 1,
            symbols: 1,
            entities: 1,
            padstacks: 1,
            packages: 1,
            parts: 1,
        },
    )
    .with_metadata("library_name", "demo")
    .with_warning("warning");

    let json = serde_json::to_string(&original).expect("report should serialize");
    let restored: ImportReport = serde_json::from_str(&json).expect("report should deserialize");
    assert_eq!(restored, original);
}

#[test]
fn detects_import_kinds_from_file_extension() {
    assert_eq!(
        detect_import_kind(Path::new("board.kicad_pcb")),
        Some(ImportKind::KiCadBoard)
    );
    assert_eq!(
        detect_import_kind(Path::new("sheet.kicad_sch")),
        Some(ImportKind::KiCadSchematic)
    );
    assert_eq!(
        detect_import_kind(Path::new("project.kicad_pro")),
        Some(ImportKind::KiCadProject)
    );
    assert_eq!(
        detect_import_kind(Path::new("legacy.brd")),
        Some(ImportKind::EagleBoard)
    );
    assert_eq!(
        detect_import_kind(Path::new("legacy.sch")),
        Some(ImportKind::EagleSchematic)
    );
    assert_eq!(
        detect_import_kind(Path::new("parts.lbr")),
        Some(ImportKind::EagleLibrary)
    );
    assert_eq!(detect_import_kind(Path::new("unknown.txt")), None);
}
