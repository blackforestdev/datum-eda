use super::*;
#[allow(clippy::too_many_arguments)]
pub(crate) fn push_native(
    selection_strokes: &mut Vec<selection_stroke::Stroke>,
    out: &mut Vec<Quad>,
    text_runs: &mut Vec<TextRun>,
    scene: &BoardReviewSceneV1,
    state: &ReviewWorkspaceState,
    viewport: RectPx,
    camera: CameraState,
    surface: datum_gui_protocol::PaneContent,
) {
    push_native_impl(
        selection_strokes,
        out,
        text_runs,
        scene,
        state,
        viewport,
        camera,
        surface,
        false,
    );
}

use datum_gui_protocol::selection_resolution::NativeSelectionResolution;
use datum_gui_protocol::selection_view::build_selection_view;
use datum_gui_protocol::{
    AuthoredSelectionClass as Class, AuthoredSelectionIdentity as Id, SelectionSubject,
};
use eda_engine::ir::geometry::Point;
#[path = "fixtures.rs"]
mod fixtures;
pub(crate) use fixtures::*;

#[test]
fn native_selection_is_immediate_and_retained_authored_bytes_and_resize_eligibility_are_stable() {
    let (f, mut state) = selected_state();
    let before = f.source_bytes();
    let selected = RetainedScene::from_workspace(&state, 1600, 1000);
    assert!(
        !selected.world_strokes().is_empty(),
        "positive retained-material proof cannot compare empty buffers"
    );
    state.selection = SelectionTarget::None;
    let baseline = RetainedScene::from_workspace(&state, 1600, 1000);
    assert_eq!(
        selected.glyph, baseline.glyph,
        "selection cannot rewrite exact glyph material or positions"
    );
    assert_eq!(
        bytemuck::cast_slice::<Vertex, u8>(selected.world_vertices()),
        bytemuck::cast_slice::<Vertex, u8>(baseline.world_vertices())
    );
    assert_eq!(
        bytemuck::cast_slice::<WorldStrokeInstance, u8>(selected.world_strokes().as_ref()),
        bytemuck::cast_slice::<WorldStrokeInstance, u8>(baseline.world_strokes().as_ref())
    );
    assert_eq!(
        selected.can_reuse_for_surface_resize(),
        baseline.can_reuse_for_surface_resize()
    );
    for (path, bytes) in before {
        assert_eq!(std::fs::read(path).unwrap(), bytes);
    }
}
#[test]
fn native_cues_are_clipped_to_each_pane_and_reproject_after_warm_camera_assignment() {
    let (_f, state) = selected_state();
    let viewport = RectPx {
        x: 100.0,
        y: 200.0,
        width: 500.0,
        height: 400.0,
    };
    let camera = CameraState::fit_to_bounds(&state.scene.bounds);
    let mut first = Vec::new();
    let mut first_strokes = Vec::new();
    let mut text = Vec::new();
    push_native(
        &mut first_strokes,
        &mut first,
        &mut text,
        &state.scene,
        &state,
        viewport,
        camera,
        datum_gui_protocol::PaneContent::Board,
    );
    assert!(!first_strokes.is_empty());
    assert!(
        first_strokes
            .iter()
            .all(|s| s.clip == [100.0, 200.0, 600.0, 600.0])
    );
    assert!(
        first
            .iter()
            .flat_map(|q| q.points)
            .all(|(x, y)| x >= viewport.x
                && x <= viewport.x + viewport.width
                && y >= viewport.y
                && y <= viewport.y + viewport.height)
    );
    let mut second = Vec::new();
    let mut second_strokes = Vec::new();
    let mut warm = camera;
    warm.center_x_nm += 5_000_000.0;
    push_native(
        &mut second_strokes,
        &mut second,
        &mut text,
        &state.scene,
        &state,
        viewport,
        warm,
        datum_gui_protocol::PaneContent::Board,
    );
    assert_ne!(first_strokes, second_strokes);
    let hover = datum_gui_protocol::HoverTarget {
        surface: datum_gui_protocol::PaneContent::Board,
        object_id: "stable-track-display".into(),
    };
    assert!(selection_beats_hover(&state, &hover));
}

