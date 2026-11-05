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
    if review.fields.is_some() {
        return;
    } // The same review is presented in its row table.
    if scene.schematic_document != review.basis.document
        || scene.scene_id != review.scene.scene_id
        || scene.project_uuid != review.basis.project.to_string()
    {
        return;
    }
    if review.current(state) && proposed_visible(state) && review.response.is_some() {
        let (ghost_scene, ids, role) = if let Some(child) = &review.child_sheet {
            (
                &child.preview,
                std::collections::BTreeSet::from([child.summary.instance.to_string()]),
                "proposed_preview",
            )
        } else if let Some(gate) = &review.gate_placement {
            let ids = std::iter::once(gate.symbol.uuid.to_string())
                .chain(gate.symbol.pins.iter().map(|pin| pin.uuid.to_string()))
                .collect();
            (&gate.preview, ids, "proposed_preview")
        } else {
            (&review.scene, review.deleted_sources(), "proposed_delete")
        };
        for graphic in &ghost_scene.board_graphics {
            if graphic.object_kind == "schematic_field_tether"
                || !ids.contains(&graphic.source_object_uuid)
                || !layer_visible(state, &graphic.layer_id)
            {
                continue;
            }
            let ghost = ProposalOverlayPrimitive {
                overlay_id: format!("review:{}", graphic.object_id),
                primitive_kind: "track_path".into(),
                proposal_action_id: review.proposal_id.to_string(),
                layer_id: Some(graphic.layer_id.clone()),
                render_role: role.into(),
                width_nm: graphic.width_nm,
                drill_nm: None,
                diameter_nm: None,
                path: graphic.path.clone(),
            };
            // Same proposal painter, no new authored/selection hit targets.
            push_overlay_channels(out, plan, &ghost, projection, PROPOSAL_BASE, false);
        }
        let assets = glyph_mesh_lookup::MeshIndex::new(&ghost_scene.glyph_mesh_assets, out);
        for owned in ghost_scene.board_texts.iter().filter(|t| {
            t.source_object_uuid
                .as_ref()
                .is_some_and(|id| ids.contains(id))
        }) {
            if !layer_visible(state, &owned.layer_id) {
                continue;
            }
            if let (Some(geometry), Some(assets)) = (
                ghost_scene
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
        if review.child_sheet.is_some() {
            "Tab accept child sheet · Esc dismiss"
        } else if review.gate_placement.is_some() {
            "Tab accept Gate placement · Esc dismiss"
        } else if review.annotation.is_some() {
            "Tab accept annotation · Esc dismiss"
        } else if review.assignment.is_some() {
            "Tab accept assignment · Esc dismiss"
        } else if review.disposition.is_some() {
            "Tab accept disposition · Esc dismiss"
        } else {
            "Tab accept deletion · Esc dismiss"
        }
    } else {
        "Proposal unavailable · Esc dismiss"
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
#[path = "native_proposal_review_tests.rs"]
mod tests;
