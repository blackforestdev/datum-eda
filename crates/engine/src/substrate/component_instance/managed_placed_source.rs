//! Common managed revisions qualify a placed source before its legacy birth
//! fallback. Discovery heads alone never select a definition for a placement.
use super::*;
use crate::{
    library_assets::{AssetKind, ClosureLimits, inspect_closure},
    substrate::DesignModel,
    substrate::library_asset_resolver::{counter_policy, read_state, resolve_state, server_policy},
};
use serde_json::Value;

pub(super) struct Definitions {
    pub(super) symbol: Value,
    pub(super) entity: Value,
}

fn invalid(message: &str) -> EngineError {
    EngineError::Validation(format!("Managed placed library source: {message}"))
}
fn id(raw: &Value, field: &str) -> Result<Uuid, EngineError> {
    raw.get(field)
        .and_then(Value::as_str)
        .and_then(|value| value.parse().ok())
        .ok_or_else(|| invalid("placed component chain is incomplete"))
}

pub(super) struct Placement<'a> {
    pub(super) source_path: String,
    pub(super) sheet: Value,
    pub(super) raw: Value,
    pub(super) owners: Vec<&'a ComponentInstance>,
}

/// Shared typed membership and declared-ownership qualification, retained from
/// the original placed Symbol reader. Entity queries must enforce it as well.
pub(super) fn qualify(
    model: &DesignModel,
    placed: Uuid,
    library_symbol: Uuid,
) -> Result<Placement<'_>, EngineError> {
    if model.journal_cursor.applied_transaction_count != model.journal.len() {
        return Err(invalid("journal tip is not the resolved source basis"));
    }
    let object = model
        .objects
        .get(&placed)
        .filter(|o| o.domain == "schematic")
        .ok_or_else(|| invalid("placed object is missing or foreign"))?;
    let shard = model
        .source_shards
        .iter()
        .find(|s| s.shard_id == object.source_shard_id)
        .ok_or_else(|| invalid("placed source shard is unavailable"))?;
    let sheet = model.materialized_source_shard_value_by_relative_path(&shard.relative_path)?;
    if !matches!(
        shard.kind,
        SourceShardKind::SchematicSheet
            | SourceShardKind::SchematicRoot
            | SourceShardKind::SchematicDefinition
    ) {
        return Err(invalid("placed source is not a schematic partition"));
    }
    // Registry kinds for UUID-keyed maps are their key, not "symbols".
    // Authenticate the typed source membership and embedded identity instead.
    let key = placed.to_string();
    let mut rows = Vec::new();
    if let Some(row) = sheet.get("symbols").and_then(|s| s.get(&key)) {
        rows.push((sheet["uuid"].clone(), row));
    }
    if let Some(sheets) = sheet.get("sheets").and_then(Value::as_object) {
        rows.extend(sheets.values().filter_map(|s| {
            s.get("symbols")
                .and_then(|rows| rows.get(&key))
                .map(|row| (s["uuid"].clone(), row))
        }));
    }
    if rows.len() != 1 {
        return Err(invalid(
            "placed Symbol source membership is unavailable or ambiguous",
        ));
    }
    let (sheet_id, raw) = &rows[0];
    if raw
        .get("uuid")
        .and_then(Value::as_str)
        .and_then(|id| Uuid::parse_str(id).ok())
        != Some(placed)
    {
        return Err(invalid(
            "placed source identity differs from its membership",
        ));
    }
    if raw
        .get("lib_id")
        .and_then(Value::as_str)
        .and_then(|id| Uuid::parse_str(id).ok())
        != Some(library_symbol)
    {
        return Err(invalid("library identity does not match the placed source"));
    }
    let owners: Vec<_> = model
        .component_instances
        .values()
        .filter(|ci| {
            ci.authority == ComponentInstanceAuthority::Authored
                && ci.placed_symbol_refs.contains(&placed)
        })
        .collect();
    for shard in model
        .source_shards
        .iter()
        .filter(|s| s.kind == SourceShardKind::ComponentInstance)
    {
        let raw = model.materialized_source_shard_value_by_relative_path(&shard.relative_path)?;
        let persisted: super::ComponentInstanceShard = serde_json::from_value(raw)?;
        let input = persisted.component_instance;
        if input
            .placed_symbol_refs
            .iter()
            .any(|r| r.object_id == placed)
            && !owners.iter().any(|owner| owner.id == input.uuid)
        {
            return Err(invalid(
                "declared placed ownership failed canonical binding resolution",
            ));
        }
    }
    if owners.len() > 1 {
        return Err(invalid("placed binding ownership is ambiguous"));
    }
    Ok(Placement {
        source_path: shard.relative_path.clone(),
        sheet: sheet_id.clone(),
        raw: (*raw).clone(),
        owners,
    })
}

