use std::collections::BTreeSet;

use serde_json::{Map, Value};

use super::SemanticSchema;

impl SemanticSchema {
    pub(super) fn validate(self, value: &Value) -> bool {
        match self {
            Self::ObjectOpacity => object_opacity(value),
            Self::ObjectSnapTypes => object_snap_types(value),
            Self::GridMarkStyle => grid_mark_style(value),
            Self::VersionedKeymap => versioned_keymap(value),
            Self::RgbaColor => rgba_color(value),
            Self::SheetFormat => sheet_format(value),
            Self::PublishSetNaming => publish_set_naming(value),
            Self::TerminalLaunchProfiles => terminal_launch_profiles(value),
            Self::TerminalTextRendering => terminal_text_rendering(value),
            Self::TerminalCursor => terminal_cursor(value),
            Self::TerminalFeedback => terminal_feedback(value),
            Self::TerminalScrollback => terminal_scrollback(value),
            Self::DefaultLocations => default_locations(value),
            Self::AutosavePolicy => autosave_policy(value),
            Self::ProjectDisplayUnits => project_display_units(value),
        }
    }
}

fn exact_object<'a>(value: &'a Value, fields: &[&str]) -> Option<&'a Map<String, Value>> {
    let object = value.as_object()?;
    let actual: BTreeSet<_> = object.keys().map(String::as_str).collect();
    let expected: BTreeSet<_> = fields.iter().copied().collect();
    (actual == expected).then_some(object)
}

fn integer_between(value: Option<&Value>, min: u64, max: u64) -> bool {
    value
        .and_then(Value::as_u64)
        .is_some_and(|candidate| (min..=max).contains(&candidate))
}

fn one_of(value: Option<&Value>, allowed: &[&str]) -> bool {
    value
        .and_then(Value::as_str)
        .is_some_and(|candidate| allowed.contains(&candidate))
}

fn object_opacity(value: &Value) -> bool {
    exact_object(value, &["track", "via", "pad", "zone"]).is_some_and(|object| {
        ["track", "via", "pad", "zone"]
            .iter()
            .all(|field| integer_between(object.get(*field), 0, 100))
    })
}

fn object_snap_types(value: &Value) -> bool {
    const KINDS: &[&str] = &[
        "pad_center",
        "via_center",
        "track_endpoint",
        "track_vertex",
        "junction",
        "pad_on_grid",
        "pin_endpoint",
        "wire_endpoint",
        "wire_vertex",
        "bus_connection",
        "label_anchor",
        "no_connect",
    ];
    exact_object(value, &["types", "current_layer_only"]).is_some_and(|object| {
        let Some(values) = object.get("types").and_then(Value::as_array) else {
            return false;
        };
        let unique: BTreeSet<_> = values.iter().filter_map(Value::as_str).collect();
        unique.len() == values.len()
            && unique.iter().all(|kind| KINDS.contains(kind))
            && object
                .get("current_layer_only")
                .is_some_and(Value::is_boolean)
    })
}

fn grid_mark_style(value: &Value) -> bool {
    exact_object(value, &["shape", "size", "min_spacing_px"]).is_some_and(|object| {
        one_of(object.get("shape"), &["cross", "dot", "line"])
            && integer_between(object.get("size"), 0, u8::MAX.into())
            && integer_between(object.get("min_spacing_px"), 0, u8::MAX.into())
    })
}

fn versioned_keymap(value: &Value) -> bool {
    exact_object(value, &["version", "bindings"]).is_some_and(|object| {
        integer_between(object.get("version"), 1, u32::MAX.into())
            && object.get("bindings").is_some_and(Value::is_object)
    })
}

fn rgba_color(value: &Value) -> bool {
    if value.as_str().is_some_and(|token| !token.trim().is_empty()) {
        return true;
    }
    exact_object(value, &["r", "g", "b", "a"]).is_some_and(|object| {
        ["r", "g", "b", "a"]
            .iter()
            .all(|field| integer_between(object.get(*field), 0, 255))
    })
}

fn sheet_format(value: &Value) -> bool {
    exact_object(value, &["size", "orientation"]).is_some_and(|object| {
        object
            .get("size")
            .and_then(Value::as_str)
            .is_some_and(|size| !size.trim().is_empty())
            && one_of(object.get("orientation"), &["portrait", "landscape"])
    })
}

