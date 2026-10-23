//! M7-INT-001 authored-object selection ownership regression coverage.
//!
//! These tests prove selection ownership stability on the canonical
//! `datum-test` fixture: selecting one component must change retained
//! world geometry only inside that component's pad bounds, switching
//! selection must clear the prior owner, and hover must stay preview-only.
//!
//! Assertions are lane-aware (vertex-state diffs against an unselected
//! baseline), not exact-color-token checks.

use std::path::PathBuf;

use datum_gui_protocol::{
    HoverTarget, LiveReviewRequest, PaneContent, RectNm, ReviewWorkspaceState, SelectionTarget,
};
use datum_gui_render::{RetainedScene, Vertex};

fn datum_test_request() -> LiveReviewRequest {
    LiveReviewRequest {
        project_root: PathBuf::from("/home/bfadmin/Documents/kicad_projects/Datum-eda/datum-test"),
        board_file: Some(PathBuf::from(
            "/home/bfadmin/Documents/kicad_projects/Datum-eda/datum-test/datum-test.kicad_pcb",
        )),
        artifact_path: None,
        net_uuid: None,
        from_anchor_pad_uuid: None,
        to_anchor_pad_uuid: None,
        profile: None,
        kicad_board_source: None,
    }
}

fn load_datum_test_state() -> ReviewWorkspaceState {
    datum_gui_protocol::load_board_editor_workspace_state(&datum_test_request())
        .expect("datum-test workspace should load")
}

fn component_object_and_uuid(state: &ReviewWorkspaceState, reference: &str) -> (String, String) {
    state
        .scene
        .components
        .iter()
        .find(|component| component.reference == reference)
        .map(|component| {
            (
                component.object_id.clone(),
                component.component_uuid.clone(),
            )
        })
        .unwrap_or_else(|| panic!("{reference} should exist"))
}

fn pad_bounds_for_component(state: &ReviewWorkspaceState, component_uuid: &str) -> Vec<RectNm> {
    state
        .scene
        .pads
        .iter()
        .filter(|pad| pad.component_uuid == component_uuid)
        .map(|pad| pad.bounds)
        .collect()
}

fn changed_vertex_count_in_pad_bounds(
    before: &[Vertex],
    after: &[Vertex],
    pad_bounds: &[RectNm],
) -> usize {
    before
        .iter()
        .zip(after.iter())
        .filter(|(before, _)| {
            pad_bounds.iter().any(|pad| {
                before.pos[0] >= pad.min_x as f32
                    && before.pos[0] <= pad.max_x as f32
                    && before.pos[1] >= pad.min_y as f32
                    && before.pos[1] <= pad.max_y as f32
            })
        })
        .filter(|(before, after)| before.color != after.color)
        .count()
}

#[test]
fn datum_test_q3_selection_does_not_select_c1_pads() {
    let mut state = load_datum_test_state();
    let (q3_object_id, q3_component_uuid) = component_object_and_uuid(&state, "Q3");
    let (_, c1_component_uuid) = component_object_and_uuid(&state, "C1");
    let baseline = RetainedScene::from_workspace(&state, 1280, 800);
    state.select_authored_object(&q3_object_id);
    let retained = RetainedScene::from_workspace(&state, 1280, 800);
    let q3_pad_bounds = pad_bounds_for_component(&state, &q3_component_uuid);
    let c1_pad_bounds = pad_bounds_for_component(&state, &c1_component_uuid);
    let changed_in_q3 = changed_vertex_count_in_pad_bounds(
        baseline.world_vertices(),
        retained.world_vertices(),
        &q3_pad_bounds,
    );
    let changed_in_c1 = changed_vertex_count_in_pad_bounds(
        baseline.world_vertices(),
        retained.world_vertices(),
        &c1_pad_bounds,
    );
    assert!(
        changed_in_q3 > 0,
        "selecting Q3 should change retained vertex state inside Q3 pad bounds"
    );
    assert_eq!(
        changed_in_c1, 0,
        "selecting Q3 must not change retained vertex state inside C1 pad bounds"
    );
}

