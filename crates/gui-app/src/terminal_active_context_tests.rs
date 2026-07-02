use super::*;
use crate::terminal_session::TerminalLaunchContext;
use crate::terminal_session_context::TerminalSessionContextSummary;
use datum_gui_protocol::{
    CheckRunReviewState, DatumCursorContext, DatumProjectionContext, DatumSelectionContext,
    ProductionArtifactDetail, ProductionArtifactFileSummary, ProductionStatus,
};
use eda_engine::substrate::{
    CommitProvenance, CommitSource, Operation, OperationBatch, ProjectResolver,
};
use std::fs;
use std::path::Path;
use uuid::Uuid;

fn write_json(path: &Path, value: serde_json::Value) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("fixture directory should create");
    }
    fs::write(path, format!("{value}\n")).expect("fixture JSON should write");
}

fn write_minimal_native_project(root: &Path) {
    write_json(
        &root.join("project.json"),
        serde_json::json!({
            "schema_version": 1,
            "uuid": Uuid::new_v4(),
            "name": "GUI Terminal Context Demo",
            "pools": [],
            "schematic": "schematic/schematic.json",
            "board": "board/board.json",
            "rules": "rules/rules.json",
            "forward_annotation_review": {}
        }),
    );
    write_json(
        &root.join("schematic/schematic.json"),
        serde_json::json!({
            "schema_version": 1,
            "uuid": Uuid::new_v4(),
            "sheets": {},
            "definitions": {},
            "instances": [],
            "variants": {},
            "waivers": [],
            "deviations": []
        }),
    );
    write_json(
        &root.join("board/board.json"),
        serde_json::json!({
            "schema_version": 1,
            "uuid": Uuid::new_v4(),
            "name": "GUI Terminal Context Demo Board",
            "stackup": { "layers": [] },
            "outline": { "vertices": [], "closed": true },
            "packages": {},
            "component_silkscreen": {},
            "component_pads": {},
            "pads": {},
            "tracks": {},
            "vias": {},
            "zones": {},
            "nets": {},
            "net_classes": {},
            "dimensions": {},
            "texts": {},
            "keepouts": {}
        }),
    );
    write_json(
        &root.join("rules/rules.json"),
        serde_json::json!({
            "schema_version": 1,
            "uuid": Uuid::new_v4(),
            "object_revision": 0,
            "rules": []
        }),
    );
}

fn commit_review_sidecar(root: &Path) -> String {
    let mut model = ProjectResolver::new(root)
        .resolve()
        .expect("native project should resolve");
    let report = model
        .commit_journaled(
            root,
            OperationBatch {
                batch_id: Uuid::new_v4(),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "gui-terminal-context-test".to_string(),
                    source: CommitSource::Cli,
                    reason: "record review sidecar".to_string(),
                },
                operations: vec![Operation::SetForwardAnnotationReview {
                    relative_path: ".datum/forward_annotation_review/review.json".to_string(),
                    previous_review: None,
                    review: serde_json::json!({
                        "schema_version": 1,
                        "reviews": {}
                    }),
                }],
            },
        )
        .expect("review sidecar should commit");
    report.transaction.transaction_id.to_string()
}

fn context_for_root(root: &std::path::Path) -> TerminalLaunchContext {
    TerminalLaunchContext {
        project_root: root.to_path_buf(),
        project_id: None,
        project_name: None,
        board_id: None,
        board_name: None,
        scene_id: None,
        source_revision: None,
        production_status: ProductionStatus::default(),
        source_shard_status: datum_gui_protocol::SourceShardStatusSummary::default(),
        check_status: CheckRunReviewState::default(),
        selection_context: DatumSelectionContext {
            kind: "none".to_string(),
            id: None,
        },
        cursor_context: DatumCursorContext {
            screen_px: None,
            hovered_object_id: None,
            active_dock_tab: None,
            active_tool: "select".to_string(),
        },
        projection_context: DatumProjectionContext {
            scene_id: "test-scene".to_string(),
            board_id: None,
            board_name: None,
            scene_bounds_nm: None,
            active_projection_id: None,
        },
        terminal_sessions: TerminalSessionContextSummary::default(),
    }
}

