//! Board document parsing and report accounting share one snapshot.
use super::skeleton::parse_board_skeleton;
use super::*;

pub fn import_board_file(path: &Path) -> Result<ImportReport, EngineError> {
    let (_board, report) = import_board_document(path)?;
    Ok(report)
}

pub fn import_board_document(
    path: &Path,
) -> Result<(crate::board::Board, ImportReport), EngineError> {
    import_board_document_inner(path, None).map(|(board, report, _)| (board, report))
}

pub fn import_board_document_with_import_map(
    path: &Path,
    import_map: &std::collections::BTreeMap<ImportKey, ImportMapEntry>,
) -> Result<(crate::board::Board, ImportReport), EngineError> {
    import_board_document_inner(path, Some(import_map)).map(|(board, report, _)| (board, report))
}

pub fn import_board_document_with_import_map_identities(
    path: &Path,
    import_map: &std::collections::BTreeMap<ImportKey, ImportMapEntry>,
) -> Result<
    (
        crate::board::Board,
        ImportReport,
        Vec<KiCadBoardImportIdentity>,
    ),
    EngineError,
> {
    import_board_document_inner(path, Some(import_map))
}

fn import_board_document_inner(
    path: &Path,
    import_map: Option<&std::collections::BTreeMap<ImportKey, ImportMapEntry>>,
) -> Result<
    (
        crate::board::Board,
        ImportReport,
        Vec<KiCadBoardImportIdentity>,
    ),
    EngineError,
> {
    let import_started = std::time::Instant::now();
    let read_started = std::time::Instant::now();
    let contents = std::fs::read_to_string(path)?;
    trace_kicad_import_timing(format!(
        "board document read {}ms bytes={}",
        read_started.elapsed().as_millis(),
        contents.len()
    ));
    let parse_started = std::time::Instant::now();
    let mut import_identities = Vec::new();
    let (board, board_warnings) =
        parse_board_skeleton(path, &contents, import_map, Some(&mut import_identities))?;
    trace_kicad_import_timing(format!(
        "board skeleton parse {}ms packages={} pads={} tracks={} vias={} zones={} nets={}",
        parse_started.elapsed().as_millis(),
        board.packages.len(),
        board.pads.len(),
        board.tracks.len(),
        board.vias.len(),
        board.zones.len(),
        board.nets.len()
    ));
    let mut report =
        ImportReport::new(ImportKind::KiCadBoard, path, ImportObjectCounts::default()).with_warning(
            "parsed KiCad board skeleton into canonical nets, packages, tracks, vias, zones, and stackup; full geometry and rule import is not implemented yet",
        );
    for msg in board_warnings {
        report = report.with_warning(msg);
    }

    if let Some(version) = extract_kicad_board_version(&contents) {
        report = report.with_metadata("kicad_version", version);
    }

    let metadata_started = std::time::Instant::now();
    let counts = count_top_level_form_lines_by_form(
        &contents,
        &[
            "footprint",
            "segment",
            "via",
            "zone",
            "net",
            "gr_line",
            "gr_arc",
            "dimension",
            "gr_text",
        ],
    );
    report = report
        .with_metadata(
            "footprint_count",
            counts
                .get("footprint")
                .copied()
                .unwrap_or_default()
                .to_string(),
        )
        .with_metadata(
            "segment_count",
            counts
                .get("segment")
                .copied()
                .unwrap_or_default()
                .to_string(),
        )
        .with_metadata(
            "via_count",
            counts.get("via").copied().unwrap_or_default().to_string(),
        )
        .with_metadata(
            "zone_count",
            counts.get("zone").copied().unwrap_or_default().to_string(),
        )
        .with_metadata(
            "net_count",
            counts.get("net").copied().unwrap_or_default().to_string(),
        )
        .with_metadata(
            "gr_line_count",
            counts
                .get("gr_line")
                .copied()
                .unwrap_or_default()
                .to_string(),
        )
        .with_metadata(
            "gr_arc_count",
            counts
                .get("gr_arc")
                .copied()
                .unwrap_or_default()
                .to_string(),
        )
        .with_metadata(
            "dimension_count",
            counts
                .get("dimension")
                .copied()
                .unwrap_or_default()
                .to_string(),
        )
        .with_metadata(
            "gr_text_count",
            counts
                .get("gr_text")
                .copied()
                .unwrap_or_default()
                .to_string(),
        );
    trace_kicad_import_timing(format!(
        "board report metadata {}ms",
        metadata_started.elapsed().as_millis()
    ));
    trace_kicad_import_timing(format!(
        "board document total {}ms",
        import_started.elapsed().as_millis()
    ));

    Ok((board, report, import_identities))
}

fn trace_kicad_import_timing(message: String) {
    if std::env::var_os("DATUM_TRACE_TIMING").is_some() {
        eprintln!("[datum-engine-import] {message}");
    }
}
