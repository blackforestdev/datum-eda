//! Wrapped inventory previews use only inherited clipped Inspector rows.
use super::*;
use datum_gui_protocol::AuthoredSelectionIdentity;

pub(super) fn row_count(view: &SelectionView, id: &AuthoredSelectionIdentity) -> usize {
    1 + usize::from(view.has_inventory_states(id))
        + usize::from(view.has_unavailable_fields(id))
        + usize::from(!id.instance_path.is_empty())
}

pub(super) fn rows(
    view: &SelectionView,
    id: &AuthoredSelectionIdentity,
    ordinal: usize,
    columns: usize,
) -> Vec<String> {
    // Display order never becomes an authored identity or selection focus.
    let identity = truncate_text(&format!("{ordinal}: {}", id.id), columns / 2);
    let separator = " · ";
    let name = truncate_text(
        &view.member_display_label(id, ordinal),
        columns.saturating_sub(identity.chars().count() + separator.chars().count()),
    );
    let mut result = vec![
        [name, identity]
            .into_iter()
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>()
            .join(separator),
    ];
    if view.has_inventory_states(id) {
        let mut states = Vec::new();
        if view.hidden.contains(id) {
            states.push("hidden");
        }
        if view.locked.contains(id) {
            states.push("locked");
        }
        if view.unprojected.contains(id) {
            states.push("unprojected");
        }
        result.push(head_preview(&states.join(", "), columns));
    }
    let unavailable = view.unavailable_fields(id);
    if let Some(first) = unavailable.first() {
        let more = if unavailable.len() == 1 {
            String::new()
        } else {
            format!(" (+{})", unavailable.len() - 1)
        };
        let prefix = "Unavailable: ";
        let reason_columns = columns.saturating_sub(prefix.chars().count() + more.chars().count());
        let reason = head_preview(
            &datum_gui_protocol::selection_view::field_label(first),
            reason_columns,
        );
        result.push(head_preview(&format!("{prefix}{reason}{more}"), columns));
    }
    if !id.instance_path.is_empty() {
        // This is explicitly a shortened occurrence path, not the full path.
        let prefix = "Sheet path … / ";
        let path = id
            .instance_path
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(" / ");
        let clue = truncate_text(&path, columns.saturating_sub(prefix.chars().count()));
        result.push(head_preview(&format!("{prefix}{clue}"), columns));
    }
    result
}

fn head_preview(value: &str, columns: usize) -> String {
    if value.chars().count() <= columns {
        value.into()
    } else if columns == 0 {
        String::new()
    } else {
        format!("{}…", value.chars().take(columns - 1).collect::<String>())
    }
}