#[test]
fn region_evaluation_feedback_is_pane_clipped_and_preserves_authored_sources() {
    let (fixture, mut state) = selected_state();
    let source = fixture.source_bytes();
    let pane = state.ui.layout.focused;
    state.ui.selection_region = Some(datum_gui_protocol::SelectionRegionFeedback {
        pane,
        path: vec![
            PointNm { x: 0, y: 0 },
            PointNm { x: 10, y: 0 },
            PointNm { x: 10, y: 10 },
        ],
        subtractive: false,
        phase: 0,
        evaluating: Some(4096),
    });
    let rect = RectPx {
        x: 20.0,
        y: 30.0,
        width: 200.0,
        height: 100.0,
    };
    let camera = CameraState::fit_to_bounds(&state.scene.bounds);
    let mut quads = Vec::new();
    let mut text = Vec::new();
    push_region(
        &mut quads,
        &mut text,
        &state,
        pane,
        &state.scene,
        rect,
        camera,
    );
    assert!(
        text.iter()
            .any(|run| run.text.contains("Evaluating selection")
                && run.text.contains("Escape cancels")
                && run.clip_bounds == Some(rect))
    );
    let mut other_text = Vec::new();
    push_region(
        &mut Vec::new(),
        &mut other_text,
        &state,
        datum_gui_protocol::PaneId(pane.0 + 1),
        &state.scene,
        rect,
        camera,
    );
    assert!(other_text.is_empty());
    for (path, bytes) in source {
        assert_eq!(std::fs::read(path).unwrap(), bytes);
    }
}

#[test]
fn native_projection_uses_typed_surface_and_refuses_foreign_or_stale_scene_basis() {
    use datum_gui_protocol::PaneContent;
    let (fixture, mut state) = selected_state();
    let source = fixture.source_bytes();
    let viewport = RectPx {
        x: 0.0,
        y: 0.0,
        width: 500.0,
        height: 400.0,
    };
    let camera = CameraState::fit_to_bounds(&state.scene.bounds);
    // Names carry no surface authority: this is still the configured board pane.
    state.scene.scene_id = "schematic-misleading-display-name".into();
    let render = |state: &ReviewWorkspaceState, surface| {
        let mut quads = Vec::new();
        let mut strokes = Vec::new();
        push_native(
            &mut strokes,
            &mut quads,
            &mut Vec::new(),
            &state.scene,
            state,
            viewport,
            camera,
            surface,
        );
        strokes
    };
    assert!(!render(&state, PaneContent::Board).is_empty());
    assert!(render(&state, PaneContent::Schematic).is_empty());
    if let SelectionTarget::Native(view) = &mut state.selection {
        view.schematic_keys.insert("stable-track-display".into());
        view.board_keys.clear();
    }
    state.scene.scene_id = "board-misleading-display-name".into();
    assert!(render(&state, PaneContent::Board).is_empty());
    assert!(!render(&state, PaneContent::Schematic).is_empty());
    let revision = state.scene.source_revision.clone();
    state.scene.source_revision = "older-scene-revision".into();
    assert!(render(&state, PaneContent::Schematic).is_empty());
    state.scene.source_revision = revision;
    state.scene.project_uuid = uuid::Uuid::new_v4().to_string();
    assert!(render(&state, PaneContent::Schematic).is_empty());
    for (path, bytes) in source {
        assert_eq!(std::fs::read(path).unwrap(), bytes);
    }
}

