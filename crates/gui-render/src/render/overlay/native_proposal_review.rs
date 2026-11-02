//! The adopted inline ghost review: human diff, Tab accept, Escape dismiss.
use super::*;

pub(super) fn render(
    out: &mut Vec<Quad>,
    text: &mut Vec<TextRun>,
    scene: &BoardReviewSceneV1,
    viewport: RectPx,
    projection: &Projection,
    state: &ReviewWorkspaceState,
    plan: &mut native_selection_overlay::cue_plan::PanePlan,
) {
    let Some(review) = &state.schematic_authoring.delete_review else {
        return;
    };
    if scene.schematic_document != review.basis.document
        || scene.scene_id != review.scene.scene_id
        || scene.project_uuid != review.basis.project.to_string()
    {
        return;
    }
    if review.current(state) && proposed_visible(state) && review.response.is_some() {
        let ids = review.deleted_sources();
        for graphic in &review.scene.board_graphics {
            if graphic.object_kind == "schematic_field_tether"
                || !ids.contains(&graphic.source_object_uuid)
                || !layer_visible(state, &graphic.layer_id)
            {
                continue;
            }
            let ghost = ProposalOverlayPrimitive {
                overlay_id: format!("delete:{}", graphic.object_id),
                primitive_kind: "track_path".into(),
                proposal_action_id: review.proposal_id.to_string(),
                layer_id: Some(graphic.layer_id.clone()),
                render_role: "proposed_delete".into(),
                width_nm: graphic.width_nm,
                drill_nm: None,
                diameter_nm: None,
                path: graphic.path.clone(),
            };
            // Same proposal painter, no new authored/selection hit targets.
            push_overlay_channels(out, plan, &ghost, projection, PROPOSAL_BASE, false);
        }
        let assets = glyph_mesh_lookup::MeshIndex::new(&review.scene.glyph_mesh_assets, out);
        for owned in review.scene.board_texts.iter().filter(|t| {
            t.source_object_uuid
                .as_ref()
                .is_some_and(|id| ids.contains(id))
        }) {
            if !layer_visible(state, &owned.layer_id) {
                continue;
            }
            if let (Some(geometry), Some(assets)) = (
                review
                    .scene
                    .board_text_geometries
                    .iter()
                    .find(|g| g.object_id == owned.object_id),
                assets.as_ref(),
            ) {
                let mut world = Vec::new();
                push_board_text_geometry_world(
                    &mut world,
                    geometry,
                    assets,
                    PROPOSAL_BASE,
                    projection,
                );
                let mut screen = plan.base_output(out, false);
                for mut quad in world {
                    for point in &mut quad.points {
                        *point = projection.project_point(PointNm {
                            x: point.0.round() as i64,
                            y: point.1.round() as i64,
                        });
                    }
                    screen.push(quad);
                }
            }
        }
    }
    let width = (viewport.width - 28.).clamp(80., 520.);
    let chip = RectPx {
        x: viewport.x + (viewport.width - width) * 0.5,
        y: viewport.y + viewport.height - 88.,
        width,
        height: 74.,
    };
    // open-decisions.html .ghostchip: surface03, accent border, six-pixel radius.
    let mut chrome = Vec::new();
    push_projected_round_rect(&mut chrome, chip, design_tokens::chrome::ACCENT, 6.);
    push_projected_round_rect(
        &mut chrome,
        inset_rect(chip, 1., 1., 1., 1.),
        design_tokens::chrome::SURFACE_03,
        5.,
    );
    plan.base_output(out, false).extend(chrome);
    let Some(text_clip) = chip.intersect(viewport) else {
        return;
    };
    let columns = ((width - 20.) / 6.7).max(1.) as usize;
    let summary = truncate_text(&review.summary(), columns);
    let detail = review.refusal.clone().unwrap_or_else(|| review.names());
    let action = if review.pending.is_some() {
        "Preparing Proposal…"
    } else if review.accepted() && review.ready() && review.current(state) {
        "Tab retry apply · Esc close review"
    } else if review.ready() && review.current(state) {
        "Tab accept deletion · Esc dismiss"
    } else {
        "Deletion unavailable · Esc dismiss"
    };
    for (line, y, color) in [
        (summary, chip.y + 10., TEXT_PRIMARY),
        (truncate_text(&detail, columns), chip.y + 29., TEXT_MUTED),
        (action.into(), chip.y + 49., design_tokens::chrome::ACCENT),
    ] {
        draw_text_clipped(
            &line,
            chip.x + 10.,
            y,
            11.,
            color,
            TextFace::Mono,
            text_clip,
            text,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use datum_gui_protocol::schematic_authoring::delete_review::DeleteReview;
    use eda_engine::{
        api::native_write::schematic_delete_proposal::{
            DeleteClass, DeleteTarget, DeletedObject, NativeProposalResponse,
        },
        schematic::{Schematic, Sheet},
        substrate::ProposalStatus,
    };
    #[test]
    fn batch_delete_inline_review_uses_names_counts_and_exact_source_without_modal_or_cancel_write()
    {
        let mut state = datum_gui_protocol::load_fixture_workspace_state();
        state.selection = SelectionTarget::None;
        let project = uuid::Uuid::new_v4();
        state.scene.project_uuid = project.to_string();
        let sheet_id = uuid::Uuid::new_v4();
        let source = Sheet {
            uuid: sheet_id,
            name: "Main".into(),
            frame: None,
            symbols: Default::default(),
            wires: Default::default(),
            junctions: Default::default(),
            labels: Default::default(),
            buses: Default::default(),
            bus_entries: Default::default(),
            ports: Default::default(),
            noconnects: Default::default(),
            texts: Default::default(),
            drawings: Default::default(),
        };
        let wire = uuid::Uuid::new_v4();
        let symbol = uuid::Uuid::new_v4();
        let proposal_id = uuid::Uuid::new_v4();
        let mut scene = state.scene.clone();
        scene.schematic_document = None;
        scene
            .board_graphics
            .push(datum_gui_protocol::BoardGraphicPrimitive {
                object_id: "wire-test".into(),
                object_kind: "schematic_wire".into(),
                primitive_kind: "line".into(),
                source_object_uuid: wire.to_string(),
                layer_id: "L0".into(),
                path: vec![PointNm { x: 0, y: 0 }, PointNm { x: 2_000_000, y: 0 }],
                holes: vec![],
                width_nm: Some(120_000),
            });
        state.schematic_documents.panes.clear();
        state.ui.layout.focused = datum_gui_protocol::PaneId(1);
        state.supervision.journal.accepted_transaction_tip = None;
        state.schematic_authoring.source = Some(Schematic {
            uuid: sheet_id,
            sheets: [(sheet_id, source.clone())].into_iter().collect(),
            sheet_definitions: Default::default(),
            sheet_instances: Default::default(),
            variants: Default::default(),
            waivers: vec![],
        });
        state.schematic_authoring.delete_review = Some(DeleteReview {
            basis: datum_gui_protocol::AuthoringViewBasis {
                project,
                revision: scene.source_revision.clone(),
                pane: datum_gui_protocol::PaneId(1),
                content: datum_gui_protocol::PaneContent::Schematic,
                source: sheet_id,
                document: None,
            },
            source,
            scene: scene.clone(),
            targets: vec![
                DeleteTarget {
                    class: DeleteClass::Wire,
                    id: wire,
                },
                DeleteTarget {
                    class: DeleteClass::Symbol,
                    id: symbol,
                },
            ],
            proposal_id,
            revision: scene.source_revision.clone(),
            tip: None,
            pending: None,
            refusal: None,
            response: Some(NativeProposalResponse {
                proposal_id,
                status: ProposalStatus::Draft,
                prepared_against: scene.source_revision.clone(),
                current_model_revision: scene.source_revision.clone(),
                transaction_tip: None,
                deleted: vec![
                    DeletedObject {
                        id: wire,
                        class: "wire".into(),
                        name: String::new(),
                    },
                    DeletedObject {
                        id: symbol,
                        class: "symbol".into(),
                        name: "R1".into(),
                    },
                ],
                selected_count: Some(2),
                owned_count: Some(0),
                transaction_id: None,
            }),
        });
        let source = state.schematic_authoring.source.clone();
        let viewport = RectPx {
            x: 0.,
            y: 0.,
            width: 400.,
            height: 500.,
        };
        let camera = CameraState::fit_to_bounds(&scene.bounds);
        let projection = Projection::new(
            inset_rect(viewport, 10., 10., 10., 10.),
            &scene.bounds,
            camera,
        );
        let mut plan =
            native_selection_overlay::cue_plan::PanePlan::new(&state, viewport, camera).unwrap();
        let mut quads = vec![];
        let mut text = vec![];
        render(
            &mut quads,
            &mut text,
            &scene,
            viewport,
            &projection,
            &state,
            &mut plan,
        );
        assert!(!quads.is_empty());
        assert!(text.iter().any(|t| t.text == "R1"));
        assert!(text.iter().any(|t| t.text.contains("Tab accept deletion")));
        assert!(text.iter().any(|t| t.text.contains("2 selected")));
        assert!(!text.iter().any(|t|t.text.contains(&wire.to_string())||t.text.contains(&symbol.to_string())));
        // Exercise the actual frame route: there is no Board pane to render
        // these commands, and the camera is installed after frame construction.
        let pane = state.ui.layout.open_beside_root(
            datum_gui_protocol::PaneContent::Schematic,
            datum_gui_protocol::SplitOrientation::Vertical,
            0.5,
            true,
        );
        state.ui.layout.zoomed = Some(pane);
        state.schematic_documents.project = None;
        state
            .schematic_authoring
            .delete_review
            .as_mut()
            .unwrap()
            .basis
            .pane = pane;
        state.schematic_scene = Some(
            state
                .schematic_authoring
                .delete_review
                .as_ref()
                .unwrap()
                .preview_scene(),
        );
        let retained = RetainedScene::from_workspace(&state, 1280, 800);
        let mut frame =
            PreparedScene::from_workspace(&state, 1280, 800, camera, &retained).unwrap();
        frame.set_surface_camera(pane, camera);
        frame.append_native_selection(&state).unwrap();
        assert!(
            frame
                .text_runs
                .iter()
                .any(|t| t.text.contains("Tab accept deletion"))
        );
        assert!(
            frame
                .text_runs
                .iter()
                .any(|t| t.text.contains("2 selected"))
        );
        assert!(
            frame
                .viewport_overlay_vertices
                .iter()
                .any(|v| v.color == PROPOSAL_BASE)
        );
        let narrow = RectPx {
            x: 20.,
            y: 20.,
            width: 60.,
            height: 55.,
        };
        let mut narrow_plan =
            native_selection_overlay::cue_plan::PanePlan::new(&state, narrow, camera).unwrap();
        let mut narrow_quads = Vec::new();
        let mut narrow_text = Vec::new();
        render(
            &mut narrow_quads,
            &mut narrow_text,
            &scene,
            narrow,
            &projection,
            &state,
            &mut narrow_plan,
        );
        assert!(!narrow_quads.is_empty());
        assert!(
            narrow_quads
                .iter()
                .all(|q| q.points.iter().all(|p| narrow.contains(p.0, p.1)))
        );
        let mut wrong = scene.clone();
        wrong.scene_id.push_str("other");
        quads.clear();
        text.clear();
        render(
            &mut quads,
            &mut text,
            &wrong,
            viewport,
            &projection,
            &state,
            &mut plan,
        );
        assert!(quads.is_empty() && text.is_empty());
        state.schematic_authoring.delete_review = None;
        render(
            &mut quads,
            &mut text,
            &scene,
            viewport,
            &projection,
            &state,
            &mut plan,
        );
        assert!(quads.is_empty() && text.is_empty());
        assert_eq!(state.schematic_authoring.source, source);
    }
}
