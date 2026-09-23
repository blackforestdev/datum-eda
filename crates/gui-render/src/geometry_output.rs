//! Shared geometry emission with explicit retained-construction admission.
use std::ops::{Deref, DerefMut};

pub(crate) trait Output<T>: DerefMut<Target = [T]> + Extend<T> {
    fn push(&mut self, value: T);
    fn reserve(&mut self, additional: usize);
}
impl<T> Output<T> for Vec<T> {
    fn push(&mut self, value: T) {
        Vec::push(self, value);
    }
    fn reserve(&mut self, additional: usize) {
        Vec::reserve(self, additional);
    }
}

/// A failed emission never publishes a partial scene. Ordinary screen emitters
/// retain Vec output; retained board and schematic builders explicitly opt in.
pub(crate) struct Admitted<T, F> {
    values: Vec<T>,
    admit: F,
    failure: Option<anyhow::Error>,
}
impl<T, F: FnMut(usize) -> anyhow::Result<()>> Admitted<T, F> {
    pub fn new(admit: F) -> Self {
        Self {
            values: Vec::new(),
            admit,
            failure: None,
        }
    }
    pub fn finish(self) -> anyhow::Result<Vec<T>> {
        match self.failure {
            Some(error) => Err(error),
            None => Ok(self.values),
        }
    }
    fn grow(&mut self, additional: usize) -> anyhow::Result<()> {
        let needed = self
            .values
            .len()
            .checked_add(additional)
            .ok_or_else(|| anyhow::anyhow!("geometry output length overflow"))?;
        if needed <= self.values.capacity() {
            return Ok(());
        }
        let capacity = needed.max(self.values.capacity().saturating_mul(2)).max(4);
        let layout = std::alloc::Layout::array::<T>(capacity)?;
        // Existing capacity remains in the construction scope while its
        // replacement is admitted, covering a moving allocator's overlap.
        (self.admit)(crate::cpu_alloc::heap::allocation_bytes(layout))?;
        self.values
            .try_reserve_exact(capacity - self.values.len())?;
        Ok(())
    }
}
impl<T, F> Deref for Admitted<T, F> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        &self.values
    }
}
impl<T, F: FnMut(usize) -> anyhow::Result<()>> Output<T> for Admitted<T, F> {
    fn reserve(&mut self, additional: usize) {
        if self.failure.is_none() {
            self.failure = self.grow(additional).err();
        }
    }
    fn push(&mut self, value: T) {
        self.reserve(1);
        if self.failure.is_none() {
            self.values.push(value);
        }
    }
}

impl<T, F: FnMut(usize) -> anyhow::Result<()>> Extend<T> for Admitted<T, F> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, values: I) {
        for value in values {
            self.push(value);
            if self.failure.is_some() {
                break;
            }
        }
    }
}

impl<T, F> DerefMut for Admitted<T, F> {
    fn deref_mut(&mut self) -> &mut [T] {
        &mut self.values
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    #[test]
    fn retained_emission_preserves_geometry_and_commands_and_refuses_before_growth() {
        let state = crate::gpu_surface_pass::board_fixture_state();
        let projection = Projection::new(
            RectPx {
                x: 0.0,
                y: 0.0,
                width: 960.0,
                height: 720.0,
            },
            &state.scene.bounds,
            CameraState::fit_to_bounds(&state.scene.bounds),
        );
        let emit = |quads: &mut Vec<Quad>,
                    strokes: &mut Vec<WorldStrokeInstance>,
                    commands: &mut Vec<RetainedDrawCommand>| {
            push_retained_scene_geometry(
                quads,
                strokes,
                commands,
                &state.scene,
                &projection,
                &state,
            );
            push_retained_board_text_geometry_batches(
                quads,
                commands,
                &state.scene,
                &projection,
                &state,
            );
            push_retained_board_graphic_batches(
                quads,
                strokes,
                commands,
                &state.scene,
                &projection,
                &state,
            );
        };
        let (mut quads, mut strokes, mut commands) = (Vec::new(), Vec::new(), Vec::new());
        emit(&mut quads, &mut strokes, &mut commands);
        let scope = crate::cpu_alloc::Scope::new("admitted-geometry-emission");
        let admit = |bytes: usize| {
            let live = scope.usage();
            anyhow::ensure!(
                live.payload_bytes + live.tracking_bytes + bytes as u64 <= 64 * 1024 * 1024,
                "emission limit"
            );
            Ok(())
        };
        let (a, b, c) = scope.with(|| {
            let (mut a, mut b, mut c) = (
                Admitted::new(admit),
                Admitted::new(admit),
                Admitted::new(admit),
            );
            push_retained_scene_geometry(&mut a, &mut b, &mut c, &state.scene, &projection, &state);
            push_retained_board_text_geometry_batches(
                &mut a,
                &mut c,
                &state.scene,
                &projection,
                &state,
            );
            push_retained_board_graphic_batches(
                &mut a,
                &mut b,
                &mut c,
                &state.scene,
                &projection,
                &state,
            );
            (
                a.finish().unwrap(),
                b.finish().unwrap(),
                c.finish().unwrap(),
            )
        });
        assert_eq!(a, quads);
        assert_eq!(b, strokes);
        assert_eq!(c, commands);
        drop((a, b, c));
        assert_eq!(scope.usage().allocations, 0);
        let mut refused = Admitted::new(|_: usize| anyhow::bail!("refuse before allocation"));
        refused.push(1u64);
        assert_eq!(refused.values.capacity(), 0);
        refused.extend(0..1000);
        assert_eq!(refused.values.capacity(), 0);
        assert!(refused.finish().is_err());
    }
}
