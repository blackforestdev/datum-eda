//! Assemble an exact selected update through the existing verified closure owner.
use super::*;
use crate::library_assets::{ClosureLimits, inspect_closure};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn update_roots(
    context: &NativeWriteContext<'_>,
    expected_catalog: &AssetDigest,
    selected: &AssetRevisionRef,
    limits: LibraryAssetLimits,
) -> Result<Vec<AssetRevisionRef>, EngineError> {
    if context.model.project.schema_version != Some(2) {
        return Err(refusal("separate forward-only format adoption is required"));
    }
    let previous = read_state(context.project_root, limits)?;
    let snapshot = resolve_state(context.project_root, &previous, &BTreeMap::new(), limits)?;
    if &snapshot.catalog().manifest_ref != expected_catalog {
        return Err(refusal("catalog changed since update selection"));
    }
    if !snapshot.assets().contains_key(selected) {
        return Err(refusal("selected exact revision is unavailable locally"));
    }
    if snapshot
        .lock()
        .assets
        .iter()
        .any(|asset| &asset.asset == selected)
    {
        return Err(refusal("selected exact revision is already active"));
    }
    let bounds = ClosureLimits {
        max_nodes: limits.files,
        max_depth: limits.depth,
    };
    let catalog = inspect_closure(&snapshot.manifest().roots, snapshot.assets(), bounds)?;
    if !catalog.graph_report().valid {
        return Err(refusal("local candidate graph is invalid"));
    }
    let catalog_by_id: BTreeMap<_, _> = catalog
        .assets()
        .keys()
        .map(|reference| (reference.asset_id, reference.clone()))
        .collect();
    let selected_closure =
        inspect_closure(std::slice::from_ref(selected), snapshot.assets(), bounds)?;
    // Intermediate closures authenticate exact forward dependencies. A Map's
    // Part owner is contextual, so its isolated native graph is incomplete;
    // validate the native graph only after assembling the coherent group below.
    // A retained shared leaf must bring its recorded dependent revisions with
    // it. Search only each explicit head's ancestry; inventory order and dates
    // never choose a substitute or manufacture a repinned parent record.
    let mut affected_heads = BTreeMap::new();
    for active in &snapshot.lock().assets {
        let closure = inspect_closure(
            std::slice::from_ref(&active.asset),
            snapshot.assets(),
            bounds,
        )?;
        if closure
            .assets()
            .keys()
            .any(|r| r.asset_id == selected.asset_id)
        {
            affected_heads.insert(
                active.asset.asset_id,
                catalog_by_id
                    .get(&active.asset.asset_id)
                    .unwrap_or(&active.asset)
                    .clone(),
            );
        }
    }
    // Include a newly captured candidate's dependent roots as well as existing
    // placed users, without selecting unrelated pending catalog heads.
    for root in &snapshot.manifest().roots {
        let closure = inspect_closure(std::slice::from_ref(root), snapshot.assets(), bounds)?;
        if closure
            .assets()
            .keys()
            .any(|r| r.asset_id == selected.asset_id)
        {
            affected_heads.insert(root.asset_id, root.clone());
        }
    }
    let mut roots = BTreeSet::from([selected.clone()]);
    for (id, head) in affected_heads {
        if id == selected.asset_id {
            continue;
        }
        roots.insert(compatible_ancestor(
            &head,
            selected,
            &selected_closure,
            snapshot.assets(),
            bounds,
        )?);
    }
    let update = inspect_closure(
        &roots.iter().cloned().collect::<Vec<_>>(),
        snapshot.assets(),
        bounds,
    )?;
    if !update.graph_report().valid {
        return Err(refusal(
            "selected update has an invalid native library graph",
        ));
    }
    let replaced: BTreeSet<_> = update
        .assets()
        .keys()
        .map(|reference| reference.asset_id)
        .collect();
    roots.extend(
        snapshot
            .lock()
            .assets
            .iter()
            .filter(|asset| !replaced.contains(&asset.asset.asset_id))
            .map(|asset| asset.asset.clone()),
    );
    let roots: Vec<_> = roots.into_iter().collect();
    let complete = inspect_closure(&roots, snapshot.assets(), bounds)?;
    if !complete.graph_report().valid {
        return Err(refusal(
            "selected update conflicts with retained active definitions; review a coherent candidate group",
        ));
    }
    Ok(roots)
}

/// Keep only the unique most-descendant compatible recorded revision. A fork
/// needs a separate explicit group review; hash order is not revision authority.
fn compatible_ancestor(
    head: &AssetRevisionRef,
    selected: &AssetRevisionRef,
    selected_closure: &crate::library_assets::AssetClosure<'_>,
    available: &BTreeMap<AssetRevisionRef, crate::library_assets::VerifiedAsset>,
    bounds: ClosureLimits,
) -> Result<AssetRevisionRef, EngineError> {
    fn visit(
        reference: &AssetRevisionRef,
        available: &BTreeMap<AssetRevisionRef, crate::library_assets::VerifiedAsset>,
        bounds: ClosureLimits,
        visiting: &mut BTreeSet<AssetRevisionRef>,
        ancestry: &mut BTreeMap<AssetRevisionRef, BTreeSet<AssetRevisionRef>>,
    ) -> Result<(), EngineError> {
        if ancestry.contains_key(reference) {
            return Ok(());
        }
        if !visiting.insert(reference.clone()) {
            return Err(refusal("dependent revision ancestry is cyclic"));
        }
        if visiting.len() > bounds.max_depth
            || ancestry.len().saturating_add(visiting.len()) > bounds.max_nodes
        {
            return Err(refusal(
                "dependent revision ancestry exceeds library policy",
            ));
        }
        let asset = available
            .get(reference)
            .ok_or_else(|| refusal("dependent revision ancestry is unavailable"))?;
        let mut ancestors = BTreeSet::new();
        for parent in &asset.revision().parents {
            if parent.asset_id != reference.asset_id {
                return Err(refusal("dependent revision has a foreign parent identity"));
            }
            visit(parent, available, bounds, visiting, ancestry)?;
            ancestors.insert(parent.clone());
            ancestors.extend(ancestry[parent].iter().cloned());
        }
        visiting.remove(reference);
        ancestry.insert(reference.clone(), ancestors);
        Ok(())
    }
    let mut ancestry = BTreeMap::new();
    visit(head, available, bounds, &mut BTreeSet::new(), &mut ancestry)?;
    let target: BTreeMap<_, _> = selected_closure
        .assets()
        .keys()
        .map(|reference| (reference.asset_id, reference))
        .collect();
    let mut candidates = BTreeSet::new();
    for reference in ancestry.keys() {
        let closure = inspect_closure(std::slice::from_ref(reference), available, bounds)?;
        if closure.assets().contains_key(selected)
            && closure.assets().keys().all(|reference| {
                target
                    .get(&reference.asset_id)
                    .is_none_or(|wanted| **wanted == *reference)
            })
        {
            candidates.insert(reference.clone());
        }
    }
    let choices: Vec<_> = candidates
        .iter()
        .filter(|candidate| {
            !candidates
                .iter()
                .any(|other| other != *candidate && ancestry[other].contains(*candidate))
        })
        .cloned()
        .collect();
    match choices.as_slice() {
        [choice] => Ok(choice.clone()),
        [] => Err(refusal(
            "no retained dependent revision contains the exact selected revision; review a coherent candidate group",
        )),
        _ => Err(refusal(
            "retained dependent revisions are ambiguous; review an explicit coherent candidate group",
        )),
    }
}

#[cfg(test)]
#[path = "library_asset_selection/tests.rs"]
mod tests;
