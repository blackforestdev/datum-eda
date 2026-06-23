#[test]
fn output_jobs_payload_maps_artifact_files_to_production_status() {
    let payload = r#"{
        "contract": "output_job_list_v1",
        "project_id": "00000000-0000-0000-0000-000000000001",
        "model_revision": "revision-a",
        "output_job_count": 1,
        "output_jobs": [{
            "id": "00000000-0000-0000-0000-00000000job1",
            "name": "Release fabrication",
            "include": ["gerber_set"],
            "prefix": "release-a",
            "output_dir": "generated/release-a",
            "status": "succeeded",
            "execution_count": 1,
            "latest_run": {
                "run_id": "00000000-0000-0000-0000-00000000run1",
                "artifact_id": "00000000-0000-0000-0000-00000000art1"
            },
            "artifacts": [{
                "artifact_id": "00000000-0000-0000-0000-00000000art1",
                "kind": "gerber_set",
                "project_id": "00000000-0000-0000-0000-000000000001",
                "model_revision": "revision-a",
                "output_job": "00000000-0000-0000-0000-00000000job1",
                "variant": "00000000-0000-0000-0000-00000000var1",
                "generator_version": "datum-test",
                "output_dir": "/tmp/fab",
                "validation_state": "not_validated",
                "files": [{
                    "path": "fabrication/board-F_Cu.gbr",
                    "sha256": "sha256:abc123"
                }],
                "production_projections": [{
                    "projection_kind": "gerber_copper_layer",
                    "projection_contract": "datum.production_projection.gerber_copper_layer.v1",
                    "model_revision": "revision-a",
                    "byte_count": 128,
                    "sha256": "sha256:def456"
                }]
            }]
        }]
    }"#;

    let production = datum_gui_protocol::production_status_from_output_jobs_json(payload)
        .expect("output-job list payload should decode");

    assert_eq!(production.output_job_count, 1);
    assert_eq!(production.artifact_count, 1);
    assert_eq!(production.latest_status.as_deref(), Some("succeeded"));
    assert_eq!(
        production.latest_run_id.as_deref(),
        Some("00000000-0000-0000-0000-00000000run1")
    );
    let job = production.output_jobs.first().expect("job summary");
    assert_eq!(job.name, "Release fabrication");
    assert_eq!(job.prefix, "release-a");
    assert_eq!(job.output_dir.as_deref(), Some("generated/release-a"));
    assert_eq!(job.family, "GERBER SET");
    assert_eq!(
        job.latest_run_artifact_id.as_deref(),
        Some("00000000-0000-0000-0000-00000000art1")
    );
    assert_eq!(job.artifact_count, 1);
    let artifact = job.artifacts.first().expect("artifact summary");
    assert_eq!(artifact.kind, "gerber_set");
    assert_eq!(
        artifact.project_id.as_deref(),
        Some("00000000-0000-0000-0000-000000000001")
    );
    assert_eq!(artifact.model_revision.as_deref(), Some("revision-a"));
    assert_eq!(
        artifact.output_job.as_deref(),
        Some("00000000-0000-0000-0000-00000000job1")
    );
    assert_eq!(
        artifact.variant.as_deref(),
        Some("00000000-0000-0000-0000-00000000var1")
    );
    assert_eq!(artifact.generator_version.as_deref(), Some("datum-test"));
    assert_eq!(artifact.output_dir.as_deref(), Some("/tmp/fab"));
    assert_eq!(artifact.validation_state.as_deref(), Some("not_validated"));
    assert_eq!(artifact.file_count, 1);
    assert_eq!(artifact.production_projection_count, 1);
    let file = artifact.files.first().expect("artifact file summary");
    assert_eq!(file.path, "fabrication/board-F_Cu.gbr");
    assert_eq!(file.sha256, "sha256:abc123");
    let projection = artifact
        .production_projections
        .first()
        .expect("artifact projection summary");
    assert_eq!(projection.projection_kind, "gerber_copper_layer");
    assert_eq!(
        projection.projection_contract,
        "datum.production_projection.gerber_copper_layer.v1"
    );
    assert_eq!(projection.model_revision, "revision-a");
    assert_eq!(projection.byte_count, 128);
    assert_eq!(projection.sha256, "sha256:def456");
}

