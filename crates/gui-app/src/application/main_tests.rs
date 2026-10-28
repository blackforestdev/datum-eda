use super::*;
use crate::app_bootstrap::parse_window_size;

#[test]
fn parses_visual_window_size() {
    assert_eq!(parse_window_size("1280x768").unwrap(), (1280, 768));
}

#[test]
fn bounds_from_points_applies_padding() {
    let bounds = bounds_from_points([PointNm { x: 10, y: 20 }, PointNm { x: 30, y: -10 }], 5)
        .expect("bounds should exist");
    assert_eq!(
        bounds,
        SceneBounds {
            min_x: 5,
            min_y: -15,
            max_x: 35,
            max_y: 25
        }
    );
}

#[test]
fn rejects_invalid_visual_window_size() {
    assert!(parse_window_size("1280").is_err());
    assert!(parse_window_size("0x768").is_err());
    assert!(parse_window_size("1280x0").is_err());
}

#[test]
fn marking_menu_key_maps_phase_one_board_objects() {
    // Single-object (count 1) resolves to the per-type wheel.
    assert_eq!(
        marking_menu_key_for_target(Some("component:U1"), 1),
        "pcb.component"
    );
    assert_eq!(marking_menu_key_for_target(Some("pad:P1"), 1), "pcb.pad");
    assert_eq!(
        marking_menu_key_for_target(Some("track:T1"), 1),
        "pcb.track"
    );
    assert_eq!(marking_menu_key_for_target(Some("via:V1"), 1), "pcb.via");
    assert_eq!(marking_menu_key_for_target(Some("zone:Z1"), 1), "pcb.zone");
    // No object resolves to the empty-canvas wheel.
    assert_eq!(marking_menu_key_for_target(None, 0), "pcb.empty");
}

#[test]
fn marking_menu_key_resolves_multiselect_over_per_type() {
    // A multi-object selection opens the multi-select wheel regardless of which
    // object is under the cursor (its Align/Distribute sub-wheels live only here).
    assert_eq!(
        marking_menu_key_for_target(Some("component:U1"), 3),
        "pcb.multiselect"
    );
    assert_eq!(marking_menu_key_for_target(None, 2), "pcb.multiselect");
    // The boundary: exactly one selected object is still the per-type wheel.
    assert_eq!(
        marking_menu_key_for_target(Some("component:U1"), 1),
        "pcb.component"
    );
}

#[test]
fn marking_slot_for_delta_uses_screen_direction_wheel() {
    assert_eq!(marking_slot_for_delta(0, -40).as_deref(), Some("N"));
    assert_eq!(marking_slot_for_delta(40, 0).as_deref(), Some("E"));
    assert_eq!(marking_slot_for_delta(0, 40).as_deref(), Some("S"));
    assert_eq!(marking_slot_for_delta(-40, 0).as_deref(), Some("W"));
    assert_eq!(marking_slot_for_delta(30, -30).as_deref(), Some("NE"));
    assert_eq!(marking_slot_for_delta(3, 3), None);
}

#[test]
fn terminal_paste_bytes_wraps_when_bracketed_paste_is_enabled() {
    assert_eq!(terminal_paste_bytes("alpha\nbeta", false), b"alpha\nbeta");
    assert_eq!(
        terminal_paste_bytes("alpha\nbeta", true),
        b"\x1b[200~alpha\nbeta\x1b[201~"
    );
}

#[test]
fn assistant_activity_command_is_session_scoped() {
    assert!(ASSISTANT_ACTIVITY_COMMAND.contains("context session-activity"));
    assert!(ASSISTANT_ACTIVITY_COMMAND.contains("$DATUM_SESSION_ID"));
    assert!(ASSISTANT_ACTIVITY_COMMAND.contains("--limit 20"));
    assert_eq!(
        ASSISTANT_ACTIVITY_COMMAND,
        "datum-eda context session-activity --session \"$DATUM_SESSION_ID\" --limit 20"
    );
}

#[cfg(feature = "visual")]
#[test]
fn converts_bgra_readback_to_rgba() {
    let mut pixels = vec![1, 2, 3, 255, 10, 20, 30, 255];
    convert_texture_pixels_to_rgba(&mut pixels, wgpu::TextureFormat::Bgra8UnormSrgb).unwrap();
    assert_eq!(pixels, vec![3, 2, 1, 255, 30, 20, 10, 255]);
}
