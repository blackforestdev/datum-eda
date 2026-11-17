//! Pins name actual pool shards and journal incarnations, not display aliases.
use super::*;
use crate::substrate::{Operation, SourceShardKind, SourceShardTaxon};
use serde_json::Value;

pub(crate) fn hash_value(value: &Value) -> Result<String, EngineError> {
    Ok(crate::substrate::sha256_hex(
        crate::ir::serialization::to_json_deterministic(value)?.as_bytes(),
    ))
}
fn historical_root_kind(model: &DesignModel, pin: &PinnedPadLibrarySource) -> Option<&'static str> {
    let kind = |name: &str| match name {
        "parts" => Some("parts"),
        "packages" if pin.object_id == pin.owner_id => Some("packages"),
        "pin_pad_maps" => Some("pin_pad_maps"),
        _ => None,
    };
    std::path::Path::new(&pin.relative_path)
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|p| p.to_str())
        .and_then(kind)
        .or_else(|| {
            model
                .objects
                .get(&pin.owner_id)
                .filter(|o| o.domain == "pool")
                .and_then(|o| kind(&o.kind))
        })
}
pub(crate) fn source_value(
    model: &DesignModel,
    pin: &PinnedPadLibrarySource,
) -> Result<Value, EngineError> {
    let context = PinFrame::Accepted(model);
    source_value_in(&context, pin)
}

pub(super) fn source_value_in(
    context: &PinFrame<'_>,
    pin: &PinnedPadLibrarySource,
) -> Result<Value, EngineError> {
    let model = context.model();
    let shard = model
        .source_shards
        .iter()
        .find(|s| {
            s.kind == SourceShardKind::Pool
                && s.shard_id == pin.source_shard_id
                && s.relative_path == pin.relative_path
        })
        .ok_or_else(|| invalid("missing pinned pool shard"))?;
    let value = context.owner_value(&shard.relative_path)?;
    if value.get("uuid").and_then(Value::as_str) != Some(pin.owner_id.to_string().as_str()) {
        return Err(invalid("foreign pinned library owner"));
    }
    if let Some(kind) = historical_root_kind(model, pin) {
        if pin.object_id != pin.owner_id {
            return Err(invalid("Definition pin must name its root definition"));
        }
        if !model.objects.get(&pin.owner_id).is_some_and(|o| {
            o.domain == "pool" && o.kind == kind && o.source_shard_id == pin.source_shard_id
        }) {
            return Err(invalid("Definition pin has a foreign source owner or kind"));
        }
        let historical = context.definition(pin.owner_id, kind, pin.object_revision)?;
        let hash = hash_value(&historical)?;
        if hash != pin.owner_value_hash
            || hash != pin.record_hash
            || context.incarnation(&pin.relative_path, pin.object_id)? != pin.incarnation
        {
            return Err(invalid(
                "Definition pin differs from authenticated historical source",
            ));
        }
        return Ok(historical);
    }
    Ok(value)
}
fn record(value: &Value, id: Uuid) -> Option<&Value> {
    if value.get("uuid").and_then(Value::as_str) == Some(id.to_string().as_str()) {
        return Some(value);
    }
    value
        .get("pads")?
        .as_object()?
        .get(&id.to_string())
        .filter(|p| p.get("uuid").and_then(Value::as_str) == Some(id.to_string().as_str()))
}
pub(crate) fn incarnation(model: &DesignModel, path: &str, id: Uuid) -> Option<Uuid> {
    let mut birth = None;
    for t in &model.journal {
        for op in &t.operations {
            let added = match op {
                Operation::CreatePoolLibraryObject {
                    relative_path,
                    object,
                    ..
                } if relative_path == path => record(object, id).is_some(),
                Operation::CreatePoolPackage {
                    relative_path,
                    package,
                    ..
                } if relative_path == path => record(package, id).is_some(),
                Operation::CreatePoolPadstack {
                    relative_path,
                    padstack,
                    ..
                } if relative_path == path => record(padstack, id).is_some(),
                Operation::SetPoolLibraryObject {
                    relative_path,
                    previous_object,
                    object,
                    ..
                } if relative_path == path => {
                    record(previous_object, id).is_none() && record(object, id).is_some()
                }
                _ => false,
            };
            if added {
                birth = Some(t.transaction_id);
            }
        }
    }
    birth
}
fn capture(
    model: &DesignModel,
    path: &str,
    id: Uuid,
) -> Result<PinnedPadLibrarySource, EngineError> {
    capture_in(&PinFrame::Accepted(model), path, id)
}

