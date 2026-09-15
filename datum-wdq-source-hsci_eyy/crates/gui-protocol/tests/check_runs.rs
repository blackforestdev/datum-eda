use serde_json::json;

#[test]
fn check_run_v1_payload_maps_to_review_state() {
    let state = datum_gui_protocol::check_run_review_state_from_json(
        r#"{
          "contract": "check_run_v1",
          "persisted": true,
          "check_run_id": "run-current",
          "project_id": "project-a",
          "model_revision": "revision-a",
          "profile_id": "native-combined",
          "status": "warning",
          "summary": {"status": "warning"},
          "finding_count": 1,
          "proposal_refs": ["proposal-a"],
          "proposal_links": [{
            "proposal_id": "proposal-a",
            "status": "draft"
          }],
          "profile_basis": {
            "profile_id": "native-combined",
            "domains": ["relationships", "erc", "drc", "standards", "manufacturing"],
            "description": "Combined profile"
          },
          "coverage": [{
            "domain": "erc",
            "rule_id": "schematic_connectivity",
            "status": "evaluated",
            "target_scope": "schematic",
            "basis_id": "datum.check.coverage.erc.schematic_connectivity.v1",
            "rule_revision": "v1"
          }],
          "findings": [{
            "finding_id": "finding-a",
            "index": 0,
            "source": "erc",
            "code": "unconnected_component_pin",
            "severity": "warning",
            "fingerprint": "sha256:finding-a",
            "domain": "erc",
            "rule_id": "unconnected_component_pin",
            "standards_basis": null,
            "rule_revision": null,
            "import_key": null,
            "status": "active",
            "primary_target": {"kind": "pin", "object_id": "pin-a"},
            "related_targets": [{"kind": "net", "object_id": "net-a"}],
            "message": "Pin is unconnected",
            "explanation": "Rule unconnected_component_pin produced this finding.",
            "suggested_next_action": "Fix, waive, or accept this finding.",
            "evidence": [{"net": "N/C"}],
            "payload": {"component": "U1", "pin": "1"},
            "proposal_refs": ["proposal-a"],
            "proposal_links": [{
              "proposal_id": "proposal-a",
              "matched_fingerprint": "sha256:finding-a"
            }],
            "waiver_refs": [],
            "deviation_refs": []
          }]
        }"#,
    )
    .expect("check_run_v1 should decode");

    assert_eq!(state.check_run_id.as_deref(), Some("run-current"));
    assert_eq!(state.project_id.as_deref(), Some("project-a"));
    assert_eq!(state.model_revision.as_deref(), Some("revision-a"));
    assert_eq!(state.profile_id.as_deref(), Some("native-combined"));
    assert_eq!(state.status.as_deref(), Some("warning"));
    assert!(state.persisted);
    assert_eq!(state.finding_count, 1);
    assert_eq!(state.proposal_links[0]["proposal_id"], "proposal-a");
    assert_eq!(state.profile_basis.profile_id, "native-combined");
    assert_eq!(
        state.profile_basis.domains,
        vec!["relationships", "erc", "drc", "standards", "manufacturing"]
    );
    assert_eq!(state.coverage[0].domain, "erc");
    assert_eq!(state.coverage[0].rule_id, "schematic_connectivity");
    assert_eq!(state.coverage[0].status, "evaluated");

    let finding = state.findings.first().expect("finding summary");
    assert_eq!(finding.finding_id.as_deref(), Some("finding-a"));
    assert_eq!(finding.severity, "warning");
    assert_eq!(finding.fingerprint, "sha256:finding-a");
    assert_eq!(finding.domain, "erc");
    assert_eq!(finding.rule_id, "unconnected_component_pin");
    assert_eq!(finding.standards_basis, None);
    assert_eq!(finding.rule_revision, None);
    assert_eq!(finding.import_key, None);
    assert_eq!(finding.status, "active");
    assert!(finding.explanation.contains("unconnected_component_pin"));
    assert_eq!(
        finding.suggested_next_action.as_deref(),
        Some("Fix, waive, or accept this finding.")
    );
    assert_eq!(
        finding.proposal_links[0]["matched_fingerprint"],
        "sha256:finding-a"
    );
}

