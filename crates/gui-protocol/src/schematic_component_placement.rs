//! Normal component capture uses complete reusable Parts; legacy Symbols stay explicit.
use super::*;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ComponentPlacement {
    pub active: bool,
    pub choices: Vec<LibrarySymbolChoice>,
    pub unavailable: Vec<(String, String)>,
}

impl SchematicAuthoring {
    pub fn begin_component_placement(&mut self) {
        self.components.active = true;
        self.picker = true;
        self.chosen = None;
        self.field = PlacementField::Reference;
        self.choice_offset = 0;
        if !self.components.choices.is_empty() {
            self.select(0);
        }
    }
    pub fn component_placement_refusal(&self) -> Option<String> {
        if !self.components.active {
            return None;
        }
        let component = self
            .chosen
            .and_then(|i| self.picker_choices().get(i))
            .and_then(|choice| choice.component.as_ref());
        match component {
            None => Some("Choose a complete component with a value, package, footprint and pin map".into()),
            Some(component) if component.value != self.value =>
                Some("Choose the component with the required value; its physical definition stays together".into()),
            Some(_) => None,
        }
    }
}
