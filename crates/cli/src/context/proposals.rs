use eda_engine::substrate::DesignModel;
use serde_json::{Map, Value};

pub(super) fn visible_proposal_ids(model: &DesignModel) -> Value {
    Value::Array(
        model
            .proposals
            .keys()
            .map(|proposal| Value::String(proposal.to_string()))
            .collect(),
    )
}

pub(super) fn latest_proposal_id_from_context(
    object: &Map<String, Value>,
    model: &DesignModel,
) -> Option<String> {
    let selected = object
        .get("selection_context")
        .and_then(Value::as_object)
        .filter(|selection| selection.get("kind").and_then(Value::as_str) == Some("proposal"))
        .and_then(|selection| selection.get("id").and_then(Value::as_str));
    if let Some(selected) = selected {
        if model
            .proposals
            .keys()
            .any(|proposal| proposal.to_string() == selected)
        {
            return Some(selected.to_string());
        }
    }
    model
        .proposals
        .iter()
        .find(|(_, proposal)| {
            matches!(
                format!("{:?}", proposal.status).as_str(),
                "Accepted" | "Draft" | "Deferred"
            )
        })
        .or_else(|| model.proposals.iter().next())
        .map(|(proposal_id, _)| proposal_id.to_string())
}
