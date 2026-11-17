//! Disposable retained assignment target; Library commits remain independent.
use crate::schematic_authoring::assignment::AssignmentEvidence;
use crate::{AuthoredSelectionIdentity, AuthoringViewBasis, ReviewWorkspaceState};
use serde_json::Value;
use uuid::Uuid;
#[path = "component_configuration.rs"]
pub mod configuration;
#[cfg(test)]
#[path = "component_resolution_tests.rs"]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolutionAction {
    Entity,
    Footprint,
    Part,
    Map,
    Assign,
    Return,
    Cancel,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ComponentResolution {
    pub root: std::path::PathBuf,
    pub basis: AuthoringViewBasis,
    pub evidence: AssignmentEvidence,
    pub sheet_preimage: Value,
    pub library_pane: Option<crate::PaneId>,
    pub transfer: bool,
    pub in_library: bool,
    pub authored: Vec<(String, Uuid)>,
    pub refusal: Option<String>,
    pub focus: Option<ResolutionAction>,
}
impl ComponentResolution {
    pub fn capture(
        state: &ReviewWorkspaceState,
        target: &AuthoredSelectionIdentity,
        transfer: bool,
    ) -> Result<Self, String> {
        let basis = AssignmentEvidence::admit_target(state, target)?;
        let evidence = AssignmentEvidence::capture_resolution(state, target)?;
        let sheet = state
            .schematic_authoring
            .source
            .as_ref()
            .and_then(|s| s.sheets.get(&basis.source))
            .ok_or("Target sheet unavailable")?;
        Ok(Self {
            root: state
                .backing
                .as_ref()
                .ok_or("Persistent project source unavailable")?
                .request
                .project_root
                .clone(),
            basis,
            evidence,
            sheet_preimage: serde_json::to_value(sheet).map_err(|e| e.to_string())?,
            library_pane: None,
            transfer,
            in_library: false,
            authored: vec![],
            refusal: None,
            focus: None,
        })
    }
    pub fn verify(&self, state: &ReviewWorkspaceState) -> Result<(), String> {
        if state
            .backing
            .as_ref()
            .is_none_or(|b| b.request.project_root != self.root)
        {
            return Err("Project source root changed; select the current Symbol again".into());
        }
        let focused = state.ui.layout.focused;
        let library_return = self.in_library
            && self.library_pane == Some(focused)
            && state.ui.layout.content_for(focused) == Some(crate::PaneContent::Library);
        if (focused != self.basis.pane && !library_return)
            || state.ui.layout.content_for(self.basis.pane) != Some(crate::PaneContent::Schematic)
            || state.schematic_documents.panes.get(&self.basis.pane) != self.basis.document.as_ref()
            || state
                .library
                .basis
                .as_ref()
                .is_none_or(|m| m.project.project_id != self.basis.project)
        {
            return Err("Assignment context changed; close and select the Symbol again".into());
        }
        let sheet = state
            .schematic_authoring
            .source
            .as_ref()
            .and_then(|s| s.sheets.get(&self.basis.source))
            .ok_or("Target sheet unavailable")?;
        if serde_json::to_value(sheet).map_err(|e| e.to_string())? != self.sheet_preimage {
            return Err(
                "Schematic changed during Library work; select the current Symbol again".into(),
            );
        }
        Ok(())
    }
    pub fn reference(&self) -> &str {
        &self.evidence.options.original.reference
    }
    /// One eligibility projection for paint and ordinary major-control traversal.
    pub fn actions(&self, browser: &super::LibraryBrowser) -> Vec<ResolutionAction> {
        use ResolutionAction as A;
        let mut actions = vec![A::Entity, A::Footprint];
        if !browser.choices("entities").is_empty() && !browser.choices("packages").is_empty() {
            actions.push(A::Part);
        }
        if !browser.choices("parts").is_empty() && !browser.choices("footprints").is_empty() {
            actions.push(A::Map);
        }
        if !self.evidence.options.choices.is_empty() {
            actions.push(A::Assign);
        }
        actions.push(A::Cancel);
        actions
    }
}