pub(super) fn capture_in(
    context: &PinFrame<'_>,
    path: &str,
    id: Uuid,
) -> Result<PinnedPadLibrarySource, EngineError> {
    let model = context.model();
    let shard = model
        .source_shards
        .iter()
        .find(|s| s.kind == SourceShardKind::Pool && s.relative_path == path)
        .ok_or_else(|| invalid("missing library source shard"))?;
    let value = context.owner_value(path)?;
    let owner_id = serde_json::from_value(
        value
            .get("uuid")
            .cloned()
            .ok_or_else(|| invalid("missing library owner UUID"))?,
    )?;
    let source = record(&value, id)
        .ok_or_else(|| invalid("library object does not belong to pinned owner"))?;
    let mut historical_revision = source
        .get("object_revision")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    for t in &model.journal {
        for operation in &t.operations {
            if let Operation::SetPoolLibraryObject {
                relative_path,
                previous_object,
                object,
                ..
            } = operation
                && relative_path == path
            {
                match (record(previous_object, id), record(object, id)) {
                    (None, Some(_)) => historical_revision = 0,
                    (Some(old), Some(new)) if old != new || id == owner_id => {
                        historical_revision = historical_revision
                            .checked_add(1)
                            .ok_or_else(|| invalid("library revision overflow"))?
                    }
                    _ => {}
                }
            }
        }
    }
    if context
        .objects()
        .get(&id)
        .is_some_and(|o| o.domain == "pool" && o.source_shard_id != shard.shard_id)
    {
        return Err(invalid(
            "ambiguous library UUID belongs to another source owner",
        ));
    }
    let object_revision = context
        .objects()
        .get(&id)
        .filter(|o| o.domain == "pool" && o.source_shard_id == shard.shard_id)
        .map_or(ObjectRevision(historical_revision), |o| o.object_revision);
    Ok(PinnedPadLibrarySource {
        object_id: id,
        owner_id,
        source_shard_id: shard.shard_id,
        relative_path: path.into(),
        object_revision,
        owner_value_hash: hash_value(&value)?,
        record_hash: hash_value(source)?,
        incarnation: context.incarnation(path, id)?,
        journal_tip: context.anchor_tip(),
    })
}
pub(crate) fn capture_root(
    model: &DesignModel,
    id: Uuid,
    taxon: SourceShardTaxon,
) -> Result<PinnedPadLibrarySource, EngineError> {
    let mut matches = Vec::new();
    for shard in model
        .source_shards
        .iter()
        .filter(|s| s.kind == SourceShardKind::Pool && s.taxon == Some(taxon))
    {
        let value = model.materialized_source_shard_value_by_relative_path(&shard.relative_path)?;
        if value.get("uuid").and_then(Value::as_str) == Some(id.to_string().as_str()) {
            matches.push(shard.relative_path.as_str());
        }
    }
    if matches.len() != 1 {
        return Err(invalid("library source owner is absent or ambiguous"));
    }
    capture(model, matches[0], id)
}
pub(crate) fn capture_pad(
    model: &DesignModel,
    owner: &PinnedPadLibrarySource,
    id: Uuid,
) -> Result<PinnedPadLibrarySource, EngineError> {
    capture(model, &owner.relative_path, id)
}
pub(crate) fn check_pin(
    model: &DesignModel,
    pin: &PinnedPadLibrarySource,
) -> Result<(), EngineError> {
    check_pin_in(&PinFrame::Accepted(model), pin)
}

