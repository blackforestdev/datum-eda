use super::*;
use crate::native_selection_overlay::tests::selected_state;

fn checks(state: &mut ReviewWorkspaceState) {
    state.checks.project_id = Some(state.scene.project_uuid.clone());
    state.checks.model_revision = Some(state.scene.source_revision.clone());
    state.checks.findings = datum_gui_protocol::check_run_review_state_from_json(&format!(r#"{{"findings":[{{"fingerprint":"sha256:actual-producing-finding","domain":"drc","severity":"warning","primary_target":{{"kind":"object_uuid","id":"{}"}},"payload":{{"location":{{"x_nm":5000000,"y_nm":0}}}}}}]}}"#,state.scene.tracks[0].source_object_uuid)).unwrap().findings;
}
#[test]
fn marker_severity_and_source_identity_survive_selection_and_pane_clipping() {
    let (fixture, mut state) = selected_state();
    let source_bytes = fixture.source_bytes();
    checks(&mut state);
    let viewport = RectPx {
        x: 10.0,
        y: 20.0,
        width: 500.0,
        height: 400.0,
    };
    let camera = CameraState::fit_to_bounds(&state.scene.bounds);
    let mut core = Vec::new();
    let mut hits = Vec::new();
    push(&mut core, &mut hits, &state.scene, &state, viewport, camera);
    assert_eq!(hits.len(), 1);
    assert_eq!(
        hits[0].target,
        HitTarget::CheckFinding(state.checks.findings[0].fingerprint.clone())
    );
    if let SelectionTarget::Native(view) = &mut state.selection {
        view.subject = SelectionSubject::Diagnostic(state.checks.findings[0].fingerprint.clone());
    }
    let mut selected = Vec::new();
    let mut selected_hits = Vec::new();
    push(
        &mut selected,
        &mut selected_hits,
        &state.scene,
        &state,
        viewport,
        camera,
    );
    assert_eq!(selected_hits, hits);
    assert_eq!(&selected[3..], core.as_slice()); // semantic core bytes never recolored
    assert!(selected[..3].iter().all(|q| q.color == TEXT_ACCENT));
    assert!(
        selected
            .iter()
            .flat_map(|q| q.points)
            .all(|(x, y)| viewport.contains(x, y))
    );
    if let SelectionTarget::Native(view) = &mut state.selection {
        view.stale = true;
    }
    let mut stale = Vec::new();
    push(
        &mut stale,
        &mut Vec::new(),
        &state.scene,
        &state,
        viewport,
        camera,
    );
    assert_eq!(stale, core);
    for (path, bytes) in source_bytes {
        assert_eq!(std::fs::read(path).unwrap(), bytes);
    }
}
#[test]
fn marker_projection_uses_producing_location_and_refuses_foreign_revision_occurrence_or_ambiguity()
{
    let (_fixture, mut state) = selected_state();
    checks(&mut state);
    let anchor = |state: &ReviewWorkspaceState| {
        marker::MarkerProjectionIndex::capture(&state.scene, &state.checks, |_| true)
            .anchor(&state.checks.findings[0])
    };
    assert_eq!(anchor(&state), Ok(PointNm { x: 5_000_000, y: 0 }));
    state.checks.findings[0].payload = Default::default();
    assert_eq!(anchor(&state), Ok(PointNm { x: 0, y: 0 }));
    let primary = state.checks.findings[0].primary_target.clone();
    state.checks.findings[0].primary_target = datum_gui_protocol::check_run_review_state_from_json(
        r#"{"findings":[{"primary_target":{"instance_path":["non-root-occurrence"]}}]}"#,
    )
    .unwrap()
    .findings
    .remove(0)
    .primary_target;
    assert!(anchor(&state).is_err());
    state.checks.findings[0].primary_target = primary;
    state.scene.tracks.push(state.scene.tracks[0].clone());
    assert!(anchor(&state).is_err());
    state.scene.tracks.pop();
    state.checks.findings.push(state.checks.findings[0].clone());
    assert!(anchor(&state).is_err());
    state.checks.findings.pop();
    state.checks.model_revision = Some("obsolete".into());
    assert!(anchor(&state).is_err());
    state.checks.model_revision = Some(state.scene.source_revision.clone());
    state.checks.project_id = Some(uuid::Uuid::new_v4().to_string());
    assert!(anchor(&state).is_err());
}
#[test]
fn root_schematic_marker_preserves_domain_and_hidden_navigation_is_unavailable() {
    let (_fixture, mut state) = selected_state();
    checks(&mut state);
    state.scene.scene_id = "schematic-root-projection".into();
    state.checks.findings[0].domain = "erc".into();
    state.checks.findings[0].payload = Default::default();
    let projected = marker::MarkerProjectionIndex::capture(&state.scene, &state.checks, |_| true);
    assert!(projected.anchor(&state.checks.findings[0]).is_ok());
    let hidden = marker::MarkerProjectionIndex::capture(&state.scene, &state.checks, |_| false);
    assert!(hidden.anchor(&state.checks.findings[0]).is_err());
    state.checks.findings[0].domain = "drc".into();
    assert!(projected.anchor(&state.checks.findings[0]).is_err());
}
