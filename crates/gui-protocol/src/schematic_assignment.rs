//! Compatible library choices configure the shared source-qualified Proposal review.
use crate::{
    AuthoredSelectionClass, AuthoredSelectionIdentity, AuthoringViewBasis, ReviewWorkspaceState,
};
use eda_engine::api::native_write::{
    schematic_assignment::{self, AssignmentChoice, AssignmentOptions},
    schematic_delete_proposal::{AssignmentSummary, NativeProposalAction},
};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
pub struct AssignmentEvidence {
    pub target: AuthoredSelectionIdentity,
    pub options: AssignmentOptions,
    pub selected: usize,
}
impl AssignmentEvidence {
    /// Paint admission uses the already captured model only; compatibility is read on entry.
    pub fn admit_target(
        state: &ReviewWorkspaceState,
        target: &AuthoredSelectionIdentity,
    ) -> Result<AuthoringViewBasis, String> {
        if target.class != AuthoredSelectionClass::Symbol {
            return Err("Select one native pool Symbol".into());
        }
        if state.project_unopened || !crate::gui_menu_model::native_project_edit_available(state) {
            return Err("Open a persistent native Project".into());
        }
        if state.schematic_authoring.delete_review.is_some()
            || state.schematic_authoring.picker
            || state.schematic_authoring.active()
        {
            return Err("Finish or cancel the current editor operation".into());
        }
        let basis = AuthoringViewBasis::capture(state, state.ui.layout.focused)?;
        if basis.content != crate::PaneContent::Schematic
            || basis
                .document
                .as_ref()
                .is_none_or(|d| d.instance_path != target.instance_path)
        {
            return Err("Select a Symbol in the focused supported schematic occurrence".into());
        }
        let model = state
            .library
            .basis
            .as_ref()
            .ok_or("Current library source is unavailable")?;
        if model.project.project_id != basis.project || model.model_revision.0 != basis.revision {
            return Err("Library source changed; select the current Symbol again".into());
        }
        let source = state
            .schematic_authoring
            .source
            .as_ref()
            .and_then(|s| s.sheets.get(&basis.source))
            .and_then(|sheet| sheet.symbols.get(&target.id))
            .ok_or("Selected Symbol source unavailable")?;
        let symbol: Uuid = source
            .lib_id
            .as_deref()
            .and_then(|id| id.parse().ok())
            .ok_or("Assignment requires a native pool Symbol")?;
        if !model
            .objects
            .get(&symbol)
            .is_some_and(|o| o.domain == "pool" && o.kind == "symbols")
        {
            return Err("Native pool Symbol is unavailable".into());
        }
        if state.schematic_authoring.move_paths.get(&target.id) != Some(&target.instance_path) {
            return Err("Assignment requires one supported Symbol occurrence".into());
        }
        if let crate::SelectionTarget::Native(view) = &state.selection
            && view.locked.contains(target)
        {
            return Err("Selected Symbol is locked".into());
        }
        let mut owners = model
            .component_instances
            .values()
            .filter(|c| c.placed_symbol_refs.contains(&target.id));
        if let Some(owner) = owners.next() {
            if owners.next().is_some()
                || owner.placed_symbol_refs.len() != 1
                || !owner.placed_package_refs.is_empty()
                || owner.authority != eda_engine::substrate::ComponentInstanceAuthority::Authored
            {
                return Err(
                    "Assignment supports one schematic-only Symbol per authored ComponentInstance"
                        .into(),
                );
            }
        }
        Ok(basis)
    }
    pub fn admit_selected(state: &ReviewWorkspaceState) -> Result<(), String> {
        let (_, target) = super::selected::target(state)?;
        Self::admit_target(state, &target).map(|_| ())
    }
    pub fn capture(
        state: &ReviewWorkspaceState,
        target: &AuthoredSelectionIdentity,
    ) -> Result<Self, String> {
        let basis = Self::admit_target(state, target)?;
        let model = state
            .library
            .basis
            .as_ref()
            .ok_or("Current library source unavailable")?;
        let options = schematic_assignment::options(model, target.id, &target.instance_path)
            .map_err(|e| e.to_string())?;
        if options.sheet != basis.source {
            return Err("Symbol belongs to another sheet".into());
        }
        if options.choices.is_empty() {
            return Err("No compatible Part/Gate for this Symbol's existing Unit".into());
        }
        Ok(Self {
            target: target.clone(),
            options,
            selected: 0,
        })
    }
    pub fn selected(state: &ReviewWorkspaceState) -> Result<Self, String> {
        let (_, target) = super::selected::target(state)?;
        Self::capture(state, &target)
    }
    pub fn choice(&self) -> Option<&AssignmentChoice> {
        self.options.choices.get(self.selected)
    }
    pub fn cycle(&mut self, gate: bool) {
        let Some(current) = self.choice().cloned() else {
            return;
        };
        let choices = &self.options.choices;
        for offset in 1..=choices.len() {
            let index = (self.selected + offset) % choices.len();
            let candidate = &choices[index];
            if if gate {
                candidate.part == current.part
            } else {
                candidate.part != current.part
            } {
                self.selected = index;
                return;
            }
        }
    }
    pub fn part_label(&self) -> String {
        self.choice().map_or_else(
            || "No compatible Part".into(),
            |c| {
                let mut parts = Vec::new();
                for choice in &self.options.choices {
                    if !parts.contains(&choice.part) {
                        parts.push(choice.part);
                    }
                }
                format!(
                    "{} · {}/{}",
                    if c.part_name.is_empty() {
                        "Unnamed Part"
                    } else {
                        &c.part_name
                    },
                    parts.iter().position(|id| *id == c.part).unwrap_or(0) + 1,
                    parts.len()
                )
            },
        )
    }
    pub fn gate_label(&self) -> String {
        self.choice().map_or_else(
            || "No compatible Gate".into(),
            |c| format!("{} · {}", c.gate_name, c.entity_name),
        )
    }
    pub fn action(&self, proposal_id: Uuid) -> Result<NativeProposalAction, String> {
        let choice = self.choice().ok_or("Choose a compatible Part/Gate")?;
        let original = &self.options.original;
        if self.options.component_instance.is_some()
            && original.part == Some(choice.part)
            && original.entity == Some(choice.entity)
            && original.gate == Some(choice.gate)
        {
            return Err("This Part/Gate is already assigned; choose another or cancel".into());
        }
        Ok(NativeProposalAction::CreateAssignment {
            proposal_id,
            symbol: self.target.id,
            instance_path: self.target.instance_path.clone(),
            part: choice.part,
            gate: choice.gate,
        })
    }
    pub fn matches(&self, summary: &AssignmentSummary) -> bool {
        self.choice().is_some_and(|choice| {
            summary.component_instance != Uuid::nil()
                && summary.symbol == self.target.id
                && summary.sheet == self.options.sheet
                && summary.instance_path == self.target.instance_path
                && summary.previous_part == self.options.original.part
                && summary.reference == self.options.original.reference
                && summary.choice.part == choice.part
                && summary.choice.entity == choice.entity
                && summary.choice.gate == choice.gate
                && self
                    .options
                    .component_instance
                    .is_none_or(|id| summary.component_instance == id)
        })
    }
}