#[test]
fn datum_test_q2_selection_does_not_select_q1_pads() {
    let mut state = load_datum_test_state();
    let (q2_object_id, q2_component_uuid) = component_object_and_uuid(&state, "Q2");
    let (_, q1_component_uuid) = component_object_and_uuid(&state, "Q1");
    let baseline = RetainedScene::from_workspace(&state, 1280, 800);
    state.select_authored_object(&q2_object_id);
    let retained = RetainedScene::from_workspace(&state, 1280, 800);
    let q2_pad_bounds = pad_bounds_for_component(&state, &q2_component_uuid);
    let q1_pad_bounds = pad_bounds_for_component(&state, &q1_component_uuid);
    let changed_in_q2 = changed_vertex_count_in_pad_bounds(
        baseline.world_vertices(),
        retained.world_vertices(),
        &q2_pad_bounds,
    );
    let changed_in_q1 = changed_vertex_count_in_pad_bounds(
        baseline.world_vertices(),
        retained.world_vertices(),
        &q1_pad_bounds,
    );
    assert!(
        changed_in_q2 > 0,
        "selecting Q2 should change retained vertex state inside Q2 pad bounds"
    );
    assert_eq!(
        changed_in_q1, 0,
        "selecting Q2 must not change retained vertex state inside Q1 pad bounds"
    );
}

#[test]
fn datum_test_switching_q1_to_q2_rebuilds_selected_geometry() {
    let mut state = load_datum_test_state();
    let (q1_object_id, q1_component_uuid) = component_object_and_uuid(&state, "Q1");
    let (q2_object_id, q2_component_uuid) = component_object_and_uuid(&state, "Q2");
    let q1_pad_bounds = pad_bounds_for_component(&state, &q1_component_uuid);
    let q2_pad_bounds = pad_bounds_for_component(&state, &q2_component_uuid);

    state.select_authored_object(&q1_object_id);
    let _q1_selected = RetainedScene::from_workspace(&state, 1280, 800);

    let baseline = RetainedScene::from_workspace(&load_datum_test_state(), 1280, 800);
    state.select_authored_object(&q2_object_id);
    let second = RetainedScene::from_workspace(&state, 1280, 800);

    let changed_in_q1 = changed_vertex_count_in_pad_bounds(
        baseline.world_vertices(),
        second.world_vertices(),
        &q1_pad_bounds,
    );
    let changed_in_q2 = changed_vertex_count_in_pad_bounds(
        baseline.world_vertices(),
        second.world_vertices(),
        &q2_pad_bounds,
    );

    assert_eq!(
        changed_in_q1, 0,
        "after switching to Q2, retained vertex state inside Q1 pad bounds should return to baseline"
    );
    assert!(
        changed_in_q2 > 0,
        "switching from Q1 to Q2 must change retained vertex state inside Q2 pad bounds"
    );
}

#[test]
fn datum_test_q2_selection_emits_selected_geometry_only_in_q2_pad_bounds() {
    // Lane-aware replacement for the retired color-exact selected-pad test:
    // selecting Q2 must visibly change geometry inside Q2 pad bounds and must
    // leave every other component's pad bounds at baseline vertex state.
    let mut state = load_datum_test_state();
    let (q2_object_id, q2_component_uuid) = component_object_and_uuid(&state, "Q2");
    let baseline = RetainedScene::from_workspace(&state, 1280, 800);
    state.select_authored_object(&q2_object_id);
    let retained = RetainedScene::from_workspace(&state, 1280, 800);

    let q2_pad_bounds = pad_bounds_for_component(&state, &q2_component_uuid);
    let changed_in_q2 = changed_vertex_count_in_pad_bounds(
        baseline.world_vertices(),
        retained.world_vertices(),
        &q2_pad_bounds,
    );
    assert!(
        changed_in_q2 > 0,
        "selecting Q2 should emit changed vertex state inside Q2 pad bounds"
    );

    for component in &state.scene.components {
        if component.component_uuid == q2_component_uuid {
            continue;
        }
        let other_pad_bounds = pad_bounds_for_component(&state, &component.component_uuid);
        if other_pad_bounds.is_empty() {
            continue;
        }
        let changed_in_other = changed_vertex_count_in_pad_bounds(
            baseline.world_vertices(),
            retained.world_vertices(),
            &other_pad_bounds,
        );
        assert_eq!(
            changed_in_other, 0,
            "selecting Q2 must not change retained vertex state inside {} pad bounds",
            component.reference
        );
    }
}

#[test]
fn datum_test_hover_is_preview_only_and_does_not_overwrite_selection() {
    let mut state = load_datum_test_state();
    let (q2_object_id, _) = component_object_and_uuid(&state, "Q2");
    let (c1_object_id, _) = component_object_and_uuid(&state, "C1");

    state.select_authored_object(&q2_object_id);
    let selected_only = RetainedScene::from_workspace(&state, 1280, 800);

    state.ui.hovered_object = Some(HoverTarget {
        object_id: c1_object_id,
        surface: PaneContent::Board,
    });
    let selected_with_hover = RetainedScene::from_workspace(&state, 1280, 800);

    assert_eq!(
        state.selection,
        SelectionTarget::AuthoredObject(q2_object_id.clone()),
        "hovering another object must not replace the explicit authored selection"
    );
    assert_eq!(
        selected_only.world_vertices(),
        selected_with_hover.world_vertices(),
        "with an explicit selection active, hover must not change retained world geometry"
    );
}