pub(crate) fn selected_points_state() -> (Fixture, ReviewWorkspaceState) {
    use eda_engine::{
        api::native_write::{self, BatchComposer, WriteProvenance},
        substrate::{CommitSource, Operation},
    };
    let (mut fixture, mut state) = selected_state();
    let sheet = uuid::Uuid::new_v4();
    let junction = uuid::Uuid::new_v4();
    let second_junction = uuid::Uuid::new_v4();
    let provenance = || {
        WriteProvenance::new(
            "selection-point-fixture",
            CommitSource::Test,
            "canonical low-zoom marker source",
        )
    };
    let mut source = fixture
        .model
        .materialized_source_shard_value(eda_engine::substrate::SourceShardKind::SchematicRoot)
        .unwrap();
    source.as_object_mut().unwrap().clear();
    let empty = source.clone();
    source["schema_version"] = 1.into();
    source["uuid"] = sheet.to_string().into();
    source["name"] = "Main".into();
    for field in [
        "symbols",
        "wires",
        "labels",
        "ports",
        "junctions",
        "buses",
        "bus_entries",
        "noconnects",
        "texts",
        "drawings",
    ] {
        source[field] = empty.clone();
    }
    let mut junction_source = empty.clone();
    junction_source["uuid"] = junction.to_string().into();
    junction_source["position"] = empty.clone();
    junction_source["position"]["x"] = 0.into();
    junction_source["position"]["y"] = 0.into();
    let mut second_source = junction_source.clone();
    second_source["uuid"] = second_junction.to_string().into();
    second_source["position"]["x"] = 1_000_000.into();
    let write = native_write::schematic_sheets::build_create_schematic_sheet(
        &fixture.model,
        provenance(),
        fixture.schematic,
        sheet,
        "sheets/main.json",
        source,
    )
    .unwrap();
    native_write::commit_prepared(&mut fixture.model, &fixture.root, write).unwrap();
    let write = BatchComposer::compose(&fixture.model, provenance())
        .push_ops([
            Operation::CreateSchematicJunction {
                sheet_id: sheet,
                junction_id: junction,
                junction: junction_source,
            },
            Operation::CreateSchematicJunction {
                sheet_id: sheet,
                junction_id: second_junction,
                junction: second_source,
            },
        ])
        .finish()
        .unwrap();
    native_write::commit_prepared(&mut fixture.model, &fixture.root, write).unwrap();
    let schematic =
        datum_gui_protocol::project_native_schematic_scene(&fixture.model, &fixture.root)
            .unwrap()
            .unwrap();
    state.scene.source_revision = fixture.model.model_revision.0.clone();
    let authority = NativeSelectionResolution::capture(&fixture.model).unwrap();
    let subject = SelectionSubject::Compound(
        datum_gui_protocol::CompoundSelection::new(
            [
                Id {
                    class: Class::Junction,
                    id: junction,
                    instance_path: vec![],
                },
                Id {
                    class: Class::Junction,
                    id: second_junction,
                    instance_path: vec![],
                },
            ],
            None,
        )
        .unwrap(),
    );
    let view = build_selection_view(
        &fixture.model,
        &authority,
        subject,
        &state.scene,
        Some(&schematic),
        |_| true,
    )
    .unwrap();
    state.schematic_scene = Some(schematic);
    state.selection = SelectionTarget::Native(Box::new(view));
    (fixture, state)
}

pub(crate) fn selected_wire_state() -> (Fixture, ReviewWorkspaceState) {
    use eda_engine::{
        api::native_write::{self, WriteProvenance},
        substrate::CommitSource,
    };
    let (mut f, mut state) = selected_points_state();
    let before = NativeSelectionResolution::capture(&f.model).unwrap();
    let sheets = &before.snapshot().schematic_source().unwrap().sheets;
    assert_eq!(sheets.len(), 1);
    let sheet = *sheets.keys().next().unwrap();
    let wire = eda_engine::schematic::SchematicWire {
        uuid: uuid::Uuid::new_v4(),
        from: Point::new(0, 0),
        to: Point::new(10_000_000, 0),
    };
    let write = native_write::schematic_connectivity::build_create_schematic_wire(
        &f.model,
        WriteProvenance::new(
            "selection-wire-fixture",
            CommitSource::Test,
            "canonical selection stroke source",
        ),
        sheet,
        &wire,
    )
    .unwrap();
    native_write::commit_prepared(&mut f.model, &f.root, write).unwrap();
    let schematic = datum_gui_protocol::project_native_schematic_scene(&f.model, &f.root)
        .unwrap()
        .unwrap();
    state.scene.source_revision = f.model.model_revision.0.clone();
    let authority = NativeSelectionResolution::capture(&f.model).unwrap();
    let subject = SelectionSubject::Object(Id {
        class: Class::Wire,
        id: wire.uuid,
        instance_path: vec![],
    });
    let view = build_selection_view(
        &f.model,
        &authority,
        subject,
        &state.scene,
        Some(&schematic),
        |_| true,
    )
    .unwrap();
    state.schematic_scene = Some(schematic);
    state.selection = SelectionTarget::Native(Box::new(view));
    (f, state)
}