fn publish_set_naming(value: &Value) -> bool {
    exact_object(value, &["mode", "pattern"]).is_some_and(|object| {
        one_of(object.get("mode"), &["ask", "pattern"])
            && object.get("pattern").is_some_and(Value::is_string)
    })
}

fn terminal_launch_profiles(value: &Value) -> bool {
    let Some(profiles) = value.as_array() else {
        return false;
    };
    let mut identities = BTreeSet::new();
    profiles.iter().all(|profile| {
        exact_object(
            profile,
            &["name", "executable", "argv", "cwd", "environment"],
        )
        .is_some_and(|object| {
            let Some(name) = object.get("name").and_then(Value::as_str) else {
                return false;
            };
            identities.insert(name)
                && !name.trim().is_empty()
                && object.get("executable").is_some_and(Value::is_string)
                && object.get("argv").is_some_and(|value| {
                    value
                        .as_array()
                        .is_some_and(|values| values.iter().all(Value::is_string))
                })
                && object.get("cwd").is_some_and(Value::is_string)
                && object.get("environment").is_some_and(Value::is_object)
        })
    })
}

fn terminal_text_rendering(value: &Value) -> bool {
    exact_object(value, &["zoom", "ligatures"]).is_some_and(|object| {
        integer_between(object.get("zoom"), 60, 200)
            && object
                .get("zoom")
                .and_then(Value::as_u64)
                .is_some_and(|zoom| zoom.is_multiple_of(10))
            && object.get("ligatures").is_some_and(Value::is_boolean)
    })
}

fn terminal_cursor(value: &Value) -> bool {
    exact_object(value, &["shape", "blink"]).is_some_and(|object| {
        one_of(object.get("shape"), &["block", "bar", "underline"])
            && object.get("blink").is_some_and(Value::is_boolean)
    })
}

fn terminal_feedback(value: &Value) -> bool {
    exact_object(value, &["bell", "activity", "copy_confirmation"]).is_some_and(|object| {
        one_of(object.get("bell"), &["visual", "audible", "both", "none"])
            && object.get("activity").is_some_and(Value::is_boolean)
            && object
                .get("copy_confirmation")
                .is_some_and(Value::is_boolean)
    })
}

fn terminal_scrollback(value: &Value) -> bool {
    exact_object(value, &["max_lines", "max_mib"]).is_some_and(|object| {
        integer_between(object.get("max_lines"), 100, 100_000)
            && integer_between(object.get("max_mib"), 1, 64)
    })
}

fn default_locations(value: &Value) -> bool {
    exact_object(
        value,
        &["projects", "libraries", "output_jobs", "new_project"],
    )
    .is_some_and(|object| object.values().all(Value::is_string))
}

fn autosave_policy(value: &Value) -> bool {
    exact_object(
        value,
        &[
            "interval",
            "retained_versions",
            "backup_age",
            "recovery",
            "reminder",
        ],
    )
    .is_some_and(|object| {
        one_of(object.get("interval"), &["off", "5m", "10m", "30m"])
            && integer_between(object.get("retained_versions"), 1, u16::MAX.into())
            && object
                .get("backup_age")
                .is_some_and(|value| value.is_string() || value.as_u64().is_some_and(|age| age > 0))
            && object.get("recovery").is_some_and(Value::is_boolean)
            && (object.get("reminder").is_some_and(Value::is_boolean)
                || object.get("reminder").is_some_and(Value::is_string))
    })
}

fn project_display_units(value: &Value) -> bool {
    if value.is_null() {
        return true;
    }
    exact_object(
        value,
        &[
            "system",
            "board_length",
            "board_length_precision",
            "drill_hole",
            "drill_hole_precision",
            "schematic_geometry",
            "schematic_geometry_precision",
            "angle_precision",
        ],
    )
    .is_some_and(|object| {
        one_of(object.get("system"), &["metric", "imperial"])
            && one_of(
                object.get("board_length"),
                &["follow_system", "mm", "um", "mil", "inch"],
            )
            && precision(object.get("board_length_precision"))
            && one_of(
                object.get("drill_hole"),
                &["follow_system", "mm", "mil", "inch"],
            )
            && precision(object.get("drill_hole_precision"))
            && one_of(
                object.get("schematic_geometry"),
                &["follow_system", "mm", "mil"],
            )
            && precision(object.get("schematic_geometry_precision"))
            && one_of(
                object.get("angle_precision"),
                &["decimal_0", "decimal_1", "decimal_2", "decimal_3"],
            )
    })
}

