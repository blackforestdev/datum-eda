use super::*;

fn prepared() -> PreparedScene {
    let mut state = crate::gpu_surface_pass::board_fixture_state();
    state.selection = datum_gui_protocol::SelectionTarget::None;
    let retained = crate::RetainedScene::from_workspace_for_surface(&state, 1344, 806, 1.0);
    PreparedScene::from_workspace_for_surface(
        &state,
        1344,
        806,
        1.0,
        crate::CameraState::fit_to_bounds(&state.scene.bounds),
        &retained,
    )
    .unwrap()
}

#[test]
fn native_content_retains_actual_wrong_none_text_not_expected_selection_labels() {
    let prepared = prepared();
    let observed = prepared.native_content_text();
    assert!(
        observed
            .iter()
            .any(|r| r.region == "inspector" && r.text == "No selection")
    );
    assert!(
        observed
            .iter()
            .any(|r| r.region == "footer" && r.text == "none")
    );
}

#[test]
fn native_content_excludes_terminal_and_retains_exact_rich_spans_and_clip() {
    let mut prepared = prepared();
    let original = prepared
        .text_runs
        .iter()
        .find(|r| r.text == "No selection")
        .unwrap()
        .clone();
    let mut private = original.clone();
    private.origin = TextOrigin::TerminalLeaf(0);
    private.text = "must remain outside host observation".into();
    prepared.text_runs.push(private);
    let run = prepared
        .text_runs
        .iter_mut()
        .find(|r| r.text == "No selection")
        .unwrap();
    run.rich_spans = vec![TextRunSpan {
        text: "actual shaped span".into(),
        color: [0.1, 0.2, 0.3],
        bold: true,
        italic: false,
    }];
    run.clip_bounds = Some(RectPx {
        x: 1060.0,
        y: 75.0,
        width: 200.0,
        height: 20.0,
    });
    let observed = prepared.native_content_text();
    assert!(!observed.iter().any(|r| r.text.contains("must remain")));
    let row = observed.iter().find(|r| r.text == "No selection").unwrap();
    assert_eq!(row.spans[0].text, "actual shaped span");
    assert_eq!(row.clip, Some([1060.0, 75.0, 200.0, 20.0]));
}
