//! Complete workspace refresh projects one fresh engine-owned model. The edit
//! caller reuses that same immutable basis for selection before atomic publication.
use super::*;
use eda_engine::substrate::DesignModel;
use std::sync::Arc;

pub fn load_board_editor_workspace_with_model(
    request: &LiveReviewRequest,
) -> Result<(ReviewWorkspaceState, Arc<DesignModel>)> {
    anyhow::ensure!(
        request.board_file.is_none(),
        "Native edit requires a native Project"
    );
    let (state, model) = load_workspace(request, false, None)?;
    Ok((state, model.context("Native Project basis unavailable")?))
}

/// Project a caller's fresh engine-resolved basis. Native edit publication still
/// qualifies revision and journal tip against the canonical writer afterward.
pub fn project_board_editor_workspace(
    request: &LiveReviewRequest,
    model: Arc<DesignModel>,
) -> Result<ReviewWorkspaceState> {
    anyhow::ensure!(
        request.board_file.is_none(),
        "Native projection requires a native Project"
    );
    Ok(load_workspace(request, false, Some(model))?.0)
}

pub(super) fn load_workspace_state_impl(
    request: &LiveReviewRequest,
    include_review: bool,
) -> Result<ReviewWorkspaceState> {
    Ok(load_workspace(request, include_review, None)?.0)
}

fn load_workspace(
    request: &LiveReviewRequest,
    include_review: bool,
    resolved: Option<Arc<DesignModel>>,
) -> Result<(ReviewWorkspaceState, Option<Arc<DesignModel>>)> {
    let workspace_started = std::time::Instant::now();
    let cli = cli_prefix()?;
    let review_started = std::time::Instant::now();
    let review = if include_review && request.board_file.is_none() {
        load_live_route_review(&cli, request)?
    } else {
        empty_route_review_payload(request)
    };
    trace_protocol_timing(format!(
        "workspace review load {}ms",
        review_started.elapsed().as_millis()
    ));
    let selected_path_started = std::time::Instant::now();
    let selected_path_points = if include_review && request.board_file.is_none() {
        load_selected_candidate_path(&cli, request, review.selected_candidate.as_deref())?
    } else {
        None
    };
    trace_protocol_timing(format!(
        "workspace selected path load {}ms",
        selected_path_started.elapsed().as_millis()
    ));
    let basis_started = std::time::Instant::now();
    let model = if request.board_file.is_none() {
        Some(match resolved {
            Some(model) => model,
            None => Arc::new(ProjectResolver::new(&request.project_root).resolve()?),
        })
    } else {
        None
    };
    trace_protocol_timing(format!(
        "workspace authoritative basis {}ms",
        basis_started.elapsed().as_millis()
    ));
    let scene_started = std::time::Instant::now();
    let (scene, board_path) = if let Some(board_file) = &request.board_file {
        load_scene_from_kicad_import(board_file)?
    } else {
        project_scene(
            request,
            model.as_deref().context("Native scene basis unavailable")?,
        )?
    };
    trace_protocol_timing(format!(
        "workspace scene load {}ms",
        scene_started.elapsed().as_millis()
    ));
    let mut scene = scene;
    let review_attach_started = std::time::Instant::now();
    attach_review_primitives(&mut scene, &review, selected_path_points.as_deref());
    trace_protocol_timing(format!(
        "workspace review attach {}ms",
        review_attach_started.elapsed().as_millis()
    ));
    let mut state = ReviewWorkspaceState::new(scene, review);
    // Locate the companion schematic from the ORIGINAL KiCad source when this
    // request was materialized into a native project (the materialized
    // `board/board.json` has no sibling `.kicad_sch`); otherwise derive it from
    // the loaded board path (the direct `--board <file>.kicad_pcb` path).
    let schematic_companion_base = request
        .kicad_board_source
        .as_deref()
        .unwrap_or(board_path.as_path());
    state.schematic_scene = if request.board_file.is_none() {
        native_schematic_scene::project_native_schematic_scene(
            model
                .as_deref()
                .context("Native schematic basis unavailable")?,
            &request.project_root,
        )?
        .or_else(|| load_sibling_schematic_scene(schematic_companion_base))
    } else {
        load_sibling_schematic_scene(schematic_companion_base)
    };
    state.production = if model.as_deref().is_some_and(production_is_empty) {
        production_payloads_to_production_status(
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
        )
    } else {
        load_production_status(&cli, request)?
    };
    state.library = model
        .as_ref()
        .map(|model| crate::library_browser::LibraryBrowser::capture(model.clone()))
        .unwrap_or_default();
    state.schematic_authoring = model
        .as_deref()
        .map(crate::schematic_authoring::SchematicAuthoring::from_model)
        .transpose()?
        .unwrap_or_default();
    state.source_shards = model
        .as_deref()
        .map(source_shard_status::project_source_shard_status)
        .unwrap_or_default();
    state.checks = if model
        .as_deref()
        .is_some_and(|model| model.check_runs.is_empty())
    {
        CheckRunReviewState::default()
    } else {
        check_runs::loading::load(&cli, request)?
    };
    state.supervision = load_gui_supervision_snapshot(
        request,
        &state.scene,
        &state.production,
        &state.source_shards,
        &state.checks,
        model.as_deref(),
    )?;
    state
        .ui
        .console_journal
        .begin_session(state.supervision.journal.applied_transaction_count);
    state.backing = Some(WorkspaceBacking {
        request: request.clone(),
        board_path,
    });
    trace_protocol_timing(format!(
        "workspace total {}ms",
        workspace_started.elapsed().as_millis()
    ));
    Ok((state, model))
}

