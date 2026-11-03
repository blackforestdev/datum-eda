use super::*;
use datum_gui_protocol::SelectionSubject;
use datum_gui_protocol::selection_view::{InspectionValue, ScopeInspection, SelectionView};
use std::collections::BTreeMap;

fn state() -> ReviewWorkspaceState {
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    state.scene.project_uuid = "7c214888-2760-4698-a97e-67a6f2302ab0".into();
    state.scene.source_revision = "readable-revision".into();
    state.checks=datum_gui_protocol::check_run_review_state_from_json(r#"{
      "contract":"check_run_v1","persisted":false,
      "project_id":"7c214888-2760-4698-a97e-67a6f2302ab0","model_revision":"readable-revision",
      "check_run_id":"91bd1440-7546-4694-8210-e289668ec7cb","profile_id":"erc",
      "summary":{"check_run_id":"91bd1440-7546-4694-8210-e289668ec7cb","model_revision":"readable-revision"},
      "finding_count":1,"findings":[{"fingerprint":"private-fingerprint","domain":"erc","status":"active","severity":"warning",
      "rule_id":"unconnected_component_pin","message":"Component pin Q1.3 has no connected net; connect it or explicitly mark the pin as intentionally unused.",
      "explanation":"Component pin Q1.3 has no connected net; connect it or explicitly mark the pin as intentionally unused.",
      "evidence":[{"objects":[{"key":"Q1.3","kind":"pin"}]}]}] }"#).unwrap();
    state.selection = SelectionTarget::Native(Box::new(SelectionView {
        project: state.scene.project_uuid.parse().unwrap(),
        revision: datum_gui_protocol::selection_resolution::SelectionModelRevision(
            state.scene.source_revision.clone(),
        ),
        subject: SelectionSubject::Diagnostic("private-fingerprint".into()),
        members: Default::default(),
        related: Default::default(),
        scope: None,
        values: Default::default(),
        authored_anchors: Default::default(),
        inspection: vec![ScopeInspection {
            scope: None,
            fields: BTreeMap::from([(
                "artifact_basis".into(),
                InspectionValue::Common(state.checks.summary.clone()),
            )]),
            field_details: Default::default(),
            display_labels: Default::default(),
            quantities: Default::default(),
        }],
        zone_regions: Default::default(),
        board_keys: Default::default(),
        schematic_keys: Default::default(),
        schematic_documents: Default::default(),
        board_presentations: Default::default(),
        schematic_presentations: Default::default(),
        related_board_keys: Default::default(),
        related_schematic_keys: Default::default(),
        hidden: Default::default(),
        unprojected: Default::default(),
        locked: Default::default(),
        notice: None,
        dropped: Default::default(),
        stale: false,
    }));
    state
}
fn render(state: &ReviewWorkspaceState) -> (Vec<TextRun>, Vec<HitRegion>) {
    let mut text = vec![];
    let mut hits = vec![];
    super::super::inspector_dispatch::render_active_inspector(
        state,
        RectPx {
            x: 400.,
            y: 80.,
            width: 240.,
            height: 500.,
        },
        &mut vec![],
        &mut text,
        &mut hits,
    )
    .unwrap();
    (text, hits)
}
#[test]
fn erc_readable_inspector_native_dispatch_wraps_without_generic_ids_or_duplicate_message() {
    let mut state = state();
    let (text, hits) = render(&state);
    let content = text
        .iter()
        .map(|t| t.text.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    assert!(content.contains("Check finding"));
    assert!(content.contains("Q1.3"));
    assert!(content.contains("read-only report"));
    assert!(!content.contains("private-fingerprint"));
    assert!(!content.contains("Diagnostic — 0"));
    assert_eq!(content.matches("Component pin").count(), 1);
    let (total, visible) = hits
        .iter()
        .find_map(|h| {
            if let HitTarget::SelectionInspectorScroll { total, visible, .. } = &h.target {
                Some((*total, *visible))
            } else {
                None
            }
        })
        .unwrap();
    assert!(
        total > 0 && total < 40,
        "no inherited generic artifact row inventory"
    );
    assert!(visible > 0);
    state.ui.selection_inspector.inventory_expanded = true;
    state.ui.selection_inspector.scroll_offset = usize::MAX;
    let (text, hits) = render(&state);
    assert!(
        text.iter().any(|t| t.text.contains("private-fingerprint")),
        "clamped scroll reaches actual expanded detail rows"
    );
    let expanded = hits
        .iter()
        .find_map(|h| {
            if let HitTarget::SelectionInspectorScroll { total, .. } = &h.target {
                Some(*total)
            } else {
                None
            }
        })
        .unwrap();
    assert!(expanded > total);
    assert!(
        text.iter().all(|t| t.clip_bounds.is_some()),
        "shared dispatch clips every row"
    );
}
#[test]
fn erc_readable_inspector_respects_higher_priority_owners_and_unavailable_report() {
    let mut state = state();
    state.checks.check_run_id = Some(uuid::Uuid::new_v4().to_string());
    let (text, _) = render(&state);
    assert!(text.iter().any(|t| t.text.contains("Unavailable")));
    assert!(!text.iter().any(|t| t.text.contains("Component pin")));
    state.ui.layout.open_beside_root(
        datum_gui_protocol::PaneContent::Library,
        datum_gui_protocol::SplitOrientation::Vertical,
        0.5,
        true,
    );
    let (text, _) = render(&state);
    assert!(!text.iter().any(|t| t.text == "Check finding"));
    state.ui.layout.open_beside_root(
        datum_gui_protocol::PaneContent::Revision(datum_gui_protocol::RevisionPane::Surface(
            datum_gui_protocol::RevisionSurface::Evidence,
        )),
        datum_gui_protocol::SplitOrientation::Vertical,
        0.5,
        true,
    );
    let (text, _) = render(&state);
    assert!(!text.iter().any(|t| t.text == "Check finding"));
    state.inspector_context.owner =
        datum_gui_protocol::inspector_context::InspectorOwner::Navigator;
    let (text, _) = render(&state);
    assert!(!text.iter().any(|t| t.text == "Check finding"));
}
