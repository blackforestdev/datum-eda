//! UI previews never change canonical fields, scope or complete membership.
use datum_gui_protocol::selection_view::InspectionValue;
pub(super) fn value(value: Option<&InspectionValue>, columns: usize) -> String {
    value
        .map(|value| value.display_preview(columns))
        .unwrap_or_else(|| "Unavailable".into())
}
