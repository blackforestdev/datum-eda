//! Exact form/preview integration; ordinary input and async outcomes need GUI proof.
use super::*;
use datum_gui_protocol::{AuthoringViewBasis, CoordinateEntryControl as C};
use eda_engine::ir::units::UnitsProfile;
use uuid::Uuid;
fn point(x: i64, y: i64) -> PointNm {
    PointNm { x, y }
}
fn form() -> CoordinateEntryForm {
    let project = Uuid::from_u128(1);
    CoordinateEntryForm {
        entry: CoordinateEntry::new(
            project,
            LengthQuantity::SchematicGeometry,
            UnitsProfile::default().resolve().unwrap(),
            point(2_731, 5_019),
            point(3_731, 7_019),
            CoordinateEntryMode::Absolute,
        )
        .unwrap(),
        focus: C::X,
        label: "Position".into(),
        refusal: None,
        restore_position: point(3_731, 7_019),
        basis: AuthoringViewBasis {
            project,
            revision: "captured".into(),
            pane: datum_gui_protocol::PaneId(1),
            content: PaneContent::Schematic,
            source: Uuid::from_u128(2),
            document: None,
        },
        pending: false,
        stale: false,
    }
}
#[test]
fn captured_delta_form_keeps_exact_reference_restore_pose_and_field_editor_selection() {
    let mut form = form();
    form.entry.set_mode(CoordinateEntryMode::Delta).unwrap();
    assert_eq!(form.entry.destination().unwrap(), form.restore_position);
    form.entry.x = "1nm".into();
    form.entry.y = "-2nm".into();
    assert_eq!(form.entry.destination().unwrap(), point(2732, 5017));
    form.entry.x_editor.select_all(&form.entry.x);
    form.entry
        .x_editor
        .replace(&mut form.entry.x, "0.000003mm")
        .unwrap();
    assert_eq!(form.entry.destination().unwrap(), point(2734, 5017));
    form.entry.set_mode(CoordinateEntryMode::Absolute).unwrap();
    assert_eq!(form.entry.destination().unwrap(), point(2734, 5017));
    assert_eq!(
        form.restore_position,
        point(3731, 7019),
        "closing form restores preform pose, not edited fields"
    );
    assert_eq!(form.basis.revision, "captured");
}
#[test]
fn checked_grid_displacement_preserves_offgrid_residue_and_refuses_integer_overflow() {
    let original = point(2_731, -5_019);
    let right = checked_move_delta(original, 1000, 0).unwrap();
    let up = checked_move_delta(right, 0, -1000).unwrap();
    assert_eq!(up, point(3731, -6019));
    assert_eq!(checked_move_delta(up, -1000, 1000).unwrap(), original);
    assert!(checked_move_delta(point(i64::MAX, 0), 1, 0).is_err());
    assert!(checked_move_delta(point(0, i64::MIN), 0, -1).is_err());
}
// Reuse demo geometry only; native Move requires a real UUID identity. Keep
// every typed owner reference coherent without changing designator/text labels.
fn native_identity_scene() -> datum_gui_protocol::BoardReviewSceneV1 {
    let mut scene = datum_gui_protocol::load_fixture_workspace_state().scene;
    let old = scene
        .components
        .first()
        .expect("fixture component")
        .component_uuid
        .clone();
    let native = Uuid::from_u128(301).to_string();
    for component in &mut scene.components {
        if component.component_uuid == old {
            component.component_uuid = native.clone();
        }
    }
    for pad in &mut scene.pads {
        if pad.component_uuid == old {
            pad.component_uuid = native.clone();
        }
    }
    for graphic in &mut scene.component_graphics {
        if graphic.component_uuid == old {
            graphic.component_uuid = native.clone();
        }
    }
    for text in &mut scene.component_texts {
        if text.component_uuid == old {
            text.component_uuid = native.clone();
        }
    }
    for wire in &mut scene.unrouted_primitives {
        if wire.from_component == old {
            wire.from_component = native.clone();
        }
        if wire.to_component == old {
            wire.to_component = native.clone();
        }
    }
    scene
}
#[test]
fn exact_numeric_footprint_action_bypasses_enabled_grid_without_rebuilding_authority() {
    let scene = native_identity_scene();
    let component = scene.components.first().expect("fixture component");
    let snap = datum_gui_protocol::AuthoringSnapState {
        enabled: true,
        grid_nm: 1_000_000,
        ..Default::default()
    };
    let preview = NativeMovePreview::Footprint(
        FootprintMovePreview::begin(&scene, &component.object_id, component.position, snap)
            .unwrap(),
    );
    let reference = preview.reference_position();
    let exact = checked_move_delta(reference, 2731, 5019).unwrap();
    let action = preview.action(exact).unwrap();
    assert!(
        matches!(action,NativeEditAction::Move{component,x_nm,y_nm} if component==Uuid::from_u128(301) && x_nm==exact.x && y_nm==exact.y)
    );
    preview.scene_at(exact).unwrap();
    assert_eq!(
        preview.reference_position(),
        reference,
        "numeric action retains immutable source anchor"
    );
}

#[test]
fn live_coordinate_fields_preflight_exact_ghost_and_keep_restore_on_invalid_input() {
    let scene = native_identity_scene();
    let component = scene.components.first().expect("fixture component");
    let preview = NativeMovePreview::Footprint(
        FootprintMovePreview::begin(
            &scene,
            &component.object_id,
            component.position,
            Default::default(),
        )
        .unwrap(),
    );
    let mut form = form();
    let restore = form.restore_position;
    form.entry.x = "0.000007mm".into();
    form.entry.y = "0.000011mm".into();
    let valid = coordinate_preview_position(&form.entry, &preview).unwrap();
    assert_eq!(valid, point(7, 11));
    let valid_scene = preview.scene_at(valid).unwrap();
    form.entry.x = "not-a-length".into();
    let text = form.entry.x.clone();
    let focus = form.focus;
    assert!(coordinate_preview_position(&form.entry, &preview).is_err());
    assert_eq!(form.entry.x, text);
    assert_eq!(form.focus, focus);
    assert_eq!(form.restore_position, restore);
    assert_eq!(
        preview.scene_at(valid).unwrap(),
        valid_scene,
        "invalid buffered input cannot replace last valid ghost"
    );
    form.entry.x = "7nm".into();
    form.entry.set_mode(CoordinateEntryMode::Delta).unwrap();
    assert_eq!(
        coordinate_preview_position(&form.entry, &preview).unwrap(),
        valid
    );
    assert_eq!(form.restore_position, restore);
}

#[test]
fn current_menu_and_modal_ownership_admission_keeps_form_and_passive_focus() {
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    state.authoring.coordinates = Some(form());
    state.ui.focus = datum_gui_protocol::ApplicationFocus::Terminal;
    let before = state.authoring.coordinates.clone();
    // Terminal focus alone permits an explicit opener/field click to transfer
    // focus; an existing menu/modal must instead retain its current authority.
    assert!(coordinate_owner_available(&state));
    for owner in 0..4 {
        state.ui.active_menu = (owner == 0).then(|| "View".into());
        state.ui.new_project.open = owner == 1;
        state.ui.project_open.open = owner == 2;
        state.library.acquisition = (owner == 3).then(|| {
            datum_gui_protocol::library_browser::acquisition::LibraryAcquisition::begin(
                "captured".into(),
                None,
            )
        });
        assert!(!coordinate_owner_available(&state));
        assert_eq!(state.authoring.coordinates, before);
        assert_eq!(
            state.ui.focus,
            datum_gui_protocol::ApplicationFocus::Terminal
        );
    }
}
