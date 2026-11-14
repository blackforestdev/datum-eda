//! Disposable explicit Part/gate rows in the existing Symbol picker.
use super::*;
use eda_engine::pool::{Entity, Part};

fn object(model: &DesignModel, id: Uuid, kind: &str) -> anyhow::Result<serde_json::Value> {
    let object = model
        .objects
        .get(&id)
        .filter(|o| o.domain == "pool" && o.kind == kind)
        .ok_or_else(|| anyhow::anyhow!("Selected {kind} unavailable"))?;
    let shard = model
        .source_shards
        .iter()
        .find(|s| s.shard_id == object.source_shard_id)
        .ok_or_else(|| anyhow::anyhow!("Selected source unavailable"))?;
    model
        .materialized_source_shard_value_by_relative_path(&shard.relative_path)
        .map_err(Into::into)
}
pub(super) fn append(state: &mut SchematicAuthoring, model: &DesignModel) {
    let symbols = state.choices.clone();
    let mut added = Vec::new();
    for record in model
        .objects
        .values()
        .filter(|o| o.domain == "pool" && o.kind == "parts")
    {
        if !model.source_shards.iter().any(|s| {
            s.shard_id == record.source_shard_id && s.relative_path.starts_with("pool/parts/")
        }) {
            continue;
        }
        let result = (|| -> anyhow::Result<()> {
            let part: Part = serde_json::from_value(object(model, record.object_id, "parts")?)?;
            let entity: Entity = serde_json::from_value(object(model, part.entity, "entities")?)?;
            for gate in entity.gates.values() {
                let Some(mut choice) = symbols.iter().find(|c| c.id == gate.symbol).cloned() else {
                    return Err(anyhow::anyhow!("A Part gate Symbol is unavailable"));
                };
                choice.name = format!(
                    "Part: {} {} · {} · {} / {}",
                    part.manufacturer, part.mpn, gate.name, entity.name, choice.symbol.name
                );
                choice.prefix = entity.prefix.clone();
                choice.value = part.value.clone();
                match PartGateSelection::capture(model, part.uuid, gate.uuid, gate.symbol) {
                    Ok(intent) => {
                        choice.explicit_part = Some(intent);
                        choice.binding_status = "explicit_part_gate".into();
                        choice.binding_diagnostics.clear();
                    }
                    Err(error) => {
                        state.refusal = Some(format!("Some Part choices are unavailable: {error}"));
                        continue;
                    }
                }
                match eda_engine::api::native_write::schematic_library::part_placement::CompletePartSelection::capture(model, part.uuid, gate.uuid, gate.symbol) {
                    Ok(component) => {
                        choice.component = Some(component);
                        state.components.choices.push(choice.clone());
                    }
                    Err(error) => state.components.unavailable.push((choice.name.clone(), error.to_string())),
                }
                added.push(choice);
            }
            Ok(())
        })();
        if let Err(error) = result {
            state.refusal = Some(format!("Some Part choices are unavailable: {error}"));
        }
    }
    added.sort_by(|a, b| {
        a.name.cmp(&b.name).then_with(|| {
            let key = |c: &LibrarySymbolChoice| c.explicit_part.as_ref().map(|s| (s.part, s.gate));
            key(a).cmp(&key(b))
        })
    });
    state.choices.extend(added);
    state.components.choices.sort_by(|a, b| a.name.cmp(&b.name));
}

#[cfg(test)]
#[path = "schematic_part_choices_tests.rs"]
mod tests;