/// Supplemental production-renderer goldens over the admitted canonical input.
/// Ordinary native acquisition and public identity proof remain separate.
#[cfg(feature = "visual")]
#[test]
#[ignore = "requires the admitted crossprobe Project, revision, Net and reviewed goldens"]
fn native_crossprobe_same_engine_goldens() -> anyhow::Result<()> {
    use datum_gui_protocol::{
        AuthoredSelectionClass as Class, AuthoredSelectionIdentity as Id, CompoundSelection,
        SelectionSubject as Subject, WorkspacePreset,
        selection_artifacts::build_workspace_selection_view,
        selection_resolution::NativeSelectionResolution,
    };
    let root = PathBuf::from(std::env::var("DATUM_CROSSPROBE_PROJECT")?);
    let output = PathBuf::from(std::env::var("DATUM_CROSSPROBE_VISUAL_OUTPUT")?);
    let revision = std::env::var("DATUM_CROSSPROBE_REVISION")?;
    let net = std::env::var("DATUM_CROSSPROBE_NET")?.parse()?;
    let candidate_only = std::env::var("DATUM_CROSSPROBE_CANDIDATE_ONLY").as_deref() == Ok("1");
    std::fs::create_dir_all(&output)?;
    let mut request = datum_test_request();
    request.project_root = root;
    request.board_file = None;
    let (mut state, model) = datum_gui_protocol::load_board_editor_workspace_with_model(&request)?;
    anyhow::ensure!(
        model.model_revision.0 == revision,
        "admitted revision changed"
    );
    state
        .ui
        .layout
        .apply_preset(WorkspacePreset::BoardSchematic);
    let authority = NativeSelectionResolution::capture(&model)
        .map_err(|e| anyhow::anyhow!("canonical selection authority: {e:?}"))?;
    let identity = |class, suffix| -> anyhow::Result<Id> {
        Ok(Id {
            class,
            id: format!("78000000-0000-4000-8000-{suffix:012x}").parse()?,
            instance_path: vec![],
        })
    };
    let symbol = identity(Class::Symbol, 1)?;
    let footprint = identity(Class::Footprint, 2)?;
    let cases = [
        ("bound-symbol", Subject::Object(symbol.clone()), 1, 1),
        ("bound-footprint", Subject::Object(footprint.clone()), 1, 1),
        ("global-net", Subject::GlobalNet(net), 6, 2),
        (
            "unlinked-symbol",
            Subject::Object(identity(Class::Symbol, 5)?),
            1,
            0,
        ),
        (
            "explicit-parent-pair",
            Subject::Compound(
                CompoundSelection::new([symbol, footprint.clone()], Some(footprint))
                    .map_err(|e| anyhow::anyhow!("parent pair: {e:?}"))?,
            ),
            2,
            0,
        ),
    ];
    let golden_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("testdata/golden/crossprobe");
    let mut renderer = datum_gui_render::visual_capture::OffscreenRenderer::new(1344, 806)?;
    for (name, subject, members, related) in cases {
        let view = build_workspace_selection_view(&model, &authority, subject, &state)
            .map_err(|e| anyhow::anyhow!("{name} projection: {e:?}"))?;
        assert_eq!(view.members.len(), members, "{name} complete membership");
        assert_eq!(
            view.related.len(),
            related,
            "{name} related-only membership"
        );
        state.selection = SelectionTarget::Native(Box::new(view));
        let actual = renderer.render_workspace(&state, None)?;
        actual.save(output.join(format!("{name}.png")))?;
        if candidate_only {
            println!("CANDIDATE_ONLY {name}: rendered; golden comparison NOT RUN");
            continue;
        }
        let expected = image::open(golden_root.join(format!("{name}.golden.png")))?.to_rgba8();
        assert_eq!(
            actual.dimensions(),
            expected.dimensions(),
            "{name} dimensions"
        );
        assert_eq!(
            actual.as_raw(),
            expected.as_raw(),
            "{name} exact renderer golden"
        );
        println!("GOLDEN_MATCH {name}: exact same-engine pixels");
    }
    Ok(())
}
