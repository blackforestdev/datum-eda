//! Fixed-owner Gate placement configures the existing picker, ghost and Proposal.
use super::{LibrarySymbolChoice, assignment::AssignmentEvidence, delete_review::DeleteReview};
use crate::{AuthoredSelectionIdentity, BoardReviewSceneV1, PointNm, ReviewWorkspaceState};
use eda_engine::{
    api::native_write::{
        schematic_delete_proposal::NativeProposalAction,
        schematic_gate_placement::{
            self as engine, GatePlacementMaterial, GatePlacementOptions, GatePlacementRequest,
            GatePlacementSummary,
        },
    },
    ir::geometry::Point,
    schematic::PlacedSymbol,
};

#[derive(Debug, Clone, PartialEq)]
pub struct GatePlacementEvidence {
    pub symbol: PlacedSymbol,
    pub summary: GatePlacementSummary,
    pub preview: BoardReviewSceneV1,
}
#[derive(Debug, Clone, PartialEq)]
pub struct GatePlacementDraft {
    pub review: Box<DeleteReview>,
    pub options: GatePlacementOptions,
    pub choices: Vec<LibrarySymbolChoice>,
    pub selected: Option<usize>,
    pub material: Option<GatePlacementMaterial>,
}
impl GatePlacementDraft {
    /// Captured facts only; complete canonical census/compatibility is read on entry.
    pub fn admit_target(
        state: &ReviewWorkspaceState,
        target: &AuthoredSelectionIdentity,
    ) -> Result<(), String> {
        AssignmentEvidence::admit_source(state, target)?;
        let model = state.library.basis.as_ref().unwrap();
        let mut owners = model
            .component_instances
            .values()
            .filter(|ci| ci.placed_symbol_refs.contains(&target.id));
        let owner = owners
            .next()
            .ok_or("Assign a Part before placing another Gate")?;
        if owners.next().is_some()
            || !owner.placed_package_refs.is_empty()
            || owner.authority != eda_engine::substrate::ComponentInstanceAuthority::Authored
        {
            return Err("Place another Gate requires one schematic-only authored component".into());
        }
        Ok(())
    }
    pub fn admit_selected(state: &ReviewWorkspaceState) -> Result<(), String> {
        let (_, target) = super::selected::target(state)?;
        Self::admit_target(state, &target)
    }
    pub fn capture(state: &ReviewWorkspaceState) -> Result<Self, String> {
        let (_, target) = super::selected::target(state)?;
        Self::admit_target(state, &target)?;
        let review = DeleteReview::capture_source(state)?;
        let model = state.library.basis.as_ref().unwrap();
        let options =
            engine::options(model, target.id, &target.instance_path).map_err(|e| e.to_string())?;
        if options.sheet != review.basis.source || options.instance_path != target.instance_path {
            return Err("Gate owner source changed".into());
        }
        if options.choices.is_empty() {
            return Err("This component has no unused compatible Gates".into());
        }
        let choices = options
            .choices
            .iter()
            .map(|gate| {
                let mut choice = state
                    .schematic_authoring
                    .choices
                    .iter()
                    .find(|c| c.id == gate.library_symbol)
                    .cloned()
                    .ok_or("Gate library Symbol is unavailable")?;
                choice.name = format!("{} · {}", gate.gate_name, gate.symbol_name);
                Ok(choice)
            })
            .collect::<Result<Vec<_>, String>>()?;
        Ok(Self {
            review: Box::new(review),
            options,
            choices,
            selected: None,
            material: None,
        })
    }
    pub fn choose(&mut self, state: &ReviewWorkspaceState, index: usize) -> Result<(), String> {
        if !self.review.current(state) {
            return Err("Gate source changed; cancel and reopen placement".into());
        }
        let model = state
            .library
            .basis
            .as_ref()
            .ok_or("Gate library source unavailable")?;
        if model.project.project_id != self.review.basis.project
            || model.model_revision.0 != self.review.basis.revision
        {
            return Err("Gate source model changed".into());
        }
        let choice = self
            .options
            .choices
            .get(index)
            .ok_or("Choose an unused Gate")?;
        let material =
            engine::materialize(model, &self.options, choice.gate, self.review.proposal_id)
                .map_err(|e| e.to_string())?;
        self.material = Some(material);
        self.selected = Some(index);
        Ok(())
    }
    pub fn placed(
        &self,
        reference: &str,
        value: &str,
        at: PointNm,
        rotation: i32,
        mirrored: bool,
    ) -> Result<PlacedSymbol, String> {
        self.material
            .as_ref()
            .ok_or("Choose an unused Gate")?
            .placed(
                reference,
                value,
                Point { x: at.x, y: at.y },
                rotation,
                mirrored,
            )
            .map_err(|e| e.to_string())
    }
    pub fn stage(
        mut self,
        state: &ReviewWorkspaceState,
        at: PointNm,
    ) -> Result<DeleteReview, String> {
        if !self.review.current(state) {
            return Err("Gate source changed; cancel and reopen placement".into());
        }
        let model = state
            .library
            .basis
            .as_ref()
            .ok_or("Gate library source unavailable")?;
        if model.project.project_id != self.review.basis.project
            || model.model_revision.0 != self.review.basis.revision
        {
            return Err("Gate source model changed".into());
        }
        let e = &state.schematic_authoring;
        let choice = self
            .selected
            .and_then(|i| self.options.choices.get(i))
            .ok_or("Choose an unused Gate")?;
        let request = GatePlacementRequest {
            proposal_id: self.review.proposal_id,
            component_instance: self.options.component_instance,
            sheet: self.options.sheet,
            instance_path: self.options.instance_path.clone(),
            gate: choice.gate,
            library_symbol: choice.library_symbol,
            reference: e.reference.clone(),
            value: e.value.clone(),
            x_nm: at.x,
            y_nm: at.y,
            rotation_deg: e.rotation_deg,
            mirrored: e.mirrored,
        };
        let planned = engine::preview(model, &request).map_err(|e| e.to_string())?;
        if planned.symbol != self.placed(&e.reference, &e.value, at, e.rotation_deg, e.mirrored)? {
            return Err("Gate placement preview changed; cancel and reopen".into());
        }
        let mut source = e.source.clone().ok_or("Gate source unavailable")?;
        let sheet = source
            .sheets
            .get_mut(&self.review.basis.source)
            .ok_or("Gate sheet unavailable")?;
        if *sheet != self.review.source || sheet.symbols.contains_key(&planned.symbol.uuid) {
            return Err("Gate source changed".into());
        }
        sheet
            .symbols
            .insert(planned.symbol.uuid, planned.symbol.clone());
        let root = &state
            .backing
            .as_ref()
            .ok_or("Native project source unavailable")?
            .request
            .project_root;
        let preview = e
            .project_source(&source, self.options.sheet, &self.review.scene, root)
            .map_err(|e| e.to_string())?;
        self.review.gate_placement = Some(GatePlacementEvidence {
            symbol: planned.symbol,
            summary: planned.summary,
            preview,
        });
        self.review.pending = Some(NativeProposalAction::CreateGatePlacement(request));
        Ok(*self.review)
    }
}