fn precision(value: Option<&Value>) -> bool {
    one_of(
        value,
        &[
            "automatic",
            "decimal_0",
            "decimal_1",
            "decimal_2",
            "decimal_3",
            "decimal_4",
            "decimal_5",
            "decimal_6",
            "exact_nm",
        ],
    )
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn every_structured_schema_accepts_its_canonical_shape_and_rejects_empty_object() {
        let cases = [
            (
                SemanticSchema::ObjectOpacity,
                json!({"track":100,"via":100,"pad":100,"zone":70}),
            ),
            (
                SemanticSchema::ObjectSnapTypes,
                json!({"types":["pad_center","pin_endpoint"],"current_layer_only":false}),
            ),
            (
                SemanticSchema::GridMarkStyle,
                json!({"shape":"dot","size":1,"min_spacing_px":8}),
            ),
            (
                SemanticSchema::VersionedKeymap,
                json!({"version":1,"bindings":{}}),
            ),
            (
                SemanticSchema::RgbaColor,
                json!({"r":255,"g":0,"b":64,"a":255}),
            ),
            (
                SemanticSchema::SheetFormat,
                json!({"size":"A3","orientation":"landscape"}),
            ),
            (
                SemanticSchema::PublishSetNaming,
                json!({"mode":"pattern","pattern":"{project}-{set}"}),
            ),
            (
                SemanticSchema::TerminalLaunchProfiles,
                json!([{"name":"default","executable":"/bin/sh","argv":[],"cwd":".","environment":{}}]),
            ),
            (
                SemanticSchema::TerminalTextRendering,
                json!({"zoom":100,"ligatures":false}),
            ),
            (
                SemanticSchema::TerminalCursor,
                json!({"shape":"block","blink":true}),
            ),
            (
                SemanticSchema::TerminalFeedback,
                json!({"bell":"visual","activity":true,"copy_confirmation":false}),
            ),
            (
                SemanticSchema::TerminalScrollback,
                json!({"max_lines":50000,"max_mib":64}),
            ),
            (
                SemanticSchema::DefaultLocations,
                json!({"projects":"projects","libraries":"libraries","output_jobs":"output-jobs","new_project":"projects"}),
            ),
            (
                SemanticSchema::AutosavePolicy,
                json!({"interval":"10m","retained_versions":10,"backup_age":"30d","recovery":true,"reminder":true}),
            ),
            (
                SemanticSchema::ProjectDisplayUnits,
                json!({"system":"metric","board_length":"follow_system","board_length_precision":"automatic","drill_hole":"follow_system","drill_hole_precision":"automatic","schematic_geometry":"follow_system","schematic_geometry_precision":"automatic","angle_precision":"decimal_1"}),
            ),
        ];
        for (schema, valid) in cases {
            assert!(schema.validate(&valid), "{schema:?} rejected {valid}");
            assert!(
                !schema.validate(&json!({})),
                "{schema:?} accepted empty object"
            );
        }
    }

    #[test]
    fn structured_schemas_reject_unknown_fields_and_invalid_domains() {
        assert!(
            !SemanticSchema::ObjectOpacity
                .validate(&json!({"track":101,"via":100,"pad":100,"zone":70}))
        );
        assert!(
            !SemanticSchema::ObjectSnapTypes
                .validate(&json!({"types":["future_kind"],"current_layer_only":false}))
        );
        assert!(
            !SemanticSchema::TerminalTextRendering.validate(&json!({"zoom":125,"ligatures":false}))
        );
        assert!(
            !SemanticSchema::TerminalScrollback.validate(&json!({"max_lines":100001,"max_mib":64}))
        );
        assert!(
            !SemanticSchema::TerminalCursor
                .validate(&json!({"shape":"block","blink":true,"color":"red"}))
        );
    }
}
