//! Navigator and Inspector profiles over the existing Accessible/child/state service.
use super::*;
use crate::selection_accessibility::SelectionAccessibilitySnapshot as Snapshot;
use crate::selection_accessibility::workspace::WorkspaceInspection;
use datum_gui_protocol::inspector_context::InspectorSectionId;
use datum_gui_protocol::schematic_documents::SchematicDocumentKey;

pub(in crate::terminal_accessibility_platform) const NAVIGATOR: &str =
    "/org/a11y/atspi/accessible/selection/navigator";
const SECTIONS: &str = "/org/a11y/atspi/accessible/selection/sections/";

pub(super) fn document_path(key: &SchematicDocumentKey) -> String {
    format!(
        "{NAVIGATOR}/n{}",
        encoded(&serde_json::to_string(key).expect("typed document identity"))
    )
}
pub(super) fn section_path(id: InspectorSectionId) -> String {
    format!(
        "{SECTIONS}{}",
        encoded(&serde_json::to_string(&id).expect("typed section identity"))
    )
}
pub(super) fn field_path(id: InspectorSectionId, index: usize) -> String {
    format!("{}/fields/{index}", section_path(id))
}
pub(super) fn node<'a>(ui: &'a WorkspaceInspection, path: &str) -> Option<Node<'a>> {
    if path == NAVIGATOR {
        return Some(Node::Navigator);
    }
    if let Some(key) = path.strip_prefix(&format!("{NAVIGATOR}/n")) {
        let key: SchematicDocumentKey = serde_json::from_str(&decoded(key)?).ok()?;
        return ui
            .documents
            .iter()
            .find(|row| row.key == key)
            .map(Node::Document);
    }
    let rest = path.strip_prefix(SECTIONS)?;
    let (key, suffix) = rest.split_once('/').unwrap_or((rest, ""));
    let id: InspectorSectionId = serde_json::from_str(&decoded(key)?).ok()?;
    let section = ui.sections.iter().find(|section| section.id == id)?;
    if suffix.is_empty() {
        return Some(Node::Section(section));
    }
    if suffix == "run_erc" && section.id == InspectorSectionId::Checks {
        return Some(Node::RunErc);
    }
    let index = suffix.strip_prefix("fields/")?.parse::<usize>().ok()?;
    (index < section.fields.len()).then_some(Node::SectionField(section, index))
}

pub(super) fn paths<'a>(
    ui: &'a WorkspaceInspection,
    node: Node<'a>,
) -> Option<Box<dyn Iterator<Item = String> + 'a>> {
    match node {
        Node::Root => Some(Box::new(
            ui.sections.iter().map(|section| section_path(section.id)),
        )),
        Node::Navigator => Some(Box::new(
            ui.documents
                .iter()
                .filter(|row| row.parent.is_none())
                .map(|row| document_path(&row.key)),
        )),
        Node::Document(parent) => Some(Box::new(
            ui.documents
                .iter()
                .filter(move |row| row.parent.as_ref() == Some(&parent.key))
                .map(|row| document_path(&row.key)),
        )),
        Node::Section(section) => Some(Box::new(
            (0..section.fields.len())
                .map(move |index| field_path(section.id, index))
                .chain(
                    (section.id == InspectorSectionId::Checks)
                        .then(|| format!("{}/run_erc", section_path(section.id))),
                ),
        )),
        Node::SectionField(_, _) | Node::RunErc => Some(Box::new(std::iter::empty())),
        _ => None,
    }
}

