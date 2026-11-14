//! The adopted inline ghost review: human diff, Tab accept, Escape dismiss.
use super::*;

/// A fit refusal uses the existing foreground channel above Console, below menus.
pub(crate) struct ReviewForeground {
    pub(crate) quads: Vec<Quad>,
    pub(crate) text: Vec<TextRun>,
}
impl crate::Renderer {
    pub fn board_field_review_layout(
        evidence: &datum_gui_protocol::board_transfer::BoardFieldRevisionEvidence,
        refusal: Option<&str>,
        width: f32,
        height: f32,
        clearance: f32,
    ) -> Result<datum_gui_protocol::schematic_authoring::review_layout::ProposalReviewLayout, String>
    {
        let measure_width = |text| {
            crate::measured_text_run_width_px(text, 11., TextFace::Mono)
                .map_err(|error| error.to_string())
        };
        let apply_width = measure_width("Apply changes [Tab]")?
            .max(measure_width("Preparing Proposal…")?)
            .max(measure_width("Apply unavailable")?);
        let label_widths = (apply_width, measure_width("Dismiss [Esc]")?);
        evidence.layout(
            refusal,
            width,
            height,
            clearance,
            label_widths,
            |text, width| {
                crate::text_metrics::measured_text_run_height_px(text, width, 11., TextFace::Mono)
                    .map_err(|error| error.to_string())
            },
        )
    }
}

