//! Canonical aggregate values compiled once for every complete Inspector scope.
use super::*;
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScopeInspection {
    pub scope: Option<Class>,
    pub fields: BTreeMap<String, InspectionValue>,
}
#[derive(Default)]
struct Aggregate {
    count: usize,
    values: BTreeMap<String, (usize, Value, bool)>,
}
pub(super) fn compile(
    members: &BTreeSet<Id>,
    values: &BTreeMap<Id, BTreeMap<String, Value>>,
) -> Vec<ScopeInspection> {
    let mut scopes: BTreeMap<Option<Class>, Aggregate> = BTreeMap::new();
    scopes.entry(None).or_default();
    for id in members {
        for scope in [None, Some(id.class)] {
            let aggregate = scopes.entry(scope).or_default();
            aggregate.count += 1;
            for (name, value) in values.get(id).into_iter().flat_map(|v| v.iter()) {
                let entry = aggregate
                    .values
                    .entry(name.clone())
                    .or_insert_with(|| (0, value.clone(), false));
                entry.0 += 1;
                entry.2 |= entry.1 != *value;
            }
        }
    }
    scopes
        .into_iter()
        .map(|(scope, aggregate)| ScopeInspection {
            scope,
            fields: aggregate
                .values
                .into_iter()
                .map(|(name, (count, value, mixed))| {
                    let state = if count != aggregate.count {
                        InspectionValue::Unavailable(
                            "Property is absent or incompatible in the complete scope".into(),
                        )
                    } else if mixed {
                        InspectionValue::Mixed
                    } else {
                        InspectionValue::Common(value)
                    };
                    (name, state)
                })
                .collect(),
        })
        .collect()
}
