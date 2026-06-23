use datum_gui_render::{CameraState, HitTarget, PreparedScene, RetainedScene};

fn production_commands(prepared: &PreparedScene) -> Vec<(String, String)> {
    prepared
        .hit_regions
        .iter()
        .filter_map(|region| match &region.target {
            HitTarget::ProductionTerminalCommand(handoff)
            | HitTarget::ProductionOutputJobRun(handoff) => {
                Some((handoff.command_id.clone(), handoff.command.clone()))
            }
            _ => None,
        })
        .collect()
}

#[test]
fn outputs_dock_exposes_finer_output_job_run_command() {
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    state.ui.active_dock_tab = Some(datum_gui_protocol::DockTab::Outputs);
    state.ui.dock_height_px = 320;
    state.production = datum_gui_protocol::ProductionStatus {
        output_job_count: 2,
        artifact_count: 0,
        artifact_run_count: 0,
        latest_status: Some("running".to_string()),
        artifact_runs: Vec::new(),
        output_jobs: vec![
            datum_gui_protocol::ProductionOutputJobSummary {
                id: "00000000-0000-0000-0000-00000000job2".to_string(),
                name: "Release drills".to_string(),
                include: vec!["drill".to_string()],
                prefix: "release-a".to_string(),
                output_dir: Some("$DATUM_PROJECT_ROOT/fab".to_string()),
                family: "DRILL".to_string(),
                status: "never_run".to_string(),
                execution_count: 0,
                artifact_count: 0,
                latest_run_id: None,
                latest_run_artifact_id: None,
                artifacts: Vec::new(),
            },
            datum_gui_protocol::ProductionOutputJobSummary {
                id: "00000000-0000-0000-0000-00000000job3".to_string(),
                name: "Release package".to_string(),
                include: vec!["manufacturing-set".to_string()],
                prefix: "release-b".to_string(),
                output_dir: Some("$DATUM_PROJECT_ROOT/fab".to_string()),
                family: "MANUFACTURING SET".to_string(),
                status: "running".to_string(),
                execution_count: 1,
                artifact_count: 0,
                latest_run_id: Some("00000000-0000-0000-0000-00000000run3".to_string()),
                latest_run_artifact_id: None,
                artifacts: Vec::new(),
            },
        ],
        ..datum_gui_protocol::ProductionStatus::default()
    };

    let prepared = PreparedScene::from_workspace(
        &state,
        1280,
        800,
        CameraState::fit_to_bounds(&state.scene.bounds),
        &RetainedScene::from_workspace(&state, 1280, 800),
    );
    assert!(prepared.hit_regions.iter().any(|region| matches!(
        &region.target,
        HitTarget::ProductionOutputJobRun(handoff)
            if handoff.command_id == "datum.artifact.generate"
                && handoff.mcp_alias.as_deref() == Some("datum.artifact.generate")
                && handoff.command
                    == "datum-eda artifact generate \"$DATUM_PROJECT_ROOT\" --output-job 00000000-0000-0000-0000-00000000job2"
    )));
    assert!(prepared.hit_regions.iter().any(|region| matches!(
        &region.target,
        HitTarget::ProductionTerminalCommand(handoff)
            if handoff.command_id == "datum.artifact.start_output_job_run"
                && handoff.mcp_alias.as_deref() == Some("datum.artifact.start_output_job_run")
                && handoff.command
                    == "datum-eda artifact start-output-job-run \"$DATUM_PROJECT_ROOT\" --output-job 00000000-0000-0000-0000-00000000job2"
    )));
    assert!(prepared.hit_regions.iter().any(|region| matches!(
        &region.target,
        HitTarget::ProductionTerminalCommand(handoff)
            if handoff.command_id == "datum.artifact.cancel_output_job_run"
                && handoff.mcp_alias.as_deref() == Some("datum.artifact.cancel_output_job_run")
                && handoff.command
                    == "datum-eda artifact cancel-output-job-run \"$DATUM_PROJECT_ROOT\" --run 00000000-0000-0000-0000-00000000run3"
    )));
}

