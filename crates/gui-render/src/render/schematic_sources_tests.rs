//! Focused retained-source and lifetime controls; ordinary native proof remains separate.
use super::*;
use datum_gui_protocol::{PaneId, SplitOrientation};

struct Input(std::path::PathBuf);
impl Drop for Input {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).unwrap();
    }
}
fn copy_source(from: &std::path::Path, to: &std::path::Path) {
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
fn rect(state: &ReviewWorkspaceState, pane: PaneId) -> crate::RectPx {
    ShellLayout::for_surface(1400, 900, 1.0, None)
        .viewport_panes(&state.ui.layout)
        .panes
        .into_iter()
        .find(|p| p.id == pane)
        .unwrap()
        .rect
        .scene
}
fn native_occurrences() -> (
    Input,
    eda_engine::substrate::DesignModel,
    ReviewWorkspaceState,
    Vec<SchematicDocumentKey>,
    PaneId,
    PaneId,
) {
    let repository = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let source = repository.join("crates/test-harness/testdata/selection/s5a_v1/replay-inputs/native/s5a-c03-u5-semantic-v1/variants/semantic-scope");
    let input = Input(
        repository
            .join(".datum/scratch/gui-p2-inspector-source-tests")
            .join(uuid::Uuid::new_v4().to_string()),
    );
    copy_source(&source, &input.0);
    let model = eda_engine::substrate::ProjectResolver::new(&input.0)
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
    for key in &children {
        state
            .schematic_documents
            .prepare_scene(key, &input.0, &model)
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
    (input, model, state, children, a, b)
}
#[test]
fn qualified_sources_keep_occurrences_and_pinned_frames_distinct_on_close() {
    let (_input, _model, mut state, children, a, b) = native_occurrences();
    let primary = RetainedScene::try_from_workspace_schematic_for_surface(&state, 1400, 900, 1.0)
        .unwrap()
        .unwrap();
    let sources = SchematicSources::capture(
        &state,
        Some(&primary),
        &SchematicSources::default(),
        1400,
        900,
        1.0,
    )
    .unwrap();
    let source_a = sources.scene(Some(&children[0]), rect(&state, a)).unwrap();
    let source_b = sources.scene(Some(&children[1]), rect(&state, b)).unwrap();
    assert_ne!(source_a.presentation.source, source_b.presentation.source);
    assert!(!source_a.world_vertices.ptr_eq(&source_b.world_vertices));
    assert!(!source_a.all_draw_commands().is_empty());
    let primary_budget = primary.world_vertices.document_budget().unwrap();
    assert!(Arc::ptr_eq(
        primary_budget,
        source_a.world_vertices.document_budget().unwrap()
    ));
    assert!(Arc::ptr_eq(
        primary_budget,
        source_b.world_vertices.document_budget().unwrap()
    ));
    let pinned = sources.clone();
    let observer_a = source_a.geometry_observer();
    state.ui.layout.focused = a;
    state.ui.layout.close_focused();
    let next = SchematicSources::capture(&state, Some(&primary), &sources, 1400, 900, 1.0).unwrap();
    assert!(next.iter().all(|(key, _, _)| key != Some(&children[0])));
    drop(sources);
    assert!(observer_a.is_live());
    assert!(pinned.iter().any(|(key, _, _)| key == Some(&children[0])));
    drop(pinned);
    assert!(!observer_a.is_live());
    assert!(next.iter().any(|(key, _, _)| key == Some(&children[1])));
    // The production session retains only visible native documents, including
    // a child-only layout after the original root pane closes.
    state.ui.layout.focused = PaneId(1);
    state.ui.layout.close_focused();
    let mut session = crate::RenderSession::default();
    let epoch = crate::render_input::SourceEpoch::default();
    session
        .ensure_sources(&state, Some(epoch.revision()), 1400, 900, 1.0)
        .unwrap();
    assert!(session.schematic().is_none());
    assert_eq!(session.schematic_sources.iter().count(), 1);
    assert!(
        session
            .schematic_sources
            .iter()
            .all(|(key, _, _)| key == Some(&children[1]))
    );
}

#[test]
fn same_window_layout_changes_rebuild_only_projection_dependent_sources() {
    // A transient mechanical projection exercises the existing world-stroke
    // size-dependent path; this is a cache control, not authored/native evidence.
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    let mut scene = state.scene.clone();
    let bounds = scene.components[0].bounds;
    scene
        .component_graphics
        .push(datum_gui_protocol::ComponentGraphicPrimitive {
            graphic_id: "retention-control".into(),
            component_uuid: scene.components[0].component_uuid.clone(),
            layer_id: None,
            primitive_kind: "polyline".into(),
            render_role: "component_mechanical".into(),
            width_nm: None,
            closed: true,
            path: vec![
                datum_gui_protocol::PointNm {
                    x: bounds.min_x,
                    y: bounds.min_y,
                },
                datum_gui_protocol::PointNm {
                    x: bounds.max_x,
                    y: bounds.min_y,
                },
                datum_gui_protocol::PointNm {
                    x: bounds.max_x,
                    y: bounds.max_y,
                },
            ],
            holes: vec![],
        });
    state.schematic_scene = Some(scene);
    let primary = RetainedScene::try_from_workspace_schematic_for_surface(&state, 1400, 900, 1.0)
        .unwrap()
        .unwrap();
    assert!(!primary.can_reuse_for_surface_resize());
    let sources = SchematicSources::capture(
        &state,
        Some(&primary),
        &SchematicSources::default(),
        1400,
        900,
        1.0,
    )
    .unwrap();
    let old_rect = rect(&state, PaneId(1));
    state.ui.layout.focused = PaneId(1);
    let other = state.ui.layout.split_focused_with(
        PaneContent::Schematic,
        SplitOrientation::Horizontal,
        false,
    );
    let root_rect = rect(&state, PaneId(1));
    let other_rect = rect(&state, other);
    assert_ne!(old_rect, root_rect);
    assert_ne!(root_rect, other_rect);
    let next = SchematicSources::capture(&state, Some(&primary), &sources, 1400, 900, 1.0).unwrap();
    assert_eq!(next.iter().count(), 2);
    let old = sources.scene(None, old_rect).unwrap();
    let root = next.scene(None, root_rect).unwrap();
    let second = next.scene(None, other_rect).unwrap();
    assert!(!old.world_vertices.ptr_eq(&root.world_vertices));
    assert!(!root.world_vertices.ptr_eq(&second.world_vertices));
    assert!(next.scene(None, old_rect).is_none());
    // Warm unchanged views preserve their actual immutable payloads.
    let warm = SchematicSources::capture(&state, Some(&primary), &next, 1400, 900, 1.0).unwrap();
    assert!(
        root.world_vertices
            .ptr_eq(&warm.scene(None, root_rect).unwrap().world_vertices)
    );
    // Genuinely independent geometry still shares through same-window splits.
    state
        .schematic_scene
        .as_mut()
        .unwrap()
        .component_graphics
        .clear();
    let independent =
        RetainedScene::try_from_workspace_schematic_for_surface(&state, 1400, 900, 1.0)
            .unwrap()
            .unwrap();
    assert!(independent.can_reuse_for_surface_resize());
    let independent_sources = SchematicSources::capture(
        &state,
        Some(&independent),
        &SchematicSources::default(),
        1400,
        900,
        1.0,
    )
    .unwrap();
    assert_eq!(independent_sources.iter().count(), 1);
    assert!(
        independent_sources
            .scene(None, root_rect)
            .unwrap()
            .world_vertices
            .ptr_eq(
                &independent_sources
                    .scene(None, other_rect)
                    .unwrap()
                    .world_vertices
            )
    );
}

#[cfg(feature = "visual")]
#[test]
#[ignore = "requires local GPU; explicit qualified document routing proof"]
fn qualified_world_passes_masks_and_retirement_use_the_frozen_source_set() {
    use datum_gui_protocol::selection_resolution::NativeSelectionResolution;
    let (_input, model, mut state, children, a, b) = native_occurrences();
    let authority = NativeSelectionResolution::capture(&model).unwrap();
    let scene_a = datum_gui_protocol::camera_scene_for_pane(&state, a).unwrap();
    let binding =
        datum_gui_protocol::selection_scene::SceneSelectionBindings::capture(scene_a, &authority);
    let selected = binding
        .iter()
        .find(|(_, binding)| {
            binding.projected.class == datum_gui_protocol::AuthoredSelectionClass::Wire
        })
        .unwrap()
        .1
        .projected
        .clone();
    let scenes: Vec<_> = state
        .ui
        .layout
        .leaves()
        .into_iter()
        .filter(|pane| state.ui.layout.content_for(*pane) == Some(PaneContent::Schematic))
        .filter_map(|pane| datum_gui_protocol::camera_scene_for_pane(&state, pane))
        .collect();
    let view = datum_gui_protocol::selection_view::build_selection_view_for_documents(
        &model,
        &authority,
        datum_gui_protocol::SelectionSubject::Object(selected),
        &state.scene,
        &scenes,
        |_| true,
    )
    .unwrap();
    state.selection = datum_gui_protocol::SelectionTarget::Native(Box::new(view));
    let instance = wgpu::Instance::default();
    let adapter =
        pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
            .unwrap();
    let (device, queue) =
        pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default())).unwrap();
    let format = wgpu::TextureFormat::Rgba8UnormSrgb;
    let target = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("qualified-document-routing"),
        size: wgpu::Extent3d {
            width: 1400,
            height: 900,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    let target = target.create_view(&wgpu::TextureViewDescriptor::default());
    let mut renderer = crate::Renderer::new(&device, &queue, format, 4).unwrap();
    let epoch = crate::render_input::SourceEpoch::default();
    let prepare = |renderer: &mut crate::Renderer, state: &ReviewWorkspaceState| {
        renderer
            .prepare_session_workspace(
                state,
                crate::WorkspaceView {
                    source_revision: Some(epoch.revision()),
                    width: 1400,
                    height: 900,
                    scale: 1.0,
                    camera: crate::CameraState::fit_to_bounds(&state.scene.bounds),
                    schematic_camera: None,
                    include_preferences_overlay: false,
                    single_terminal_snapshot: true,
                    pane_cameras: &[],
                },
                &[],
            )
            .unwrap();
    };
    prepare(&mut renderer, &state);
    let frozen = renderer.render_session().prepared().unwrap().clone();
    let board = renderer.render_session().board().unwrap().clone();
    let a_pass = frozen
        .surface_passes()
        .iter()
        .find(|p| p.pane_id == a)
        .unwrap();
    let b_pass = frozen
        .surface_passes()
        .iter()
        .find(|p| p.pane_id == b)
        .unwrap();
    assert_eq!(a_pass.schematic_document.as_ref(), Some(&children[0]));
    assert_eq!(b_pass.schematic_document.as_ref(), Some(&children[1]));
    let a_scene = frozen.schematic_for_pass(a_pass, None).unwrap();
    let b_scene = frozen.schematic_for_pass(b_pass, None).unwrap();
    let observer_a = a_scene.geometry_observer();
    assert_ne!(a_scene.presentation.source, b_scene.presentation.source);
    assert!(frozen.selection_masks.iter().any(|(pane, _, _)| *pane == a));
    assert!(!frozen.selection_masks.iter().any(|(pane, _, _)| *pane == b));
    renderer
        .render(&device, &queue, &target, &frozen, &board, None, 1400, 900)
        .unwrap();
    assert!(
        renderer
            .encoded_world_admission()
            .any(|p| p.pane_id == a && p.ranges().next().is_some())
    );
    assert!(
        renderer
            .encoded_world_admission()
            .any(|p| p.pane_id == b && p.ranges().next().is_some())
    );
    let a_gpu = renderer.qualified_schematic_gpu.get(a_scene).unwrap();
    let b_gpu = renderer.qualified_schematic_gpu.get(b_scene).unwrap();
    assert_ne!(a_gpu.vertices.buffer(), b_gpu.vertices.buffer());
    let selected: Vec<_> = frozen.selected_paint_ranges(&board, None).collect();
    assert!(!selected.is_empty());
    assert!(selected.iter().all(|range| range.pane == a));
    let consumers: Vec<_> = renderer.retained_surface_resource_consumers().collect();
    assert!(consumers.iter().any(|(_, pane, _)| *pane == a));
    assert!(consumers.iter().any(|(_, pane, _)| *pane == b));
    // Exercise the real FramePlan and encoder, rather than treating a cloned
    // PreparedScene as an in-flight frame. This remains offscreen GPU proof;
    // completion feedback below is controlled, not a native compositor receipt.
    let old_hits = renderer
        .render_session()
        .prepared()
        .unwrap()
        .hit_regions
        .as_ptr();
    let old_plan = renderer
        .render_session_mut()
        .prepare_frame(1, 1, 1, true, 1400, 900)
        .unwrap();
    drop(frozen);
    assert!(
        observer_a.is_live(),
        "the actual plan owns the closed source"
    );
    // Publish a new document layout after planning and before encoding.
    state.ui.layout.focused = a;
    state.ui.layout.close_focused();
    prepare(&mut renderer, &state);
    let current = renderer.render_session().prepared().unwrap().clone();
    let current_hits = renderer
        .render_session()
        .prepared()
        .unwrap()
        .hit_regions
        .as_ptr();
    assert_ne!(old_hits, current_hits);
    assert!(current.surface_passes().iter().all(|p| p.pane_id != a));
    let old_frame = renderer
        .encode_frame(
            old_plan,
            &device,
            &queue,
            &mut (),
            &mut |_| Ok(Some(&target)),
            &mut |_, _| {},
        )
        .unwrap()
        .expect("the already warmed source set encodes a complete old frame");
    assert!(renderer.encoded_world_admission().any(|p| p.pane_id == a));
    assert!(renderer.encoded_world_admission().any(|p| p.pane_id == b));
    assert!(
        renderer
            .render_session_mut()
            .complete_submitted(old_frame, 1, 1, 1, true)
    );
    assert_eq!(
        renderer
            .render_session_mut()
            .take_published_frame()
            .unwrap()
            .0
            .as_ptr(),
        old_hits
    );
    assert_eq!(
        renderer
            .render_session()
            .prepared()
            .unwrap()
            .hit_regions
            .as_ptr(),
        current_hits
    );
    assert!(renderer.render_session().has_pending_frame());
    let current_plan = renderer
        .render_session_mut()
        .prepare_frame(1, 1, 1, true, 1400, 900)
        .unwrap();
    let current_frame = renderer
        .encode_frame(
            current_plan,
            &device,
            &queue,
            &mut (),
            &mut |_| Ok(Some(&target)),
            &mut |_, _| {},
        )
        .unwrap()
        .expect("the retained current source set encodes a complete new frame");
    assert!(renderer.encoded_world_admission().all(|p| p.pane_id != a));
    assert!(
        renderer
            .render_session_mut()
            .complete_submitted(current_frame, 1, 1, 1, true)
    );
    assert_eq!(
        renderer
            .render_session_mut()
            .take_published_frame()
            .unwrap()
            .0
            .as_ptr(),
        current_hits
    );
    assert!(!renderer.render_session().has_pending_frame());
    device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    assert!(!observer_a.is_live());
    assert!(
        renderer
            .qualified_schematic_gpu
            .entries
            .iter()
            .all(|entry| current
                .schematic_sources
                .iter()
                .any(|(_, _, scene)| entry.matches(scene)))
    );
}