pub(super) fn check_pin_in(
    context: &PinFrame<'_>,
    pin: &PinnedPadLibrarySource,
) -> Result<(), EngineError> {
    let model = context.model();
    let now = capture_in(context, &pin.relative_path, pin.object_id)?;
    if historical_root_kind(model, pin).is_some() {
        if now.owner_id != pin.owner_id
            || now.source_shard_id != pin.source_shard_id
            || now.incarnation != pin.incarnation
        {
            return Err(invalid("foreign or reused definition source pin"));
        }
        source_value_in(context, pin)?;
        return Ok(());
    }
    if now.owner_id != pin.owner_id
        || now.source_shard_id != pin.source_shard_id
        || now.object_revision != pin.object_revision
        || now.record_hash != pin.record_hash
        || now.incarnation != pin.incarnation
    {
        return Err(invalid("stale library source pin"));
    }
    if now.owner_value_hash != pin.owner_value_hash {
        unchanged_package_pad_owner_in(context, pin)?;
    }
    Ok(())
}

/// A legacy pad retains its own revision. Authenticate its old Package root
/// separately; only the four descriptive fields may differ from today's root.
fn unchanged_package_pad_owner(
    model: &DesignModel,
    pin: &PinnedPadLibrarySource,
) -> Result<(), EngineError> {
    unchanged_package_pad_owner_in(&PinFrame::Accepted(model), pin)
}
fn unchanged_package_pad_owner_in(
    context: &PinFrame<'_>,
    pin: &PinnedPadLibrarySource,
) -> Result<(), EngineError> {
    let model = context.model();
    let owner = model
        .objects
        .get(&pin.owner_id)
        .filter(|o| {
            o.domain == "pool"
                && o.kind == "packages"
                && o.source_shard_id == pin.source_shard_id
                && pin.object_id != pin.owner_id
        })
        .ok_or_else(|| invalid("stale library source pin"))?;
    // None is the valid source basis before the first canonical transaction.
    let start = match pin.journal_tip {
        None => 0,
        Some(id) => model
            .journal
            .iter()
            .position(|t| t.transaction_id == id)
            .map(|i| i + 1)
            .ok_or_else(|| invalid("missing Package pad history anchor"))?,
    };
    let mut revision = owner.object_revision.0;
    for op in model.journal[start..].iter().flat_map(|t| &t.operations) {
        match op {
            Operation::SetPoolLibraryObject { object_id, .. }
            | Operation::BumpObjectRevision { object_id }
                if *object_id == pin.owner_id =>
            {
                revision = revision
                    .checked_sub(1)
                    .ok_or_else(|| invalid("Package pad history counter differs"))?;
            }
            Operation::CreatePoolLibraryObject { object_id, .. }
            | Operation::DeletePoolLibraryObject { object_id, .. }
                if *object_id == pin.owner_id =>
            {
                return Err(invalid("Package pad history crosses a definition epoch"));
            }
            _ => {}
        }
    }
    let mut historical = context.definition(pin.owner_id, "packages", ObjectRevision(revision))?;
    if hash_value(&historical)? != pin.owner_value_hash {
        return Err(invalid(
            "Package pad owner differs from authenticated history",
        ));
    }
    let mut current = context.owner_value(&pin.relative_path)?;
    for key in ["name", "package_family", "package_code", "mounting_type"] {
        historical
            .as_object_mut()
            .ok_or_else(|| invalid("missing Package object"))?
            .remove(key);
        current
            .as_object_mut()
            .ok_or_else(|| invalid("missing Package object"))?
            .remove(key);
    }
    if historical != current {
        return Err(invalid("Package pad owner has structural changes"));
    }
    Ok(())
}

