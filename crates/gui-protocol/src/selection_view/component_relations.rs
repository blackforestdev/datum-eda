//! One component-binding explanation for Inspector, accessibility and context.
use super::*;
use eda_engine::substrate::{ComponentSelectionContext, ElectricalOccurrence};

fn matches(id: &Id, source: &ElectricalOccurrence) -> bool {
    id.id == source.source_id
        && id.instance_path == source.instance_path
        && matches!(
            (id.class, source.class.as_str()),
            (Class::Symbol, "symbols") | (Class::Footprint, "packages")
        )
}

pub(super) fn append(view: &mut SelectionView, context: &ComponentSelectionContext) {
    for scope in &mut view.inspection {
        let roots: Vec<_> = view
            .members
            .iter()
            .filter(|id| {
                matches!(id.class, Class::Symbol | Class::Footprint)
                    && scope.scope.is_none_or(|class| id.class == class)
            })
            .collect();
        if roots.is_empty() {
            continue;
        }
        let relations: Vec<_> = context
            .relations
            .iter()
            .filter(|r| roots.iter().any(|id| matches(id, &r.source)))
            .collect();
        let unavailable: Vec<_> = context
            .unavailable
            .iter()
            .filter(|(source, _)| roots.iter().any(|id| matches(id, source)))
            .collect();
        let summary = if relations.len() == 1 && unavailable.is_empty() {
            let relation = relations[0];
            let kind = if relation.counterpart.class == "packages" {
                "Footprint"
            } else {
                "Symbol"
            };
            format!(
                "{kind} {} — component binding; related only",
                relation
                    .counterpart_reference
                    .as_deref()
                    .unwrap_or("(unnamed)")
            )
        } else {
            format!(
                "{} component binding relations; {} unavailable; related only",
                relations.len(),
                unavailable.len()
            )
        };
        let summary = if relations.is_empty() && !unavailable.is_empty() {
            InspectionValue::Unavailable(unavailable[0].1.clone())
        } else {
            InspectionValue::Common(serde_json::json!(summary))
        };
        scope.insert_field("Related component", summary, InspectionDetailKind::Ordinary);
        let unavailable: Vec<_> = unavailable
            .into_iter()
            .map(|(source, reason)| serde_json::json!({"source":source,"reason":reason}))
            .collect();
        scope.insert_field(
            "Component bindings",
            InspectionValue::Common(serde_json::json!({
                "relations":relations,"unavailable":unavailable
            })),
            InspectionDetailKind::Members,
        );
    }
}
