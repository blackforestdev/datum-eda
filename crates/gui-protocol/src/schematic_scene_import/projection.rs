//! Shared composition configured with an engine-resolved source Sheet.
use super::*;
/// Explicit source-sheet configuration; composition remains shared with import.
pub(crate) fn project_schematic_sheet_source(
    schematic: &Schematic,
    root_sheet: &Sheet,
    schematic_file: &Path,
) -> Result<BoardReviewSceneV1> {
    let project_name = schematic_file
        .file_stem()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty())
        .unwrap_or("schematic")
        .to_string();
    let project_uuid = schematic.uuid.to_string();
    let sheet_uuid = root_sheet.uuid.to_string();
    let inspect = ProjectInspectPayload {
        project_root: schematic_file
            .parent()
            .unwrap_or(Path::new("."))
            .display()
            .to_string(),
        project_name: project_name.clone(),
        project_uuid: project_uuid.clone(),
        board_uuid: sheet_uuid.clone(),
        board_path: schematic_file.display().to_string(),
    };

    let mut graphics = Vec::new();
    let mut points = Vec::new();
    let mut text = SchematicTextSink::default();
    push_root_sheet_graphics(root_sheet, schematic, &mut graphics, &mut points, &mut text);
    tag_schematic_hit_kinds(root_sheet, &mut graphics);
    let (board_texts, board_text_geometries, glyph_mesh_assets) = text.into_parts();
    // P2.2f: the schematic pane has NO sheet border (the prototype draws none), so
    // the projection no longer emits the gold `Edge.Cuts` padded-bounds frame. The
    // padded envelope still drives the fit-to-bounds camera, so we compute it and
    // set it as `scene.bounds` directly, but pass an EMPTY outline to the builder
    // so no frame primitive renders. A proper title-block frame is future work.
    let envelope = schematic_bounds(&points);
    let mut scene = build_board_review_scene(
        &inspect,
        OutlinePayload {
            vertices: Vec::new(),
            closed: false,
        },
        Vec::new(),
        Vec::new(),
        Vec::new(),
        ScenePadExpansionSetup::default(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        graphics,
        board_texts,
        board_text_geometries,
        glyph_mesh_assets,
        Vec::new(),
        Vec::new(),
        String::new(),
    );
    scene.bounds = envelope;
    scene.kind = "schematic_review_scene".to_string();
    scene.scene_id = format!("schematic-review-scene:{sheet_uuid}");
    scene.board_name = root_sheet.name.clone();
    scene.source_revision = format!("schematic:{project_uuid}:sheet:{sheet_uuid}");
    scene.layers = schematic_scene_layers();
    Ok(scene)
}
