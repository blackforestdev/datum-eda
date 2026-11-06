//! CPU gesture/acquisition controls; the owner reproduction is read-only.
use super::super::drag::drag_subject;
use datum_gui_protocol::{
    AuthoredSelectionClass as Class, AuthoredSelectionIdentity as Id, CompoundSelection, PaneId,
    PointNm, SelectionSubject as Subject,
};
use datum_gui_viewport::selection_gesture::SelectionGesture;

fn id(class: Class, value: u128) -> Id {
    Id {
        class,
        id: uuid::Uuid::from_u128(value),
        instance_path: vec![uuid::Uuid::from_u128(91)],
    }
}

#[test]
fn connectivity_drag_picks_exact_object_without_run_members_or_symbol_fallback() {
    let wire = id(Class::Wire, 1);
    let run = Subject::Run(wire.clone().into());
    for class in [Class::Junction, Class::Wire, Class::Port] {
        let hit = id(class, 2);
        assert_eq!(
            drag_subject(Some(&run), &hit),
            Some(Subject::Object(hit.clone()))
        );
        assert_eq!(drag_subject(None, &hit), Some(Subject::Object(hit)));
    }
    assert_eq!(drag_subject(Some(&run), &id(Class::Pin, 3)), None);
    assert_eq!(drag_subject(Some(&run), &id(Class::Symbol, 4)), None);
    assert_eq!(drag_subject(None, &id(Class::BusSection, 5)), None);
}

#[test]
fn selected_symbol_and_complete_compound_keep_existing_drag_ownership() {
    let a = id(Class::Symbol, 1);
    let b = id(Class::Symbol, 2);
    let single = Subject::Object(a.clone());
    assert_eq!(drag_subject(Some(&single), &a), Some(single.clone()));
    assert_eq!(drag_subject(Some(&single), &b), None);
    let group =
        Subject::Compound(CompoundSelection::new([a.clone(), b.clone()], Some(a.clone())).unwrap());
    assert_eq!(drag_subject(Some(&group), &b), Some(group.clone()));
    let mut foreign = b;
    foreign.instance_path.clear();
    assert_eq!(drag_subject(Some(&group), &foreign), None);
    assert_eq!(drag_subject(Some(&group), &id(Class::Pin, 3)), None);
}

#[test]
fn connectivity_pickup_remains_thresholded_and_modifier_regions_keep_their_mode() {
    use datum_gui_viewport::selection_acquisition::AcquisitionMode;
    let anchor = PointNm {
        x: -2_400_000,
        y: 1_200_000,
    };
    for (shift, control, mode) in [
        (false, false, AcquisitionMode::Replace),
        (true, false, AcquisitionMode::Add),
        (false, true, AcquisitionMode::Remove),
    ] {
        let mut gesture = SelectionGesture::begin(
            PaneId(1),
            (100.0, 100.0),
            anchor,
            shift,
            control,
            false,
            "source@revision".into(),
        )
        .unwrap();
        assert!(!gesture.motion((102.0, 101.0), anchor));
        assert!(gesture.kind.is_none());
        assert!(gesture.motion(
            (100.0, 95.0),
            PointNm {
                y: 700_000,
                ..anchor
            }
        ));
        assert_eq!(gesture.mode, mode);
        assert_eq!(gesture.origin, anchor);
    }
    assert!(
        SelectionGesture::begin(
            PaneId(1),
            (0.0, 0.0),
            anchor,
            true,
            true,
            false,
            String::new()
        )
        .is_none()
    );
}

#[test]
#[ignore = "requires DATUM_JUNCTION_REPRO_ROOT frozen owner project; CPU only, no writes"]
fn owner_junction_center_qualifies_exact_pickup_and_connected_preview() {
    use datum_gui_protocol::{
        AuthoringSnapState, footprint_move_preview::native::NativeMovePreview,
        schematic_authoring::SchematicAuthoring, selection_resolution::NativeSelectionResolution,
        selection_scene::SceneSelectionBindings,
    };
    use datum_gui_render::{HitTarget, RetainedScene};
    let root = std::path::PathBuf::from(
        std::env::var_os("DATUM_JUNCTION_REPRO_ROOT").expect("frozen owner reproduction"),
    );
    let model = eda_engine::substrate::ProjectResolver::new(&root)
        .resolve()
        .unwrap();
    let authority = NativeSelectionResolution::capture(&model).unwrap();
    let scene = datum_gui_protocol::project_native_schematic_scene(&model, &root)
        .unwrap()
        .unwrap();
    let bindings = SceneSelectionBindings::capture(&scene, &authority);
    let mut workspace = datum_gui_protocol::load_fixture_workspace_state();
    workspace.schematic_scene = Some(scene.clone());
    let retained =
        RetainedScene::from_workspace_schematic_for_surface(&workspace, 1344, 806, 1.0).unwrap();
    let anchor = PointNm {
        x: -2_400_000,
        y: 1_200_000,
    };
    let target = retained.hit_test_authored_world_eligible(anchor, &workspace, |target| {
        let HitTarget::AuthoredObject(key) = target else {
            return false;
        };
        bindings
            .get(key)
            .is_some_and(|b| b.eligible(&workspace.ui.filters))
    });
    let Some(HitTarget::AuthoredObject(key)) = target else {
        panic!("owner marker must be hittable");
    };
    let hit = &bindings.get(&key).unwrap().acquired;
    assert_eq!(
        hit.class,
        Class::Junction,
        "visible marker wins over underlying Wire"
    );
    assert_eq!(hit.id.to_string(), "54bf64cc-59e4-44ea-852a-f921c765a689");
    let run = Subject::Run(id(Class::Wire, 7).into());
    assert_eq!(
        drag_subject(Some(&run), hit),
        Some(Subject::Object(hit.clone()))
    );
    let authoring = SchematicAuthoring::from_model(&model).unwrap();
    let source_before = serde_json::to_value(authoring.source.as_ref().unwrap()).unwrap();
    let preview = NativeMovePreview::connectivity(
        &scene,
        &authoring,
        hit,
        anchor,
        AuthoringSnapState {
            enabled: false,
            grid_nm: 100_000,
        },
        root,
    )
    .unwrap();
    let destination = PointNm {
        x: anchor.x,
        y: 700_000,
    };
    assert!(preview.action(destination).is_ok());
    drop(preview); // Cancellation discards intent; it never writes source geometry.
    assert_eq!(
        serde_json::to_value(authoring.source.as_ref().unwrap()).unwrap(),
        source_before
    );
}
