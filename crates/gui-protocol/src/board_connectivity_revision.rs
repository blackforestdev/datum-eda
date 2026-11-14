//! Frozen designer evidence over the existing source-qualified engine planner.
use crate::schematic_authoring::{delete_review::DeleteReview, review_layout};
use crate::{AuthoringViewBasis, ReviewWorkspaceState};
use eda_engine::{
    api::native_write::{
        schematic_board_transfer::connectivity_revision::{
            self as engine, AssignmentView, BoardConnectivityRevision, CopperImpact,
            RetainedCopperKind,
        },
        schematic_delete_proposal::NativeProposalAction,
    },
    connectivity::PhysicalQueryFailure,
    substrate::{DesignModel, SourceShardKind, ZoneFillState},
};
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
pub struct BoardConnectivityRevisionEvidence {
    pub summary: BoardConnectivityRevision,
    /// Display-only rows captured from the same qualified immutable source.
    pub rows: Vec<String>,
}

pub const ACCEPT_HINT: &str = "Apply changes [Tab] · Dismiss [Esc]";
pub const FIT_REFUSAL: &str =
    "Enlarge the pane to review every old/new connection, retained copper ID and rework warning";

fn named_source(model: &DesignModel, id: Uuid, kind: &str) -> String {
    let name = crate::board_transfer::source_label(model, id, kind);
    if name == id.to_string() {
        name
    } else {
        format!("{name} ({id})")
    }
}

