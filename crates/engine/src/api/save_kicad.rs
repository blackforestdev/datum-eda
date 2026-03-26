use super::*;

mod kicad_text;
mod transaction_state;

impl Engine {

    /// M3 save entry point for the current implemented write-back slice.
    ///
    /// Current scope: imported KiCad boards can be written back byte-identically
    /// when unmodified, and can persist the current `delete_track` slice by
    /// removing deleted top-level KiCad `segment` forms from the imported
    /// source text.
    pub fn save(&self, path: &Path) -> Result<(), EngineError> {
        let design = self.design.as_ref().ok_or(EngineError::NoProjectOpen)?;
        let imported = self.imported_source.as_ref().ok_or_else(|| {
            EngineError::Operation("save requires imported design provenance".to_string())
        })?;

        if design.board.is_none() {
            return Err(EngineError::Operation(
                "save is currently implemented only for imported KiCad boards".to_string(),
            ));
        }
        if imported.kind != ImportKind::KiCadBoard {
            return Err(EngineError::Operation(format!(
                "save is currently implemented only for imported KiCad boards; current design kind is {}",
                imported.kind.as_str()
            )));
        }

        let serialized = serialize_current_kicad_board_slice(
            &imported.original_contents,
            design
                .board
                .as_ref()
                .ok_or_else(|| EngineError::Operation("save requires imported board".to_string()))?,
            &self.pool,
            &self.undo_stack,
        )?;
        std::fs::write(path, serialized.as_bytes())?;
        persist_rule_sidecar(
            path,
            &serialized,
            design
                .board
                .as_ref()
                .map(|board| board.rules.clone())
                .unwrap_or_default(),
            imported.loaded_rule_sidecar,
        )?;
        persist_part_assignment_sidecar(
            path,
            &serialized,
            design
                .board
                .as_ref()
                .map(|board| {
                    board
                        .packages
                        .values()
                        .filter(|package| package.part != uuid::Uuid::nil())
                        .map(|package| (package.uuid, package.part))
                        .collect()
                })
                .unwrap_or_default(),
            imported.loaded_part_assignment_sidecar,
        )?;
        persist_package_assignment_sidecar(
            path,
            &serialized,
            design
                .board
                .as_ref()
                .map(|board| {
                    board
                        .packages
                        .values()
                        .filter(|package| package.package != uuid::Uuid::nil())
                        .map(|package| (package.uuid, package.package))
                        .collect()
                })
                .unwrap_or_default(),
            imported.loaded_package_assignment_sidecar,
        )?;
        persist_net_class_sidecar(
            path,
            &serialized,
            design
                .board
                .as_ref()
                .map(net_class_sidecar_payload)
                .unwrap_or_default(),
            imported.loaded_net_class_sidecar,
        )?;
        Ok(())
    }

    pub fn save_to_original(&self) -> Result<std::path::PathBuf, EngineError> {
        let imported = self.imported_source.as_ref().ok_or_else(|| {
            EngineError::Operation("save requires imported design provenance".to_string())
        })?;
        self.save(&imported.source_path)?;
        Ok(imported.source_path.clone())
    }
}

fn serialize_current_kicad_board_slice(
    original_contents: &str,
    board: &Board,
    pool: &Pool,
    undo_stack: &[TransactionRecord],
) -> Result<String, EngineError> {
    let deleted_tracks = transaction_state::active_deleted_track_uuids(undo_stack);
    let deleted_vias = transaction_state::active_deleted_via_uuids(undo_stack);
    let deleted_components = transaction_state::active_deleted_component_uuids(undo_stack);
    let moved_components = transaction_state::active_moved_components(undo_stack);
    let valued_components = transaction_state::active_set_value_components(undo_stack);
    let referenced_components = transaction_state::active_set_reference_components(undo_stack);
    let assigned_components = transaction_state::active_assigned_part_components(undo_stack);
    let package_rewritten_components = transaction_state::active_package_rewritten_components(board);
    let forms = [
        ("segment", &deleted_tracks),
        ("via", &deleted_vias),
        ("footprint", &deleted_components),
    ];
    let without_removed = if deleted_tracks.is_empty()
        && deleted_vias.is_empty()
        && deleted_components.is_empty()
    {
        original_contents.to_string()
    } else {
        kicad_text::remove_kicad_top_level_forms(original_contents, &forms)?
    };
    let package_rewritten = kicad_text::rewrite_package_footprints(
        &without_removed,
        board,
        pool,
        &package_rewritten_components,
    )?;
    let moved = kicad_text::rewrite_moved_footprints(
        &package_rewritten,
        &transaction_state::filter_component_map(&moved_components, &package_rewritten_components),
    )?;
    let assigned_values =
        transaction_state::merge_component_value_overrides(&valued_components, &assigned_components);
    let valued = kicad_text::rewrite_value_footprints(
        &moved,
        &transaction_state::filter_component_map(&assigned_values, &package_rewritten_components),
    )?;
    kicad_text::rewrite_reference_footprints(
        &valued,
        &transaction_state::filter_component_map(
            &referenced_components,
            &package_rewritten_components,
        ),
    )
}
