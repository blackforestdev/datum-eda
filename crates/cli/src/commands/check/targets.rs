pub(crate) fn check_primary_target(source: &str, payload: &serde_json::Value) -> serde_json::Value {
    if let Some(target) = occurrence_targets(payload).into_iter().next() {
        return target;
    }
    if let Some(value) = first_object_id(payload) {
        return serde_json::json!({
            "kind": "object_uuid",
            "id": value,
        });
    }
    for key in [
        "object_id",
        "object_uuid",
        "component_uuid",
        "symbol_uuid",
        "pin_uuid",
        "pad_uuid",
        "pad_id",
        "zone_id",
        "artifact_id",
        "net_uuid",
        "uuid",
    ] {
        if let Some(value) = payload.get(key) {
            let Some(id) = target_id_string(value) else {
                continue;
            };
            return serde_json::json!({
                "kind": target_kind(source, key),
                "id": id,
            });
        }
    }
    serde_json::json!({
        "kind": source,
        "id": "unknown",
    })
}

pub(crate) fn check_related_targets(
    primary_target: &serde_json::Value,
    payload: &serde_json::Value,
) -> Vec<serde_json::Value> {
    let mut qualified = occurrence_targets(payload);
    if !qualified.is_empty() {
        if let Some(net) = payload.get("net_uuid").and_then(serde_json::Value::as_str) {
            qualified.push(serde_json::json!({"kind": "net", "id": net}));
        }
        return qualified
            .into_iter()
            .filter(|target| target != primary_target)
            .collect();
    }
    object_id_array_values(payload)
        .map(|value| {
            serde_json::json!({
                "kind": "object_uuid",
                "id": value,
            })
        })
        .filter(|target| target != primary_target)
        .collect()
}

// Use the existing source UUID + ordered path target representation consumed by
// schematic navigation. The path is never sorted or replaced by a derived UUID.
fn occurrence_targets(payload: &serde_json::Value) -> Vec<serde_json::Value> {
    let Some(values) = payload
        .get("occurrences")
        .and_then(serde_json::Value::as_array)
    else {
        return Vec::new();
    };
    let mut occurrences: Vec<eda_engine::substrate::ElectricalOccurrence> = values
        .iter()
        .filter_map(|value| serde_json::from_value(value.clone()).ok())
        .collect();
    occurrences.sort();
    occurrences.dedup();
    occurrences
        .into_iter()
        .map(|occurrence| {
            serde_json::json!({
                "kind": "object_uuid", "id": occurrence.source_id,
                "instance_path": occurrence.instance_path,
            })
        })
        .collect()
}

fn target_kind(source: &str, key: &str) -> String {
    match key {
        "artifact_id" => "artifact",
        "component_uuid" => "board_component",
        "net_uuid" => "net",
        "object_id" | "object_uuid" | "uuid" => source,
        "pad_id" | "pad_uuid" => "board_pad",
        "pin_uuid" => "schematic_pin",
        "symbol_uuid" => "schematic_symbol",
        "zone_id" => "zone_fill",
        _ => key,
    }
    .to_string()
}

fn target_id_string(value: &serde_json::Value) -> Option<String> {
    value
        .as_str()
        .map(str::to_string)
        .or_else(|| value.as_u64().map(|value| value.to_string()))
        .or_else(|| value.as_i64().map(|value| value.to_string()))
}

fn first_object_id(payload: &serde_json::Value) -> Option<&str> {
    object_id_array_values(payload).next()
}

fn object_id_array_values(payload: &serde_json::Value) -> impl Iterator<Item = &str> + Clone {
    ["objects", "object_uuids"]
        .into_iter()
        .filter_map(|key| payload.get(key).and_then(serde_json::Value::as_array))
        .flatten()
        .filter_map(serde_json::Value::as_str)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn primary_target_maps_known_payload_ids_to_domain_kinds() {
        assert_eq!(
            check_primary_target("zone_fill", &serde_json::json!({ "zone_id": "zone-a" })),
            serde_json::json!({ "kind": "zone_fill", "id": "zone-a" })
        );
        assert_eq!(
            check_primary_target(
                "artifact",
                &serde_json::json!({ "artifact_id": "artifact-a" })
            ),
            serde_json::json!({ "kind": "artifact", "id": "artifact-a" })
        );
        assert_eq!(
            check_primary_target("drc", &serde_json::json!({ "pad_id": 12 })),
            serde_json::json!({ "kind": "board_pad", "id": "12" })
        );
    }

    #[test]
    fn primary_target_prefers_object_uuid_arrays_for_compatibility() {
        let target = check_primary_target(
            "drc",
            &serde_json::json!({
                "objects": ["object-a"],
                "pad_id": "pad-a"
            }),
        );
        assert_eq!(
            target,
            serde_json::json!({ "kind": "object_uuid", "id": "object-a" })
        );
    }
    #[test]
    fn native_occurrence_targets_keep_ordered_paths_and_actual_net_scope() {
        let pin = uuid::Uuid::from_u128(1);
        let outer = uuid::Uuid::from_u128(3);
        let inner = uuid::Uuid::from_u128(2);
        let net = uuid::Uuid::from_u128(4);
        let payload = serde_json::json!({
            "object_uuids": [pin], "net_uuid": net,
            "occurrences": [
                {"class":"pins", "source_id":pin, "instance_path":[outer,inner]},
                {"class":"pins", "source_id":pin, "instance_path":[inner,outer]}
            ]
        });
        let primary = check_primary_target("erc", &payload);
        assert_eq!(
            primary,
            serde_json::json!({
                "kind":"object_uuid", "id":pin, "instance_path":[inner,outer]
            })
        );
        assert_eq!(
            check_related_targets(&primary, &payload),
            vec![
                serde_json::json!({"kind":"object_uuid", "id":pin,"instance_path":[outer,inner]}),
                serde_json::json!({"kind":"net", "id":net})
            ]
        );
    }
}