/// Only the parent snapshot may borrow the qualified precommit Package basis.
/// The candidate's actual child identity, revision and geometry remain current.
pub(super) fn check_pin_with_package_basis(
    model: &DesignModel,
    accepted: &DesignModel,
    pin: &PinnedPadLibrarySource,
) -> Result<(), EngineError> {
    if !model
        .objects
        .get(&pin.owner_id)
        .is_some_and(|o| o.domain == "pool" && o.kind == "packages")
    {
        return check_pin(model, pin);
    }
    let now = capture(model, &pin.relative_path, pin.object_id)?;
    if now.owner_id != pin.owner_id
        || now.source_shard_id != pin.source_shard_id
        || now.incarnation != pin.incarnation
        || (pin.object_id != pin.owner_id
            && (now.object_revision != pin.object_revision || now.record_hash != pin.record_hash))
    {
        return Err(invalid(
            "candidate Package pad differs from its frozen source",
        ));
    }
    check_pin(accepted, pin)
}

pub(crate) fn recorded_pin(
    model: &DesignModel,
    pin: &PinnedPadLibrarySource,
) -> Result<(), EngineError> {
    if check_pin(model, pin).is_ok() {
        return Ok(());
    }
    let start = pin
        .journal_tip
        .and_then(|id| model.journal.iter().position(|t| t.transaction_id == id))
        .ok_or_else(|| invalid("missing pinned history anchor"))?;
    for transaction in &model.journal[start + 1..] {
        for operation in &transaction.operations {
            let values: Vec<&Value> = match operation {
                Operation::SetPoolLibraryObject {
                    relative_path,
                    previous_object,
                    object,
                    ..
                } if relative_path == &pin.relative_path => vec![previous_object, object],
                Operation::DeletePoolLibraryObject {
                    relative_path,
                    object,
                    ..
                } if relative_path == &pin.relative_path => vec![object],
                Operation::DeletePoolPackage {
                    relative_path,
                    package,
                    ..
                } if relative_path == &pin.relative_path => vec![package],
                Operation::DeletePoolPadstack {
                    relative_path,
                    padstack,
                    ..
                } if relative_path == &pin.relative_path => vec![padstack],
                _ => vec![],
            };
            for value in values {
                if hash_value(value)? == pin.owner_value_hash
                    && record(value, pin.object_id)
                        .is_some_and(|r| hash_value(r).is_ok_and(|h| h == pin.record_hash))
                {
                    return Ok(());
                }
            }
        }
    }
    Err(invalid(
        "pinned source snapshot has no canonical history proof",
    ))
}
pub(crate) fn continuity(
    model: &DesignModel,
    old: &PinnedPadLibrarySource,
    new: &PinnedPadLibrarySource,
) -> Result<(), EngineError> {
    continuity_in(&PinFrame::Accepted(model), old, new)
}

