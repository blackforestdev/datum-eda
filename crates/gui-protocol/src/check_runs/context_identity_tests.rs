use super::*;
use crate::selection_menu::{Command, SelectionMenu};

const PROJECT: &str = "f5c84dbb-dafc-4caf-ba86-b3b6c894651b";
const FINGERPRINT: &str = "sha256:260611ced8897cecba4f669ef163ded751b6644af9e32efa7c0a74294e60ab29";

fn context() -> Value {
    serde_json::json!({
        "project_id": PROJECT,
        "model_revision": "682aaa9b99c87077f85c00f31db98508b9c8c21cac41150719664db0585e31c2",
        "check_context": {
            "contract": "datum_check_context_v1",
            "visible_check_runs": [{
                "check_run_id": "2f808923-15a2-5910-b40e-b79052754aff",
                "model_revision": "682aaa9b99c87077f85c00f31db98508b9c8c21cac41150719664db0585e31c2",
                "profile_id": "native-combined",
                "status": "error",
                "finding_count": 1,
                "active_findings": [{
                    "fingerprint": FINGERPRINT,
                    "domain": "drc",
                    "rule_id": "connectivity_unrouted_net",
                    "severity": "error",
                    "primary_target": {"kind": "object_uuid", "id": "00000000-0000-0000-0000-00000000c200"},
                    "payload": {"location": null}
                }]
            }]
        }
    })
}

fn menu() -> SelectionMenu {
    SelectionMenu {
        project: PROJECT.parse().unwrap(),
        revision: context()["model_revision"].as_str().unwrap().into(),
        schematic: false,
        source_scene: "current-native-board".into(),
        filters: crate::WorkspaceFilterState {
            show_authored: true,
            excluded_selection_classes: Default::default(),
            show_proposed: true,
            show_unrouted: true,
            dim_unrelated: false,
            active_layer_id: None,
            layer_visibility: Default::default(),
            layer_scroll_offset: 0,
        },
        inspection_subject: None,
        items: Vec::new(),
        open: false,
        focus: 0,
        preview: None,
        preview_keys: Default::default(),
        scroll: 0,
    }
}

#[test]
fn producer_context_identity_admits_exact_checks_leaf_and_preserves_guards() {
    let actual_shape = context();
    let state = check_run_review_state_from_context_value(&actual_shape).unwrap();
    assert_eq!(state.project_id.as_deref(), Some(PROJECT));
    assert_eq!(
        state.model_revision,
        actual_shape["check_context"]["visible_check_runs"][0]["model_revision"]
            .as_str()
            .map(str::to_owned)
    );
    let mut current = menu();
    current.append_checks(&state);
    assert_eq!(current.items.len(), 1);
    assert_eq!(
        current.items[0].command,
        Command::Diagnostic(FINGERPRINT.into())
    );
    assert!(current.items[0].unavailable.is_none());

    let mut foreign = actual_shape.clone();
    foreign["project_id"] = serde_json::json!("70000000-0000-4000-8000-000000000001");
    let foreign = check_run_review_state_from_context_value(&foreign).unwrap();
    assert_ne!(foreign.project_id, state.project_id);
    let mut current = menu();
    current.append_checks(&foreign);
    assert!(
        current.items.is_empty(),
        "Foreign producer must not gain current Project authority"
    );

    let mut absent = actual_shape.clone();
    absent.as_object_mut().unwrap().remove("project_id");
    assert!(check_run_review_state_from_context_value(&absent).is_none());
    let run = &actual_shape["check_context"]["visible_check_runs"][0];
    let record = serde_json::json!({
        "contract": "check_run_record_v1",
        "project_id": PROJECT,
        "model_revision": actual_shape["model_revision"],
        "check_run": {
            "check_run_id": run["check_run_id"],
            "model_revision": run["model_revision"],
            "profile_id": run["profile_id"],
            "status": run["status"],
            "finding_count": run["finding_count"],
            "findings": run["active_findings"]
        }
    });
    let fallback = check_run_review_state_from_json(&record.to_string()).unwrap();
    assert_eq!(fallback.project_id, state.project_id);
    assert_eq!(fallback.model_revision, state.model_revision);
    let mut fallback_menu = menu();
    fallback_menu.append_checks(&fallback);
    assert_eq!(
        fallback_menu.items[0].command,
        Command::Diagnostic(FINGERPRINT.into())
    );

    for invalid in [Value::Null, serde_json::json!("not-a-project-uuid")] {
        let mut missing = actual_shape.clone();
        missing["project_id"] = invalid;
        assert!(
            check_run_review_state_from_context_value(&missing).is_none(),
            "Missing identity must use the established qualified history fallback"
        );
    }
}

