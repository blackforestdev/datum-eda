//! Source-qualified publication and normal acquisition over an admitted native input.
use super::*;
use datum_gui_protocol::{AuthoredSelectionClass as Class, PaneId, SelectionSubject};
use eda_engine::api::native_write;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

struct Input(PathBuf);
impl Drop for Input {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).unwrap();
    }
}
fn copy_source(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        if entry.file_name() == ".datum" {
            continue;
        }
        let kind = entry.file_type().unwrap();
        assert!(!kind.is_symlink());
        if kind.is_dir() {
            copy_source(&entry.path(), &to.join(entry.file_name()));
        } else {
            std::fs::copy(entry.path(), to.join(entry.file_name())).unwrap();
        }
    }
}
fn bytes(root: &Path, model: &eda_engine::substrate::DesignModel) -> BTreeMap<PathBuf, Vec<u8>> {
    model
        .source_shards
        .iter()
        .map(|shard| {
            let path = root.join(&shard.path);
            let value = std::fs::read(&path).unwrap();
            (path, value)
        })
        .chain(std::iter::once({
            let path = eda_engine::substrate::transaction_journal_path(root);
            let value = std::fs::read(&path).unwrap();
            (path, value)
        }))
        .collect()
}

#[test]
fn document_refresh_preserves_occurrences_and_rejects_changed_navigation_before_publication() {
    let repository = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let source = repository.join("crates/test-harness/testdata/selection/s5a_v1/replay-inputs/native/s5a-c03-u5-semantic-v1/variants/semantic-scope");
    let input = Input(
        repository
            .join(".datum/scratch/gui-p2-inspector-worker-tests")
            .join(uuid::Uuid::new_v4().to_string()),
    );
    copy_source(&source, &input.0);
    let mut model = eda_engine::substrate::ProjectResolver::new(&input.0)
        .resolve()
        .unwrap();
    let request = datum_gui_protocol::LiveReviewRequest {
        project_root: input.0.clone(),
        board_file: None,
        artifact_path: None,
        net_uuid: None,
        from_anchor_pad_uuid: None,
        to_anchor_pad_uuid: None,
        profile: None,
        kicad_board_source: None,
    };
    let mut state = datum_gui_protocol::workspace_loading::project_board_editor_workspace(
        &request,
        Arc::new(model.clone()),
    )
    .unwrap();
    let children: Vec<_> = state
        .schematic_documents
        .rows
        .iter()
        .filter(|row| !row.key.instance_path.is_empty())
        .map(|row| row.key.clone())
        .collect();
    assert_eq!(children.len(), 2);
    for child in &children {
        state
            .schematic_documents
            .prepare_scene(child, &input.0, &model)
            .unwrap();
    }
    let a = state
        .schematic_documents
        .open_beside(&mut state.ui.layout, &children[0])
        .unwrap();
    let b = state
        .schematic_documents
        .open_beside(&mut state.ui.layout, &children[1])
        .unwrap();
    state.schematic_documents.inspect(&children[0]);
    let prior = Arc::new(NativeSelectionResolution::capture(&model).unwrap());
    let scene_a = datum_gui_protocol::camera_scene_for_pane(&state, a).unwrap();
    let bindings =
        datum_gui_protocol::selection_scene::SceneSelectionBindings::capture(scene_a, &prior);
    let selected = bindings
        .iter()
        .find(|(_, binding)| binding.projected.class == Class::Wire)
        .unwrap()
        .1
        .projected
        .clone();
    let initial = build_workspace_selection_view(
        &model,
        &prior,
        SelectionSubject::Object(selected.clone()),
        &state,
    )
    .unwrap();
    let scene_b = datum_gui_protocol::camera_scene_for_pane(&state, b).unwrap();
    assert!(
        !initial
            .projection_keys(scene_a, datum_gui_protocol::PaneContent::Schematic)
            .unwrap()
            .is_empty()
    );
    assert!(
        initial
            .projection_keys(scene_b, datum_gui_protocol::PaneContent::Schematic)
            .unwrap()
            .is_empty()
    );
    state.selection = SelectionTarget::Native(Box::new(initial));
    let inputs = Inputs {
        selected: state.selection.clone(),
        filters: state.ui.filters.clone(),
        layout: state.ui.layout.clone(),
        navigation: state.schematic_documents.navigation(),
        prior: Some(prior),
    };
    assert!(inputs.matches_workspace(&state));
    let selected_before = state.selection.clone();
    state.schematic_documents.inspect(&children[1]);
    assert!(!inputs.matches_workspace(&state));
    assert_eq!(state.selection, selected_before);
    // A real canonical edit advances the source basis. The worker must prepare
    // every retained document against that new model before publishing any view.
    let write = native_write::project::build_set_project_name(
        &model,
        native_write::WriteProvenance::new(
            "unit-test",
            eda_engine::substrate::CommitSource::Test,
            "Inspector document refresh control",
        ),
        "Inspector refresh control",
    )
    .unwrap();
    native_write::commit_prepared(&mut model, &input.0, write).unwrap();
    let source_bytes = bytes(&input.0, &model);
    let mut next = datum_gui_protocol::workspace_loading::project_board_editor_workspace(
        &request,
        Arc::new(model.clone()),
    )
    .unwrap();
    let prepared = prepare_from_model(
        &mut next,
        inputs,
        Arc::new(model.clone()),
        Default::default(),
    )
    .unwrap();
    assert_eq!(next.ui.layout, state.ui.layout);
    assert_eq!(
        next.schematic_documents.inspected.as_ref(),
        Some(&children[0])
    );
    assert_eq!(
        next.schematic_documents.document_name,
        "Inspector refresh control"
    );
    let next_a = datum_gui_protocol::camera_scene_for_pane(&next, a).unwrap();
    let next_b = datum_gui_protocol::camera_scene_for_pane(&next, b).unwrap();
    assert_eq!(next_a.source_revision, model.model_revision.0);
    assert_eq!(next_b.source_revision, model.model_revision.0);
    assert_eq!(next_a.schematic_document.as_ref(), Some(&children[0]));
    assert_eq!(next_b.schematic_document.as_ref(), Some(&children[1]));
    let SelectionTarget::Native(view) = &prepared.selected else {
        panic!("Selected authored wire must survive metadata edit")
    };
    assert_eq!(view.members, std::collections::BTreeSet::from([selected]));
    assert!(
        !view
            .projection_keys(next_a, datum_gui_protocol::PaneContent::Schematic)
            .unwrap()
            .is_empty()
    );
    assert!(
        view.projection_keys(next_b, datum_gui_protocol::PaneContent::Schematic)
            .unwrap()
            .is_empty()
    );
    let authority = &prepared.session.authority;
    let a_candidates = runtime_selection::global::candidates_for_occurrence(
        authority,
        true,
        Some(children[0].source_sheet),
        &children[0].instance_path,
    )
    .unwrap();
    let b_candidates = runtime_selection::global::candidates_for_occurrence(
        authority,
        true,
        Some(children[1].source_sheet),
        &children[1].instance_path,
    )
    .unwrap();
    assert!(!a_candidates.is_empty());
    assert_eq!(a_candidates.len(), b_candidates.len());
    assert!(a_candidates.is_disjoint(&b_candidates));
    assert!(
        a_candidates
            .iter()
            .all(|id| id.instance_path == children[0].instance_path && id.class != Class::Pin)
    );
    assert!(
        b_candidates
            .iter()
            .all(|id| id.instance_path == children[1].instance_path && id.class != Class::Pin)
    );
    assert_eq!(source_bytes, bytes(&input.0, &model));
    assert!(next.ui.layout.leaves().contains(&PaneId(1)));
}

