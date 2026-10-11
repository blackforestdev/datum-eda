//! Transient Inspector navigation; never selection membership or authored data.
/// Read-only page navigation through the existing Inspector projection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InspectorPage {
    First,
    Previous,
    Next,
    Last,
}
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SelectionInspectorState {
    pub inventory_expanded: bool,
    pub filters_expanded: bool,
    pub scroll_offset: usize,
}
impl SelectionInspectorState {
    pub fn toggle_inventory(&mut self) {
        self.inventory_expanded = !self.inventory_expanded;
    }
    pub fn toggle_filters(&mut self) {
        self.filters_expanded = !self.filters_expanded;
    }
    pub fn reset_scroll(&mut self) {
        self.scroll_offset = 0;
    }
}

pub const ACQUISITION_CLASSES: [crate::AuthoredSelectionClass; 18] = [
    crate::AuthoredSelectionClass::Footprint,
    crate::AuthoredSelectionClass::Pad,
    crate::AuthoredSelectionClass::Track,
    crate::AuthoredSelectionClass::Via,
    crate::AuthoredSelectionClass::Zone,
    crate::AuthoredSelectionClass::BoardText,
    crate::AuthoredSelectionClass::BoardGraphic,
    crate::AuthoredSelectionClass::BoardOutline,
    crate::AuthoredSelectionClass::Symbol,
    crate::AuthoredSelectionClass::Pin,
    crate::AuthoredSelectionClass::Wire,
    crate::AuthoredSelectionClass::BusSection,
    crate::AuthoredSelectionClass::Label,
    crate::AuthoredSelectionClass::Port,
    crate::AuthoredSelectionClass::Junction,
    crate::AuthoredSelectionClass::NoConnect,
    crate::AuthoredSelectionClass::SchematicText,
    crate::AuthoredSelectionClass::SchematicGraphic,
];

pub fn class_label(class: crate::AuthoredSelectionClass) -> &'static str {
    match class {
        crate::AuthoredSelectionClass::Footprint => "Parts",
        crate::AuthoredSelectionClass::Pad => "Pads",
        crate::AuthoredSelectionClass::Track => "Tracks",
        crate::AuthoredSelectionClass::Via => "Vias",
        crate::AuthoredSelectionClass::Zone => "Zones",
        crate::AuthoredSelectionClass::BoardText => "Board text",
        crate::AuthoredSelectionClass::BoardGraphic => "Board graphics",
        crate::AuthoredSelectionClass::BoardOutline => "Board outline",
        crate::AuthoredSelectionClass::Symbol => "Symbols",
        crate::AuthoredSelectionClass::Pin => "Pins",
        crate::AuthoredSelectionClass::Wire => "Wires",
        crate::AuthoredSelectionClass::BusSection => "Bus sections",
        crate::AuthoredSelectionClass::Label => "Labels",
        crate::AuthoredSelectionClass::Port => "Ports",
        crate::AuthoredSelectionClass::Junction => "Junctions",
        crate::AuthoredSelectionClass::NoConnect => "No-connects",
        crate::AuthoredSelectionClass::SchematicText => "Schematic text",
        crate::AuthoredSelectionClass::SchematicGraphic => "Schematic graphics",
    }
}

/// Compact source-qualified basis for the rendered row budget, not its formula.
/// Complete identities remain guarded by the existing selection menu subject.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InspectorPresentationBasis {
    pub subject: std::sync::Arc<crate::SelectionSubject>,
    pub classes: std::collections::BTreeMap<crate::AuthoredSelectionClass, usize>,
    pub project: crate::SelectionProjectId,
    pub revision: super::SelectionModelRevision,
    pub navigation: SelectionInspectorState,
    pub scope: Option<crate::AuthoredSelectionClass>,
    pub stale: bool,
    pub focus: Option<crate::AuthoredSelectionIdentity>,
    pub notice: Option<String>,
    pub related_count: usize,
    pub member_count: usize,
    pub field_count: usize,
}
impl InspectorPresentationBasis {
    pub fn capture(view: &super::SelectionView, navigation: &SelectionInspectorState) -> Self {
        let mut classes = std::collections::BTreeMap::new();
        for id in &view.members {
            *classes.entry(id.class).or_insert(0) += 1;
        }
        Self {
            subject: std::sync::Arc::new(view.subject.clone()),
            classes,
            project: view.project,
            revision: view.revision.clone(),
            navigation: navigation.clone(),
            scope: view.scope,
            stale: view.stale,
            focus: view.focus().cloned(),
            notice: view.notice.clone(),
            related_count: view.related.len(),
            member_count: view.members.len(),
            field_count: view.fields().len(),
        }
    }
}