#[test]
fn context_current_revision_wins_over_history_order_without_stale_resurrection() {
    let mut value = context();
    let current = value["check_context"]["visible_check_runs"][0].clone();
    let mut stale = current.clone();
    stale["model_revision"] =
        serde_json::json!("ddb62e102b14ebf1cb74a161c3db3047f8514b14e85f45b932bdcf7098dc7772");
    stale["check_run_id"] = serde_json::json!("18a1dc99-8260-5405-a088-95b0485c2a8f");
    stale["finding_count"] = serde_json::json!(0);
    stale["active_findings"] = serde_json::json!([]);
    value["check_context"]["visible_check_runs"] =
        serde_json::json!([stale.clone(), current.clone()]);
    let selected = check_run_review_state_from_context_value(&value).unwrap();
    assert_eq!(
        selected.check_run_id,
        current["check_run_id"].as_str().map(str::to_owned)
    );
    assert_eq!(selected.findings[0].fingerprint, FINGERPRINT);

    // A current evaluated zero-result run must not reuse older findings.
    value["model_revision"] = stale["model_revision"].clone();
    value["check_context"]["visible_check_runs"] = serde_json::json!([current, stale.clone()]);
    let cleared = check_run_review_state_from_context_value(&value).unwrap();
    assert_eq!(
        cleared.check_run_id,
        stale["check_run_id"].as_str().map(str::to_owned)
    );
    assert!(cleared.findings.is_empty());
    let mut current_menu = menu();
    current_menu.append_checks(&cleared);
    assert!(current_menu.items.is_empty());

    // A supplied basis with no eligible run must suppress stale history fallback.
    value["model_revision"] = serde_json::json!("no-matching-producer-revision");
    let empty = check_run_review_state_from_context_value(&value).unwrap();
    assert_eq!(empty.project_id.as_deref(), Some(PROJECT));
    assert_eq!(
        empty.model_revision.as_deref(),
        Some("no-matching-producer-revision")
    );
    assert!(empty.check_run_id.is_none());
    assert!(empty.status.is_none());
    assert!(empty.profile_id.is_none());
    assert!(empty.findings.is_empty());
    assert_eq!(empty.finding_count, 0);
    let mut current_menu = menu();
    current_menu.append_checks(&empty);
    assert!(current_menu.items.is_empty());

    // An unqualified revision must not grant historical findings current
    // authority or manufacture a revision from the request/canvas.
    value.as_object_mut().unwrap().remove("model_revision");
    let absent = check_run_review_state_from_context_value(&value).unwrap();
    assert_eq!(absent.project_id.as_deref(), Some(PROJECT));
    assert!(absent.model_revision.is_none());
    assert!(absent.check_run_id.is_none());
    assert!(absent.status.is_none());
    assert!(absent.findings.is_empty());
    let mut current_menu = menu();
    current_menu.append_checks(&absent);
    assert!(current_menu.items.is_empty());
    for invalid in [Value::Null, serde_json::json!(0), serde_json::json!("")] {
        value["model_revision"] = invalid;
        let empty = check_run_review_state_from_context_value(&value).unwrap();
        assert!(empty.model_revision.is_none());
        assert!(empty.check_run_id.is_none());
        assert!(empty.status.is_none());
        assert!(empty.findings.is_empty());
    }
}
