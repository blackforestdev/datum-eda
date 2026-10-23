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
    // Related bindings may enumerate more identities than the selected subject.
    // The same existing detail cap protects this second path to member evidence.
    let truncated = truncated || view.related.len() > 256;
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
        let detail_members = match value {
            InspectionValue::Common(value) => member_detail_count(value),
            _ => 0,
        };
        let member_evidence = match value {
            // The structural guard also protects legacy/directly replaced
            // Common values whose provenance has not been updated.
            InspectionValue::Common(_) => {
                detail != InspectionDetailKind::Ordinary || detail_members > 0
            }
            InspectionValue::Unavailable(_) => detail != InspectionDetailKind::Ordinary,
            InspectionValue::Mixed => false,
        };
        if (truncated && member_evidence) || detail_members > 256 {
            omitted.push(name.to_owned());
            omitted_states.insert(name.to_owned(), value.state());
        } else {
            inspection.insert(name.to_owned(), value.clone());
        }
    }
    (inspection, omitted, omitted_states)
}

fn member_detail_count(value: &Value) -> usize {
    match value {
        Value::Object(fields) => {
            if fields.contains_key("class")
                && (fields.contains_key("id") || fields.contains_key("source_id"))
            {
                1
            } else {
                bounded_sum(fields.values())
            }
        }
        Value::Array(values) => bounded_sum(values.iter()),
        _ => 0,
    }
}

fn bounded_sum<'a>(values: impl Iterator<Item = &'a Value>) -> usize {
    let mut count = 0;
    for value in values {
        count += member_detail_count(value);
        if count > 256 {
            break;
        }
    }
    count
}
