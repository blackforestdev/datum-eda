//! Canonical aggregate values compiled once for every complete Inspector scope.
use super::*;
/// Producer-owned provenance for detail that may enumerate selected identities.
/// Older serialized views lack this evidence and must not imply safe text.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InspectionDetailKind {
    #[default]
    Unknown,
    Ordinary,
    Members,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScopeInspection {
    pub scope: Option<Class>,
    pub fields: BTreeMap<String, InspectionValue>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub field_details: BTreeMap<String, InspectionDetailKind>,
    #[serde(default)]
    pub display_labels: BTreeMap<String, String>,
    #[serde(default)]
    pub quantities: BTreeMap<String, quantities::QuantityPresentation>,
}
impl ScopeInspection {
    pub(super) fn insert_field(
        &mut self,
        name: &str,
        value: InspectionValue,
        detail: InspectionDetailKind,
    ) {
        self.fields.insert(name.into(), value);
        self.field_details.insert(name.into(), detail);
    }
}
#[derive(Default)]
struct Aggregate {
    count: usize,
    values: BTreeMap<String, (usize, Value, bool)>,
    classes: BTreeMap<Class, usize>,
    workspaces: BTreeMap<&'static str, usize>,
    hidden: usize,
    locked: usize,
    unprojected: usize,
}
pub(super) fn compile(view: &SelectionView) -> Vec<ScopeInspection> {
    let mut scopes: BTreeMap<Option<Class>, Aggregate> = BTreeMap::new();
    scopes.entry(None).or_default();
    for id in &view.members {
        for scope in [None, Some(id.class)] {
            let aggregate = scopes.entry(scope).or_default();
            aggregate.count += 1;
            *aggregate.classes.entry(id.class).or_default() += 1;
            let workspace = match id.class {
                Class::Footprint
                | Class::Pad
                | Class::Track
                | Class::Via
                | Class::Zone
                | Class::BoardText
                | Class::BoardGraphic
                | Class::BoardOutline => "board",
                _ => "schematic",
            };
            *aggregate.workspaces.entry(workspace).or_default() += 1;
            aggregate.hidden += usize::from(view.hidden.contains(id));
            aggregate.locked += usize::from(view.locked.contains(id));
            aggregate.unprojected += usize::from(view.unprojected.contains(id));
            for (name, value) in view.values.get(id).into_iter().flat_map(|v| v.iter()) {
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
        .map(|(scope, mut aggregate)| {
            if let Some(class)=scope {
                for (semantic,_) in properties::fields(class) {
                    aggregate.values.entry((*semantic).into()).or_insert((0,Value::Null,false));
                }
            }
            let mut fields: BTreeMap<String,InspectionValue>=if scope.is_none() {BTreeMap::new()} else { aggregate.values.into_iter().map(|(name,(count,value,mixed))| {
                let state=if count!=aggregate.count {InspectionValue::Unavailable("Property is not typed or available for every member in this complete scope".into())} else if mixed {InspectionValue::Mixed} else {InspectionValue::Common(value)};
                (name,state)
            }).collect() };
            fields.insert("member_count".into(),InspectionValue::Common(serde_json::json!(aggregate.count)));
            fields.insert("member_types".into(),InspectionValue::Common(serde_json::json!(aggregate.classes)));
            fields.insert("workspace_coverage".into(),InspectionValue::Common(serde_json::json!(aggregate.workspaces)));
            fields.insert("hidden_members".into(),InspectionValue::Common(serde_json::json!(aggregate.hidden)));
            fields.insert("locked_members".into(),InspectionValue::Common(serde_json::json!(aggregate.locked)));
            fields.insert("unprojected_members".into(),InspectionValue::Common(serde_json::json!(aggregate.unprojected)));
            let field_details = fields.keys().map(|name| (name.clone(), InspectionDetailKind::Ordinary)).collect();
            ScopeInspection { scope,fields, field_details, display_labels: BTreeMap::new(), quantities: BTreeMap::new() }
        })
        .collect()
}