#[test]
fn output_jobs_payload_maps_finer_artifact_family_runs() {
    let production = datum_gui_protocol::production_status_from_output_jobs_json(
        r#"{
          "contract": "output_job_list_v1",
          "output_job_count": 1,
          "output_jobs": [{
            "id": "job-drill",
            "name": "Drill output",
            "include": ["drill"],
            "prefix": "release-drill",
            "output_dir": "generated/drill",
            "status": "succeeded",
            "execution_count": 1,
            "latest_run": {"run_id": "run-drill", "artifact_id": "artifact-drill"},
            "artifacts": [{"artifact_id": "artifact-drill", "kind": "drill"}]
          }]
        }"#,
    )
    .expect("drill output-job payload should decode");

    let job = production.output_jobs.first().expect("job summary");
    assert_eq!(job.family, "DRILL");
    assert_eq!(job.prefix, "release-drill");
    assert_eq!(job.output_dir.as_deref(), Some("generated/drill"));
    assert_eq!(
        job.latest_run_artifact_id.as_deref(),
        Some("artifact-drill")
    );
    assert_eq!(job.artifacts[0].kind, "drill");
}

#[test]
fn artifact_list_payload_maps_ad_hoc_artifact_runs() {
    let production = datum_gui_protocol::production_status_from_artifacts_json(
        r#"{
          "contract": "artifact_metadata_list_v1",
          "artifact_count": 1,
          "artifact_run_count": 1,
          "artifacts": [{
            "artifact_id": "artifact-bom",
            "kind": "bom",
            "project_id": "project-a",
            "model_revision": "revision-a",
            "output_job": null,
            "variant": null,
            "generator_version": "test",
            "files": [],
            "validation_state": "not_validated"
          }],
          "artifact_runs": [{
            "run_id": "run-bom",
            "artifact_id": "artifact-bom",
            "run_sequence": 2,
            "status": "succeeded",
            "exit_code": 0
          }]
        }"#,
    )
    .expect("artifact list payload should decode");

    assert_eq!(production.output_job_count, 0);
    assert_eq!(production.artifact_count, 1);
    assert_eq!(production.artifact_run_count, 1);
    let run = production.artifact_runs.first().expect("artifact run");
    assert_eq!(run.run_id, "run-bom");
    assert_eq!(run.artifact_id, "artifact-bom");
    assert_eq!(run.run_source, "artifact_run");
    assert_eq!(run.output_job_id, None);
    assert_eq!(run.run_sequence, 2);
    assert_eq!(run.status, "succeeded");
    assert_eq!(run.exit_code, Some(0));
}

#[test]
fn artifact_list_payload_maps_linked_output_job_runs() {
    let production = datum_gui_protocol::production_status_from_artifacts_json(
        r#"{
          "contract": "artifact_metadata_list_v1",
          "artifact_count": 1,
          "artifact_run_count": 0,
          "output_job_run_count": 1,
          "artifacts": [{
            "artifact_id": "artifact-drill",
            "kind": "drill",
            "project_id": "project-a",
            "model_revision": "revision-a",
            "output_job": "job-drill",
            "variant": null,
            "generator_version": "test",
            "files": [],
            "validation_state": "not_validated"
          }],
          "artifact_runs": [],
          "output_job_runs": [{
            "run_id": "run-drill",
            "output_job": "job-drill",
            "artifact_id": "artifact-drill",
            "run_sequence": 5,
            "status": "succeeded",
            "exit_code": 0
          }]
        }"#,
    )
    .expect("artifact list payload should decode linked output-job runs");

    assert_eq!(production.output_job_count, 0);
    assert_eq!(production.artifact_count, 1);
    assert_eq!(production.artifact_run_count, 1);
    let run = production
        .artifact_runs
        .first()
        .expect("linked output-job run");
    assert_eq!(run.run_id, "run-drill");
    assert_eq!(run.artifact_id, "artifact-drill");
    assert_eq!(run.run_source, "output_job_run");
    assert_eq!(run.output_job_id.as_deref(), Some("job-drill"));
    assert_eq!(run.run_sequence, 5);
    assert_eq!(run.status, "succeeded");
    assert_eq!(run.exit_code, Some(0));
}

