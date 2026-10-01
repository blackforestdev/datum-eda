//! C01 identity nomination against one complete final partition.
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;

use super::{
    ElectricalOccurrence, EngineError, ModelRevision, NetAnchorReason, NetIdentityTransition,
    PreviousNetGroup, ResolvedNetIdentity,
};

/// Plan once, after topology resolution. Allocation happens only for unclaimed
/// final groups; callers record the returned UUIDs in their canonical batch.
/// This does not resolve geometry or certify a caller-supplied graph partition.
pub fn plan_net_identity_transition(
    source_revision: ModelRevision,
    previous: &[PreviousNetGroup],
    mut final_groups: Vec<BTreeSet<ElectricalOccurrence>>,
    mut allocate: impl FnMut() -> Uuid,
) -> Result<NetIdentityTransition, EngineError> {
    final_groups.sort();
    let mut membership = BTreeMap::new();
    for (index, group) in final_groups.iter().enumerate() {
        if group.is_empty() {
            return Err(invalid("empty final group"));
        }
        for member in group {
            if membership.insert(member.clone(), index).is_some() {
                return Err(invalid("duplicate final occurrence"));
            }
        }
    }
    let mut old_ids = BTreeSet::new();
    let mut old_members = BTreeSet::new();
    let mut nominees = BTreeMap::<usize, BTreeMap<Uuid, ElectricalOccurrence>>::new();
    for old in previous {
        if old.net_id.is_nil() || !old_ids.insert(old.net_id) {
            return Err(invalid("duplicate/nil old NetId"));
        }
        if !old.members.contains(&old.anchor) {
            return Err(invalid("old anchor not in old group"));
        }
        for member in &old.members {
            if !old_members.insert(member.clone()) {
                return Err(invalid("duplicate old occurrence"));
            }
        }
        let replacement = if membership.contains_key(&old.anchor) {
            Some(&old.anchor)
        } else {
            old.members
                .iter()
                .find(|member| membership.contains_key(*member))
        };
        if let Some(anchor) = replacement {
            nominees
                .entry(membership[anchor])
                .or_default()
                .insert(old.net_id, anchor.clone());
        }
    }
    let mut retained = BTreeSet::new();
    let mut allocated_ids = BTreeSet::new();
    let mut resolved = Vec::new();
    for (index, members) in final_groups.into_iter().enumerate() {
        let claims = nominees.remove(&index).unwrap_or_default();
        let (net_id, anchor, allocated) = if let Some((id, anchor)) = claims.first_key_value() {
            retained.insert(*id);
            (*id, anchor.clone(), false)
        } else {
            let id = allocate();
            if id.is_nil() || old_ids.contains(&id) || !allocated_ids.insert(id) {
                return Err(invalid("allocator reused/nil NetId"));
            }
            (id, members.first().expect("nonempty checked").clone(), true)
        };
        resolved.push(ResolvedNetIdentity {
            net_id,
            anchor_reason: if allocated {
                NetAnchorReason::UnclaimedFinalGroup
            } else if previous
                .iter()
                .any(|old| old.net_id == net_id && old.anchor == anchor)
            {
                NetAnchorReason::SurvivingAnchor
            } else {
                NetAnchorReason::DeletedAnchorFallback
            },
            anchor,
            members,
            predecessors: claims.keys().copied().collect(),
            allocated,
        });
    }
    Ok(NetIdentityTransition {
        source_revision,
        final_groups: resolved,
        retired: old_ids.difference(&retained).copied().collect(),
    })
}

fn invalid(reason: &str) -> EngineError {
    EngineError::Validation(format!("invalid Net identity transition: {reason}"))
}