#[path = "dense_native_tests.rs"]
mod dense_native_tests;

#[cfg(feature = "visual")]
#[path = "dense_preparation_diagnostics.rs"]
mod dense_preparation_diagnostics;

#[path = "related_fixture.rs"]
mod related_fixture;
pub(crate) use related_fixture::selected_related_state;

#[path = "owned_graphic_tests.rs"]
mod owned_graphic_tests;

// Observe final opaque mesh coverage in painter order, independently of the
// dash construction. Palette equality avoids inventing a luminance threshold.
fn region_sample(quads: &[Quad], p: (f32, f32), background: [f32; 3]) -> [f32; 3] {
    let mut color = background;
    for q in quads {
        let area: f32 = (0..4)
            .map(|i| {
                let a = q.points[i];
                let b = q.points[(i + 1) % 4];
                a.0 * b.1 - b.0 * a.1
            })
            .sum();
        if area == 0.0 {
            continue;
        }
        let mut positive = false;
        let mut negative = false;
        for i in 0..4 {
            let a = q.points[i];
            let b = q.points[(i + 1) % 4];
            let cross = (b.0 - a.0) * (p.1 - a.1) - (b.1 - a.1) * (p.0 - a.0);
            positive |= cross > 0.0;
            negative |= cross < 0.0;
        }
        if !(positive && negative) {
            color = q.color;
        }
    }
    color
}

fn region_distance(path: &[(i64, i64)], p: (f32, f32)) -> f32 {
    (0..path.len())
        .map(|i| {
            let a = (path[i].0 as f32, path[i].1 as f32);
            let b = path[(i + 1) % path.len()];
            let d = (b.0 as f32 - a.0, b.1 as f32 - a.1);
            let length2 = d.0 * d.0 + d.1 * d.1;
            let t = if length2 == 0.0 {
                0.0
            } else {
                (((p.0 - a.0) * d.0 + (p.1 - a.1) * d.1) / length2).clamp(0.0, 1.0)
            };
            (p.0 - a.0 - t * d.0).hypot(p.1 - a.1 - t * d.1)
        })
        .fold(f32::INFINITY, f32::min)
}

