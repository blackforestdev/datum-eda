use super::*;
use crate::native_selection_overlay::tests::{selected_state, selected_wire_state};
#[test]
fn canonical_mask_source_preserves_identity_and_refuses_foreign_revision_without_writes() {
    for schematic in [false, true] {
        let (fixture, state) = if schematic {
            selected_wire_state()
        } else {
            selected_state()
        };
        let bytes = fixture.source_bytes();
        let SelectionTarget::Native(view) = &state.selection else {
            panic!()
        };
        let (scene, surface, retained) = if schematic {
            (
                state.schematic_scene.as_ref().unwrap(),
                datum_gui_protocol::PaneContent::Schematic,
                RetainedScene::from_workspace_schematic_for_surface(&state, 1280, 800, 1.).unwrap(),
            )
        } else {
            (
                &state.scene,
                datum_gui_protocol::PaneContent::Board,
                RetainedScene::from_workspace(&state, 1280, 800),
            )
        };
        let request = Request::capture(view, scene, surface).unwrap().unwrap();
        assert!(request.matches(&retained.presentation));
        assert!(
            retained
                .presentation
                .strokes
                .iter()
                .chain(retained.presentation.quads.iter())
                .any(|span| request.label(span) == 1),
            "schematic={schematic}, request={request:?}, presentation={:?}",
            retained.presentation
        );
        let mut stale = scene.clone();
        stale.source_revision.push('x');
        assert!(Request::capture(view, &stale, surface).unwrap().is_none());
        stale = scene.clone();
        stale.project_uuid = uuid::Uuid::new_v4().to_string();
        assert!(Request::capture(view, &stale, surface).unwrap().is_none());
        let presentation = Presentation {
            source: Some(Source {
                project: scene.project_uuid.clone(),
                revision: scene.source_revision.clone(),
                scene: "another scene".into(),
            }),
            ..Default::default()
        };
        assert!(!request.matches(&presentation));
        let mut stale_view = view.clone();
        stale_view.stale = true;
        assert!(
            Request::capture(&stale_view, scene, surface)
                .unwrap()
                .is_none()
        );
        let mut forged = view.clone();
        match surface {
            datum_gui_protocol::PaneContent::Board => {
                forged.board_keys.insert("forged-key".into());
            }
            datum_gui_protocol::PaneContent::Schematic => {
                forged.schematic_keys.insert("forged-key".into());
            }
            _ => unreachable!(),
        }
        assert!(Request::capture(&forged, scene, surface).is_err());
        assert_eq!(fixture.source_bytes(), bytes);
    }
}
#[test]
fn mask_intent_shares_admission_until_last_owner_and_refuses_before_publication() {
    let (_fixture, state) = selected_state();
    let SelectionTarget::Native(view) = &state.selection else {
        panic!()
    };
    let budget = crate::text_gpu::budget::Budget::new(1_000_000);
    let request = Request::capture_with_budget(
        view,
        &state.scene,
        datum_gui_protocol::PaneContent::Board,
        &budget,
    )
    .unwrap()
    .unwrap();
    let charged = budget.used();
    assert!(charged > 0);
    let cloned = request.clone();
    assert_eq!(budget.used(), charged);
    drop(request);
    assert_eq!(budget.used(), charged);
    drop(cloned);
    assert_eq!(budget.used(), 0);
    let refuse = crate::text_gpu::budget::Budget::new(1);
    assert!(
        Request::capture_with_budget(
            view,
            &state.scene,
            datum_gui_protocol::PaneContent::Board,
            &refuse
        )
        .is_err()
    );
    assert_eq!(refuse.used(), 0);
}
#[test]
fn zone_qualifier_and_void_construction_never_invent_membership() {
    // Construction-only role oracle; not a native Zone acquisition proof.
    let budget = crate::text_gpu::budget::Budget::new(1_000_000);
    let request = Request(Arc::new(Inner {
        source: Source::default(),
        high_contrast: false,
        dim_unrelated: false,
        choices: vec![Choice {
            key: "zone".into(),
            label: 1,
            boundary: false,
            minimum: false,
            islands: vec![1],
        }],
        _permit: Some(budget.reserve(1).unwrap()),
    }));
    let label = |key: &str, role| {
        request.label(&Span {
            key: Some(key.into()),
            role,
            range: 0..3,
        })
    };
    assert_eq!(label("zone", Role::ZoneIsland(1)), 1);
    for role in [Role::Void, Role::ZoneIsland(0), Role::ZoneBoundary] {
        assert_eq!(label("zone", role), 0);
    }
    assert_eq!(label("forged-key", Role::Material), 0);
    assert_eq!(label("zone", Role::Material), 1);
}