#[test]
fn production_payload_maps_panel_and_plan_summaries() {
    let output_jobs = r#"{
        "contract": "output_job_list_v1",
        "project_id": "00000000-0000-0000-0000-000000000001",
        "model_revision": "revision-a",
        "output_job_count": 0,
        "output_jobs": []
    }"#;
    let manufacturing_plans = r#"{
        "contract": "manufacturing_plan_list_v1",
        "manufacturing_plan_count": 1,
        "manufacturing_plans": [{
            "id": "00000000-0000-0000-0000-00000000fab1",
            "name": "Release fabrication",
            "board_or_panel": "00000000-0000-0000-0000-00000000pan1",
            "variant": null,
            "prefix": "release-a",
            "object_revision": 2
        }]
    }"#;
    let panel_projections = r#"{
        "contract": "panel_projection_list_v1",
        "panel_projection_count": 1,
        "panel_projections": [{
            "id": "00000000-0000-0000-0000-00000000pan1",
            "name": "Release panel",
            "board_instances": [{
                "board": "00000000-0000-0000-0000-00000000brd1",
                "x_nm": 1000,
                "y_nm": 2000,
                "rotation_deg": 90
            }],
            "object_revision": 3
        }]
    }"#;

    let production = datum_gui_protocol::production_status_from_production_json(
        output_jobs,
        manufacturing_plans,
        panel_projections,
    )
    .expect("production payloads should decode");

    assert_eq!(production.manufacturing_plan_count, 1);
    assert_eq!(production.panel_projection_count, 1);
    assert_eq!(production.manufacturing_plans[0].prefix, "release-a");
    assert_eq!(production.manufacturing_plans[0].object_revision, 2);
    assert_eq!(production.panel_projections[0].board_instance_count, 1);
    assert_eq!(production.panel_projections[0].first_x_nm, Some(1000));
    assert_eq!(production.panel_projections[0].first_rotation_deg, Some(90));
    assert_eq!(production.panel_projections[0].object_revision, 3);
}

#[test]
fn production_status_decodes_persisted_proposals() {
    let status = datum_gui_protocol::production_status_from_proposals_json(
        r#"{
          "proposal_count": 1,
          "proposals": {
            "00000000-0000-0000-0000-00000000aa01": {
              "status": "draft",
              "source": "check",
              "rationale": "repair process-aperture standards findings",
              "batch": {"operations": [{"op": "SetBoardPad"}]}
            }
          }
        }"#,
    )
    .expect("proposal production status should decode");

    assert_eq!(status.proposal_count, 1);
    assert_eq!(status.proposals.len(), 1);
    assert_eq!(
        status.proposals[0].proposal_id,
        "00000000-0000-0000-0000-00000000aa01"
    );
    assert_eq!(status.proposals[0].status, "draft");
    assert_eq!(status.proposals[0].source, "check");
    assert_eq!(status.proposals[0].operation_count, 1);
}

#[test]
fn artifact_files_payload_maps_focused_artifact_detail() {
    let detail = datum_gui_protocol::production_artifact_detail_from_files_json(
        r#"{
          "contract": "artifact_files_v1",
          "artifact_id": "00000000-0000-0000-0000-00000000art1",
          "kind": "gerber_set",
          "output_dir": "/tmp/fab",
          "validation_state": "valid",
          "files": [
            {"path": "fabrication/board-F_Cu.gbr", "sha256": "sha256:abc123"}
          ],
          "production_projections": [
            {
              "projection_kind": "gerber_copper_layer",
              "projection_contract": "datum.production_projection.gerber_copper_layer.v1",
              "model_revision": "revision-a",
              "byte_count": 128,
              "sha256": "sha256:def456"
            }
          ]
        }"#,
    )
    .expect("artifact files payload should parse");

    assert_eq!(detail.kind, "gerber_set");
    assert_eq!(detail.output_dir.as_deref(), Some("/tmp/fab"));
    assert_eq!(detail.validation_state, "valid");
    assert_eq!(detail.file_count, 1);
    assert_eq!(detail.files[0].path, "fabrication/board-F_Cu.gbr");
    assert_eq!(
        detail.focused_file.as_ref().map(|file| file.path.as_str()),
        Some("fabrication/board-F_Cu.gbr")
    );
    assert_eq!(detail.production_projection_count, 1);
    assert_eq!(detail.production_projections[0].byte_count, 128);
}

