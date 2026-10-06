//! Prepared, revision-qualified selection precedence for pointer-only drawing.
use super::*;
use crate::cpu_alloc::heap;
use datum_gui_protocol::{PaneContent, SelectionTarget};
use std::collections::BTreeSet;
use std::sync::Arc;

#[derive(Clone, Default)]
pub(crate) struct SelectionHover(Option<Arc<Keys>>);
struct Keys {
    board: Box<[Box<str>]>,
    schematic: Box<[Box<str>]>,
    _charge: Option<crate::DocumentCpuCharge>,
}
impl std::fmt::Debug for SelectionHover {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("SelectionHover")
            .field(&self.0.as_ref().map(|k| (&k.board, &k.schematic)))
            .finish()
    }
}
impl PartialEq for SelectionHover {
    fn eq(&self, other: &Self) -> bool {
        match (&self.0, &other.0) {
            (Some(a), Some(b)) => a.board == b.board && a.schematic == b.schematic,
            (None, None) => true,
            _ => false,
        }
    }
}
impl SelectionHover {
    pub(crate) fn capture(
        state: &ReviewWorkspaceState,
        retained: &RetainedScene,
    ) -> anyhow::Result<Self> {
        let SelectionTarget::Native(view) = &state.selection else {
            return Ok(Self::default());
        };
        let board = view.projection_keys(&state.scene, PaneContent::Board);
        let schematic = state
            .schematic_scene
            .as_ref()
            .and_then(|s| view.projection_keys(s, PaneContent::Schematic));
        if board.is_none_or(BTreeSet::is_empty) && schematic.is_none_or(BTreeSet::is_empty) {
            return Ok(Self::default());
        }
        let mut bytes = container_bytes();
        for keys in [board, schematic].into_iter().flatten() {
            bytes = bytes
                .checked_add(heap::capacity_bytes::<Box<str>>(keys.len()))
                .ok_or_else(|| anyhow::anyhow!("selection hover key storage overflow"))?;
            for key in keys {
                bytes = bytes
                    .checked_add(heap::capacity_bytes::<u8>(key.len()))
                    .ok_or_else(|| anyhow::anyhow!("selection hover key storage overflow"))?;
            }
        }
        // Reserve before copying keys. Refusal preserves the host's pending
        // preparation rather than silently dropping selection precedence.
        let charge = retained
            .geometry_observer()
            .try_charge_document_storage(bytes)
            .ok_or_else(|| anyhow::anyhow!("selection hover keys exceed document CPU budget"))?;
        let copy = |keys: Option<&BTreeSet<String>>| -> Box<[Box<str>]> {
            let Some(keys) = keys else {
                return Box::new([]);
            };
            let mut copied = Vec::with_capacity(keys.len());
            for key in keys {
                copied.push(Box::<str>::from(key.as_str()));
            }
            copied.into_boxed_slice()
        };
        Ok(Self(Some(Arc::new(Keys {
            board: copy(board),
            schematic: copy(schematic),
            _charge: Some(charge),
        }))))
    }
    pub(crate) fn suppresses(&self, surface: SceneSurface, target: &str) -> bool {
        self.0.as_ref().is_some_and(|keys| {
            let keys = match surface {
                SceneSurface::Board => &keys.board,
                SceneSurface::Schematic => &keys.schematic,
            };
            keys.binary_search_by(|key| key.as_ref().cmp(target))
                .is_ok()
        })
    }
}
fn container_bytes() -> usize {
    if !crate::cpu_alloc::installed() {
        return std::mem::size_of::<Keys>() + 2 * std::mem::size_of::<usize>();
    }
    static BYTES: std::sync::OnceLock<usize> = std::sync::OnceLock::new();
    *BYTES.get_or_init(|| {
        heap::arc_bytes(Keys {
            board: Box::new([]),
            schematic: Box::new([]),
            _charge: None,
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn selection_keys_are_revision_bound_and_charge_shared_storage_until_last_drop() {
        let (_fixture, mut state) =
            crate::native_selection_overlay::tests::selected_track_state(10_000_000);
        let retained = RetainedScene::from_workspace(&state, 1600, 1000);
        let observer = retained.geometry_observer();
        let baseline = observer.document_cpu_payload_bytes();
        let keys = SelectionHover::capture(&state, &retained).unwrap();
        let target = &state.scene.tracks[0].object_id;
        assert!(keys.suppresses(SceneSurface::Board, target));
        assert!(!keys.suppresses(SceneSurface::Schematic, target));
        assert!(!keys.suppresses(SceneSurface::Board, "foreign-source"));
        let charged = observer.document_cpu_payload_bytes();
        assert!(charged > baseline);
        let clone = keys.clone();
        assert_eq!(observer.document_cpu_payload_bytes(), charged);
        drop(keys);
        assert!(clone.suppresses(SceneSurface::Board, target));
        assert_eq!(observer.document_cpu_payload_bytes(), charged);
        drop(clone);
        assert_eq!(observer.document_cpu_payload_bytes(), baseline);
        state.scene.source_revision.push_str("-changed");
        assert_eq!(
            SelectionHover::capture(&state, &retained).unwrap(),
            SelectionHover::default()
        );
        assert_eq!(observer.document_cpu_payload_bytes(), baseline);
    }
    #[test]
    fn selection_hover_storage_refuses_document_exhaustion_and_recovers_after_release() {
        let (_fixture, state) =
            crate::native_selection_overlay::tests::selected_track_state(10_000_000);
        let retained = RetainedScene::from_workspace(&state, 1600, 1000);
        let observer = retained.geometry_observer();
        let baseline = observer.document_cpu_payload_bytes();
        let pressure = observer
            .try_charge_document_storage(64 * 1024 * 1024 - baseline)
            .unwrap();
        assert!(SelectionHover::capture(&state, &retained).is_err());
        drop(pressure);
        assert!(SelectionHover::capture(&state, &retained).is_ok());
        assert_eq!(observer.document_cpu_payload_bytes(), baseline);
    }
}