pub(super) fn name(snapshot: &Snapshot, node: Node<'_>) -> Option<String> {
    let ui = snapshot.inspection.as_ref()?;
    Some(match node {
        Node::Root => format!("Inspector — {}", ui.title),
        Node::Navigator => "Navigator".into(),
        Node::Document(row) => row.name.clone(),
        Node::Section(section) => section.id.label().into(),
        Node::SectionField(section, index) => {
            let field = &section.fields[index];
            let value = field
                .native_field
                .as_ref()
                .and_then(|name| {
                    snapshot
                        .view
                        .as_ref()
                        .map(|view| view.field_display_preview(name, usize::MAX))
                })
                .or_else(|| field.text.clone())
                .unwrap_or_else(|| "Unavailable".into());
            format!("{}: {value}", field.label)
        }
        Node::RunErc => "Rerun ERC".into(),
        _ => return None,
    })
}
pub(super) fn description(snapshot: &Snapshot, node: Node<'_>) -> Option<String> {
    let ui = snapshot.inspection.as_ref()?;
    Some(match node {
        Node::Root => format!("Read-only Inspector follows one inspection owner. {}. Project {}; revision {}. Arrow keys navigate named sections; Left/Right collapse/expand; Space toggles; R on Checks explicitly reruns ERC. Tab returns to the editor; Shift+Tab reaches Navigator. Inspection does not change authored selection.", ui.title, ui.project, ui.revision),
        Node::Navigator => format!("Native schematic document occurrence tree. {} complete rows; Project {}; revision {}. Arrow keys inspect; Enter opens the exact document beside the focused pane; Tab reaches Inspector. Opening and inspection write no authored data.", ui.documents.len(), ui.project, ui.revision),
        Node::Document(row) => row.description.clone(),
        Node::Section(section) => format!("{} section, {}. Read-only values; Left/Right collapse/expand; Space toggles.", section.id.label(), if section.collapsed { "collapsed" } else { "expanded" }),
        Node::SectionField(_, _) => name(snapshot, node)?,
        Node::RunErc => if ui.rerun_available { "Explicit Project ERC job available. Focus Checks and press R. Starting is request intent; Running requires the actual producer acknowledgement; Complete requires matching persisted readback." } else { "Unavailable while the source basis is unavailable or a job is already pending." }.into(),
        _ => return None,
    })
}
pub(super) fn parent(node: Node<'_>) -> Option<String> {
    Some(match node {
        Node::Navigator => ROOT_PATH.into(),
        Node::Document(row) => row
            .parent
            .as_ref()
            .map(document_path)
            .unwrap_or_else(|| NAVIGATOR.into()),
        Node::Section(_) => PATH.into(),
        Node::SectionField(section, _) => section_path(section.id),
        Node::RunErc => section_path(InspectorSectionId::Checks),
        _ => return None,
    })
}
pub(super) fn role(node: Node<'_>) -> Option<GlobalPreferencesAccessibleRole> {
    use GlobalPreferencesAccessibleRole as R;
    Some(match node {
        Node::Navigator => R::Navigation,
        Node::Document(_) => R::ListItem,
        Node::Section(_) => R::Group,
        Node::SectionField(_, _) => R::TextBox,
        Node::RunErc => R::Button,
        _ => return None,
    })
}
pub(super) fn focus(snapshot: &Snapshot, node: Node<'_>) -> Option<bool> {
    let ui = snapshot.inspection.as_ref()?;
    Some(match node {
        Node::Root => ui.inspector_focused,
        Node::Navigator => ui.navigator_focused,
        Node::Document(row) => ui.navigator_focused && row.inspected && row.visible,
        Node::Section(section) => ui.inspector_focused && section.focused,
        _ => false,
    })
}
pub(super) fn attributes(snapshot: &Snapshot, node: Node<'_>) -> Vec<(&'static str, String)> {
    match node {
        Node::Document(row) => vec![
            (
                "document-identity",
                serde_json::to_string(&row.key).expect("document identity"),
            ),
            ("inspected", row.inspected.to_string()),
            ("expanded", row.expanded.to_string()),
            ("selected", "false".into()),
        ],
        Node::Section(section) => vec![("expanded", (!section.collapsed).to_string())],
        Node::RunErc => vec![
            (
                "available",
                snapshot
                    .inspection
                    .as_ref()
                    .is_some_and(|ui| ui.rerun_available)
                    .to_string(),
            ),
            ("keyboard-action", "Focus Checks; R".into()),
        ],
        _ => Vec::new(),
    }
}