pub(super) fn continuity_in(
    context: &PinFrame<'_>,
    old: &PinnedPadLibrarySource,
    new: &PinnedPadLibrarySource,
) -> Result<(), EngineError> {
    let model = context.model();
    recorded_pin(model, old)?;
    check_pin_in(context, new)?;
    if old.object_id == new.object_id
        && (old.owner_id != new.owner_id || old.incarnation != new.incarnation)
    {
        return Err(invalid("library UUID rebirth is not Pad continuity"));
    }
    if old.owner_id == new.owner_id && old.object_id == new.object_id {
        let start = old
            .journal_tip
            .and_then(|id| model.journal.iter().position(|t| t.transaction_id == id))
            .ok_or_else(|| invalid("continuity needs a recorded source-history anchor"))?;
        for t in &model.journal[start + 1..] {
            for op in &t.operations {
                match op {
                    Operation::SetPoolLibraryObject {
                        relative_path,
                        previous_object,
                        object,
                        ..
                    } if relative_path == &old.relative_path => {
                        if record(previous_object, old.object_id).is_none()
                            || record(object, old.object_id).is_none()
                        {
                            return Err(invalid("library Pad did not survive source history"));
                        }
                    }
                    Operation::DeletePoolLibraryObject { relative_path, .. }
                    | Operation::DeletePoolPackage { relative_path, .. }
                        if relative_path == &old.relative_path =>
                    {
                        return Err(invalid("land-pattern retirement is not continuity"));
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(())
}

/// Read context is private to the source owner. Pending anchors must already
/// exist in accepted history; a transaction ID is never predicted for a birth.
pub(super) enum PinFrame<'a> {
    Accepted(&'a DesignModel),
    Pending(&'a crate::substrate::journal_source_frame::VerifiedPendingSourceFrame),
    Planned(&'a super::target::PlannedPadSources<'a>),
}
impl PinFrame<'_> {
    pub(super) fn model(&self) -> &DesignModel {
        match self {
            Self::Accepted(model) => model,
            Self::Pending(frame) => frame.candidate(),
            Self::Planned(target) => target.accepted(),
        }
    }
    fn objects(&self) -> &std::collections::BTreeMap<Uuid, crate::substrate::DomainObject> {
        match self {
            Self::Planned(target) => target.objects(),
            _ => &self.model().objects,
        }
    }
    pub(super) fn owner_value(&self, path: &str) -> Result<Value, EngineError> {
        match self {
            Self::Accepted(model) => model.materialized_source_shard_value_by_relative_path(path),
            Self::Planned(target) => target.value(path),
            Self::Pending(frame) => {
                let head = frame
                    .candidate()
                    .source_shards
                    .iter()
                    .find(|s| s.kind == SourceShardKind::Pool && s.relative_path == path)
                    .ok_or_else(|| invalid("missing certified pending library owner"))?;
                frame.value(head)
            }
        }
    }
    pub(super) fn definition(
        &self,
        id: Uuid,
        kind: &str,
        revision: ObjectRevision,
    ) -> Result<Value, EngineError> {
        match self {
            Self::Accepted(model) => model.materialized_pool_object_revision(id, kind, revision),
            Self::Pending(frame) => frame.pinned_definition(id, kind, revision),
            Self::Planned(target) => target.definition(id, kind, revision),
        }
    }
    fn incarnation(&self, path: &str, id: Uuid) -> Result<Option<Uuid>, EngineError> {
        match self {
            Self::Accepted(model) => Ok(incarnation(model, path, id)),
            Self::Planned(target) => {
                let accepted = target.accepted().objects.get(&id);
                let planned = target.objects().get(&id);
                if accepted.zip(planned).is_none_or(|(old, new)| {
                    old.domain != "pool"
                        || new.domain != "pool"
                        || old.source_shard_id != new.source_shard_id
                }) {
                    return Err(invalid(
                        "new or reborn planned source is not same-pad continuity",
                    ));
                }
                Ok(incarnation(target.accepted(), path, id))
            }
            Self::Pending(frame) => {
                let old = frame
                    .accepted()
                    .objects
                    .get(&id)
                    .filter(|o| o.domain == "pool");
                let current = frame
                    .candidate()
                    .objects
                    .get(&id)
                    .filter(|o| o.domain == "pool");
                if old.is_none()
                    || old
                        .zip(current)
                        .is_none_or(|(a, b)| a.source_shard_id != b.source_shard_id)
                    || incarnation(frame.accepted(), path, id)
                        != incarnation(frame.candidate(), path, id)
                {
                    return Err(invalid(
                        "new or reborn library pin requires committed acquisition before materialization",
                    ));
                }
                Ok(incarnation(frame.accepted(), path, id))
            }
        }
    }
    fn anchor_tip(&self) -> Option<Uuid> {
        let model = match self {
            Self::Accepted(model) => *model,
            Self::Pending(frame) => frame.accepted(),
            Self::Planned(target) => target.accepted(),
        };
        model.journal.last().map(|t| t.transaction_id)
    }
}
