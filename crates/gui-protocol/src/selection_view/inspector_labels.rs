//! Product wording over existing complete Inspector properties and identities.
use super::*;
use crate::selection_view::inspector_state::class_label;
pub fn field_label(field: &str) -> String {
    let label = match field {
        "placement_position_nm" | "position_nm" | "terminal_position_nm" => "Position",
        "placement_rotation" | "text_rotation" => "Rotation",
        "combined_bounds_nm" => "Bounds",
        "projected_bounds_nm" => "Visible bounds",
        "member_types" => "Object types",
        "member_count" => "Objects",
        "workspace_coverage" => "Workspaces",
        "layer_coverage" => "Layers",
        "projected_layer_coverage" => "Visible layers",
        "net_coverage" => "Nets",
        "resolved_net_coverage" => "Resolved nets",
        "non_electrical_members" => "Non-electrical",
        "hidden_members" => "Hidden objects",
        "locked_members" => "Locked objects",
        "unprojected_members" => "Unavailable geometry",
        "pad_materialization_status" => "Pad correspondence",
        "occupied_fill_status" => "Copper fill",
        "board_layer" => "Layer",
        "board_net_id" => "Net",
        "section_start_nm" => "Start",
        "section_end_nm" => "End",
        "section_midpoint_nm" => "Midpoint",
        "copper_width_nm" | "pad_width_nm" => "Width",
        "pad_height_nm" => "Height",
        "copper_diameter_nm" | "pad_diameter_nm" => "Diameter",
        "drill_diameter_nm" => "Drill",
        "locked" => "Locked",
        "reference" => "Reference",
        "value" => "Value",
        _ => return field.trim_end_matches("_nm").replace('_', " "),
    };
    label.into()
}
impl SelectionView {
    pub fn member_display_label(&self, id: &Id, display_ordinal: usize) -> String {
        let class = match id.class {
            Class::Footprint => "Part",
            Class::Symbol => "Symbol",
            _ => class_label(id.class).trim_end_matches('s'),
        };
        let name = self.values.get(id).and_then(|values| {
            [
                "reference",
                "pin_number",
                "net_label_name",
                "port_name",
                "text",
            ]
            .into_iter()
            .filter_map(|key| values.get(key).and_then(Value::as_str))
            .find(|name| !name.is_empty())
        });
        if let Some(name) = name {
            return format!("{class}: {name}");
        }
        // A display ordinal is not an authored identity or focus authority.
        format!("{class}: object {display_ordinal}")
    }
}

pub(super) fn append(scopes: &mut [ScopeInspection], shards: &BTreeMap<uuid::Uuid, Value>) {
    let Some(board) = shards
        .values()
        .find(|value| value.get("stackup").is_some() && value.get("nets").is_some())
    else {
        return;
    };
    for scope in scopes {
        for (field, value) in &scope.fields {
            let InspectionValue::Common(value) = value else {
                continue;
            };
            let label = match field.as_str() {
                "board_net_id" => value
                    .as_str()
                    .and_then(|id| board["nets"][id]["name"].as_str())
                    .map(ToOwned::to_owned),
                "board_layer" | "from_layer" | "to_layer" => value
                    .as_i64()
                    .and_then(|id| {
                        board["stackup"]["layers"]
                            .as_array()?
                            .iter()
                            .find(|layer| layer["id"].as_i64() == Some(id))?["name"]
                            .as_str()
                    })
                    .map(ToOwned::to_owned),
                "pad_materialization_status" => value.as_str().map(|status| {
                    match status {
                        "current" => "Current",
                        "stale" => "Stale — source has changed",
                        _ => "Not source-verified",
                    }
                    .to_owned()
                }),
                "layer_coverage" | "projected_layer_coverage" => {
                    value.as_object().map(|coverage| {
                        let mut labels = Vec::new();
                        for (key, members) in coverage.iter().take(3) {
                            let layer = key.rsplit(':').next().unwrap_or(key);
                            let name = layer
                                .strip_prefix('L')
                                .and_then(|id| id.parse::<i64>().ok())
                                .and_then(|id| {
                                    board["stackup"]["layers"]
                                        .as_array()?
                                        .iter()
                                        .find(|layer| layer["id"].as_i64() == Some(id))?["name"]
                                        .as_str()
                                })
                                .unwrap_or(layer);
                            labels.push(format!(
                                "{name}: {} objects",
                                members.as_array().map_or(0, Vec::len)
                            ));
                        }
                        if coverage.len() > 3 {
                            labels.push(format!("{} layers total", coverage.len()));
                        }
                        if labels.is_empty() {
                            "None".into()
                        } else {
                            labels.join(" · ")
                        }
                    })
                }
                _ => None,
            };
            if let Some(label) = label {
                scope.display_labels.insert(field.clone(), label);
            }
        }
    }
}
