//! Read-only proposed-current process inspection uses explicit persisted identities.
use super::*;
use serde_json::json;

pub fn inspect(
    model: &DesignModel,
    board: &Value,
    pad: &ComponentPadSource,
    placement: &PlacedPackage,
) -> Result<Value, EngineError> {
    let binding = pad
        .source_binding
        .as_ref()
        .ok_or_else(|| invalid("process inspection requires a source binding"))?;
    let mut reasons = Vec::new();
    for (name, pin) in [
        ("part", &binding.part),
        ("package", &binding.package),
        ("land_pattern", &binding.land_pattern),
        ("library_pad", &binding.library_pad),
        ("padstack", &binding.padstack),
    ] {
        if let Err(error) = crate::substrate::component_pad_source::check_pin(model, pin) {
            reasons.push(
                json!({"source": name, "object_id": pin.object_id, "reason": error.to_string()}),
            );
        }
    }
    if let Some(snapshot) = &binding.process_snapshot {
        if let Err(error) = snapshot.check_current(model) {
            reasons.push(json!({"source": "process_snapshot", "reason": error.to_string()}));
        }
    } else {
        reasons.push(json!({"source": "process_snapshot", "reason": "explicit process materialization is absent"}));
    }
    let frozen = binding
        .process_snapshot
        .as_ref()
        .map(|s| s.project(pad, placement))
        .transpose()?;
    let proposed = current(model, board, pad, placement);
    let (proposed, error) = match proposed {
        Ok(value) => (Some(value), None),
        Err(error) => (None, Some(error.to_string())),
    };
    let mut changes = Vec::new();
    if let Some(proposed) = &proposed {
        differences(
            "",
            &serde_json::to_value(&frozen)?,
            &proposed["geometry"],
            &mut changes,
        );
        let old = binding.process_snapshot.as_ref().map(|s| {
            json!({
            "raw_footprint": s.raw_footprint, "raw_padstack": s.raw_padstack,
            "stackup_value_hash": s.stackup_value_hash, "profile": s.profile})
        });
        differences(
            "/inputs",
            &serde_json::to_value(old)?,
            &proposed["inputs"],
            &mut changes,
        );
    }
    Ok(
        json!({"frozen_process": frozen, "proposed_current": proposed,
        "proposed_current_error": error, "exact_differences": changes, "stale_reasons": reasons}),
    )
}

