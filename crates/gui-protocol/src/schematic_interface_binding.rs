//! Real parent-port choices configure the existing review form.
use super::delete_review::DeleteReview;
use crate::{AuthoredSelectionClass, AuthoringViewBasis, ReviewWorkspaceState, SelectionTarget};
use eda_engine::api::native_write::{
    schematic_delete_proposal::NativeProposalAction,
    schematic_interface_binding::{
        self as engine, InterfaceBindingRequest, InterfaceBindingSummary,
    },
};

#[derive(Debug, Clone, PartialEq)]
pub struct InterfaceBindingDraft {
    pub review: Box<DeleteReview>,
    pub choices: Vec<(InterfaceBindingRequest, InterfaceBindingSummary)>,
    pub selected: usize,
}
impl InterfaceBindingDraft {
    pub fn capture(state: &ReviewWorkspaceState) -> Result<Self, String> {
        let basis =
            AuthoringViewBasis::capture_qualified_schematic(state, state.ui.layout.focused)?;
        let SelectionTarget::Native(view) = &state.selection else {
            return Err("Select one child sheet instance".into());
        };
        if view.stale
            || view.project != basis.project
            || view.revision.0 != basis.revision
            || view.members.len() != 1
        {
            return Err("Select one current child sheet instance".into());
        }
        let target = view.members.iter().next().unwrap();
        if target.class != AuthoredSelectionClass::SheetInstance {
            return Err("Select the child sheet body to bind its interface".into());
        }
        let source = state
            .schematic_authoring
            .source
            .as_ref()
            .ok_or("Native source unavailable")?;
        let instance = source
            .sheet_instances
            .get(&target.id)
            .ok_or("Selected instance is missing")?;
        let mut expected_path = basis
            .document
            .as_ref()
            .ok_or("Qualified parent unavailable")?
            .instance_path
            .clone();
        expected_path.push(instance.uuid);
        if target.instance_path != expected_path || instance.parent_sheet != Some(basis.source) {
            return Err("Selected instance does not belong to the focused parent".into());
        }
        let model = state
            .library
            .basis
            .as_ref()
            .ok_or("Native model unavailable")?;
        if model.model_revision.0 != basis.revision || model.project.project_id != basis.project {
            return Err("Selected source changed".into());
        }
        let review = DeleteReview::capture_source_basis(state, basis)?;
        let parent = source
            .sheets
            .get(&review.basis.source)
            .ok_or("Parent source unavailable")?;
        let mut choices = vec![];
        for port in parent.ports.values() {
            let request = InterfaceBindingRequest {
                proposal_id: review.proposal_id,
                instance: instance.uuid,
                instance_path: target.instance_path.clone(),
                parent_port: port.uuid,
                bind: !instance.ports.contains(&port.uuid),
            };
            if let Ok(summary) = engine::preview(model, &request) {
                choices.push((request, summary));
            }
        }
        if choices.is_empty() {
            return Err("No valid interface: place a real parent Port and one matching child Hierarchical Label; repeated sources cannot be edited".into());
        }
        Ok(Self {
            review: Box::new(review),
            choices,
            selected: 0,
        })
    }
    pub fn cycle(&mut self) {
        self.selected = (self.selected + 1) % self.choices.len();
    }
    pub fn port_label(&self) -> String {
        let summary = &self.choices[self.selected].1;
        format!("{} · {}", summary.name, summary.parent_port)
    }
    pub fn action_label(&self) -> &'static str {
        if self.choices[self.selected].1.bind {
            "Bind interface"
        } else {
            "Unbind interface"
        }
    }
    pub fn stage(mut self, state: &ReviewWorkspaceState) -> Result<DeleteReview, String> {
        if !self.review.current(state) {
            return Err("Parent view changed; cancel and select the current instance".into());
        }
        let (request, expected) = &self.choices[self.selected];
        let actual = engine::preview(
            state
                .library
                .basis
                .as_ref()
                .ok_or("Native model unavailable")?,
            request,
        )
        .map_err(|e| e.to_string())?;
        if actual != *expected {
            return Err("Interface changed; select the current instance again".into());
        }
        self.review.interface_binding = Some(actual);
        self.review.pending = Some(NativeProposalAction::CreateInterfaceBinding(
            request.clone(),
        ));
        Ok(*self.review)
    }
}

/// Shared card measurement for presentation and acceptance visibility, using the
/// existing 11px monospace Proposal typography and padding.
pub const INTERFACE_ACCEPT_HINT: &str = "Tab accept interface change · Esc dismiss";
pub type InterfaceReviewLayout = super::review_layout::ProposalReviewLayout;
pub fn review_layout(
    summary: &InterfaceBindingSummary,
    refusal: Option<&str>,
    viewport_width: f32,
    viewport_height: f32,
) -> InterfaceReviewLayout {
    let path = |ids: &[uuid::Uuid]| {
        if ids.is_empty() {
            "root".to_owned()
        } else {
            ids.iter()
                .map(uuid::Uuid::to_string)
                .collect::<Vec<_>>()
                .join("/")
        }
    };
    let mut rows = vec![
        format!("Interface: {}", summary.name),
        format!("Parent source: {}", summary.parent),
        format!("Parent path: {}", path(&summary.parent_path)),
        format!("Parent Port: {}", summary.parent_port),
        format!("Child source: {}", summary.child),
        format!("Child Label: {}", summary.child_label),
        format!("Child path: {}", path(&summary.instance_path)),
    ];
    if let Some(refusal) = refusal {
        rows.push(format!("Unavailable: {refusal}"));
    }
    super::review_layout::measure(
        &rows,
        INTERFACE_ACCEPT_HINT,
        viewport_width,
        viewport_height,
        0.,
    )
}