impl DeleteReview {
    pub fn capture_board_connectivity_revision(
        state: &ReviewWorkspaceState,
    ) -> Result<Self, String> {
        let basis =
            AuthoringViewBasis::capture_unique_root_schematic(state, state.ui.layout.focused)?;
        let model = state
            .library
            .basis
            .as_ref()
            .ok_or("Native model unavailable")?;
        if model.model_revision.0 != basis.revision || model.project.project_id != basis.project {
            return Err("Design changed; reopen the PCB connectivity review".into());
        }
        let summary = engine::preview(model, basis.source).map_err(|error| error.to_string())?;
        if summary.changes.is_empty() {
            return Err("PCB pin/net correspondence already matches the schematic".into());
        }
        let mut review = Self::capture_source_basis(state, basis)?;
        let board = model
            .materialized_source_shard_value(SourceShardKind::BoardRoot)
            .map_err(|error| error.to_string())?;
        let net_name = |id: Uuid| -> String {
            board["nets"][id.to_string()]["name"]
                .as_str()
                .filter(|name| !name.is_empty())
                .map_or_else(|| id.to_string(), |name| format!("{name} ({id})"))
        };
        let mut rows = Vec::new();
        let components: BTreeSet<_> = summary
            .changes
            .iter()
            .map(|change| change.component_instance)
            .collect();
        for id in components {
            let component = model
                .component_instances
                .get(&id)
                .ok_or("Component source unavailable")?;
            let ([symbol], [package]) = (
                component.placed_symbol_refs.as_slice(),
                component.placed_package_refs.as_slice(),
            ) else {
                return Err("Component counterpart is ambiguous".into());
            };
            let symbol = review
                .source
                .symbols
                .get(symbol)
                .ok_or("Captured Symbol unavailable")?;
            let package = &board["packages"][package.to_string()];
            let part = component.part_ref.ok_or("Component Part unavailable")?;
            let maps: Vec<_> = component
                .library_bindings
                .values()
                .filter(|binding| {
                    binding.binding_role == eda_engine::substrate::LibraryBindingRole::PinPadMap
                })
                .collect();
            let [binding] = maps.as_slice() else {
                return Err("Pinned physical Map is ambiguous".into());
            };
            let map = binding.target_object_id;
            let map_object = model
                .objects
                .get(&map)
                .ok_or("Pinned Map source unavailable")?;
            let map_value = model
                .capture_source_shard_value(map_object.source_shard_id)
                .map_err(|error| error.to_string())?;
            let footprint: Uuid = serde_json::from_value(map_value["footprint"].clone())
                .map_err(|error| error.to_string())?;
            rows.push(format!(
                "{} {} · component {id} · read-only physical choice",
                symbol.reference, symbol.value
            ));
            if package["reference"].as_str() != Some(symbol.reference.as_str()) {
                rows.push(format!(
                    "Existing PCB reference: {} (unchanged by this review)",
                    package["reference"]
                        .as_str()
                        .ok_or("PCB reference unavailable")?
                ));
            }
            rows.push(format!(
                "Part: {} · Footprint: {}",
                named_source(model, part, "parts"),
                named_source(model, footprint, "footprints")
            ));
            for change in summary
                .changes
                .iter()
                .filter(|change| change.component_instance == id)
            {
                let pin = symbol
                    .pins
                    .iter()
                    .find(|pin| pin.uuid == change.terminal.source_id)
                    .ok_or("Changed terminal unavailable")?;
                rows.push(format!(
                    "Terminal: {} pin {} {} ({}) → pad {} (retained)",
                    symbol.reference, pin.number, pin.name, pin.uuid, change.pad
                ));
                rows.push(format!(
                    "Logical net: old {} → new {}",
                    change.previous_logical_net, change.logical_net
                ));
                rows.push(format!(
                    "Board net: old {} → new {}",
                    net_name(change.previous_board_net),
                    net_name(change.board_net)
                ));
            }
        }
        rows.push("Both nets already exist. Package/pad identities, placement and pinned physical choice stay unchanged.".into());
        rows.push("Retained copper — geometry and OLD net assignments; conservative candidates are labelled:".into());
        for copper in &summary.retained_copper {
            let kind = match copper.kind {
                RetainedCopperKind::Track => "Track",
                RetainedCopperKind::Via => "Via",
                RetainedCopperKind::Zone => "Zone",
            };
            let scope = match copper.impact {
                CopperImpact::QualifiedNetChange => "affected net",
                CopperImpact::ForeignContact => "foreign contact",
                CopperImpact::ConservativeUnknownContacts => {
                    "conservative candidate; contacts unavailable"
                }
                CopperImpact::FillBasisInvalidated => "fill basis invalidated",
            };
            rows.push(format!(
                "{kind} {} · {} · {scope}",
                copper.source_id,
                copper.net.map_or_else(|| "unassigned".into(), net_name)
            ));
        }
        for zone in &summary.zone_fills {
            let status = if zone.current {
                "current fill will require explicit refill after Apply"
            } else if zone
                .recorded_model_revision
                .as_ref()
                .is_some_and(|basis| basis != &summary.model_revision)
                || zone
                    .recorded_zone_revision
                    .is_some_and(|revision| revision != zone.source_revision)
            {
                "fill already stale; current contacts unavailable"
            } else {
                "no qualified current fill; contacts unavailable"
            };
            rows.push(format!("Zone {}: {status}", zone.source_id));
        }
        let mut unknown: BTreeMap<(Uuid, Option<ZoneFillState>, bool), BTreeSet<Uuid>> =
            BTreeMap::new();
        for contact in &summary.contact_basis {
            if let Some(reason) = &contact.unavailable {
                let PhysicalQueryFailure::UnavailableFill { source_id, state } = reason else {
                    return Err("Unsupported physical contact reason; reopen the review".into());
                };
                unknown
                    .entry((
                        *source_id,
                        *state,
                        contact.assignment == AssignmentView::Proposed,
                    ))
                    .or_default()
                    .insert(contact.board_net);
            }
        }
        for ((zone, state, proposed), nets) in unknown {
            let state = match state {
                Some(ZoneFillState::Stale) => "stale",
                Some(ZoneFillState::Unfilled) => "unfilled",
                Some(ZoneFillState::Unsupported) => "unsupported",
                Some(ZoneFillState::Filled) => "not current",
                None => "no recorded fill",
            };
            rows.push(format!(
                "{} contacts unavailable: Zone {zone} ({state}); board nets {}",
                if proposed { "Proposed" } else { "Previous" },
                nets.into_iter()
                    .map(|net| net.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        rows.push("Apply changes only pad/net correspondence. Tracks, vias and zones retain geometry and old net assignments; no automatic reroute, deletion, copper reassignment or fill. Repair resulting shorts and unrouted connections with Route, explicit refill and DRC. Complete review includes disclosed unknowns; it does not certify physical connectivity, current fill or a complete Global Net.".into());
        review.pending = Some(NativeProposalAction::CreateBoardConnectivityRevision {
            proposal_id: review.proposal_id,
            sheet: summary.sheet,
            review: summary.clone(),
        });
        review.board_connectivity_revision =
            Some(BoardConnectivityRevisionEvidence { summary, rows });
        Ok(review)
    }
}

impl BoardConnectivityRevisionEvidence {
    pub fn layout(
        &self,
        refusal: Option<&str>,
        width: f32,
        height: f32,
        bottom_clearance: f32,
        label_widths: (f32, f32),
        measure_height: impl FnMut(&str, f32) -> Result<f32, String>,
    ) -> Result<review_layout::ProposalReviewLayout, String> {
        let mut rows = self.rows.clone();
        if let Some(reason) = refusal {
            rows.push(format!("Unavailable: {reason}"));
        }
        let mut layout = review_layout::measure_rows(
            &rows,
            label_widths,
            width,
            height,
            bottom_clearance,
            measure_height,
        )?;
        layout.fits &= !self.summary.changes.is_empty() && !self.rows.is_empty();
        Ok(layout)
    }
}