#[test]
fn check_finding_labels_expose_target_and_standards_basis() {
    let finding = datum_gui_protocol::CheckFindingSummary {
        fingerprint: "sha256:finding-a".to_string(),
        standards_basis: Some("IPC-7351B".to_string()),
        primary_target: json!({
            "kind": "pad_uuid",
            "id": "00000000-0000-0000-0000-00000000pad1"
        }),
        ..datum_gui_protocol::CheckFindingSummary::default()
    };

    assert_eq!(
        finding.target_label(),
        Some("pad_uuid:00000000-0000-0000-0000-00000000pad1".to_string())
    );
    assert_eq!(
        finding.standards_basis_label(),
        Some("IPC-7351B".to_string())
    );
}

#[test]
fn check_finding_standards_basis_label_falls_back_to_evidence() {
    let finding = datum_gui_protocol::CheckFindingSummary {
        fingerprint: "sha256:finding-a".to_string(),
        evidence: vec![json!({
            "evidence_kind": "standards_basis",
            "basis_id": "DATUM-PROCESS-APERTURE-V1"
        })],
        ..datum_gui_protocol::CheckFindingSummary::default()
    };

    assert_eq!(
        finding.standards_basis_label(),
        Some("DATUM-PROCESS-APERTURE-V1".to_string())
    );
}

#[test]
fn check_run_record_v1_uses_inner_run_and_outer_identity_defaults() {
    let state = datum_gui_protocol::check_run_review_state_from_json(
        r#"{
          "contract": "check_run_record_v1",
          "project_id": "project-from-record",
          "model_revision": "revision-from-record",
          "check_run": {
            "check_run_id": "run-persisted",
            "profile_id": "standards",
            "status": "error",
            "findings": [{
              "id": "legacy-finding-id",
              "severity": "error",
              "fingerprint": "sha256:process-aperture",
              "domain": "standards",
              "rule_id": "process_aperture_policy",
              "standards_basis": "datum.process_aperture_and_geometry.current",
              "rule_revision": "v1",
              "import_key": "kicad:board:/pads/0",
              "status": "active",
              "explanation": "Process aperture rule failed.",
              "suggested_next_action": null,
              "proposal_links": [{"proposal_id": "repair-a"}]
            }],
            "proposal_links": [{"proposal_id": "repair-a"}]
            ,"coverage": [{
              "domain": "standards",
              "rule_id": "process_aperture_policy",
              "status": "evaluated",
              "target_scope": "board_pads_tracks_vias",
              "standards_basis": "datum.process_aperture_and_geometry.current"
            }]
          }
        }"#,
    )
    .expect("check_run_record_v1 should decode");

    assert_eq!(state.check_run_id.as_deref(), Some("run-persisted"));
    assert_eq!(state.project_id.as_deref(), Some("project-from-record"));
    assert_eq!(
        state.model_revision.as_deref(),
        Some("revision-from-record")
    );
    assert_eq!(state.profile_id.as_deref(), Some("standards"));
    assert_eq!(state.status.as_deref(), Some("error"));
    assert_eq!(state.finding_count, 1);
    assert_eq!(state.proposal_links[0]["proposal_id"], "repair-a");
    assert_eq!(
        state.findings[0].finding_id.as_deref(),
        Some("legacy-finding-id")
    );
    assert_eq!(state.findings[0].rule_id, "process_aperture_policy");
    assert_eq!(state.findings[0].domain, "standards");
    assert_eq!(
        state.findings[0].standards_basis.as_deref(),
        Some("datum.process_aperture_and_geometry.current")
    );
    assert_eq!(state.findings[0].rule_revision.as_deref(), Some("v1"));
    assert_eq!(
        state.findings[0].import_key.as_deref(),
        Some("kicad:board:/pads/0")
    );
    assert_eq!(state.findings[0].suggested_next_action, None);
    assert_eq!(
        state.coverage[0].standards_basis.as_deref(),
        Some("datum.process_aperture_and_geometry.current")
    );
}

#[test]
fn review_workspace_state_defaults_check_run_surface_empty() {
    let state = datum_gui_protocol::load_fixture_workspace_state();

    assert_eq!(
        state.checks,
        datum_gui_protocol::CheckRunReviewState::default()
    );
    assert!(state.checks.findings.is_empty());
    assert!(state.checks.proposal_links.is_empty());
}