impl super::SchematicAuthoring {
    /// Gate configuration of the shared Preview control. Never assigns a reference
    /// or stages a Proposal; only a qualified placement click stages the request.
    pub fn start_gate_preview(&mut self) -> Option<Result<(), String>> {
        let draft = self.gate_draft.as_ref()?;
        let result = (|| {
            let index = self
                .chosen
                .filter(|index| Some(*index) == draft.selected)
                .ok_or("Choose an unused Gate")?;
            let choice = draft
                .options
                .choices
                .get(index)
                .ok_or("Gate choice changed")?;
            let material = draft.material.as_ref().ok_or("Choose an unused Gate")?;
            if material.symbol.gate != Some(choice.gate)
                || material.symbol.lib_id.as_deref()
                    != Some(choice.library_symbol.to_string().as_str())
            {
                return Err("Gate preview identity changed".into());
            }
            if self.reference.is_empty()
                || self
                    .reference
                    .chars()
                    .any(|c| c.is_whitespace() || c.is_control())
                || self.taken_references.contains(&self.reference)
                || self.value.chars().any(char::is_control)
            {
                return Err(
                    "Supply a unique nonempty reference without whitespace and a single-line value"
                        .into(),
                );
            }
            Ok(())
        })();
        if result.is_ok() {
            self.picker = false;
            self.preview_dirty = true;
            self.refusal = None;
        }
        Some(result)
    }
}