// Existing scene/projection and output sinks remain explicit at this review boundary.
#[allow(clippy::too_many_arguments)]
pub(super) fn render(
    out: &mut Vec<Quad>,
    text: &mut Vec<TextRun>,
    scene: &BoardReviewSceneV1,
    viewport: RectPx,
    projection: &Projection,
    state: &ReviewWorkspaceState,
    console_clearance: f32,
    hits: &mut Vec<HitRegion>,
    plan: &mut native_selection_overlay::cue_plan::PanePlan,
) -> Option<ReviewForeground> {
    let Some(review) = &state.schematic_authoring.delete_review else {
        return None;
    };
    if review.fields.is_some() {
        return None;
    } // The same review is presented in its row table.
    if scene.schematic_document != review.basis.document
        || scene.scene_id != review.scene.scene_id
        || scene.project_uuid != review.basis.project.to_string()
    {
        return None;
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
    let interface_layout = review.interface_binding.as_ref().map(|binding| {
        datum_gui_protocol::schematic_authoring::interface_binding::review_layout(
            binding,
            review.refusal.as_deref(),
            viewport.width,
            viewport.height,
        )
    });
    let board_layout = review.board_field_revision.as_ref().map(|fields| {
        crate::Renderer::board_field_review_layout(
            fields,
            review.refusal.as_deref(),
            viewport.width,
            viewport.height,
            console_clearance,
        )
    });
    match board_layout.as_ref() {
        Some(Err(_)) => {
            return Some(fit_refusal(
                viewport,
                "Text measurement unavailable; reopen the review",
                hits,
            ));
        }
        Some(Ok(layout)) if !layout.fits => {
            return Some(fit_refusal(
                viewport,
                "Enlarge the pane to review every old and new Reference and Value",
                hits,
            ));
        }
        _ => {}
    }
    let board_layout = board_layout.and_then(Result::ok);
    let measured = board_layout.or(interface_layout);
    let fits = measured.as_ref().is_none_or(|layout| layout.fits);
    let height = measured
        .as_ref()
        .filter(|_| fits)
        .map_or(74., |layout| layout.height);
    let control_widths = measured.as_ref().and_then(|layout| layout.control_widths);
    let detail_lines = measured
        .filter(|_| fits)
        .map(|layout| (layout.lines, layout.row_heights));
    let clearance = if review.board_field_revision.is_some() {
        console_clearance
    } else {
        0.
    };
    let chip = RectPx {
        x: viewport.x + (viewport.width - width) * 0.5,
        y: viewport.y + viewport.height - height - 14. - clearance,
        width,
        height,
    };
    // Adopted schematic-authoring S-P inline review uses shared Proposal chrome.
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
        return None;
    };
    let columns = ((width - 20.) / 6.7).max(1.) as usize;
    let summary = truncate_text(&review.summary(), columns);
    let detail = if !fits {
        if review.board_field_revision.is_some() {
            "Enlarge the pane to review every old and new Reference and Value".into()
        } else {
            "Enlarge the parent pane to review every interface identity".into()
        }
    } else {
        review.refusal.clone().unwrap_or_else(|| review.names())
    };
    let action = if !fits {
        "Acceptance unavailable · Esc dismiss"
    } else if review.pending.is_some() {
        "Preparing Proposal…"
    } else if review.accepted() && review.ready() && review.current(state) {
        "Tab retry apply · Esc close review"
    } else if review.ready() && review.current(state) {
        if review.board_field_revision.is_some() {
            datum_gui_protocol::board_transfer::FIELD_ACCEPT_HINT
        } else if review.interface_binding.is_some() {
            datum_gui_protocol::schematic_authoring::interface_binding::INTERFACE_ACCEPT_HINT
        } else if review.child_sheet.is_some() {
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
    let mut lines = vec![(summary, chip.y + 10., TEXT_PRIMARY)];
    let mut measured_rows = Vec::new();
    if let Some((details, row_heights)) = detail_lines {
        let mut top = chip.y + 29.;
        for (line, row_height) in details.into_iter().zip(row_heights) {
            if review.board_field_revision.is_some() {
                measured_rows.push((lines.len(), row_height));
            }
            lines.push((line, top, TEXT_MUTED));
            top += row_height;
        }
        if review.board_field_revision.is_none() {
            lines.push((
                action.into(),
                chip.y + height - 20.,
                design_tokens::chrome::ACCENT,
            ));
        }
    } else {
        lines.push((truncate_text(&detail, columns), chip.y + 29., TEXT_MUTED));
        if review.board_field_revision.is_none() {
            lines.push((action.into(), chip.y + 49., design_tokens::chrome::ACCENT));
        }
    }
    if review.board_field_revision.is_some() {
        use datum_gui_protocol::schematic_authoring::field_table::FieldTableControl as C;
        let available = fits && review.ready() && review.current(state) && review.pending.is_none();
        let labels = [
            (
                if available {
                    "Apply changes [Tab]"
                } else if review.pending.is_some() {
                    "Preparing Proposal…"
                } else {
                    "Apply unavailable"
                },
                C::Accept,
                available,
            ),
            ("Dismiss [Esc]", C::Cancel, true),
        ];
        let (apply_width, dismiss_width) =
            control_widths.expect("complete board review measured its controls");
        for (index, (label, control, enabled)) in labels.into_iter().enumerate() {
            let rect = RectPx {
                x: chip.x + 10. + if index == 0 { 0. } else { apply_width + 8. },
                y: chip.y + height - 25.,
                width: if index == 0 {
                    apply_width
                } else {
                    dismiss_width
                },
                height: 20.,
            };
            let Some(rect) = rect.intersect(viewport) else {
                continue;
            };
            let mut chrome = Vec::new();
            push_projected_round_rect(&mut chrome, rect, design_tokens::chrome::SURFACE_01, 3.);
            plan.base_output(out, false).extend(chrome);
            draw_text_clipped(
                label,
                rect.x + 3.,
                rect.y + 3.,
                11.,
                if enabled {
                    design_tokens::chrome::ACCENT
                } else {
                    TEXT_MUTED
                },
                TextFace::Mono,
                rect,
                text,
            );
            if enabled {
                hits.push(HitRegion {
                    rect,
                    target: HitTarget::SchematicFieldTable(control),
                });
            }
        }
    }
    for (index, (line, y, color)) in lines.into_iter().enumerate() {
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
        if let Some((_, row_height)) = measured_rows.iter().find(|(row, _)| *row == index) {
            text.last_mut().unwrap().layout_size =
                Some(((width - 20.).floor().max(1.), *row_height));
        }
    }
    None
}

fn fit_refusal(viewport: RectPx, reason: &str, hits: &mut Vec<HitRegion>) -> ReviewForeground {
    use datum_gui_protocol::schematic_authoring::field_table::FieldTableControl as C;
    let width = (viewport.width - 28.).clamp(80., 520.).min(viewport.width);
    let content_width = (width - 20.).floor().max(1.);
    let reason_height = crate::text_metrics::measured_text_run_height_px(
        reason,
        content_width,
        11.,
        TextFace::Mono,
    )
    .unwrap_or(28.)
    .ceil();
    let height = (74. + reason_height).min(viewport.height.max(0.));
    let card = RectPx {
        x: viewport.x + (viewport.width - width) * 0.5,
        y: viewport.y + ((viewport.height - height) * 0.5).min(14.).max(0.),
        width,
        height,
    };
    let mut foreground = ReviewForeground {
        quads: Vec::new(),
        text: Vec::new(),
    };
    push_projected_round_rect(
        &mut foreground.quads,
        card,
        design_tokens::chrome::ACCENT,
        6.,
    );
    push_projected_round_rect(
        &mut foreground.quads,
        inset_rect(card, 1., 1., 1., 1.),
        design_tokens::chrome::SURFACE_03,
        5.,
    );
    draw_text_clipped(
        "PCB update",
        card.x + 10.,
        card.y + 8.,
        11.,
        TEXT_PRIMARY,
        TextFace::Mono,
        card,
        &mut foreground.text,
    );
    draw_text_clipped(
        reason,
        card.x + 10.,
        card.y + 25.,
        11.,
        TEXT_MUTED,
        TextFace::Mono,
        card,
        &mut foreground.text,
    );
    foreground.text.last_mut().unwrap().layout_size = Some((content_width, reason_height));
    draw_text_clipped(
        "Apply unavailable",
        card.x + 10.,
        card.y + height - 44.,
        11.,
        TEXT_MUTED,
        TextFace::Mono,
        card,
        &mut foreground.text,
    );
    let dismiss = RectPx {
        x: card.x + 10.,
        y: card.y + height - 22.,
        width: (width - 20.).max(1.),
        height: 20.,
    };
    draw_text_clipped(
        "Dismiss [Esc]",
        dismiss.x,
        dismiss.y + 3.,
        11.,
        design_tokens::chrome::ACCENT,
        TextFace::Mono,
        card,
        &mut foreground.text,
    );
    if let Some(rect) = dismiss.intersect(viewport) {
        hits.push(HitRegion {
            rect,
            target: HitTarget::SchematicFieldTable(C::Cancel),
        });
    }
    foreground
}

#[cfg(test)]
#[path = "native_proposal_review_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "native_interface_review_tests.rs"]
mod interface_tests;

#[cfg(test)]
#[path = "native_board_field_review_tests.rs"]
mod board_field_tests;