#[test]
fn artifact_preview_payload_maps_preview_summary() {
    let preview = datum_gui_protocol::production_artifact_file_preview_from_json(
        r#"{
          "contract": "artifact_file_preview_v1",
          "file": "fabrication/board-F_Cu.gbr",
          "preview_kind": "gerber_rs274x",
          "hash_matches_metadata": true,
          "primitive_count": 4,
          "primitives": [
            {
              "kind": "stroke",
              "aperture_diameter_nm": 250000,
              "points": [
                {"x_nm": 0, "y_nm": 0},
                {"x_nm": 1000000, "y_nm": 1000000}
              ]
            }
          ],
          "inspection": {
            "geometry_count": 4,
            "stroke_count": 4
          }
        }"#,
    )
    .expect("artifact preview payload should parse");

    assert_eq!(preview.file, "fabrication/board-F_Cu.gbr");
    assert_eq!(preview.preview_kind, "gerber_rs274x");
    assert_eq!(preview.hash_matches_metadata, true);
    assert_eq!(preview.primitive_count, 4);
    assert_eq!(preview.primitives.len(), 1);
    assert_eq!(preview.primitives[0].kind, "stroke");
    assert_eq!(preview.primitives[0].aperture_diameter_nm, Some(250000));
    assert_eq!(preview.primitives[0].points[1].x_nm, 1000000);
    assert_eq!(preview.geometry_count, Some(4));
    assert_eq!(preview.hit_count, None);
    assert_eq!(preview.row_count, None);
    assert!(preview.csv_columns.is_empty());
    assert!(preview.csv_rows.is_empty());
}

#[test]
fn artifact_csv_preview_payload_maps_table_summary() {
    let preview = datum_gui_protocol::production_artifact_file_preview_from_json(
        r#"{
          "contract": "artifact_file_preview_v1",
          "file": "fabrication/board-bom.csv",
          "preview_kind": "bom_csv",
          "hash_matches_metadata": true,
          "primitive_count": 0,
          "inspection": {
            "family": "bom",
            "header": "ref,value,footprint",
            "row_count": 2,
            "columns": ["ref", "value", "footprint"],
            "rows": [
              ["R1", "10k", "0603"],
              ["C1", "100n", "0603"]
            ]
          }
        }"#,
    )
    .expect("artifact CSV preview payload should parse");

    assert_eq!(preview.preview_kind, "bom_csv");
    assert_eq!(preview.row_count, Some(2));
    assert_eq!(preview.csv_columns, vec!["ref", "value", "footprint"]);
    assert_eq!(preview.csv_rows.len(), 2);
    assert_eq!(preview.csv_rows[0], vec!["R1", "10k", "0603"]);
}

