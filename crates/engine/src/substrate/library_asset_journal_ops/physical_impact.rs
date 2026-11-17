//! Targeted refusal of unsupported library uptake into existing frozen PCB
//! sources. This is not an ECO producer or a ban on lawful stale native edits.
use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;
use uuid::Uuid;

use crate::{
    error::EngineError,
    library_assets::{AssetKind, AssetRevisionRef, VerifiedAsset, lock::ProjectLibraryLock},
    substrate::{
        DesignModel, ElectricalSelectionSnapshot, SourceShardKind,
        component_pad_source::{ComponentPadSource, PinnedPadLibrarySource},
        library_asset_operations::refusal,
        library_asset_resolver::counter_policy,
    },
};

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct UnsupportedPhysicalAdoption {
    pub(crate) component_instance: Option<Uuid>,
    pub(crate) placed_package: Uuid,
    pub(crate) reference: String,
    pub(crate) assets: Vec<Uuid>,
}

/// All inputs belong to existing captured/verified owners. No IO, latest lookup,
/// generated pins, mutation or independent graph authority occurs here.
pub(crate) fn assess(
    model: &DesignModel,
    sources: &ElectricalSelectionSnapshot,
    before_pool: &BTreeMap<String, Value>,
    target: &ProjectLibraryLock,
    available: &BTreeMap<AssetRevisionRef, VerifiedAsset>,
) -> Result<Vec<UnsupportedPhysicalAdoption>, EngineError> {
    let mut changes = BTreeMap::new();
    for entry in &target.assets {
        let asset = available
            .get(&entry.asset)
            .ok_or_else(|| refusal("physical review lacks the exact target record"))?;
        let kind = asset.revision().kind;
        if !matches!(
            kind,
            AssetKind::Part
                | AssetKind::Package
                | AssetKind::Footprint
                | AssetKind::Padstack
                | AssetKind::PinPadMap
        ) {
            continue;
        }
        let same = match before_pool.get(&entry.source_path) {
            Some(value) => physical_matches(kind, asset, value)?,
            None => false,
        };
        if !same {
            changes.insert(entry.asset.asset_id, (kind, asset));
        }
    }
    if changes.is_empty() {
        return Ok(Vec::new());
    }
    let heads: Vec<_> = model
        .source_shards
        .iter()
        .filter(|head| head.kind == SourceShardKind::BoardRoot)
        .collect();
    if heads.len() > 1 {
        return Err(refusal("physical review has ambiguous Board ownership"));
    }
    let Some(head) = heads.first() else {
        return Ok(Vec::new());
    };
    let board = sources
        .source_shard_value(&model.model_revision, head.shard_id)
        .map_err(|error| refusal(format!("physical review source unavailable: {error:?}")))?;
    let Some(packages) = board.get("packages").and_then(Value::as_object) else {
        return Ok(Vec::new());
    };
    let mut affected = Vec::new();
    for package in packages.keys() {
        let package_id: Uuid = package
            .parse()
            .map_err(|_| refusal("invalid placed package identity"))?;
        let reference: String = serde_json::from_value(packages[package]["reference"].clone())?;
        let pads: Vec<ComponentPadSource> =
            match board.get("component_pads").and_then(|v| v.get(package)) {
                Some(value) => serde_json::from_value(value.clone())?,
                None => Vec::new(),
            };
        let mut impacts: BTreeMap<Option<Uuid>, BTreeSet<Uuid>> = BTreeMap::new();
        for pad in &pads {
            let Some(binding) = &pad.source_binding else {
                continue;
            };
            pad.validate_frozen()?;
            if binding.package_id != package_id || binding.project_id != model.project.project_id {
                return Err(refusal("physical review has a foreign frozen Pad source"));
            }
            let owner = binding.component_instance.as_ref().map(|r| r.object_id);
            let roots = [
                &binding.part,
                &binding.package,
                &binding.land_pattern,
                &binding.library_pad,
                &binding.padstack,
            ]
            .into_iter()
            .chain(binding.pin_pad_map.iter())
            .collect::<Vec<_>>();
            for pin in &roots {
                let Some((kind, next)) = changes.get(&pin.owner_id) else {
                    continue;
                };
                // A legacy subsidiary is covered by its separately recorded
                // root-owner pin. That root authenticates the entire old tree,
                // including embedded geometry; do not treat a child as a root.
                if pin.object_id != pin.owner_id
                    && roots.iter().any(|root| {
                        root.object_id == pin.owner_id
                            && root.owner_id == pin.owner_id
                            && root.source_shard_id == pin.source_shard_id
                            && root.relative_path == pin.relative_path
                    })
                {
                    if !same_incarnation(model, pin) {
                        impacts.entry(owner).or_default().insert(pin.owner_id);
                    }
                    continue;
                }
                if !matches_frozen(model, sources, pin, *kind, next)? {
                    impacts.entry(owner).or_default().insert(pin.owner_id);
                }
            }
        }
        // No qualified frozen sources means no asserted old geometry to bless.
        // Scope this conservative fallback to a changed selected Part closure;
        // unrelated legacy placements do not block a local library operation.
        if pads.is_empty() || pads.iter().any(|p| p.source_binding.is_none()) {
            let part = packages[package]
                .get("part")
                .and_then(Value::as_str)
                .and_then(|id| id.parse::<Uuid>().ok());
            if let Some(part) = part
                && let Some(root) = target.assets.iter().find(|a| a.asset.asset_id == part)
            {
                let mut visited = BTreeSet::new();
                let mut pending = vec![root.asset.clone()];
                while let Some(reference) = pending.pop() {
                    if !visited.insert(reference.clone()) {
                        continue;
                    }
                    let asset = available
                        .get(&reference)
                        .ok_or_else(|| refusal("verified physical dependency is unavailable"))?;
                    if changes.contains_key(&reference.asset_id) {
                        impacts.entry(None).or_default().insert(reference.asset_id);
                    }
                    // These edges were already admitted by inspect_closure.
                    pending.extend(
                        asset
                            .revision()
                            .dependencies
                            .iter()
                            .map(|d| d.target.clone()),
                    );
                }
            }
        }
        affected.extend(impacts.into_iter().map(|(component_instance, assets)| {
            UnsupportedPhysicalAdoption {
                component_instance,
                placed_package: package_id,
                reference: reference.clone(),
                assets: assets.into_iter().collect(),
            }
        }));
    }
    affected.sort_by_key(|impact| (impact.placed_package, impact.component_instance));
    Ok(affected)
}