pub(super) fn definitions(
    model: &DesignModel,
    placement: &Placement<'_>,
) -> Result<Option<Definitions>, EngineError> {
    if model.project.schema_version != Some(2) {
        return Ok(None);
    }
    let placed = id(&placement.raw, "uuid")?;
    let source_path = &placement.source_path;
    let placed_raw = &placement.raw;
    let [owner] = placement.owners.as_slice() else {
        return Ok(None);
    };
    let heads: Vec<_> = model
        .source_shards
        .iter()
        .filter(|head| head.kind == SourceShardKind::LibraryLock)
        .collect();
    let [lock_head] = heads.as_slice() else {
        return Err(invalid("common library lock ownership is unavailable"));
    };
    let root = lock_head
        .path
        .parent()
        .and_then(Path::parent)
        .ok_or_else(|| invalid("common library lock path is unavailable"))?;
    if lock_head.relative_path != "library/lock.json"
        || lock_head.path != root.join(&lock_head.relative_path)
    {
        return Err(invalid("common library lock path is foreign"));
    }
    let limits = server_policy()?;
    let state = read_state(root, limits)?;
    // Bind the existing verified library reader to this accepted model's heads;
    // a held model cannot accidentally read a later on-disk lock or catalog.
    for (kind, relative, bytes) in [
        (
            SourceShardKind::LibraryLock,
            "library/lock.json",
            &state.lock,
        ),
        (
            SourceShardKind::LibraryCatalog,
            "library/library.json",
            &state.catalog,
        ),
    ] {
        let heads: Vec<_> = model
            .source_shards
            .iter()
            .filter(|head| head.kind == kind)
            .collect();
        let [head] = heads.as_slice() else {
            return Err(invalid("common library metadata ownership is ambiguous"));
        };
        if head.relative_path != relative
            || head.path != root.join(relative)
            || head.content_hash != crate::substrate::sha256_hex(bytes)
        {
            return Err(invalid(
                "common library metadata differs from accepted model",
            ));
        }
    }
    let snapshot = resolve_state(root, &state, &BTreeMap::new(), limits)?;
    let bindings: Vec<_> = snapshot
        .lock()
        .bindings
        .iter()
        .filter(|binding| binding.component_instance_id == owner.id)
        .collect();
    if bindings.is_empty() {
        return Ok(None);
    }
    let schematic = crate::substrate::electrical_topology_source::schematic(model, &[])?;
    let origins: Vec<_> = crate::connectivity::authored_occurrences(&schematic)
        .map_err(|error| invalid(&format!("placed occurrence is unavailable: {error:?}")))?
        .into_iter()
        .filter(|source| source.class == "symbols" && source.source_id == placed)
        .collect();
    let [origin] = origins.as_slice() else {
        return Err(invalid("placed occurrence is absent or repeated"));
    };
    let matching: Vec<_> = bindings
        .into_iter()
        .filter(|binding| {
            binding.source_path == *source_path && binding.instance_path == origin.instance_path
        })
        .collect();
    let [binding] = matching.as_slice() else {
        return Err(invalid(
            "common placed occurrence binding is absent or ambiguous",
        ));
    };
    if owner.part_ref != Some(binding.component.asset_id) {
        return Err(invalid("common component differs from the authored Part"));
    }
    let closure = inspect_closure(
        std::slice::from_ref(&binding.component),
        snapshot.assets(),
        ClosureLimits {
            max_nodes: limits.files,
            max_depth: limits.depth,
        },
    )?;
    if !closure.graph_report().valid {
        return Err(invalid("managed component graph is invalid"));
    }
    let ci_object = model
        .objects
        .get(&owner.id)
        .ok_or_else(|| invalid("managed component source is unavailable"))?;
    if model
        .source_shards
        .iter()
        .filter(|head| {
            head.shard_id == ci_object.source_shard_id
                && head.kind == SourceShardKind::ComponentInstance
                && head.path == root.join(&head.relative_path)
        })
        .count()
        != 1
    {
        return Err(invalid("managed component source ownership is ambiguous"));
    }
    let (declaration, ci_bytes) = model.capture_source_shard_value_bounded(
        root,
        ci_object.source_shard_id,
        limits.blob_bytes,
    )?;
    let declaration: ComponentInstanceShard = serde_json::from_value(declaration)?;
    if declaration.schema_version != COMPONENT_INSTANCE_SHARD_SCHEMA_VERSION
        || super::component_instance_from_persisted(&declaration.component_instance) != **owner
        || ci_object.object_revision != owner.object_revision
    {
        return Err(invalid(
            "managed component differs from its authored source",
        ));
    }
    if let Some(reference) = &declaration.component_instance.part_ref {
        if reference.object_id != binding.component.asset_id {
            return Err(invalid(
                "authored Part pin differs from the common component",
            ));
        }
        let part = &snapshot.assets()[&binding.component];
        part.payload().verify_materialization(
            &model.materialized_pool_object_revision(
                reference.object_id,
                "parts",
                reference.object_revision,
            )?,
            counter_policy(AssetKind::Part, part.revision().payload_schema_version)?,
        )?;
    }
    let mut native = BTreeMap::new();
    let mut remaining = limits
        .total_bytes
        .checked_sub(ci_bytes)
        .ok_or_else(|| invalid("managed native closure exceeds library policy"))?;
    for (reference, asset) in closure.assets() {
        let entries: Vec<_> = snapshot
            .lock()
            .assets
            .iter()
            .filter(|entry| entry.asset.asset_id == reference.asset_id)
            .collect();
        let [entry] = entries.as_slice() else {
            return Err(invalid(
                "managed dependency is absent or ambiguous in the lock",
            ));
        };
        let kind = asset.revision().kind;
        let pool_kind = kind
            .pool_kind()
            .ok_or_else(|| invalid("owned model is not a placed definition"))?;
        let object = model
            .objects
            .get(&reference.asset_id)
            .filter(|object| object.domain == "pool" && object.kind == pool_kind)
            .ok_or_else(|| invalid("managed native definition is absent or foreign"))?;
        if entry.asset != *reference
            || entry.local_object_id != reference.asset_id
            || entry.payload_ref != asset.revision().payload_ref
            || entry.source_path != format!("pool/{pool_kind}/{}.json", reference.asset_id)
        {
            return Err(invalid(
                "managed dependency does not match the exact active revision",
            ));
        }
        if model
            .source_shards
            .iter()
            .filter(|head| {
                head.shard_id == object.source_shard_id
                    && head.kind == SourceShardKind::Pool
                    && head.relative_path == entry.source_path
                    && head.path == root.join(&entry.source_path)
            })
            .count()
            != 1
        {
            return Err(invalid(
                "managed native source ownership differs from the lock",
            ));
        }
        let (value, observed_bytes) = model.capture_source_shard_value_bounded(
            root,
            object.source_shard_id,
            limits.blob_bytes.min(remaining),
        )?;
        remaining = remaining
            .checked_sub(observed_bytes)
            .ok_or_else(|| invalid("managed native closure exceeds library policy"))?;
        asset.payload().verify_materialization(
            &value,
            counter_policy(kind, asset.revision().payload_schema_version)?,
        )?;
        native.insert(reference.asset_id, (kind, value));
    }
    for local in owner.library_bindings.values() {
        if !local.local_override_refs.is_empty() {
            return Err(invalid(
                "local override requires its own reviewed managed intent",
            ));
        }
        if let Some(asset) = closure
            .assets()
            .values()
            .find(|asset| asset.reference().asset_id == local.target_object_id)
        {
            let pinned = model.materialized_pool_object_revision(
                local.target_object_id,
                asset.revision().kind.pool_kind().unwrap(),
                local.pinned_object_revision,
            )?;
            asset.payload().verify_materialization(
                &pinned,
                counter_policy(
                    asset.revision().kind,
                    asset.revision().payload_schema_version,
                )?,
            )?;
        } else {
            return Err(invalid(
                "local definition conflicts with the managed component closure",
            ));
        }
    }
    let take = |id: Uuid, kind: AssetKind| -> Result<Value, EngineError> {
        native
            .get(&id)
            .filter(|(actual, _)| *actual == kind)
            .map(|(_, value)| value.clone())
            .ok_or_else(|| invalid("placed chain is outside its exact managed component closure"))
    };
    let part = take(binding.component.asset_id, AssetKind::Part)?;
    let entity_id = id(placed_raw, "entity")?;
    let gate_id = id(placed_raw, "gate")?;
    let symbol_id = id(placed_raw, "lib_id")?;
    if id(&part, "entity")? != entity_id {
        return Err(invalid("placed Entity differs from the managed Part"));
    }
    let entity = take(entity_id, AssetKind::Entity)?;
    let gate = entity
        .get("gates")
        .and_then(|gates| gates.get(gate_id.to_string()))
        .ok_or_else(|| invalid("placed gate is absent from the managed Entity"))?;
    let symbol = take(symbol_id, AssetKind::Symbol)?;
    if id(gate, "symbol")? != symbol_id || id(gate, "unit")? != id(&symbol, "unit")? {
        return Err(invalid("placed gate, Symbol and Unit disagree"));
    }
    take(id(&symbol, "unit")?, AssetKind::Unit)?;
    Ok(Some(Definitions { symbol, entity }))
}