#[test]
fn focus_production_artifact_command_selects_artifact_summary() {
    let mut workspace = datum_gui_protocol::load_fixture_workspace_state();
    workspace.production = datum_gui_protocol::ProductionStatus {
        output_job_count: 1,
        artifact_count: 2,
        artifact_run_count: 0,
        proposal_count: 0,
        manufacturing_plan_count: 0,
        panel_projection_count: 0,
        latest_status: Some("succeeded".to_string()),
        latest_run_id: None,
        artifact_runs: Vec::new(),
        proposals: Vec::new(),
        manufacturing_plans: Vec::new(),
        panel_projections: Vec::new(),
        focused_artifact: None,
        output_jobs: vec![datum_gui_protocol::ProductionOutputJobSummary {
            id: "job-1".to_string(),
            name: "Release fabrication".to_string(),
            include: vec!["gerber_set".to_string()],
            prefix: "release-a".to_string(),
            output_dir: None,
            family: "GERBER SET".to_string(),
            status: "succeeded".to_string(),
            execution_count: 1,
            artifact_count: 2,
            latest_run_id: None,
            latest_run_artifact_id: None,
            artifacts: vec![
                datum_gui_protocol::ProductionArtifactSummary {
                    artifact_id: "artifact-a".to_string(),
                    kind: "gerber_set".to_string(),
                    project_id: None,
                    model_revision: None,
                    output_job: None,
                    variant: None,
                    generator_version: None,
                    output_dir: None,
                    validation_state: None,
                    file_count: 0,
                    files: Vec::new(),
                    production_projection_count: 0,
                    production_projections: Vec::new(),
                },
                datum_gui_protocol::ProductionArtifactSummary {
                    artifact_id: "artifact-b".to_string(),
                    kind: "manufacturing_set".to_string(),
                    project_id: None,
                    model_revision: None,
                    output_job: None,
                    variant: None,
                    generator_version: None,
                    output_dir: Some("/tmp/release".to_string()),
                    validation_state: None,
                    file_count: 1,
                    files: vec![
                        datum_gui_protocol::ProductionArtifactFileSummary {
                            path: "fabrication/release.zip".to_string(),
                            sha256: "sha256:zip".to_string(),
                        },
                        datum_gui_protocol::ProductionArtifactFileSummary {
                            path: "fabrication/release.gbr".to_string(),
                            sha256: "sha256:gbr".to_string(),
                        },
                    ],
                    production_projection_count: 0,
                    production_projections: Vec::new(),
                },
            ],
        }],
    };
    let mut session = datum_gui_protocol::LiveDesignSession::new(workspace);

    let result = session.apply(datum_gui_protocol::SessionCommand::FocusProductionArtifact(
        "artifact-b".to_string(),
    ));

    assert!(result.handled);
    assert_eq!(
        session
            .workspace()
            .production
            .focused_artifact
            .as_ref()
            .map(|artifact| artifact.artifact_id.as_str()),
        Some("artifact-b")
    );
    assert_eq!(
        session
            .workspace()
            .production
            .focused_artifact
            .as_ref()
            .and_then(|artifact| artifact.output_dir.as_deref()),
        Some("/tmp/release")
    );
    assert_eq!(
        session
            .workspace()
            .production
            .focused_artifact
            .as_ref()
            .map(|artifact| artifact.files[0].path.as_str()),
        Some("fabrication/release.zip")
    );
    assert_eq!(
        session
            .workspace()
            .production
            .focused_artifact
            .as_ref()
            .and_then(|artifact| artifact.focused_file.as_ref())
            .map(|file| file.path.as_str()),
        Some("fabrication/release.zip")
    );

    let file_result = session.apply(
        datum_gui_protocol::SessionCommand::FocusProductionArtifactFile(
            "fabrication/release.gbr".to_string(),
        ),
    );

    assert!(file_result.handled);
    assert_eq!(
        session
            .workspace()
            .production
            .focused_artifact
            .as_ref()
            .and_then(|artifact| artifact.focused_file.as_ref())
            .map(|file| file.path.as_str()),
        Some("fabrication/release.gbr")
    );
}

#[test]
fn artifact_preview_viewport_commands_update_ui_state() {
    let workspace = datum_gui_protocol::load_fixture_workspace_state();
    let mut session = datum_gui_protocol::LiveDesignSession::new(workspace);

    let zoom = session.apply(datum_gui_protocol::SessionCommand::ZoomArtifactPreviewIn);
    assert!(zoom.handled);
    assert_eq!(session.workspace().ui.artifact_preview.zoom_ppm, 1_200_000);

    let pan = session.apply(datum_gui_protocol::SessionCommand::PanArtifactPreview {
        delta_x_ppm: 125_000,
        delta_y_ppm: -250_000,
    });
    assert!(pan.handled);
    assert_eq!(session.workspace().ui.artifact_preview.pan_x_ppm, 125_000);
    assert_eq!(session.workspace().ui.artifact_preview.pan_y_ppm, -250_000);

    let geometry = session.apply(datum_gui_protocol::SessionCommand::ToggleArtifactPreviewGeometry);
    assert!(geometry.handled);
    assert_eq!(session.workspace().ui.artifact_preview.show_geometry, false);

    let drills = session.apply(datum_gui_protocol::SessionCommand::ToggleArtifactPreviewDrills);
    assert!(drills.handled);
    assert_eq!(session.workspace().ui.artifact_preview.show_drills, false);

    let reset = session.apply(datum_gui_protocol::SessionCommand::ResetArtifactPreviewViewport);
    assert!(reset.handled);
    assert_eq!(
        session.workspace().ui.artifact_preview,
        datum_gui_protocol::ArtifactPreviewViewportState::default()
    );
}