#[test]
fn prepared_paint_ranges_follow_visible_commands_and_survive_shared_clone() {
    let (fixture, state) = selected_state();
    let before = fixture.source_bytes();
    let retained = RetainedScene::from_workspace(&state, 1280, 800);
    let mut prepared = PreparedScene::from_workspace(
        &state,
        1280,
        800,
        CameraState::fit_to_bounds(&state.scene.bounds),
        &retained,
    )
    .unwrap();
    prepared.append_native_selection(&state).unwrap();
    let ranges: Vec<_> = prepared.selected_paint_ranges(&retained, None).collect();
    assert!(!ranges.is_empty());
    assert!(ranges.iter().all(|range| range.group == 1 && range.stroke));
    let cloned = prepared.clone();
    assert_eq!(
        cloned
            .selected_paint_ranges(&retained, None)
            .collect::<Vec<_>>(),
        ranges
    );
    let mut closed = cloned.clone();
    closed.surface_passes.clear();
    assert_eq!(closed.selected_paint_ranges(&retained, None).count(), 0);
    prepared.visible_draw_commands.clear();
    assert_eq!(prepared.selected_paint_ranges(&retained, None).count(), 0);
    let mut other = state.clone();
    other.scene.project_uuid = uuid::Uuid::new_v4().to_string();
    let foreign = RetainedScene::from_workspace(&other, 1280, 800);
    assert_eq!(cloned.selected_paint_ranges(&foreign, None).count(), 0);
    assert_eq!(fixture.source_bytes(), before);
}

#[test]
fn nonresolving_pane_creates_no_mask_request_or_selection_resource() {
    let (fixture, state) = selected_wire_state();
    let bytes = fixture.source_bytes();
    let SelectionTarget::Native(view) = &state.selection else {
        unreachable!()
    };
    assert!(view.board_keys.is_empty());
    assert!(!view.schematic_keys.is_empty());
    assert!(
        Request::capture(view, &state.scene, datum_gui_protocol::PaneContent::Board)
            .unwrap()
            .is_none()
    );
    let scene = state.schematic_scene.as_ref().unwrap();
    assert!(
        Request::capture(view, scene, datum_gui_protocol::PaneContent::Schematic)
            .unwrap()
            .is_some()
    );
    assert_eq!(fixture.source_bytes(), bytes);
}

#[test]
fn canonical_related_parent_keeps_baseline_role_while_selected_pin_keeps_material_role() {
    let (fixture, mut state) = crate::native_selection_overlay::tests::selected_related_state();
    let source = fixture.source_bytes();
    let selection = state.selection.clone();
    state.ui.layout.open_beside_root(
        datum_gui_protocol::PaneContent::Schematic,
        datum_gui_protocol::SplitOrientation::Vertical,
        0.5,
        true,
    );
    let retained = RetainedScene::from_workspace(&state, 1280, 800);
    for enabled in [false, true] {
        state.ui.filters.dim_unrelated = enabled;
        let mut prepared = PreparedScene::from_workspace(
            &state,
            1280,
            800,
            CameraState::fit_to_bounds(&state.scene.bounds),
            &retained,
        )
        .unwrap();
        prepared.append_native_selection(&state).unwrap();
        let request = &prepared
            .selection_masks
            .iter()
            .find(|(_, surface, _)| *surface == SceneSurface::Schematic)
            .unwrap()
            .2;
        assert_eq!(request.dim_unrelated(), enabled);
        let scene = state.schematic_scene.as_ref().unwrap();
        let SelectionTarget::Native(view) = &state.selection else {
            unreachable!()
        };
        for key in &view.related_schematic_keys {
            if !view.schematic_keys.contains(key) {
                let span = Span {
                    key: Some(key.clone()),
                    role: Role::Material,
                    range: 0..1,
                };
                assert_eq!(request.material_role(&span), 2);
                assert_eq!(request.label(&span), 0);
            }
        }
        for key in &view.schematic_keys {
            let span = Span {
                key: Some(key.clone()),
                role: Role::Material,
                range: 0..1,
            };
            assert_eq!(request.material_role(&span), 1);
        }
        assert_eq!(
            request.material_role(&Span {
                key: None,
                role: Role::Material,
                range: 0..1
            }),
            3
        );
        let unrelated = scene
            .board_texts
            .iter()
            .find(|v| {
                !view.schematic_keys.contains(&v.object_id)
                    && !view.related_schematic_keys.contains(&v.object_id)
            })
            .unwrap();
        assert_eq!(
            request.material_role(&Span {
                key: Some(unrelated.object_id.clone()),
                role: Role::Material,
                range: 0..1
            }),
            0
        );
    }
    assert_eq!(state.selection, selection);
    assert_eq!(fixture.source_bytes(), source);
}
