use super::*;

#[test]
fn selected_board_text_numeric_rows_have_step_and_center_edit_zones() {
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    let object_id = "board-text:test-hit-zones".to_string();
    state
        .scene
        .board_texts
        .push(datum_gui_protocol::BoardTextPrimitive {
            object_id: object_id.clone(),
            object_kind: "board_text".to_string(),
            text_uuid: "test-hit-zones".to_string(),
            source_object_uuid: None,
            text: "TEST".to_string(),
            layer_id: "F.Silks".to_string(),
            position: PointNm { x: 0, y: 0 },
            rotation_degrees: 0,
            height_nm: 1_000_000,
            stroke_width_nm: 100_000,
            render_intent: "annotation".to_string(),
            family: "inter".to_string(),
            style: "regular".to_string(),
            style_class: None,
            h_align: "center".to_string(),
            v_align: "center".to_string(),
            mirrored: false,
            keep_upright: true,
            line_spacing_ratio_ppm: 1_000_000,
            bold: false,
            italic: false,
        });
    state.selection = SelectionTarget::AuthoredObject(object_id);

    let retained = RetainedScene::from_workspace(&state, 1280, 800);
    let prepared = PreparedScene::from_workspace(
        &state,
        1280,
        800,
        CameraState::fit_to_bounds(&state.scene.bounds),
        &retained,
    )
    .unwrap();

    assert_three_zone_row(
        &prepared,
        HitTarget::DecreaseSelectedBoardTextHeight,
        HitTarget::EditSelectedBoardTextHeight,
        HitTarget::IncreaseSelectedBoardTextHeight,
    );
    assert_three_zone_row(
        &prepared,
        HitTarget::RotateSelectedBoardTextCounterClockwise90,
        HitTarget::EditSelectedBoardTextRotation,
        HitTarget::RotateSelectedBoardTextClockwise90,
    );
    assert_three_zone_row(
        &prepared,
        HitTarget::DecreaseSelectedBoardTextLineSpacing,
        HitTarget::EditSelectedBoardTextLineSpacing,
        HitTarget::IncreaseSelectedBoardTextLineSpacing,
    );
    assert_three_zone_row(
        &prepared,
        HitTarget::CycleSelectedBoardTextRenderIntent,
        HitTarget::EditSelectedBoardTextRenderIntent,
        HitTarget::CycleSelectedBoardTextRenderIntent,
    );
    assert_three_zone_row(
        &prepared,
        HitTarget::CycleSelectedBoardTextFamily,
        HitTarget::EditSelectedBoardTextFamily,
        HitTarget::CycleSelectedBoardTextFamily,
    );
    assert_three_zone_row(
        &prepared,
        HitTarget::CycleSelectedBoardTextHAlign,
        HitTarget::EditSelectedBoardTextAlignment,
        HitTarget::CycleSelectedBoardTextVAlign,
    );
}

fn assert_three_zone_row(
    prepared: &PreparedScene,
    left: HitTarget,
    center: HitTarget,
    right: HitTarget,
) {
    let left_rect = hit_rect(prepared, &left);
    let center_rect = hit_rect(prepared, &center);
    let right_rect = hit_rect_from_end(prepared, &right);
    assert!(
        (left_rect.y - center_rect.y).abs() < f32::EPSILON
            && (center_rect.y - right_rect.y).abs() < f32::EPSILON,
        "three-zone hit regions must share one row"
    );
    assert!(left_rect.x < center_rect.x);
    assert!(center_rect.x < right_rect.x);
    assert!(center_rect.width > left_rect.width);
    assert!(center_rect.width > right_rect.width);

    assert_eq!(hit_center(prepared, left_rect), left);
    assert_eq!(hit_center(prepared, center_rect), center);
    assert_eq!(hit_center(prepared, right_rect), right);
}

fn hit_rect(prepared: &PreparedScene, target: &HitTarget) -> RectPx {
    prepared
        .hit_regions
        .iter()
        .find(|region| &region.target == target)
        .map(|region| region.rect)
        .unwrap_or_else(|| panic!("expected hit region for {target:?}"))
}

fn hit_rect_from_end(prepared: &PreparedScene, target: &HitTarget) -> RectPx {
    prepared
        .hit_regions
        .iter()
        .rev()
        .find(|region| &region.target == target)
        .map(|region| region.rect)
        .unwrap_or_else(|| panic!("expected hit region for {target:?}"))
}

fn hit_center(prepared: &PreparedScene, rect: RectPx) -> HitTarget {
    prepared
        .hit_test(rect.x + rect.width * 0.5, rect.y + rect.height * 0.5)
        .cloned()
        .expect("hit target should exist at rect center")
}