#[test]
fn region_short_boundaries_retain_light_dashes_at_every_phase_without_fill_or_halo() {
    let (fixture, mut state) = selected_state();
    let before = fixture.source_bytes();
    // Inset 1000px over 1000nm with this camera gives an independent 1:1
    // placement: feedback point (x-10,y-10) must paint at screen (x,y).
    state.scene.bounds = datum_gui_protocol::SceneBounds {
        min_x: 0,
        min_y: 0,
        max_x: 1000,
        max_y: 1000,
    };
    let viewport = RectPx {
        x: 0.0,
        y: 0.0,
        width: 1020.0,
        height: 1020.0,
    };
    let camera = CameraState {
        center_x_nm: 500.0,
        center_y_nm: 500.0,
        zoom: 1.0,
    };
    let pane = state.ui.layout.focused;
    let shapes: &[(&str, &[(i64, i64)], bool)] = &[
        (
            "observed14x13",
            &[(100, 100), (114, 100), (114, 113), (100, 113)],
            true,
        ),
        (
            "threshold4x4",
            &[(100, 100), (104, 100), (104, 104), (100, 104)],
            true,
        ),
        (
            "narrow4x1",
            &[(100, 100), (104, 100), (104, 101), (100, 101)],
            true,
        ),
        (
            "degenerate",
            &[(100, 100), (104, 100), (104, 100), (100, 100)],
            false,
        ),
        (
            "retraced",
            &[(100, 100), (114, 100), (100, 100), (100, 113)],
            false,
        ),
        (
            "larger",
            &[(100, 100), (228, 100), (228, 196), (100, 196)],
            true,
        ),
        (
            "lasso",
            &[(100, 100), (106, 98), (115, 105), (112, 114), (98, 111)],
            true,
        ),
    ];
    for &(name, path, has_interior) in shapes {
        for subtractive in [false, true] {
            let mut phase_zero = Vec::new();
            let mut phase_one = Vec::new();
            for phase in 0..12 {
                state.ui.global_preferences.reduced_motion = false;
                state.ui.selection_region = Some(datum_gui_protocol::SelectionRegionFeedback {
                    pane,
                    path: path
                        .iter()
                        .map(|&(x, y)| PointNm {
                            x: x - 10,
                            y: y - 10,
                        })
                        .collect(),
                    subtractive,
                    phase,
                    evaluating: None,
                });
                let render = |state: &ReviewWorkspaceState| {
                    let mut quads = Vec::new();
                    let mut text = Vec::new();
                    push_region(
                        &mut quads,
                        &mut text,
                        state,
                        pane,
                        &state.scene,
                        viewport,
                        camera,
                    );
                    assert!(text.is_empty());
                    assert!(
                        quads
                            .iter()
                            .flat_map(|q| q.points)
                            .all(|(x, y)| viewport.contains(x, y))
                    );
                    quads
                };
                let quads = render(&state);
                assert!(!quads.is_empty(), "nonvacuous {name}");
                let mut light = 0;
                let mut dark = 0;
                for y in path.iter().map(|p| p.1).min().unwrap() - 1
                    ..=path.iter().map(|p| p.1).max().unwrap() + 1
                {
                    for x in path.iter().map(|p| p.0).min().unwrap() - 1
                        ..=path.iter().map(|p| p.0).max().unwrap() + 1
                    {
                        let point = (x as f32 + 0.25, y as f32 + 0.25);
                        let on_dark = region_sample(&quads, point, [0.0; 3]);
                        let on_light = region_sample(&quads, point, TEXT_PRIMARY);
                        light += usize::from(on_dark == TEXT_PRIMARY);
                        dark += usize::from(on_light == [0.0; 3]);
                        if on_dark != [0.0; 3] || on_light != TEXT_PRIMARY {
                            assert!(
                                region_distance(path, point) <= 0.5,
                                "fill/halo {name} {phase} {point:?}"
                            );
                        }
                    }
                }
                assert!(
                    light > 0,
                    "dark-only boundary {name} subtractive={subtractive} phase={phase}"
                );
                if has_interior {
                    assert!(
                        dark > 0,
                        "missing dark contrast {name} subtractive={subtractive} phase={phase}"
                    );
                }
                if phase == 0 {
                    phase_zero = quads.clone();
                }
                if phase == 1 {
                    phase_one = quads;
                }
                state.ui.global_preferences.reduced_motion = true;
                assert_eq!(render(&state), phase_zero, "reduced motion {name} {phase}");
            }
            assert_ne!(
                phase_zero, phase_one,
                "animation remains enabled for {name}"
            );
        }
    }
    for (path, bytes) in before {
        assert_eq!(std::fs::read(path).unwrap(), bytes);
    }
}

#[test]
fn region_empty_or_single_point_feedback_cannot_paint_a_fill() {
    let (_fixture, mut state) = selected_state();
    let pane = state.ui.layout.focused;
    for path in [vec![], vec![PointNm { x: 0, y: 0 }]] {
        state.ui.selection_region = Some(datum_gui_protocol::SelectionRegionFeedback {
            pane,
            path,
            subtractive: false,
            phase: 6,
            evaluating: None,
        });
        let mut quads = Vec::new();
        push_region(
            &mut quads,
            &mut Vec::new(),
            &state,
            pane,
            &state.scene,
            RectPx {
                x: 0.0,
                y: 0.0,
                width: 100.0,
                height: 100.0,
            },
            CameraState::fit_to_bounds(&state.scene.bounds),
        );
        assert!(quads.is_empty());
    }
}
