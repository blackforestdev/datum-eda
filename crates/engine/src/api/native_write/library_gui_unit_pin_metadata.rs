//! Sparse saved Unit pin metadata edits against exact captured raw rows.
use super::*;
use serde::Deserialize;

pub(super) const KEY: &str = "pin_metadata";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PinMetadata {
    previous: Value,
    #[serde(default)]
    alternate_edits: Vec<AlternateEdit>,
    #[serde(default)]
    alternate_removals: Vec<CapturedAlternate>,
    #[serde(default)]
    alternate_additions: Vec<crate::pool::AlternateName>,
    #[serde(default)]
    swap_group: Option<u32>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CapturedAlternate {
    index: usize,
    previous: Value,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AlternateEdit {
    index: usize,
    previous: Value,
    name: String,
    kind: String,
}

fn check_row(
    rows: &[Value],
    addressed: &mut HashSet<usize>,
    index: usize,
    previous: &Value,
) -> Result<(), EngineError> {
    if rows.get(index) != Some(previous) || !addressed.insert(index) {
        return Err(refusal(
            "Saved alternate name row has changed or was addressed twice",
        ));
    }
    Ok(())
}

fn apply_pin(intent: PinMetadata, pin: &mut Value) -> Result<(), EngineError> {
    if let Some(group) = intent.swap_group {
        pin["swap_group"] = json!(group);
    }
    if !intent.alternate_edits.is_empty()
        || !intent.alternate_removals.is_empty()
        || !intent.alternate_additions.is_empty()
    {
        let original = intent.previous["alternates"]
            .as_array()
            .ok_or_else(|| refusal("Saved alternate names must be an ordered array"))?;
        let mut rows = original.clone();
        let mut addressed = HashSet::new();
        for edit in intent.alternate_edits {
            check_row(original, &mut addressed, edit.index, &edit.previous)?;
            let row = rows[edit.index]
                .as_object_mut()
                .ok_or_else(|| refusal("Saved alternate name is malformed"))?;
            row.insert("name".into(), json!(edit.name));
            row.insert("kind".into(), json!(edit.kind));
        }
        let mut removed = HashSet::new();
        for row in intent.alternate_removals {
            check_row(original, &mut addressed, row.index, &row.previous)?;
            removed.insert(row.index);
        }
        let mut next: Vec<_> = rows
            .into_iter()
            .enumerate()
            .filter_map(|(index, row)| (!removed.contains(&index)).then_some(row))
            .collect();
        for row in intent.alternate_additions {
            next.push(serde_json::to_value(row)?);
        }
        pin["alternates"] = Value::Array(next);
    }
    let _: crate::pool::Pin = serde_json::from_value(pin.clone())?;
    Ok(())
}

pub(super) fn apply(
    context: &NativeWriteContext<'_>,
    request: &LibraryDefinitionRequest,
    previous: &Value,
    object: &mut Value,
    removed: &HashSet<Uuid>,
) -> Result<(), EngineError> {
    let edits: std::collections::BTreeMap<String, PinMetadata> =
        serde_json::from_value(request.fields[KEY].clone())?;
    for (key, intent) in edits {
        let id =
            Uuid::parse_str(&key).map_err(|_| refusal("Saved logical pin identity is invalid"))?;
        let pin_id = intent.previous["uuid"]
            .as_str()
            .and_then(|raw| raw.parse().ok());
        let owner = &context.model.objects[&request.object_id];
        let qualified = context.model.objects.get(&id).is_some_and(|pin| {
            pin.domain == "pool"
                && pin.source_shard_id == owner.source_shard_id
                && (pin.kind == "pins" || pin.kind.parse::<Uuid>().ok() == Some(id))
        });
        if id.is_nil()
            || pin_id != Some(id)
            || !qualified
            || removed.contains(&id)
            || previous["pins"].get(&key) != Some(&intent.previous)
            || previous["pins"].as_object().is_none_or(|pins| {
                pins.keys()
                    .filter(|key| key.parse::<Uuid>().ok() == Some(id))
                    .count()
                    != 1
            })
        {
            return Err(refusal(
                "Saved logical pin has changed, is ambiguous or is being removed",
            ));
        }
        let pin = object["pins"]
            .get_mut(&key)
            .ok_or_else(|| refusal("Saved logical pin is unavailable"))?;
        apply_pin(intent, pin)?;
    }
    Ok(())
}