#[test]
fn terminal_context_projects_active_artifact_and_check_commands() {
    let root = std::env::temp_dir().join(format!(
        "datum-terminal-active-context-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("terminal active context root should create");
    write_minimal_native_project(&root);
    let transaction_tip = commit_review_sidecar(&root);
    let mut context = context_for_root(&root);
    context.production_status.focused_artifact = Some(ProductionArtifactDetail {
        artifact_id: "artifact-gerber".to_string(),
        kind: "gerber_set".to_string(),
        output_dir: Some("build/fab".to_string()),
        validation_state: "valid".to_string(),
        file_count: 1,
        files: Vec::new(),
        focused_file: Some(ProductionArtifactFileSummary {
            path: "build/fab/doa2526.drl".to_string(),
            sha256: "sha256-drill".to_string(),
        }),
        focused_preview: None,
        production_projection_count: 0,
        production_projections: Vec::new(),
    });
    context.production_status.latest_output_job_run_id = Some("run-gerber-2".to_string());
    context.production_status.latest_artifact_id = Some("artifact-gerber".to_string());
    context.production_status.artifact_runs = vec![
        datum_gui_protocol::ProductionArtifactRunSummary {
            run_id: "run-gerber-1".to_string(),
            artifact_id: "artifact-previous".to_string(),
            run_source: "artifact_run".to_string(),
            output_job_id: None,
            run_sequence: 1,
            status: "succeeded".to_string(),
            exit_code: Some(0),
        },
        datum_gui_protocol::ProductionArtifactRunSummary {
            run_id: "run-gerber-2".to_string(),
            artifact_id: "artifact-gerber".to_string(),
            run_source: "output_job_run".to_string(),
            output_job_id: Some("job-gerber".to_string()),
            run_sequence: 2,
            status: "succeeded".to_string(),
            exit_code: Some(0),
        },
    ];
    context.production_status.output_jobs = vec![datum_gui_protocol::ProductionOutputJobSummary {
        id: "job-gerber".to_string(),
        name: "Gerber Job".to_string(),
        prefix: "doa2526".to_string(),
        include: vec!["gerber-set".to_string()],
        output_dir: Some("build/fab".to_string()),
        family: "GERBER SET".to_string(),
        status: "succeeded".to_string(),
        execution_count: 1,
        artifact_count: 0,
        latest_run_id: Some("run-gerber-2".to_string()),
        latest_run_artifact_id: Some("artifact-gerber".to_string()),
        artifacts: Vec::new(),
    }];
    context.production_status.proposals = vec![datum_gui_protocol::ProductionProposalSummary {
        proposal_id: "proposal-repair".to_string(),
        status: "draft".to_string(),
        source: "Check".to_string(),
        rationale: "standards repair".to_string(),
        operation_count: 1,
        can_apply: Some(true),
        blocker_codes: Vec::new(),
        preview: None,
    }];
    context.check_status.check_run_id = Some("check-run-visible".to_string());
    context.selection_context = DatumSelectionContext {
        kind: "check_finding".to_string(),
        id: Some("sha256:selected-finding".to_string()),
    };

    let terminal_context = write_terminal_context(&context).expect("write terminal context");
    let envelope: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&terminal_context.context_path).unwrap()).unwrap();
    let root_arg = root.display().to_string();
    assert_eq!(
        envelope["active_context_commands"]["artifact_list"],
        format!("datum-eda artifact list {root_arg}")
    );
    assert_eq!(
        envelope["active_context_commands"]["artifact_show"],
        format!("datum-eda artifact show {root_arg} --artifact artifact-gerber")
    );
    assert_eq!(
        envelope["active_context_commands"]["artifact_files"],
        format!("datum-eda artifact files {root_arg} --artifact artifact-gerber")
    );
    assert_eq!(
        envelope["active_context_commands"]["artifact_preview"],
        format!(
            "datum-eda artifact preview {root_arg} --artifact artifact-gerber --file build/fab/doa2526.drl"
        )
    );
    assert_eq!(envelope["previous_artifact_id"], "artifact-previous");
    assert_eq!(
        envelope["active_context_commands"]["artifact_compare"],
        format!(
            "datum-eda artifact compare {root_arg} --before artifact-previous --after artifact-gerber"
        )
    );
    assert_eq!(
        envelope["active_context_commands"]["artifact_validate"],
        format!("datum-eda artifact validate {root_arg} --artifact artifact-gerber")
    );
    assert_eq!(
        envelope["active_context_commands"]["output_job_generate"],
        format!("datum-eda artifact generate {root_arg} --output-job job-gerber")
    );
    assert_eq!(
        envelope["active_context_commands"]["output_job_start_run"],
        format!("datum-eda artifact start-output-job-run {root_arg} --output-job job-gerber")
    );
    assert_eq!(
        envelope["active_context_commands"]["output_job_cancel_run"],
        format!("datum-eda artifact cancel-output-job-run {root_arg} --run run-gerber-2")
    );
    assert_eq!(
        envelope["active_context_commands"]["proposal_list"],
        format!("datum-eda proposal list {root_arg}")
    );
    assert_eq!(envelope["latest_proposal_id"], "proposal-repair");
    assert_eq!(
        envelope["visible_proposal_ids"],
        serde_json::json!(["proposal-repair"])
    );
    assert_eq!(
        envelope["active_context_commands"]["proposal_show"],
        format!("datum-eda proposal show {root_arg} --proposal proposal-repair")
    );
    assert_eq!(
        envelope["active_context_commands"]["proposal_preview"],
        format!("datum-eda proposal preview {root_arg} --proposal proposal-repair")
    );
    assert_eq!(
        envelope["active_context_commands"]["proposal_validate"],
        format!("datum-eda proposal validate {root_arg} --proposal proposal-repair")
    );
    assert_eq!(
        envelope["active_context_commands"]["proposal_review_accept"],
        format!(
            "datum-eda proposal review {root_arg} --proposal proposal-repair --status accepted"
        )
    );
    assert_eq!(
        envelope["active_context_commands"]["proposal_review_reject"],
        format!(
            "datum-eda proposal review {root_arg} --proposal proposal-repair --status rejected"
        )
    );
    assert_eq!(
        envelope["active_context_commands"]["proposal_defer"],
        format!("datum-eda proposal defer {root_arg} --proposal proposal-repair")
    );
    assert_eq!(
        envelope["active_context_commands"]["proposal_reject"],
        format!("datum-eda proposal reject {root_arg} --proposal proposal-repair")
    );
    assert_eq!(
        envelope["active_context_commands"]["proposal_accept_apply"],
        format!("datum-eda proposal accept-apply {root_arg} --proposal proposal-repair")
    );
    assert_eq!(
        envelope["active_context_commands"]["proposal_apply"],
        format!("datum-eda proposal apply {root_arg} --proposal proposal-repair")
    );
    assert_eq!(
        envelope["active_context_commands"]["journal_list"],
        format!("datum-eda journal list {root_arg}")
    );
    assert_eq!(envelope["accepted_transaction_tip"], transaction_tip);
    assert_eq!(
        envelope["active_context_commands"]["journal_show_tip"],
        format!("datum-eda journal show {root_arg} --transaction {transaction_tip}")
    );
    assert_eq!(
        envelope["active_context_commands"]["journal_undo"],
        format!("datum-eda journal undo {root_arg}")
    );
    assert_eq!(
        envelope["active_context_commands"]["journal_redo"],
        format!("datum-eda journal redo {root_arg}")
    );
    assert_eq!(
        envelope["active_context_commands"]["source_shards"],
        format!("datum-eda project query {root_arg} resolve-debug")
    );
    assert_eq!(
        envelope["active_context_commands"]["check_run"],
        format!("datum-eda check run {root_arg}")
    );
    assert_eq!(
        envelope["active_context_commands"]["check_list"],
        format!("datum-eda check list {root_arg}")
    );
    assert_eq!(
        envelope["active_context_commands"]["check_profiles"],
        format!("datum-eda check profiles {root_arg}")
    );
    assert_eq!(
        envelope["active_context_commands"]["check_fill_zones"],
        format!("datum-eda check fill-zones {root_arg}")
    );
    assert_eq!(
        envelope["active_context_commands"]["check_show"],
        format!("datum-eda check show {root_arg} --check-run check-run-visible")
    );
    assert_eq!(
        envelope["active_context_commands"]["check_repair_standards"],
        format!("datum-eda check repair-standards {root_arg}")
    );
    assert_eq!(
        envelope["active_context_commands"]["check_waive_finding"],
        format!(
            "datum-eda check waive {root_arg} --fingerprint 'sha256:selected-finding' --rationale '<rationale>'"
        )
    );
    assert_eq!(
        envelope["active_context_commands"]["check_accept_deviation"],
        format!(
            "datum-eda check accept-deviation {root_arg} --fingerprint 'sha256:selected-finding' --rationale '<rationale>'"
        )
    );
    assert_eq!(
        envelope["active_context_commands"]["library_list_objects"],
        format!("datum-eda query pool-library-objects {root_arg} --pool pool")
    );
    assert_eq!(
        envelope["active_context_commands"]["library_show_object"],
        format!(
            "datum-eda query pool-library-objects {root_arg} --pool pool --kind '<kind>' --object '<uuid>' --include-payload"
        )
    );
    assert_eq!(
        envelope["active_context_commands"]["project_validate_pool"],
        format!("datum-eda project validate {root_arg}")
    );
    assert_eq!(
        envelope["active_context_commands"]["project_create_pin_pad_map"],
        format!(
            "datum-eda project create-pool-pin-pad-map {root_arg} --pool pool --map '<map-uuid>' --part '<part-uuid>' --entry '<pad-uuid>:<gate-uuid>:<pin-uuid>'"
        )
    );
    assert_eq!(
        envelope["active_context_commands"]["project_set_pin_pad_map"],
        format!(
            "datum-eda project set-pool-pin-pad-map {root_arg} --pool pool --map '<map-uuid>' --mode merge --entry '<pad-uuid>:<gate-uuid>:<pin-uuid>'"
        )
    );
    assert_eq!(
        envelope["active_context_commands"]["proposal_create_pin_pad_map"],
        format!(
            "datum-eda proposal create-pool-pin-pad-map {root_arg} --pool pool --map '<map-uuid>' --part '<part-uuid>' --entry '<pad-uuid>:<gate-uuid>:<pin-uuid>' --rationale 'create PinPadMap'"
        )
    );
    assert_eq!(
        envelope["active_context_commands"]["proposal_set_pin_pad_map"],
        format!(
            "datum-eda proposal set-pool-pin-pad-map {root_arg} --pool pool --map '<map-uuid>' --mode merge --entry '<pad-uuid>:<gate-uuid>:<pin-uuid>' --rationale 'update PinPadMap'"
        )
    );
    assert_eq!(
        envelope["library_commands"]["validate_pool"],
        "datum-eda project validate \"$DATUM_PROJECT_ROOT\""
    );
    assert_eq!(
        envelope["library_commands"]["list_objects"],
        "datum-eda query pool-library-objects \"$DATUM_PROJECT_ROOT\" --pool pool"
    );
    assert_eq!(
        envelope["library_commands"]["show_object"],
        "datum-eda query pool-library-objects \"$DATUM_PROJECT_ROOT\" --pool pool --kind <kind> --object <uuid> --include-payload"
    );
    let empty_context = context_for_root(&root);
    write_terminal_context_files(&terminal_context, &empty_context)
        .expect("rewrite terminal context without active focus");
    let empty: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&terminal_context.context_path).unwrap()).unwrap();
    assert_eq!(
        empty["active_context_commands"]["artifact_list"],
        format!("datum-eda artifact list {root_arg}")
    );
    assert!(empty["active_context_commands"]["artifact_show"].is_null());
    assert!(empty["active_context_commands"]["artifact_compare"].is_null());
    assert!(empty["active_context_commands"]["output_job_generate"].is_null());
    assert!(empty["active_context_commands"]["output_job_start_run"].is_null());
    assert!(empty["active_context_commands"]["output_job_cancel_run"].is_null());
    assert_eq!(
        empty["active_context_commands"]["proposal_list"],
        format!("datum-eda proposal list {root_arg}")
    );
    assert!(empty["active_context_commands"]["proposal_show"].is_null());
    assert!(empty["active_context_commands"]["proposal_accept_apply"].is_null());
    assert_eq!(
        empty["active_context_commands"]["journal_list"],
        format!("datum-eda journal list {root_arg}")
    );
    assert_eq!(
        empty["active_context_commands"]["journal_show_tip"],
        format!("datum-eda journal show {root_arg} --transaction {transaction_tip}")
    );
    assert_eq!(
        empty["active_context_commands"]["source_shards"],
        format!("datum-eda project query {root_arg} resolve-debug")
    );
    assert_eq!(
        empty["active_context_commands"]["check_run"],
        format!("datum-eda check run {root_arg}")
    );
    assert_eq!(
        empty["active_context_commands"]["check_list"],
        format!("datum-eda check list {root_arg}")
    );
    assert_eq!(
        empty["active_context_commands"]["check_profiles"],
        format!("datum-eda check profiles {root_arg}")
    );
    assert_eq!(
        empty["active_context_commands"]["check_fill_zones"],
        format!("datum-eda check fill-zones {root_arg}")
    );
    assert!(empty["active_context_commands"]["check_show"].is_null());
    assert!(empty["active_context_commands"]["check_waive_finding"].is_null());
    assert!(empty["active_context_commands"]["check_accept_deviation"].is_null());
    assert!(empty["active_context_commands"]["library_list_objects"].is_string());
    assert!(empty["active_context_commands"]["library_show_object"].is_string());
    assert_eq!(
        empty["active_context_commands"]["project_validate_pool"],
        format!("datum-eda project validate {root_arg}")
    );
    assert!(empty["active_context_commands"]["project_create_pin_pad_map"].is_string());
    assert!(empty["active_context_commands"]["proposal_set_pin_pad_map"].is_string());
    let _ = fs::remove_dir_all(&root);
}