#[test]
fn qualified_child_split_close_fill_and_preset_preserve_selection_and_canonical_bytes() {
    use crate::pane_cameras::PaneCameras;
    use datum_gui_protocol::{PaneContent, SplitOrientation, WorkspacePreset};
    let repository = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let source = repository.join("crates/test-harness/testdata/selection/s5a_v1/replay-inputs/native/s5a-c03-u5-semantic-v1/variants/semantic-scope");
    let input = Input(
        repository
            .join(".datum/scratch/gui-p2-inspector-worker-tests")
            .join(uuid::Uuid::new_v4().to_string()),
    );
    copy_source(&source, &input.0);
    let model = Arc::new(
        eda_engine::substrate::ProjectResolver::new(&input.0)
            .resolve()
            .unwrap(),
    );
    let canonical = || {
        let mut paths: Vec<_> = model
            .source_shards
            .iter()
            .map(|s| input.0.join(&s.path))
            .collect();
        paths.push(eda_engine::substrate::transaction_journal_path(&input.0));
        paths
            .into_iter()
            .map(|p| {
                let data = std::fs::read(&p).ok();
                (p, data)
            })
            .collect::<BTreeMap<_, _>>()
    };
    let before = canonical();
    let request = datum_gui_protocol::LiveReviewRequest {
        project_root: input.0.clone(),
        board_file: None,
        artifact_path: None,
        net_uuid: None,
        from_anchor_pad_uuid: None,
        to_anchor_pad_uuid: None,
        profile: None,
        kicad_board_source: None,
    };
    let mut state = datum_gui_protocol::workspace_loading::project_board_editor_workspace(
        &request,
        model.clone(),
    )
    .unwrap();
    let child = state
        .schematic_documents
        .rows
        .iter()
        .find(|r| !r.key.instance_path.is_empty())
        .unwrap()
        .key
        .clone();
    state
        .schematic_documents
        .prepare_scene(&child, &input.0, &model)
        .unwrap();
    let initial = state.schematic_documents.panes[&PaneId(1)].clone();
    let a = state
        .schematic_documents
        .open_beside(&mut state.ui.layout, &child)
        .unwrap();
    let authority = NativeSelectionResolution::capture(&model).unwrap();
    let scene = datum_gui_protocol::camera_scene_for_pane(&state, a).unwrap();
    let bindings =
        datum_gui_protocol::selection_scene::SceneSelectionBindings::capture(scene, &authority);
    let selected = bindings
        .iter()
        .find(|(_, b)| b.projected.class == Class::Wire)
        .unwrap()
        .1
        .projected
        .clone();
    state.selection = SelectionTarget::Native(Box::new(
        build_workspace_selection_view(
            &model,
            &authority,
            SelectionSubject::Object(selected),
            &state,
        )
        .unwrap(),
    ));
    let subject = state.selection.clone();
    let framing = CameraState {
        center_x_nm: 123_000.0,
        center_y_nm: 456_000.0,
        zoom: 2.75,
    };
    let mut cameras = PaneCameras::new(a, PaneContent::Schematic, framing);
    for orientation in [SplitOrientation::Vertical, SplitOrientation::Horizontal] {
        let duplicate = state
            .schematic_documents
            .split_view(&mut state.ui.layout, orientation);
        cameras.inherit(
            duplicate,
            PaneContent::Schematic,
            cameras.camera(a, PaneContent::Schematic).unwrap(),
        );
        assert_eq!(state.ui.layout.focused, a);
        assert_eq!(
            state.ui.layout.content_for(duplicate),
            Some(PaneContent::Schematic)
        );
        assert_eq!(state.schematic_documents.panes[&duplicate], child);
        assert_eq!(
            datum_gui_protocol::camera_scene_for_pane(&state, duplicate)
                .unwrap()
                .schematic_document
                .as_ref(),
            Some(&child)
        );
        assert_eq!(
            cameras.camera(duplicate, PaneContent::Schematic),
            Some(framing)
        );
        cameras
            .entry_or_insert_with(duplicate, PaneContent::Schematic, || unreachable!())
            .zoom = 0.5;
        assert_eq!(cameras.camera(a, PaneContent::Schematic), Some(framing));
        state.ui.layout.focused = duplicate;
        state.ui.layout.close_focused();
        state.schematic_documents.prune_views(&state.ui.layout);
        cameras.retain_live(&state.ui.layout.leaves());
        assert!(!state.schematic_documents.panes.contains_key(&duplicate));
        assert!(cameras.camera(duplicate, PaneContent::Schematic).is_none());
        state.ui.layout.focused = a;
    }
    // An explicit fill and source refresh retain warm child identity, not root.
    state.ui.layout.set_focused_content(PaneContent::Board);
    let navigation = state.schematic_documents.navigation();
    let mut fresh = datum_gui_protocol::workspace_loading::project_board_editor_workspace(
        &request,
        model.clone(),
    )
    .unwrap();
    fresh.ui.layout = state.ui.layout.clone();
    fresh
        .schematic_documents
        .restore_views(&navigation, &fresh.ui.layout, &input.0, &model)
        .unwrap();
    assert_eq!(fresh.schematic_documents.panes[&a], child);
    fresh.ui.layout.set_focused_content(PaneContent::Schematic);
    fresh.schematic_documents.bind_initial_view(a);
    fresh
        .schematic_documents
        .prepare_scene(&child, &input.0, &model)
        .unwrap();
    assert_eq!(
        datum_gui_protocol::camera_scene_for_pane(&fresh, a)
            .unwrap()
            .schematic_document
            .as_ref(),
        Some(&child)
    );
    // A vanished occurrence remains unavailable, including through split/fill.
    let mut missing = child.clone();
    missing.instance_path.push(child.source_sheet);
    state.schematic_documents.panes.insert(a, missing.clone());
    state.ui.layout.set_focused_content(PaneContent::Schematic);
    state.schematic_documents.bind_initial_view(a);
    let unavailable = state
        .schematic_documents
        .split_view(&mut state.ui.layout, SplitOrientation::Vertical);
    assert_eq!(state.schematic_documents.panes[&unavailable], missing);
    assert!(datum_gui_protocol::camera_scene_for_pane(&state, unavailable).is_none());
    // Presets deliberately reset views to the already captured initial owner.
    state
        .ui
        .layout
        .apply_preset(WorkspacePreset::BoardSchematic);
    state.schematic_documents.reset_views(&state.ui.layout);
    assert!(
        state
            .schematic_documents
            .panes
            .values()
            .all(|key| key == &initial)
    );
    assert!(
        state
            .schematic_documents
            .panes
            .keys()
            .all(|pane| state.ui.layout.content_for(*pane) == Some(PaneContent::Schematic))
    );
    assert_eq!(state.selection, subject);
    assert_eq!(canonical(), before);
}
