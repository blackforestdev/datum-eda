//! Governed transport omission, never a membership or Inspector authority.
use crate::selection_view::{
    InspectionDetailKind, InspectionState, InspectionValue, SelectionView,
};
use serde_json::Value;
use std::collections::BTreeMap;

pub(super) fn capture(
    view: &SelectionView,
    truncated: bool,
) -> (
    BTreeMap<String, InspectionValue>,
    Vec<String>,
    BTreeMap<String, InspectionState>,
) {
    let mut inspection = BTreeMap::new();
    let mut omitted = Vec::new();
    let mut omitted_states = BTreeMap::new();
    for name in view.fields() {
        let Some(value) = view.field_ref(name) else {
            continue;
        };
        // Coverage/contact fields may contain the very identity list omitted
        // from `members`. Blocker evidence may likewise enumerate every member
        // in its canonical reason. Do not leak it through another field or
        // publish a prefix. Name each omitted field for explicit complete query.
        let detail = view.field_detail_kind(name);
        let member_evidence = match value {
            // The structural guard also protects legacy/directly replaced
            // Common values whose provenance has not been updated.
            InspectionValue::Common(value) => {
                detail != InspectionDetailKind::Ordinary || contains_member(value)
            }
            InspectionValue::Unavailable(_) => detail != InspectionDetailKind::Ordinary,
            InspectionValue::Mixed => false,
        };
        if truncated && member_evidence {
            omitted.push(name.to_owned());
            omitted_states.insert(name.to_owned(), value.state());
        } else {
            inspection.insert(name.to_owned(), value.clone());
        }
    }
    (inspection, omitted, omitted_states)
}

fn contains_member(value: &Value) -> bool {
    match value {
        Value::Object(fields) => {
            (fields.contains_key("class")
                && (fields.contains_key("id") || fields.contains_key("source_id")))
                || fields.values().any(contains_member)
        }
        Value::Array(values) => values.iter().any(contains_member),
        _ => false,
    }
}