/// Project the board project's companion schematic into a review scene, if
/// one exists next to the loaded board file (`<stem>.kicad_sch`). This is
/// carried alongside the board `scene` so pane B can draw real schematic
/// geometry; it reuses the existing schematic projector rather than authoring
/// new geometry. Robust by design: a missing or unparsable schematic yields
/// `None` and never disturbs the board load.
fn load_sibling_schematic_scene(board_path: &Path) -> Option<BoardReviewSceneV1> {
    let sibling = board_path.with_extension("kicad_sch");
    if !sibling.is_file() {
        return None;
    }
    match schematic_scene_import::load_scene_from_kicad_schematic_import(&sibling) {
        Ok((scene, _)) => Some(scene),
        Err(error) => {
            eprintln!(
                "datum-gui warning: skipped companion schematic {}: {error:#}",
                sibling.display()
            );
            None
        }
    }
}

pub(super) fn load_gui_supervision_snapshot(
    request: &LiveReviewRequest,
    scene: &BoardReviewSceneV1,
    production: &ProductionStatus,
    source_shards: &SourceShardStatusSummary,
    checks: &CheckRunReviewState,
    model: Option<&DesignModel>,
) -> Result<GuiSupervisionSnapshot> {
    let mut snapshot = GuiSupervisionSnapshot {
        contract: GUI_SUPERVISION_SNAPSHOT_CONTRACT.to_string(),
        project_root: request.project_root.display().to_string(),
        project_uuid: scene.project_uuid.clone(),
        project_name: scene.project_name.clone(),
        model_revision: scene.source_revision.clone(),
        scene_kind: scene.kind.clone(),
        read_only: true,
        journal: GuiJournalSupervision::default(),
        source_shards: source_shards.clone(),
        scene: GuiSceneSupervision {
            component_count: scene.components.len(),
            pad_count: scene.pads.len(),
            track_count: scene.tracks.len(),
            via_count: scene.vias.len(),
            zone_count: scene.zones.len(),
            board_text_count: scene.board_texts.len(),
            board_graphic_count: scene.board_graphics.len(),
            layer_count: scene.layers.len(),
        },
        checks: GuiCheckSupervision {
            check_run_id: checks.check_run_id.clone(),
            model_revision: checks.model_revision.clone(),
            profile_id: checks.profile_id.clone(),
            status: checks.status.clone(),
            finding_count: checks.finding_count,
            proposal_ref_count: checks.proposal_refs.len(),
        },
        data: GuiDataSupervision {
            output_job_count: production.output_job_count,
            artifact_count: production.artifact_count,
            artifact_run_count: production.artifact_run_count,
            proposal_count: production.proposal_count,
            manufacturing_plan_count: production.manufacturing_plan_count,
            panel_projection_count: production.panel_projection_count,
            latest_status: production.latest_status.clone(),
        },
    };
    if let Some(model) = model {
        snapshot.project_uuid = model.project.project_id.to_string();
        snapshot.project_name = model.project.name.clone();
        snapshot.model_revision = model.model_revision.0.clone();
        snapshot.journal.applied_transaction_count = model.journal_cursor.applied_transaction_count;
        snapshot.journal.accepted_transaction_tip = model
            .journal_cursor
            .applied_transaction_count
            .checked_sub(1)
            .and_then(|index| model.journal.get(index))
            .map(|transaction| transaction.transaction_id.to_string());
        let applied = model.journal_cursor.applied_transaction_count;
        let projection_start = applied.saturating_sub(CONSOLE_JOURNAL_PROJECTION_CAPACITY);
        snapshot.journal.projection_omitted_count = projection_start;
        snapshot.journal.projection = model.journal[projection_start..applied]
            .iter()
            .enumerate()
            .map(|(offset, transaction)| ConsoleJournalProjectionRecord {
                journal_ordinal: projection_start + offset + 1,
                transaction_id: transaction.transaction_id.to_string(),
                transaction_kind: format!("{:?}", transaction.transaction_kind).to_lowercase(),
                commit_source: format!("{:?}", transaction.provenance.source).to_lowercase(),
                reason: transaction.provenance.reason.clone(),
                operation_count: transaction.operations.len(),
                created_count: transaction.diff.created.len(),
                modified_count: transaction.diff.modified.len(),
                deleted_count: transaction.diff.deleted.len(),
            })
            .collect();
    }
    Ok(snapshot)
}

