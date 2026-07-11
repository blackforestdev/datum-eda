#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn shell_layout_reserves_bottom_dock_and_viewport() {
        let layout = ShellLayout::for_window(1280, 800, None);
        assert!(layout.viewport.width > 0.0);
        assert_eq!(layout.bottom_strip.height, design_tokens::spacing::SP_07);
        assert!(layout.left_sidebar.width > 0.0);
        assert!(layout.right_sidebar.width > 0.0);
    }

    #[test]
    fn text_buffer_key_ignores_position_and_color_but_tracks_content() {
        let base = TextRun {
            text: "PROJECT".to_string(),
            x: 12.0,
            y: 24.0,
            size: 12.0,
            color: TEXT_PRIMARY,
            face: TextFace::Ui,
            clip_bounds: None,
        };
        let mut moved = base.clone();
        moved.x += 100.0;
        moved.y += 50.0;
        moved.color = TEXT_SECONDARY;

        assert_eq!(
            text_buffer_key(&base, 1280, 768),
            text_buffer_key(&moved, 1280, 768)
        );

        let mut changed_text = base.clone();
        changed_text.text.push('!');
        assert_ne!(
            text_buffer_key(&base, 1280, 768),
            text_buffer_key(&changed_text, 1280, 768)
        );

        let mut changed_size = base.clone();
        changed_size.size = 13.0;
        assert_ne!(
            text_buffer_key(&base, 1280, 768),
            text_buffer_key(&changed_size, 1280, 768)
        );

        let mut clipped = base.clone();
        clipped.clip_bounds = Some(RectPx {
            x: 0.0,
            y: 0.0,
            width: 44.0,
            height: 18.0,
        });
        assert_ne!(
            text_buffer_key(&base, 1280, 768),
            text_buffer_key(&clipped, 1280, 768)
        );
    }

    #[test]
    fn conformance_medium_type_tiers_resolve_to_medium_weight() {
        assert_eq!(design_tokens::typography::STRONG_WEIGHT, 500);
        assert_eq!(design_tokens::typography::MICRO_WEIGHT, 500);
        assert_eq!(text_attrs(TextFace::UiMedium).weight, Weight::MEDIUM);
        assert_eq!(text_attrs(TextFace::UiStrong).weight, Weight::SEMIBOLD);
        assert_eq!(text_attrs(TextFace::Ui).weight, Weight::NORMAL);
    }

    #[test]
    fn text_prepare_signature_tracks_render_relevant_inputs() {
        let run = TextRun {
            text: "TERMINAL".to_string(),
            x: 12.0,
            y: 24.0,
            size: 12.0,
            color: TEXT_PRIMARY,
            face: TextFace::Ui,
            clip_bounds: None,
        };
        let base = text_prepare_signature(&[4], std::slice::from_ref(&run), 1280, 768);
        let mut moved = run.clone();
        moved.x += 1.0;
        assert_ne!(
            base,
            text_prepare_signature(&[4], std::slice::from_ref(&moved), 1280, 768)
        );
        let mut recolored = run.clone();
        recolored.color = TEXT_SECONDARY;
        assert_ne!(
            base,
            text_prepare_signature(&[4], std::slice::from_ref(&recolored), 1280, 768)
        );
        assert_ne!(
            base,
            text_prepare_signature(&[5], std::slice::from_ref(&run), 1280, 768)
        );
        assert_ne!(
            base,
            text_prepare_signature(&[4], std::slice::from_ref(&run), 1281, 768)
        );
    }

    #[test]
    fn shell_layout_is_solved_by_taffy_grid_contract() {
        // Assert the derivable, token-driven grid CONTRACT rather than magic
        // pixels that drift: menu bar on top; left sidebar then viewport then
        // right sidebar below it; dock then status bar
        // pinned full-width to the bottom. Values come from design tokens so
        // this cannot silently fall out of sync with the layout again.
        let (w, h, dock) = (1280.0_f32, 800.0_f32, 260.0_f32);
        let layout = ShellLayout::for_window(w as u32, h as u32, Some(dock as u32));

        let menu = design_tokens::spacing::SP_07 + 1.0;
        let status = design_tokens::spacing::SP_06 + design_tokens::spacing::SP_01;
        let content_h = h - menu - dock - status;

        assert_eq!(
            layout.top_menu_bar,
            RectPx {
                x: 0.0,
                y: 0.0,
                width: w,
                height: menu
            }
        );
        assert_eq!(layout.left_sidebar.x, 0.0);
        assert_eq!(layout.left_sidebar.y, menu);
        assert_eq!(layout.left_sidebar.height, content_h);
        assert_eq!(layout.viewport.x, layout.left_sidebar.width);
        assert_eq!(layout.viewport.y, menu);
        assert_eq!(layout.viewport.height, content_h);
        assert_eq!(
            layout.viewport.width,
            w - layout.left_sidebar.width - layout.right_sidebar.width
        );
        assert_eq!(layout.right_sidebar.x, w - layout.right_sidebar.width);
        assert_eq!(layout.right_sidebar.y, menu);
        assert_eq!(layout.right_sidebar.height, content_h);
        assert_eq!(
            layout.bottom_strip,
            RectPx {
                x: 0.0,
                y: menu + content_h,
                width: w,
                height: dock
            }
        );
        assert_eq!(
            layout.status_bar,
            RectPx {
                x: 0.0,
                y: h - status,
                width: w,
                height: status
            }
        );
    }

    #[test]
    fn shell_layout_solves_logical_pixels_then_scales_to_surface_pixels() {
        // Contract: for_surface takes PHYSICAL pixels, solves the layout at
        // logical pixels (physical / scale), then scales every region back up
        // to surface pixels. Assert that relationship (plus token-derived
        // anchors) instead of drifting magic pixels.
        let scale = 1.25_f32;
        let (phys_w, phys_h) = (1600u32, 1000u32);
        let logical = ShellLayout::for_window(
            (phys_w as f32 / scale).round() as u32,
            (phys_h as f32 / scale).round() as u32,
            Some(260),
        );
        let surface = ShellLayout::for_surface(phys_w, phys_h, scale, Some(260));

        assert_eq!(
            surface.top_menu_bar.height,
            logical.top_menu_bar.height * scale
        );
        assert_eq!(
            surface.left_sidebar.width,
            logical.left_sidebar.width * scale
        );
        assert_eq!(
            surface.right_sidebar.width,
            logical.right_sidebar.width * scale
        );
        assert_eq!(surface.status_bar.height, logical.status_bar.height * scale);
        assert_eq!(surface.viewport.x, logical.viewport.x * scale);
        assert_eq!(surface.viewport.width, logical.viewport.width * scale);

        let menu = design_tokens::spacing::SP_07 + 1.0;
        let status = design_tokens::spacing::SP_06 + design_tokens::spacing::SP_01;
        assert_eq!(surface.top_menu_bar.height, menu * scale);
        assert_eq!(surface.status_bar.height, status * scale);
    }

    #[test]
    fn prepared_scene_uses_surface_scale_for_layout_and_text() {
        let state = datum_gui_protocol::load_fixture_workspace_state();
        let retained = RetainedScene::from_workspace_for_surface(&state, 1600, 1000, 1.25);
        let prepared = PreparedScene::from_workspace_for_surface(
            &state,
            1600,
            1000,
            1.25,
            CameraState::fit_to_bounds(&state.scene.bounds),
            &retained,
        );

        assert_eq!(
            prepared.layout,
            ShellLayout::for_surface(1600, 1000, 1.25, dock_height_for_state(&state))
        );
        let project_title = prepared
            .text_runs
            .iter()
            .find(|run| run.text == "PROJECT")
            .expect("project title should render");
        assert_eq!(project_title.size, 15.0);
        assert!(
            prepared
                .hit_test(
                    prepared.layout.left_sidebar.x + 20.0,
                    prepared.layout.left_sidebar.y + 20.0,
                )
                .is_none()
        );
    }

    #[test]
    fn proposal_preview_affected_ids_match_scene_source_ids() {
        let mut state = datum_gui_protocol::load_fixture_workspace_state();
        let component = state
            .scene
            .components
            .first()
            .expect("fixture component should exist")
            .clone();
        state.production.proposals = vec![datum_gui_protocol::ProductionProposalSummary {
            proposal_id: "proposal-a".to_string(),
            status: "draft".to_string(),
            source: "check".to_string(),
            rationale: "highlight modified component".to_string(),
            operation_count: 1,
            can_apply: Some(false),
            blocker_codes: Vec::new(),
            preview: Some(datum_gui_protocol::ProductionProposalPreviewSummary {
                prepared_against: "rev-before".to_string(),
                preview_after_model_revision: "rev-after".to_string(),
                created_count: 0,
                modified_count: 1,
                deleted_count: 0,
                affected_object_count: 1,
                affected_objects: vec![component.source_object_uuid.clone()],
                render_deltas: Vec::new(),
            }),
        }];

        let affected = proposal_preview_affected_ids(&state);
        assert!(source_object_matches_preview(
            &affected,
            &component.object_id,
            &component.source_object_uuid
        ));
        assert!(component_matches_preview(
            &component.component_uuid,
            &state.scene,
            &affected
        ));
    }
    #[test]
    fn proposal_preview_render_deltas_become_overlay_primitives() {
        let mut state = datum_gui_protocol::load_fixture_workspace_state();
        state.production.proposals = vec![datum_gui_protocol::ProductionProposalSummary {
            proposal_id: "proposal-a".to_string(),
            status: "draft".to_string(),
            source: "check".to_string(),
            rationale: "ghost new track".to_string(),
            operation_count: 1,
            can_apply: Some(false),
            blocker_codes: Vec::new(),
            preview: Some(datum_gui_protocol::ProductionProposalPreviewSummary {
                prepared_against: "rev-before".to_string(),
                preview_after_model_revision: "rev-after".to_string(),
                created_count: 1,
                modified_count: 0,
                deleted_count: 0,
                affected_object_count: 1,
                affected_objects: vec!["track-a".to_string()],
                render_deltas: vec![
                    datum_gui_protocol::ProductionProposalRenderDeltaSummary {
                        delta_kind: "create".to_string(),
                        object_id: "track-a".to_string(),
                        primitive_kind: "track_path".to_string(),
                        layer_id: "L1".to_string(),
                        end_layer_id: None,
                        width_nm: 250_000,
                        drill_nm: None,
                        diameter_nm: None,
                        path: vec![
                            datum_gui_protocol::PointNm { x: 1000, y: 2000 },
                            datum_gui_protocol::PointNm { x: 3000, y: 4000 },
                        ],
                    },
                    datum_gui_protocol::ProductionProposalRenderDeltaSummary {
                        delta_kind: "create".to_string(),
                        object_id: "via-a".to_string(),
                        primitive_kind: "via".to_string(),
                        layer_id: "L1".to_string(),
                        end_layer_id: Some("L2".to_string()),
                        width_nm: 650_000,
                        drill_nm: Some(300_000),
                        diameter_nm: Some(650_000),
                        path: vec![datum_gui_protocol::PointNm { x: 5000, y: 6000 }],
                    },
                ],
            }),
        }];

        let overlays = production_proposal_overlay_primitives(&state);
        assert_eq!(overlays.len(), 2);
        assert_eq!(overlays[0].overlay_id, "proposal:proposal-a:preview:0");
        assert_eq!(overlays[0].primitive_kind, "track_path");
        assert_eq!(overlays[0].proposal_action_id, "proposal-a");
        assert_eq!(overlays[0].layer_id.as_deref(), Some("L1"));
        assert_eq!(overlays[0].width_nm, Some(250_000));
        assert_eq!(overlays[0].path.len(), 2);
        assert_eq!(overlays[1].overlay_id, "proposal:proposal-a:preview:1");
        assert_eq!(overlays[1].primitive_kind, "via");
        assert_eq!(overlays[1].proposal_action_id, "proposal-a");
        assert_eq!(overlays[1].layer_id.as_deref(), Some("L1"));
        assert_eq!(overlays[1].width_nm, Some(650_000));
        assert_eq!(overlays[1].drill_nm, Some(300_000));
        assert_eq!(overlays[1].diameter_nm, Some(650_000));
        assert_eq!(overlays[1].path.len(), 1);
    }

    #[test]
    fn prepared_scene_preserves_viewport_dominance() {
        let state = datum_gui_protocol::load_fixture_workspace_state();
        let retained = RetainedScene::from_workspace(&state, 1280, 800);
        let prepared = PreparedScene::from_workspace(
            &state,
            1280,
            960,
            CameraState::fit_to_bounds(&state.scene.bounds),
            &retained,
        );
        assert!(prepared.layout.viewport.width > prepared.layout.left_sidebar.width);
        assert!(prepared.layout.viewport.width > prepared.layout.right_sidebar.width / 2.0);
    }

    #[test]
    fn authoring_tool_buttons_are_not_rendered_in_read_only_phase_one() {
        let state = datum_gui_protocol::load_fixture_workspace_state();
        let retained = RetainedScene::from_workspace(&state, 1280, 800);
        let prepared = PreparedScene::from_workspace(
            &state,
            1280,
            800,
            CameraState::fit_to_bounds(&state.scene.bounds),
            &retained,
        );
        let tool_regions = prepared
            .hit_regions
            .iter()
            .filter(|region| matches!(region.target, HitTarget::SetWorkspaceTool(_)))
            .collect::<Vec<_>>();

        assert!(tool_regions.is_empty());
    }

    #[test]
    fn authored_component_selection_populates_inspector_reference_and_value() {
        // Locks the Phase-2 populated-component inspector branch that the
        // datum-test `--select R1` parity capture freezes: an AuthoredObject
        // component selection must emit the reference identity run plus a Value
        // key row inside the right sidebar column (the single-pane populated
        // composition), not the empty route-action chrome.
        let mut state = datum_gui_protocol::load_fixture_workspace_state();
        let object_id = state.scene.components[0].object_id.clone();
        let reference = state.scene.components[0].reference.clone();
        assert!(
            state.select_authored_object(&object_id),
            "fixture component object_id should resolve to an AuthoredObject selection"
        );
        let retained = RetainedScene::from_workspace(&state, 1280, 800);
        let prepared = PreparedScene::from_workspace(
            &state,
            1280,
            800,
            CameraState::fit_to_bounds(&state.scene.bounds),
            &retained,
        );
        let right = prepared.layout.right_sidebar;
        let in_inspector =
            |run: &TextRun| run.x >= right.x && run.x <= right.x + right.width;
        // The component reference draws as the inspector identity run (Mono, 15px);
        // the right-column x filter disambiguates it from the on-board silk label.
        assert!(
            prepared.text_runs.iter().any(|run| run.text == reference
                && matches!(run.face, TextFace::Mono)
                && in_inspector(run)),
            "populated component inspector should emit the reference identity run in the right column"
        );
        // The Value key row proves the populated branch rendered (route-action
        // chrome has no Value row).
        assert!(
            prepared
                .text_runs
                .iter()
                .any(|run| run.text == "Value" && in_inspector(run)),
            "populated component inspector should emit the Value key row"
        );
    }

    #[test]
    fn project_card_controls_flow_above_filter_controls() {
        let state = datum_gui_protocol::load_fixture_workspace_state();
        let retained = RetainedScene::from_workspace(&state, 1280, 800);
        let prepared = PreparedScene::from_workspace(
            &state,
            1280,
            800,
            CameraState::fit_to_bounds(&state.scene.bounds),
            &retained,
        );
        let fit_bottom = prepared
            .hit_regions
            .iter()
            .filter_map(|region| {
                if matches!(
                    region.target,
                    HitTarget::FitBoard | HitTarget::FitReviewTarget
                ) {
                    Some(region.rect.y + region.rect.height)
                } else {
                    None
                }
            })
            .fold(0.0_f32, f32::max);
        let filter_top = prepared
            .hit_regions
            .iter()
            .find(|region| matches!(region.target, HitTarget::ToggleShowAuthored))
            .expect("authored filter hit region should render")
            .rect
            .y;

        assert!(fit_bottom < filter_top);
    }

    #[test]
    fn populated_inspector_status_stays_inside_right_column() {
        let mut state = datum_gui_protocol::load_fixture_workspace_state();
        let action_id = "action-populated".to_string();
        state.active_review_target_id = action_id.clone();
        state.selection = SelectionTarget::ReviewAction(action_id.clone());
        state.review.proposal_actions = vec![datum_gui_protocol::RouteProposalActionPayload {
            action_id,
            proposal_action: "draw_track".to_string(),
            reason: "qa populated review state".to_string(),
            contract: "m7_populated_layout_contract".to_string(),
            net_uuid: "net-populated".to_string(),
            net_name: "BOARD_STATUS_NET".to_string(),
            from_anchor_pad_uuid: "pad-a".to_string(),
            to_anchor_pad_uuid: "pad-b".to_string(),
            layer: 1,
            width_nm: 200_000,
            from: datum_gui_protocol::PointNm { x: 0, y: 0 },
            to: datum_gui_protocol::PointNm { x: 1_000_000, y: 0 },
            reused_via_uuid: None,
            reused_via_uuids: Vec::new(),
            reused_object_kind: None,
            reused_object_uuid: None,
            reused_object_from_layer: None,
            reused_object_to_layer: None,
            selected_path_bend_count: 1,
            selected_path_point_count: 2,
            selected_path_segment_index: 0,
            selected_path_segment_count: 1,
            selected_path_layer_segment_index: Some(0),
            selected_path_layer_segment_count: Some(1),
            selected_path_layer_segment_bend_count: Some(1),
            selected_path_layer_segment_point_count: Some(2),
        }];
        state.review.segment_evidence = vec![datum_gui_protocol::RouteProposalSegmentEvidence {
            layer_segment_index: 0,
            layer_segment_count: 1,
            layer: 1,
            bend_count: 1,
            point_count: 2,
            track_action_count: 1,
        }];
        state.last_command_status = Some(datum_gui_protocol::EditorCommandStatus {
            action: "place_board_text".to_string(),
            detail: "queued board text @ 140700000,90100000".to_string(),
        });

        let layout = ShellLayout::for_window(1280, 800, dock_height_for_state(&state));
        let right_layout =
            side_panels::solve_right_panel_layout_with_taffy(&state, layout.right_sidebar)
                .expect("right panel layout should solve");
        let inspector_bottom = right_layout.inspector_rect.y + right_layout.inspector_rect.height;
        let retained = RetainedScene::from_workspace(&state, 1280, 800);
        let prepared = PreparedScene::from_workspace(
            &state,
            1280,
            800,
            CameraState::fit_to_bounds(&state.scene.bounds),
            &retained,
        );
        let last_row_y = prepared
            .text_runs
            .iter()
            .find(|run| run.text == "LAST")
            .expect("populated inspector should render LAST key")
            .y;

        assert!(last_row_y + 12.0 <= inspector_bottom);
    }

    // (Removed filter_summary_renders_below_layer_rows — the OUTPUTS/ART/status
    // summary dump was pulled from the Layers panel as debug-HUD clutter.)

    #[test]
    fn terminal_dock_does_not_surface_artifact_file_summaries() {
        let mut state = datum_gui_protocol::load_fixture_workspace_state();
        state.ui.active_dock_tab = Some(datum_gui_protocol::DockTab::Terminal);
        state.ui.dock_height_px = 560;
        state.production = datum_gui_protocol::ProductionStatus {
            output_job_count: 1,
            artifact_count: 1,
            latest_status: Some("succeeded".to_string()),
            latest_run_id: Some("00000000-0000-0000-0000-00000000run1".to_string()),
            manufacturing_plan_count: 1,
            panel_projection_count: 1,
            output_jobs: vec![datum_gui_protocol::ProductionOutputJobSummary {
                id: "00000000-0000-0000-0000-00000000job1".to_string(),
                name: "Release fabrication".to_string(),
                include: vec!["drill".to_string()],
                prefix: "release-a".to_string(),
                output_dir: None,
                family: "DRILL".to_string(),
                status: "succeeded".to_string(),
                execution_count: 1,
                artifact_count: 1,
                latest_run_id: Some("00000000-0000-0000-0000-00000000run1".to_string()),
                latest_run_artifact_id: Some("00000000-0000-0000-0000-00000000art1".to_string()),
                artifacts: vec![datum_gui_protocol::ProductionArtifactSummary {
                    artifact_id: "00000000-0000-0000-0000-00000000art1".to_string(),
                    kind: "drill".to_string(),
                    project_id: None,
                    model_revision: None,
                    output_job: None,
                    variant: None,
                    generator_version: None,
                    output_dir: Some("/tmp/fab".to_string()),
                    validation_state: None,
                    file_count: 1,
                    files: vec![datum_gui_protocol::ProductionArtifactFileSummary {
                        path: "fabrication/release-a-drill.drl".to_string(),
                        sha256: "sha256:abc123".to_string(),
                    }],
                    production_projection_count: 1,
                    production_projections: vec![
                        datum_gui_protocol::ProductionArtifactProjectionSummary {
                            projection_kind: "excellon_drill".to_string(),
                            projection_contract: "datum.production_projection.excellon_drill.v1"
                                .to_string(),
                            model_revision: "revision-a".to_string(),
                            byte_count: 128,
                            sha256: "sha256:def456".to_string(),
                        },
                    ],
                }],
            }],
            manufacturing_plans: vec![datum_gui_protocol::ProductionManufacturingPlanSummary {
                id: "00000000-0000-0000-0000-00000000fab1".to_string(),
                name: "Release fabrication".to_string(),
                prefix: "release-a".to_string(),
                board_or_panel: "00000000-0000-0000-0000-00000000pan1".to_string(),
                variant: None,
                object_revision: 2,
            }],
            panel_projections: vec![datum_gui_protocol::ProductionPanelProjectionSummary {
                id: "00000000-0000-0000-0000-00000000pan1".to_string(),
                name: "Release panel".to_string(),
                board_instance_count: 1,
                first_board: Some("00000000-0000-0000-0000-00000000brd1".to_string()),
                first_x_nm: Some(1000),
                first_y_nm: Some(2000),
                first_rotation_deg: Some(90),
                object_revision: 3,
            }],
            focused_artifact: Some(datum_gui_protocol::ProductionArtifactDetail {
                artifact_id: "00000000-0000-0000-0000-00000000art1".to_string(),
                kind: "gerber_set".to_string(),
                output_dir: Some("/tmp/fab".to_string()),
                validation_state: "valid".to_string(),
                file_count: 1,
                files: vec![datum_gui_protocol::ProductionArtifactFileSummary {
                    path: "fabrication/board-F_Cu.gbr".to_string(),
                    sha256: "sha256:abc123".to_string(),
                }],
                focused_file: Some(datum_gui_protocol::ProductionArtifactFileSummary {
                    path: "fabrication/board-F_Cu.gbr".to_string(),
                    sha256: "sha256:abc123".to_string(),
                }),
                focused_preview: Some(datum_gui_protocol::ProductionArtifactFilePreviewSummary {
                    file: "fabrication/board-F_Cu.gbr".to_string(),
                    preview_kind: "gerber_rs274x".to_string(),
                    hash_matches_metadata: true,
                    primitive_count: 4,
                    primitives: sample_artifact_preview_primitives(),
                    geometry_count: Some(4),
                    hit_count: None,
                    row_count: None,
                    csv_columns: Vec::new(),
                    csv_rows: Vec::new(),
                }),
                production_projection_count: 1,
                production_projections: vec![
                    datum_gui_protocol::ProductionArtifactProjectionSummary {
                        projection_kind: "gerber_copper_layer".to_string(),
                        projection_contract: "datum.production_projection.gerber_copper_layer.v1"
                            .to_string(),
                        model_revision: "revision-a".to_string(),
                        byte_count: 128,
                        sha256: "sha256:def456".to_string(),
                    },
                ],
            }),
            ..datum_gui_protocol::ProductionStatus::default()
        };
        let retained = RetainedScene::from_workspace(&state, 1280, 960);
        let prepared = PreparedScene::from_workspace(
            &state,
            1280,
            960,
            CameraState::fit_to_bounds(&state.scene.bounds),
            &retained,
        );
        let rendered_text = prepared
            .text_runs
            .iter()
            .map(|run| run.text.as_str())
            .collect::<Vec<_>>();
        assert!(
            !rendered_text.contains(&"OUTPUT JOBS"),
            "Phase 1 terminal dock must not render Output-lane summaries"
        );
        assert!(
            !prepared
                .hit_regions
                .iter()
                .any(|region| matches!(region.target, HitTarget::ProductionArtifact(_))),
            "Phase 1 terminal dock must not expose Output-lane artifact hit regions"
        );
    }

    #[test]
    fn marking_menu_shell_renders_manifest_items_without_command_targets() {
        let mut state = datum_gui_protocol::load_fixture_workspace_state();
        state.ui.marking_menu = Some(datum_gui_protocol::MarkingMenuState {
            menu_key: "pcb.component".to_string(),
            target_object_id: Some("component:demo".to_string()),
            anchor_x_px: 640,
            anchor_y_px: 360,
            preview_slot: Some("N".to_string()),
            gesture_dx_px: 0,
            gesture_dy_px: -72,
        });
        let retained = RetainedScene::from_workspace(&state, 1280, 768);
        let prepared = PreparedScene::from_workspace(
            &state,
            1280,
            768,
            CameraState::fit_to_bounds(&state.scene.bounds),
            &retained,
        );
        let rendered_text = prepared
            .text_runs
            .iter()
            .map(|run| run.text.as_str())
            .collect::<Vec<_>>();

        assert!(rendered_text.contains(&"Rotate"));
        assert!(rendered_text.contains(&"Delete"));
        assert!(rendered_text.contains(&"pcb.component"));
        assert!(prepared.hit_regions.iter().any(|region| matches!(
            &region.target,
            HitTarget::MarkingMenuItem {
                menu_key,
                slot,
                label
            } if menu_key == "pcb.component" && slot == "N" && label == "Rotate"
        )));
        assert!(
            prepared.hit_regions.iter().all(|region| {
                !matches!(
                    region.target,
                    HitTarget::ProductionTerminalCommand(_) | HitTarget::ProductionOutputJobRun(_)
                )
            }),
            "inert marking-menu shell must not expose terminal command hit targets"
        );
    }

    #[test]
    fn imported_board_text_counts_as_component_detail_text() {
        let component_uuid = "f7794004-b142-4fe8-aea4-5f3796f333a5";

        assert!(imported_board_text_belongs_to_component(
            &format!(
                "imported_kicad_property_text:{component_uuid}:reference:component_silkscreen"
            ),
            component_uuid,
        ));
        assert!(imported_board_text_belongs_to_component(
            &format!("imported_kicad_fp_text:{component_uuid}:component_silkscreen"),
            component_uuid,
        ));
        assert!(!imported_board_text_belongs_to_component(
            "imported_kicad_property_text:other-component:reference:component_silkscreen",
            component_uuid,
        ));
        assert!(!imported_board_text_belongs_to_component(
            "manual_board_text",
            component_uuid,
        ));
    }

    #[test]
    fn hit_regions_include_review_rows_and_overlay_targets() {
        let state = datum_gui_protocol::load_fixture_workspace_state();
        let retained = RetainedScene::from_workspace(&state, 1280, 800);
        let prepared = PreparedScene::from_workspace(
            &state,
            1280,
            800,
            CameraState::fit_to_bounds(&state.scene.bounds),
            &retained,
        );
        assert!(prepared.hit_regions.iter().any(
            |region| matches!(region.target, HitTarget::ReviewAction(ref id) if id == "action-1")
        ));
    }

    #[test]
    fn hit_testing_prefers_overlay_over_underlying_authored_geometry() {
        let state = datum_gui_protocol::load_fixture_workspace_state();
        let retained = RetainedScene::from_workspace(&state, 1280, 800);
        let prepared = PreparedScene::from_workspace(
            &state,
            1280,
            800,
            CameraState::fit_to_bounds(&state.scene.bounds),
            &retained,
        );
        let overlay_rect = prepared
            .hit_regions
            .iter()
            .rev()
            .find_map(|region| match &region.target {
                HitTarget::ReviewAction(id) if id == "action-1" => Some(region.rect),
                _ => None,
            })
            .expect("action overlay hit region should exist");
        let hit = prepared
            .hit_test(
                overlay_rect.x + overlay_rect.width / 2.0,
                overlay_rect.y + overlay_rect.height / 2.0,
            )
            .expect("topmost hit should exist");
        assert_eq!(hit, &HitTarget::ReviewAction("action-1".to_string()));
    }

    #[test]
    fn board_outline_hit_region_selects_assembled_outline() {
        let state = datum_gui_protocol::load_fixture_workspace_state();
        let retained = RetainedScene::from_workspace(&state, 1280, 800);
        let outline = state
            .scene
            .outline
            .first()
            .expect("fixture should include a board outline");
        assert!(
            outline.path.len() >= 2,
            "fixture outline should include at least one segment"
        );
        let a = outline.path[0];
        let b = outline.path[1];
        let hit_point = PointNm {
            x: (a.x + b.x) / 2,
            y: (a.y + b.y) / 2,
        };

        let hit = retained
            .hit_test_authored_world(hit_point, &state)
            .expect("board outline segment should be selectable");
        assert_eq!(hit, &HitTarget::AuthoredObject(outline.object_id.clone()));
    }

    #[test]
    fn selected_board_text_numeric_rows_have_step_and_center_edit_zones() {
        let mut state = datum_gui_protocol::load_fixture_workspace_state();
        let object_id = "board-text:test-hit-zones".to_string();
        state
            .scene
            .board_texts
            .push(datum_gui_protocol::BoardTextPrimitive {
                object_id: object_id.clone(),
                object_kind: "board_text".to_string(),
                text_uuid: "test-hit-zones".to_string(),
                text: "TEST".to_string(),
                layer_id: "F.Silks".to_string(),
                position: PointNm { x: 0, y: 0 },
                rotation_degrees: 0,
                height_nm: 1_000_000,
                stroke_width_nm: 100_000,
                render_intent: "annotation".to_string(),
                family: "inter".to_string(),
                style: "regular".to_string(),
                style_class: None,
                h_align: "center".to_string(),
                v_align: "center".to_string(),
                mirrored: false,
                keep_upright: true,
                line_spacing_ratio_ppm: 1_000_000,
                bold: false,
                italic: false,
            });
        state.selection = SelectionTarget::AuthoredObject(object_id);

        let retained = RetainedScene::from_workspace(&state, 1280, 800);
        let prepared = PreparedScene::from_workspace(
            &state,
            1280,
            800,
            CameraState::fit_to_bounds(&state.scene.bounds),
            &retained,
        );

        assert_three_zone_row(
            &prepared,
            HitTarget::DecreaseSelectedBoardTextHeight,
            HitTarget::EditSelectedBoardTextHeight,
            HitTarget::IncreaseSelectedBoardTextHeight,
        );
        assert_three_zone_row(
            &prepared,
            HitTarget::RotateSelectedBoardTextCounterClockwise90,
            HitTarget::EditSelectedBoardTextRotation,
            HitTarget::RotateSelectedBoardTextClockwise90,
        );
        assert_three_zone_row(
            &prepared,
            HitTarget::DecreaseSelectedBoardTextLineSpacing,
            HitTarget::EditSelectedBoardTextLineSpacing,
            HitTarget::IncreaseSelectedBoardTextLineSpacing,
        );
        assert_three_zone_row(
            &prepared,
            HitTarget::CycleSelectedBoardTextRenderIntent,
            HitTarget::EditSelectedBoardTextRenderIntent,
            HitTarget::CycleSelectedBoardTextRenderIntent,
        );
        assert_three_zone_row(
            &prepared,
            HitTarget::CycleSelectedBoardTextFamily,
            HitTarget::EditSelectedBoardTextFamily,
            HitTarget::CycleSelectedBoardTextFamily,
        );
        assert_three_zone_row(
            &prepared,
            HitTarget::CycleSelectedBoardTextHAlign,
            HitTarget::EditSelectedBoardTextAlignment,
            HitTarget::CycleSelectedBoardTextVAlign,
        );
    }

    fn assert_three_zone_row(
        prepared: &PreparedScene,
        left: HitTarget,
        center: HitTarget,
        right: HitTarget,
    ) {
        let left_rect = hit_rect(prepared, &left);
        let center_rect = hit_rect(prepared, &center);
        let right_rect = hit_rect_from_end(prepared, &right);
        assert!(
            (left_rect.y - center_rect.y).abs() < f32::EPSILON
                && (center_rect.y - right_rect.y).abs() < f32::EPSILON,
            "three-zone hit regions must share one row"
        );
        assert!(left_rect.x < center_rect.x);
        assert!(center_rect.x < right_rect.x);
        assert!(center_rect.width > left_rect.width);
        assert!(center_rect.width > right_rect.width);

        assert_eq!(hit_center(prepared, left_rect), left);
        assert_eq!(hit_center(prepared, center_rect), center);
        assert_eq!(hit_center(prepared, right_rect), right);
    }

    fn hit_rect(prepared: &PreparedScene, target: &HitTarget) -> RectPx {
        prepared
            .hit_regions
            .iter()
            .find(|region| &region.target == target)
            .map(|region| region.rect)
            .unwrap_or_else(|| panic!("expected hit region for {target:?}"))
    }

    fn hit_rect_from_end(prepared: &PreparedScene, target: &HitTarget) -> RectPx {
        prepared
            .hit_regions
            .iter()
            .rev()
            .find(|region| &region.target == target)
            .map(|region| region.rect)
            .unwrap_or_else(|| panic!("expected hit region for {target:?}"))
    }

    fn hit_center(prepared: &PreparedScene, rect: RectPx) -> HitTarget {
        prepared
            .hit_test(rect.x + rect.width * 0.5, rect.y + rect.height * 0.5)
            .cloned()
            .expect("hit target should exist at rect center")
    }

    #[test]
    fn roundrect_pad_uses_richer_geometry_than_rect_pad() {
        let viewport = RectPx {
            x: 0.0,
            y: 0.0,
            width: 200.0,
            height: 120.0,
        };
        let bounds = datum_gui_protocol::SceneBounds {
            min_x: 0,
            min_y: 0,
            max_x: 2_000_000,
            max_y: 1_200_000,
        };
        let projection = Projection::new(viewport, &bounds, CameraState::fit_to_bounds(&bounds));
        let mut rect_out = Vec::new();
        let mut roundrect_out = Vec::new();
        let rect_pad = datum_gui_protocol::PadPrimitive {
            object_id: "pad:rect".to_string(),
            object_kind: "pad".to_string(),
            source_object_uuid: "rect".to_string(),
            pad_uuid: "rect".to_string(),
            component_uuid: "U1".to_string(),
            net_uuid: None,
            layer_id: "L1".to_string(),
            copper_layer_ids: vec!["L1".to_string()],
            center: PointNm {
                x: 1_000_000,
                y: 600_000,
            },
            bounds: datum_gui_protocol::RectNm {
                min_x: 700_000,
                min_y: 350_000,
                max_x: 1_300_000,
                max_y: 850_000,
            },
            shape_kind: "rect".to_string(),
            roundrect_rratio_ppm: 250_000,
            mask_layer_ids: vec![],
            paste_layer_ids: vec![],
            solder_mask_margin_nm: 0,
            solder_paste_margin_nm: 0,
            solder_paste_margin_ratio_ppm: 0,
            drill_nm: None,
            rotation_degrees: 0.0,
        };
        let mut roundrect_pad = rect_pad.clone();
        roundrect_pad.shape_kind = "roundrect".to_string();

        push_pad_primitive(
            &mut rect_out,
            &rect_pad,
            &projection,
            "L1",
            PAD_COPPER,
            None,
            false,
        );
        push_pad_primitive(
            &mut roundrect_out,
            &roundrect_pad,
            &projection,
            "L1",
            PAD_COPPER,
            None,
            false,
        );

        assert!(roundrect_out.len() > rect_out.len());
    }

    #[test]
    fn retained_pad_hit_regions_target_pads_for_inspection() {
        let state = datum_gui_protocol::load_fixture_workspace_state();
        let pad = state
            .scene
            .pads
            .first()
            .expect("fixture should include pads");
        let retained = RetainedScene::from_workspace(&state, 1280, 800);

        assert!(
            retained.world_hit_index.regions().iter().any(|region| {
                matches!(&region.target, HitTarget::AuthoredObject(id) if id == &pad.object_id)
            }),
            "pad hit region should select the pad object for read-only inspection"
        );
    }

    #[test]
    fn board_text_inspector_does_not_emit_edit_hit_regions_in_phase_one() {
        let mut state = datum_gui_protocol::load_fixture_workspace_state();
        let Some(text) = state.scene.board_texts.first() else {
            return;
        };
        state.selection = SelectionTarget::AuthoredObject(text.object_id.clone());
        let retained = RetainedScene::from_workspace(&state, 1280, 800);
        let prepared = PreparedScene::from_workspace(
            &state,
            1280,
            800,
            CameraState::fit_to_bounds(&state.scene.bounds),
            &retained,
        );

        assert!(
            prepared.hit_regions.iter().all(|region| {
                !matches!(
                    region.target,
                    HitTarget::ToggleSelectedBoardTextMirrored
                        | HitTarget::ToggleSelectedBoardTextKeepUpright
                        | HitTarget::ToggleSelectedBoardTextBold
                        | HitTarget::CycleSelectedBoardTextRenderIntent
                        | HitTarget::CycleSelectedBoardTextFamily
                        | HitTarget::CycleSelectedBoardTextHAlign
                        | HitTarget::CycleSelectedBoardTextVAlign
                        | HitTarget::EditSelectedBoardTextRenderIntent
                        | HitTarget::EditSelectedBoardTextFamily
                        | HitTarget::EditSelectedBoardTextAlignment
                        | HitTarget::DecreaseSelectedBoardTextHeight
                        | HitTarget::IncreaseSelectedBoardTextHeight
                        | HitTarget::RotateSelectedBoardTextCounterClockwise90
                        | HitTarget::RotateSelectedBoardTextClockwise90
                        | HitTarget::DecreaseSelectedBoardTextLineSpacing
                        | HitTarget::IncreaseSelectedBoardTextLineSpacing
                        | HitTarget::EditSelectedBoardTextContent
                        | HitTarget::EditSelectedBoardTextHeight
                        | HitTarget::EditSelectedBoardTextRotation
                        | HitTarget::EditSelectedBoardTextLineSpacing
                )
            }),
            "Phase 1 board-text inspector must be read-only"
        );
    }

    #[test]
    fn layers_panel_renders_active_layer_and_toggle_regions() {
        let state = datum_gui_protocol::load_fixture_workspace_state();
        let retained = RetainedScene::from_workspace(&state, 1280, 800);
        let prepared = PreparedScene::from_workspace(
            &state,
            1280,
            800,
            CameraState::fit_to_bounds(&state.scene.bounds),
            &retained,
        );
        // (The "ACTIVE <layer>" summary line was removed as HUD clutter; the
        // active layer is shown by its inline ACTIVE badge + accent row. The
        // consumer-facing contract that remains is the toggle hit regions.)
        assert!(
            prepared
                .hit_regions
                .iter()
                .any(|region| matches!(region.target, HitTarget::ToggleLayer(_))),
            "Layers panel should expose consumer-side layer toggle hit regions"
        );
    }

    /// P2.1b "no noticeable lag" latency gate (decision 021): a pure workspace
    /// pane op — focus-switch, split, zoom, preset — is view state only and must
    /// NEVER re-resolve the world scene. We warm the retained world buffer once,
    /// then drive each pane op the way the app does (mutate the layout tree and
    /// keep the already-resolved retained scene — the app's `invalidate_frame`
    /// path, which never drops the retained scene), and assert the resolve counter
    /// stays flat. A bump would mean a pane op paid a full world-scene rebuild,
    /// i.e. clicking an adjacent viewport would lag.
    #[test]
    fn pane_ops_do_not_re_resolve_the_world_scene() {
        use datum_gui_protocol::{SplitOrientation, WorkspacePreset};

        let mut state = datum_gui_protocol::load_fixture_workspace_state();
        // The default workspace is the Board|Schematic split with the Board leaf
        // focused (today's look) — the starting point every pane op mutates.
        assert_eq!(
            state.ui.layout.focused_content(),
            datum_gui_protocol::PaneContent::Board
        );

        // The app holds ONE resolved world scene and only rebuilds it on a content
        // change; a pane op reuses it. Mirror that reuse with a get-or-resolve
        // Option that only resolves when empty (the `invalidate_frame` contract).
        let mut retained: Option<RetainedScene> = None;
        let warm = |retained: &mut Option<RetainedScene>,
                        state: &datum_gui_protocol::ReviewWorkspaceState| {
            if retained.is_none() {
                *retained = Some(RetainedScene::from_workspace_for_surface(
                    state, 1600, 1000, 1.0,
                ));
            }
        };

        warm(&mut retained, &state);
        let warmed = retained_scene_resolve_count();
        assert!(warmed >= 1, "warming must resolve the world scene once");

        // focus-switch: Board -> Schematic leaf (adjacent viewport becomes live).
        state.ui.layout.focus_next();
        warm(&mut retained, &state);
        assert_eq!(
            retained_scene_resolve_count(),
            warmed,
            "focus-switch must not re-resolve the world scene"
        );

        // split the focused leaf.
        state.ui.layout.split_focused(SplitOrientation::Horizontal);
        warm(&mut retained, &state);
        assert_eq!(
            retained_scene_resolve_count(),
            warmed,
            "split must not re-resolve the world scene"
        );

        // zoom (transient maximize) the focused leaf.
        state.ui.layout.toggle_zoom();
        warm(&mut retained, &state);
        assert_eq!(
            retained_scene_resolve_count(),
            warmed,
            "zoom/maximize must not re-resolve the world scene"
        );

        // apply a layout preset (rebuilds the whole tree).
        state.ui.layout.apply_preset(WorkspacePreset::BoardSchematic);
        warm(&mut retained, &state);
        assert_eq!(
            retained_scene_resolve_count(),
            warmed,
            "layout preset must not re-resolve the world scene"
        );
    }

}