fn matches_frozen(
    model: &DesignModel,
    sources: &ElectricalSelectionSnapshot,
    pin: &PinnedPadLibrarySource,
    kind: AssetKind,
    target: &VerifiedAsset,
) -> Result<bool, EngineError> {
    let Some(owner) = model.objects.get(&pin.owner_id) else {
        return Ok(false);
    };
    if pin.object_id != pin.owner_id
        || owner.source_shard_id != pin.source_shard_id
        || owner.domain != "pool"
        || Some(owner.kind.as_str()) != kind.pool_kind()
        || !same_incarnation(model, pin)
    {
        return Ok(false);
    }
    if !model.source_shards.iter().any(|s| {
        s.shard_id == pin.source_shard_id
            && s.relative_path == pin.relative_path
            && s.kind == SourceShardKind::Pool
    }) {
        return Ok(false);
    };
    let frozen = match sources.pinned_library_definition(
        &model.model_revision,
        pin.owner_id,
        kind.pool_kind().unwrap(),
        pin.object_revision,
    ) {
        Ok(value) => value,
        // An unprovable historical target is not a rollback permission.
        Err(_) => return Ok(false),
    };
    let hash = crate::substrate::component_pad_source::hash_value(&frozen)?;
    if hash != pin.owner_value_hash || hash != pin.record_hash {
        return Ok(false);
    }
    physical_matches(kind, target, &frozen)
}

fn same_incarnation(model: &DesignModel, pin: &PinnedPadLibrarySource) -> bool {
    model.objects.get(&pin.object_id).is_some_and(|object| {
        object.domain == "pool" && object.source_shard_id == pin.source_shard_id
    }) && crate::substrate::component_pad_source::source_incarnation(
        model,
        &pin.relative_path,
        pin.object_id,
    ) == pin.incarnation
}

fn physical_matches(
    kind: AssetKind,
    target: &VerifiedAsset,
    native: &Value,
) -> Result<bool, EngineError> {
    let descriptive: &'static [&'static str] = match kind {
        AssetKind::Footprint => &["name", "tags"],
        AssetKind::Padstack => &["name"],
        AssetKind::PinPadMap => &["tags"],
        // The existing PM057 Package continuity owner classifies these four.
        AssetKind::Package => &["name", "package_family", "package_code", "mounting_type"],
        AssetKind::Part => &[
            "mpn",
            "manufacturer",
            "manufacturer_jep106",
            "value",
            "description",
            "datasheet",
            "parametric",
            "orderable_mpns",
            "packaging_options",
            "tags",
            "lifecycle",
            "supply_chain_offers",
            "last_supply_chain_check",
        ],
        _ => &[],
    };
    target
        .payload()
        .matches_with_descriptive_fields(native, counter_policy(kind, 1)?, descriptive)
}

pub(crate) fn require_supported(
    impacts: &[UnsupportedPhysicalAdoption],
) -> Result<(), EngineError> {
    if impacts.is_empty() {
        return Ok(());
    }
    let users = impacts
        .iter()
        .map(|impact| {
            let component = impact
                .component_instance
                .map(|id| id.to_string())
                .unwrap_or_else(|| "not identified".into());
            let definitions = impact
                .assets
                .iter()
                .map(Uuid::to_string)
                .collect::<Vec<_>>()
                .join(", ");
            format!(
                "{} (PCB placement {}; component {}): affected definitions {}",
                impact.reference, impact.placed_package, component, definitions
            )
        })
        .collect::<Vec<_>>()
        .join("; ");
    Err(refusal(format!(
        "physical library adoption requires an unsupported reviewed PCB update; affected users: {users}; any previously saved candidate is retained; active project unchanged"
    )))
}

#[cfg(test)]
#[path = "physical_impact_tests.rs"]
mod tests;