#[test]
fn outputs_dock_exposes_ad_hoc_artifact_run_hit_target() {
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    state.ui.active_dock_tab = Some(datum_gui_protocol::DockTab::Outputs);
    state.ui.dock_height_px = 320;
    state.production = datum_gui_protocol::ProductionStatus {
        artifact_count: 2,
        artifact_run_count: 2,
        artifact_runs: vec![
            datum_gui_protocol::ProductionArtifactRunSummary {
                run_id: "00000000-0000-0000-0000-00000000adh1".to_string(),
                artifact_id: "00000000-0000-0000-0000-00000000art1".to_string(),
                run_source: "artifact_run".to_string(),
                output_job_id: None,
                run_sequence: 1,
                status: "succeeded".to_string(),
                exit_code: Some(0),
            },
            datum_gui_protocol::ProductionArtifactRunSummary {
                run_id: "00000000-0000-0000-0000-00000000adh2".to_string(),
                artifact_id: "00000000-0000-0000-0000-00000000art2".to_string(),
                run_source: "artifact_run".to_string(),
                output_job_id: None,
                run_sequence: 2,
                status: "succeeded".to_string(),
                exit_code: Some(0),
            },
        ],
        ..datum_gui_protocol::ProductionStatus::default()
    };

    let prepared = PreparedScene::from_workspace(
        &state,
        1280,
        800,
        CameraState::fit_to_bounds(&state.scene.bounds),
        &RetainedScene::from_workspace(&state, 1280, 800),
    );
    assert!(prepared.hit_regions.iter().any(|region| matches!(
        &region.target,
        HitTarget::ProductionArtifact(id)
            if id == "00000000-0000-0000-0000-00000000art1"
    )));
    assert!(prepared.hit_regions.iter().any(|region| matches!(
        &region.target,
        HitTarget::ProductionTerminalCommand(handoff)
            if handoff.command_id == "datum.artifact.compare"
                && handoff.mcp_alias.as_deref() == Some("datum.artifact.compare")
                && handoff.command
                    == "datum-eda artifact compare \"$DATUM_PROJECT_ROOT\" --before 00000000-0000-0000-0000-00000000art1 --after 00000000-0000-0000-0000-00000000art2"
    )));
    assert!(prepared.hit_regions.iter().any(|region| matches!(
        &region.target,
        HitTarget::ProductionTerminalCommand(handoff)
            if handoff.command_id == "datum.artifact.validate"
                && handoff.mcp_alias.as_deref() == Some("datum.artifact.validate")
                && handoff.command
                    == "datum-eda artifact validate \"$DATUM_PROJECT_ROOT\" --artifact 00000000-0000-0000-0000-00000000art1"
    )));
}

