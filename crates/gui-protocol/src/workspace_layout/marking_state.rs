#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarkingMenuState {
    pub menu_key: String,
    pub target_object_id: Option<String>,
    pub anchor_x_px: i32,
    pub anchor_y_px: i32,
    pub preview_slot: Option<String>,
    pub gesture_dx_px: i32,
    pub gesture_dy_px: i32,
    pub selection: Option<crate::selection_menu::SelectionMenu>,
}