fn project_scene(
    request: &LiveReviewRequest,
    model: &DesignModel,
) -> Result<(BoardReviewSceneV1, PathBuf)> {
    let root = &request.project_root;
    let board_shard = model
        .source_shards
        .iter()
        .find(|shard| shard.kind == SourceShardKind::BoardRoot)
        .context("resolved native project model is missing board root shard")?;
    let board_path = board_shard.path.clone();
    let mut board_value = model
        .materialized_source_shard_value(SourceShardKind::BoardRoot)
        .context("failed to materialize resolved board root shard")?;

    let board_uuid = board_value
        .get("uuid")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let inspect = ProjectInspectPayload {
        project_root: root.display().to_string(),
        project_name: model.project.name.clone(),
        project_uuid: model.project.project_id.to_string(),
        board_uuid: board_uuid.clone(),
        board_path: board_path.display().to_string(),
    };

    // --- Outline ---
    let outline = extract_outline(&board_value)?;

    // --- Components (packages) ---
    let components = extract_components(&board_value)?;

    let exact_projection =
        eda_engine::substrate::component_pad_source::component_pad_projection(model, &board_value)?;
    let native_processes =
        placed_pad_projection::native_processes(&board_value, &exact_projection)?;
    board_value = placed_pad_projection::project(model, board_value)?;
    // --- Pads ---
    let pads = extract_pads(&board_value)?;
    let pad_expansion_setup = extract_pad_expansion_setup(&board_value)?;

    // --- Tracks ---
    let tracks = extract_tracks(&board_value)?;

    // --- Vias ---
    let vias = extract_vias(&board_value)?;

    // --- Zones ---
    let zones = extract_zones(&board_value)?;
    let (native_board_texts, native_board_text_geometries, glyph_mesh_assets) =
        extract_native_board_texts(&board_value)?;

    // --- Component graphics (silkscreen + mechanical) ---
    let (component_graphics, component_texts) =
        extract_component_graphics(&board_value, &components)?;

    // Native-project path: resolve Edge.Cuts from the persisted stackup JSON
    // when present, else fall back to the KiCad 7 canonical id 44. A better
    // resolution (via typed engine Board access) would be a follow-on.
    let edge_cuts_layer_key = board_value
        .get("stackup")
        .and_then(|s| s.get("layers"))
        .and_then(|arr| arr.as_array())
        .and_then(|layers| {
            layers.iter().find_map(|l| {
                let name = l.get("name").and_then(|n| n.as_str())?;
                if name != "Edge.Cuts" {
                    return None;
                }
                let id = l.get("id").and_then(|v| v.as_i64())? as i32;
                Some(layer_id(id))
            })
        })
        .unwrap_or_else(|| layer_id(44));

    // Native-project path does not currently persist the original
    // per-contributor Edge.Cuts identities from import. Derive board-scoped
    // authored primitives from the persisted assembled outline so native
    // projects participate in the same Edge.Cuts authored-layer lane as
    // imported boards, while keeping scene.outline as the board-boundary view.
    let mut board_graphics =
        outline_board_graphics_from_outline(&outline, &inspect.board_uuid, &edge_cuts_layer_key);
    board_graphics.extend(native_board_graphics::project(&board_value)?);
    let unrouted_primitives: Vec<UnroutedPrimitive> = Vec::new();
    let net_display = net_display_from_native_board_value(&board_value)?;

    let mut scene = build_board_review_scene(
        &inspect,
        outline,
        components,
        component_graphics,
        component_texts,
        pad_expansion_setup,
        pads,
        tracks,
        vias,
        zones,
        board_graphics,
        native_board_texts,
        native_board_text_geometries,
        glyph_mesh_assets,
        unrouted_primitives,
        net_display,
        edge_cuts_layer_key,
    );
    placed_pad_projection::append_exact(&mut scene, &exact_projection)?;
    placed_pad_projection::attach_native_process(&mut scene, &native_processes)?;
    if let Some(layers) = scene_layers_from_native_stackup_value(&board_value) {
        scene.layers = layers;
    }
    scene.source_revision = model.model_revision.0.clone();
    for zone in &mut scene.zones {
        zone.fill_projection = ZoneFillProjection::capture(model, zone.zone_uuid.parse()?);
    }
    Ok((scene, board_path))
}

// These collections are the canonical sources of the CLI projections. Empty
// means absent records at this revision, not a retained or fabricated status.
fn production_is_empty(model: &DesignModel) -> bool {
    model.output_jobs.is_empty()
        && model.output_job_runs.is_empty()
        && model.manufacturing_plans.is_empty()
        && model.panel_projections.is_empty()
        && model.artifact_metadata.is_empty()
        && model.artifact_runs.is_empty()
        && model.proposals.is_empty()
}

#[cfg(test)]
pub(super) fn load_scene_from_engine(
    request: &LiveReviewRequest,
) -> Result<(BoardReviewSceneV1, PathBuf)> {
    let root = &request.project_root;
    let model = ProjectResolver::new(root).resolve()?;
    project_scene(request, &model)
}

#[cfg(test)]
#[path = "workspace_loading_tests.rs"]
mod tests;