#[test]
fn outputs_dock_exposes_proposal_review_terminal_commands() {
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    state.ui.active_dock_tab = Some(datum_gui_protocol::DockTab::Outputs);
    state.ui.dock_height_px = 380;
    state.production = datum_gui_protocol::ProductionStatus {
        proposal_count: 1,
        proposals: vec![datum_gui_protocol::ProductionProposalSummary {
            proposal_id: "00000000-0000-0000-0000-00000000aa01".to_string(),
            status: "draft".to_string(),
            source: "check".to_string(),
            rationale: "repair standards findings".to_string(),
            operation_count: 1,
            can_apply: Some(false),
            blocker_codes: vec!["missing_acceptance".to_string()],
            preview: Some(datum_gui_protocol::ProductionProposalPreviewSummary {
                prepared_against: "rev-before".to_string(),
                preview_after_model_revision: "rev-after".to_string(),
                created_count: 1,
                modified_count: 2,
                deleted_count: 0,
                affected_object_count: 3,
                affected_objects: vec![
                    "00000000-0000-0000-0000-00000000bb01".to_string(),
                    "00000000-0000-0000-0000-00000000bb02".to_string(),
                    "00000000-0000-0000-0000-00000000bb03".to_string(),
                ],
                render_deltas: vec![datum_gui_protocol::ProductionProposalRenderDeltaSummary {
                    delta_kind: "create".to_string(),
                    object_id: "00000000-0000-0000-0000-00000000bb01".to_string(),
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
                }],
            }),
        }],
        ..datum_gui_protocol::ProductionStatus::default()
    };

    let prepared = PreparedScene::from_workspace(
        &state,
        1280,
        800,
        CameraState::fit_to_bounds(&state.scene.bounds),
        &RetainedScene::from_workspace(&state, 1280, 800),
    );
    assert!(prepared.hit_regions.iter().any(|region| matches!(
        &region.target,
        HitTarget::ProductionTerminalCommand(handoff)
            if handoff.command_id == "datum.proposal.list"
                && handoff.mcp_alias.as_deref() == Some("datum.proposal.list")
                && handoff.command == "datum-eda proposal list \"$DATUM_PROJECT_ROOT\""
    )));
    assert!(prepared.hit_regions.iter().any(|region| matches!(
        &region.target,
        HitTarget::ProductionTerminalCommand(handoff)
            if handoff.command_id == "datum.proposal.show"
                && handoff.mcp_alias.as_deref() == Some("datum.proposal.show")
                && handoff.command
                    == "datum-eda proposal show \"$DATUM_PROJECT_ROOT\" --proposal 00000000-0000-0000-0000-00000000aa01"
    )));
    assert!(prepared.hit_regions.iter().any(|region| matches!(
        &region.target,
        HitTarget::ProductionTerminalCommand(handoff)
            if handoff.command_id == "datum.proposal.accept_apply"
                && handoff.mcp_alias.as_deref() == Some("datum.proposal.accept_apply")
                && handoff.command
                    == "datum-eda proposal accept-apply \"$DATUM_PROJECT_ROOT\" --proposal 00000000-0000-0000-0000-00000000aa01"
    )));
    assert!(prepared.hit_regions.iter().any(|region| matches!(
        &region.target,
        HitTarget::ProductionTerminalCommand(handoff)
            if handoff.command_id == "datum.proposal.preview"
                && handoff.mcp_alias.as_deref() == Some("datum.proposal.preview")
                && handoff.command
                    == "datum-eda proposal preview \"$DATUM_PROJECT_ROOT\" --proposal 00000000-0000-0000-0000-00000000aa01"
    )));
    assert!(prepared.hit_regions.iter().any(|region| matches!(
        &region.target,
        HitTarget::ProductionTerminalCommand(handoff)
            if handoff.command_id == "datum.proposal.validate"
                && handoff.mcp_alias.as_deref() == Some("datum.proposal.validate")
                && handoff.command
                    == "datum-eda proposal validate \"$DATUM_PROJECT_ROOT\" --proposal 00000000-0000-0000-0000-00000000aa01"
    )));
    assert!(prepared.hit_regions.iter().any(|region| matches!(
        &region.target,
        HitTarget::ProductionTerminalCommand(handoff)
            if handoff.command_id == "datum.proposal.defer"
                && handoff.mcp_alias.as_deref() == Some("datum.proposal.defer")
                && handoff.command
                    == "datum-eda proposal defer \"$DATUM_PROJECT_ROOT\" --proposal 00000000-0000-0000-0000-00000000aa01"
    )));
    assert!(prepared.hit_regions.iter().any(|region| matches!(
        &region.target,
        HitTarget::ProductionTerminalCommand(handoff)
            if handoff.command_id == "datum.proposal.reject"
                && handoff.mcp_alias.as_deref() == Some("datum.proposal.reject")
                && handoff.command
                    == "datum-eda proposal reject \"$DATUM_PROJECT_ROOT\" --proposal 00000000-0000-0000-0000-00000000aa01"
    )));
}