fn current(
    model: &DesignModel,
    board: &Value,
    pad: &ComponentPadSource,
    placement: &PlacedPackage,
) -> Result<Value, EngineError> {
    let binding = pad.source_binding.as_ref().unwrap();
    let footprint = source_value(model, &binding.land_pattern)?;
    let raw_pad = footprint
        .get("pads")
        .and_then(|p| p.get(binding.library_pad.object_id.to_string()))
        .ok_or_else(|| {
            invalid(
                "selected library Pad identity no longer exists; explicit correspondence required",
            )
        })?;
    let library_pad: crate::pool::Pad = serde_json::from_value(raw_pad.clone())?;
    if library_pad.uuid != binding.library_pad.object_id {
        return Err(invalid(
            "current library Pad identity differs from selected correspondence",
        ));
    }
    let effective = crate::api::native_write::process::inspect_effective_process(
        model,
        binding.land_pattern.object_id,
        library_pad.padstack,
    )?;
    let object = model
        .objects
        .get(&library_pad.padstack)
        .ok_or_else(|| invalid("current Padstack canonical identity is absent"))?;
    let shard = model
        .source_shards
        .iter()
        .find(|s| s.shard_id == object.source_shard_id)
        .ok_or_else(|| invalid("current Padstack source shard is absent"))?;
    let raw_padstack =
        model.materialized_source_shard_value_by_relative_path(&shard.relative_path)?;
    let stack: crate::pool::Padstack = serde_json::from_value(raw_padstack.clone())?;
    let mut candidate = pad.clone();
    candidate.source_binding.as_mut().unwrap().frozen_geometry = super::super::FrozenPadGeometry {
        name: library_pad.name,
        position: library_pad.position,
        layer: library_pad.layer,
        aperture: stack
            .aperture
            .ok_or_else(|| invalid("current Padstack aperture is absent"))?,
        drill_nm: stack.drill_nm,
        layer_span: stack.layer_span,
        plated: stack.plated,
    };
    let capability = capability_from_board(board)?
        .ok_or_else(|| invalid("current process capability is absent"))?;
    let stackup: Stackup = serde_json::from_value(board["stackup"].clone())?;
    let copper = super::super::projection::project_pad(&candidate, placement, &stackup)?;
    let geometry = PlacedProcessGeometry {
        pad_id: pad.uuid,
        source_capability: Some(capability.clone()),
        center_nm: copper.center(),
        rotation: placement.rotation,
        reflected: placement.layer != capability.front.copper_layer,
        membership: capability.membership(
            &stackup,
            &copper.terminal().copper_layers,
            &effective,
        )?,
        effective,
    };
    let profile = if let Some(ProcessBasisSelector::ProjectProfile { profile_id, .. }) = footprint
        .get("process_basis")
        .map(|v| serde_json::from_value(v.clone()))
        .transpose()?
    {
        let profile = crate::api::native_write::process::profiles(model)?
            .and_then(|p| p.get(&profile_id).cloned())
            .ok_or_else(|| invalid("current profile is absent"))?;
        Some(FrozenProfilePin {
            project_id: model.project.project_id,
            object_revision: model
                .objects
                .get(&profile_id)
                .ok_or_else(|| invalid("current profile identity is absent"))?
                .object_revision,
            value_hash: hash(&serde_json::to_value(&profile)?)?,
            profile,
        })
    } else {
        None
    };
    Ok(
        json!({"geometry": geometry, "inputs": {"raw_footprint": footprint,
        "raw_padstack": raw_padstack, "stackup_value_hash": stackup_hash(&stackup, &capability)?, "profile": profile}}),
    )
}

fn differences(path: &str, old: &Value, new: &Value, out: &mut Vec<Value>) {
    if old == new {
        return;
    }
    if let (Some(a), Some(b)) = (old.as_object(), new.as_object()) {
        let keys: std::collections::BTreeSet<_> = a.keys().chain(b.keys()).collect();
        for key in keys {
            let escaped = key.replace('~', "~0").replace('/', "~1");
            if a.contains_key(key) != b.contains_key(key) {
                out.push(json!({"path": format!("{path}/{escaped}"),
                    "frozen_present": a.contains_key(key), "proposed_current_present": b.contains_key(key),
                    "frozen": a.get(key), "proposed_current": b.get(key)}));
                continue;
            }
            differences(
                &format!("{path}/{escaped}"),
                a.get(key).unwrap_or(&Value::Null),
                b.get(key).unwrap_or(&Value::Null),
                out,
            );
        }
    } else {
        out.push(json!({"path": path, "frozen": old, "proposed_current": new}));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn process_inspection_diff_retains_presence_and_exact_rational_changes() {
        let mut changes = Vec::new();
        differences(
            "",
            &json!({"dimension": {"numerator": "1071", "denominator": "10"}}),
            &json!({"dimension": {"numerator": "1072", "denominator": "10"}, "raw_policy": null}),
            &mut changes,
        );
        assert_eq!(changes.len(), 2);
        assert_eq!(changes[0]["path"], "/dimension/numerator");
        assert_eq!(changes[0]["frozen"], "1071");
        assert_eq!(changes[0]["proposed_current"], "1072");
        assert_eq!(changes[1]["path"], "/raw_policy");
        assert_eq!(changes[1]["frozen_present"], false);
        assert_eq!(changes[1]["proposed_current_present"], true);
    }
}
