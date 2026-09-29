//! Source-bound pad paint dependencies and their shared CPU lifetime.
use super::{RetainedScene, document_cpu};
use crate::cpu_alloc::heap::capacity_bytes;
use crate::{HitTarget, ReviewWorkspaceState};
use std::sync::{Arc, Weak};

pub(crate) fn material_id(state: &ReviewWorkspaceState) -> Option<&str> {
    if !matches!(state.selection, datum_gui_protocol::SelectionTarget::None) {
        return None;
    }
    let hover = state.ui.hovered_object.as_ref()?;
    (hover.surface == datum_gui_protocol::PaneContent::Board
        && state
            .scene
            .pads
            .iter()
            .any(|pad| pad.object_id == hover.object_id))
    .then_some(hover.object_id.as_str())
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Membership(Option<Arc<Vec<u64>>>);
impl Membership {
    pub(crate) fn build(
        state: &ReviewWorkspaceState,
        index: &datum_gui_viewport::SpatialHitIndex<HitTarget>,
        budget: &Arc<crate::text_gpu::budget::Budget>,
        scope: &crate::cpu_alloc::Scope,
        limit: usize,
    ) -> Self {
        Self::try_build(state, index, budget, scope, limit).unwrap_or_default()
    }
    fn try_build(
        state: &ReviewWorkspaceState,
        index: &datum_gui_viewport::SpatialHitIndex<HitTarget>,
        budget: &Arc<crate::text_gpu::budget::Budget>,
        scope: &crate::cpu_alloc::Scope,
        limit: usize,
    ) -> Option<Self> {
        let words = index.regions().len().checked_add(63)? / 64;
        let bytes = capacity_bytes::<u64>(words).checked_add(container_bytes())?;
        document_cpu::admit_constructor_allocation(budget, scope, bytes, limit, "hover membership")
            .ok()?;
        let mut bits = Vec::new();
        bits.try_reserve_exact(words).ok()?;
        bits.resize(words, 0);
        for (i, region) in index.regions().iter().enumerate() {
            let id = match &region.target {
                HitTarget::AuthoredObject(id) | HitTarget::ReviewAction(id) => id,
                _ => continue,
            };
            let count = state
                .scene
                .pads
                .iter()
                .filter(|p| p.object_id == *id)
                .take(2)
                .count();
            if count > 1 {
                return None;
            }
            if count == 1 {
                bits[i / 64] |= 1 << (i % 64);
            }
        }
        Some(Self(Some(Arc::new(bits))))
    }
    pub(crate) fn available(&self) -> bool {
        self.0.is_some()
    }
    pub(crate) fn is_pad(&self, region: usize) -> Option<bool> {
        let bits = self.0.as_ref()?;
        Some(bits.get(region / 64)? & (1 << (region % 64)) != 0)
    }
    pub(crate) fn observe(&self) -> Observer {
        Observer(
            self.0
                .as_ref()
                .map(|bits| (Arc::downgrade(bits), capacity_bytes::<u64>(bits.capacity()))),
        )
    }
    pub(crate) fn bytes(&self) -> usize {
        self.0.as_ref().map_or(0, |bits| {
            container_bytes() + capacity_bytes::<u64>(bits.capacity())
        })
    }
}
#[derive(Clone, Default)]
pub(crate) struct Observer(Option<(Weak<Vec<u64>>, usize)>);
impl Observer {
    pub(crate) fn is_live(&self) -> bool {
        self.0.as_ref().is_some_and(|(w, _)| w.strong_count() != 0)
    }
    pub(crate) fn same(&self, other: &Self) -> bool {
        match (&self.0, &other.0) {
            (Some((a, _)), Some((b, _))) => a.ptr_eq(b),
            (None, None) => true,
            _ => false,
        }
    }
    pub(crate) fn bytes(&self) -> usize {
        self.0.as_ref().map_or(0, |(w, bytes)| {
            container_bytes() + if w.strong_count() != 0 { *bytes } else { 0 }
        })
    }
}
fn container_bytes() -> usize {
    if !crate::cpu_alloc::installed() {
        return std::mem::size_of::<Vec<u64>>();
    }
    static BYTES: std::sync::OnceLock<usize> = std::sync::OnceLock::new();
    *BYTES.get_or_init(|| crate::cpu_alloc::heap::arc_bytes(Vec::<u64>::new()))
}
impl RetainedScene {
    pub(crate) fn hover_is_pad(&self, region: usize) -> Option<bool> {
        self.hover_membership.is_pad(region)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn optional_membership_accounts_sharing_weak_retirement_and_refusal() {
        let state = datum_gui_protocol::load_fixture_workspace_state();
        let scene = RetainedScene::from_workspace(&state, 960, 720);
        let budget = document_cpu::for_scene(&state.scene.scene_id);
        let scope = crate::cpu_alloc::Scope::new("hover-membership-control");
        let refused =
            scope.with(|| Membership::build(&state, &scene.world_hit_index, &budget, &scope, 0));
        assert!(!refused.available());
        let membership = scope.with(|| {
            Membership::build(
                &state,
                &scene.world_hit_index,
                &budget,
                &scope,
                document_cpu::DOCUMENT_LIMIT,
            )
        });
        assert!(membership.available());
        let live = || {
            let u = scope.usage();
            (u.payload_bytes + u.tracking_bytes) as usize
        };
        assert_eq!(membership.bytes(), live());
        let clone = scope.with(|| membership.clone());
        let observer = membership.observe();
        assert!(observer.same(&clone.observe()));
        assert_eq!(observer.bytes(), live());
        for (i, r) in scene.world_hit_index.regions().iter().enumerate() {
            let expected = match &r.target {
                HitTarget::AuthoredObject(id) | HitTarget::ReviewAction(id) => {
                    state.scene.pads.iter().any(|p| p.object_id == *id)
                }
                _ => false,
            };
            assert_eq!(membership.is_pad(i), Some(expected));
        }
        drop(membership);
        assert!(observer.is_live());
        drop(clone);
        assert!(!observer.is_live());
        assert_eq!(observer.bytes(), live());
        drop(observer);
        assert_eq!(live(), 0);
    }
}