#[test]
fn outputs_dock_exposes_zone_fill_check_finding_actions() {
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    state.ui.active_dock_tab = Some(datum_gui_protocol::DockTab::Outputs);
    state.ui.dock_height_px = 320;
    state.checks = datum_gui_protocol::CheckRunReviewState {
        check_run_id: Some("00000000-0000-0000-0000-00000000chk1".to_string()),
        profile_id: Some("native-combined".to_string()),
        status: Some("error".to_string()),
        finding_count: 1,
        findings: vec![datum_gui_protocol::CheckFindingSummary {
            finding_id: Some("00000000-0000-0000-0000-00000000f001".to_string()),
            source: "zone_fill".to_string(),
            code: "zone_fill_unfilled".to_string(),
            severity: "error".to_string(),
            fingerprint: "sha256:zone-fill-finding".to_string(),
            domain: "zone_fill".to_string(),
            rule_id: "zone_fill_state".to_string(),
            status: "active".to_string(),
            message: "Zone is unfilled".to_string(),
            suggested_next_action: Some("Run fill-zones or waive.".to_string()),
            ..datum_gui_protocol::CheckFindingSummary::default()
        }],
        ..datum_gui_protocol::CheckRunReviewState::default()
    };

    let prepared = PreparedScene::from_workspace(
        &state,
        1280,
        800,
        CameraState::fit_to_bounds(&state.scene.bounds),
        &RetainedScene::from_workspace(&state, 1280, 800),
    );
    let commands = production_commands(&prepared);
    assert!(
        commands.iter().any(|(id, command)| id == "datum.check.show"
            && command
                == "datum-eda check show \"$DATUM_PROJECT_ROOT\" --check-run 00000000-0000-0000-0000-00000000chk1"),
        "{commands:?}"
    );
    assert!(
        commands
            .iter()
            .any(|(id, command)| id == "datum.check.profiles"
                && command == "datum-eda check profiles \"$DATUM_PROJECT_ROOT\""),
        "{commands:?}"
    );
    assert!(
        commands.iter().any(|(id, command)| id == "datum.check.list"
            && command == "datum-eda check list \"$DATUM_PROJECT_ROOT\""),
        "{commands:?}"
    );
    assert!(
        commands
            .iter()
            .any(|(id, command)| id == "datum.check.run_profile"
                && command == "datum-eda check run \"$DATUM_PROJECT_ROOT\" --profile standards"),
        "{commands:?}"
    );
    assert!(
        commands
            .iter()
            .any(|(id, command)| id == "datum.check.run_profile"
                && command == "datum-eda check run \"$DATUM_PROJECT_ROOT\" --profile release"),
        "{commands:?}"
    );
    assert!(
        commands
            .iter()
            .any(|(id, command)| id == "datum.check.fill_zones"
                && command == "datum-eda check fill-zones \"$DATUM_PROJECT_ROOT\""),
        "{commands:?}"
    );
    assert!(
        commands
            .iter()
            .any(|(id, command)| id == "datum.check.waive"
                && command.contains("--fingerprint")
                && command.contains("sha256:zone-fill-finding")),
        "{commands:?}"
    );
}

#[test]
fn outputs_dock_exposes_standards_finding_and_linked_proposal_actions() {
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    state.ui.active_dock_tab = Some(datum_gui_protocol::DockTab::Outputs);
    state.ui.dock_height_px = 360;
    state.checks = datum_gui_protocol::CheckRunReviewState {
        check_run_id: Some("00000000-0000-0000-0000-00000000chk2".to_string()),
        profile_id: Some("standards".to_string()),
        status: Some("error".to_string()),
        finding_count: 1,
        findings: vec![datum_gui_protocol::CheckFindingSummary {
            finding_id: Some("00000000-0000-0000-0000-00000000f002".to_string()),
            source: "drc".to_string(),
            code: "process_aperture_policy".to_string(),
            severity: "error".to_string(),
            fingerprint: "sha256:process-aperture".to_string(),
            domain: "standards".to_string(),
            rule_id: "process_aperture_policy".to_string(),
            status: "active".to_string(),
            message: "Pad aperture policy failed".to_string(),
            proposal_refs: vec!["00000000-0000-0000-0000-00000000pr01".to_string()],
            ..datum_gui_protocol::CheckFindingSummary::default()
        }],
        ..datum_gui_protocol::CheckRunReviewState::default()
    };

    let prepared = PreparedScene::from_workspace(
        &state,
        1280,
        800,
        CameraState::fit_to_bounds(&state.scene.bounds),
        &RetainedScene::from_workspace(&state, 1280, 800),
    );
    let commands = production_commands(&prepared);
    assert!(prepared.hit_regions.iter().any(|region| matches!(
        &region.target,
        HitTarget::CheckFinding(fingerprint) if fingerprint == "sha256:process-aperture"
    )));
    assert!(
        commands
            .iter()
            .any(|(id, command)| id == "datum.check.repair_standards"
                && command == "datum-eda check repair-standards \"$DATUM_PROJECT_ROOT\""),
        "{commands:?}"
    );
    assert!(
        commands.iter().any(|(id, command)| id == "datum.proposal.show"
            && command
                == "datum-eda proposal show \"$DATUM_PROJECT_ROOT\" --proposal 00000000-0000-0000-0000-00000000pr01"),
        "{commands:?}"
    );
    assert!(
        commands.iter().any(|(id, command)| id == "datum.proposal.preview"
            && command
                == "datum-eda proposal preview \"$DATUM_PROJECT_ROOT\" --proposal 00000000-0000-0000-0000-00000000pr01"),
        "{commands:?}"
    );
    assert!(
        commands.iter().any(|(id, command)| id == "datum.proposal.accept_apply"
            && command
                == "datum-eda proposal accept-apply \"$DATUM_PROJECT_ROOT\" --proposal 00000000-0000-0000-0000-00000000pr01"),
        "{commands:?}"
    );
}
