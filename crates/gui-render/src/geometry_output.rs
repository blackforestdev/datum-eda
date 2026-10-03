//! Shared geometry emission with explicit retained-construction admission.
use crate::presentation_ranges::{Role, Span};
use std::ops::{Deref, DerefMut};

pub(crate) trait Output<T>: DerefMut<Target = [T]> + Extend<T> {
    fn push(&mut self, value: T);
    fn source(&mut self, _key: Option<&str>, _role: Role) {}
    fn role(&mut self, _role: Role) {}
    fn reserve(&mut self, additional: usize);
    fn admit_temporary(&mut self, _bytes: usize) -> bool {
        true
    }
    fn allocation_failed(&mut self, error: anyhow::Error) {
        panic!("geometry allocation failed: {error}");
    }
    fn scratch<U>(&mut self, capacity: usize) -> Option<Vec<U>> {
        let result = (|| -> anyhow::Result<Option<Vec<U>>> {
            let layout = std::alloc::Layout::array::<U>(capacity)?;
            if !self.admit_temporary(crate::cpu_alloc::heap::allocation_bytes(layout)) {
                return Ok(None);
            }
            let mut values = Vec::new();
            values.try_reserve_exact(capacity)?;
            Ok(Some(values))
        })();
        match result {
            Ok(values) => values,
            Err(error) => {
                self.allocation_failed(error);
                None
            }
        }
    }
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
    tracking: bool,
    spans: Vec<Span>,
    key: Option<String>,
    role: Role,
    start: usize,
}
impl<T, F: FnMut(usize) -> anyhow::Result<()>> Admitted<T, F> {
    pub fn new(admit: F) -> Self {
        Self {
            values: Vec::new(),
            admit,
            failure: None,
            tracking: false,
            spans: Vec::new(),
            key: None,
            role: Role::Material,
            start: 0,
        }
    }
    pub fn track_sources(mut self) -> Self {
        self.tracking = true;
        self
    }
    pub fn finish_with_sources(mut self) -> anyhow::Result<(Vec<T>, Vec<Span>)> {
        if self.failure.is_none() {
            self.failure = self.flush_source().err();
        }
        match self.failure {
            Some(error) => Err(error),
            None => Ok((self.values, self.spans)),
        }
    }
    fn copy_key(&mut self, key: &str) -> anyhow::Result<String> {
        let layout = std::alloc::Layout::array::<u8>(key.len())?;
        (self.admit)(crate::cpu_alloc::heap::allocation_bytes(layout))?;
        let mut value = String::new();
        value.try_reserve_exact(key.len())?;
        value.push_str(key);
        Ok(value)
    }
    fn flush_source(&mut self) -> anyhow::Result<()> {
        if !self.tracking || self.start == self.values.len() {
            return Ok(());
        }
        if self.spans.len() == self.spans.capacity() {
            let capacity = self
                .spans
                .len()
                .checked_mul(2)
                .ok_or_else(|| anyhow::anyhow!("presentation range overflow"))?
                .max(4);
            (self.admit)(crate::cpu_alloc::heap::allocation_bytes(
                std::alloc::Layout::array::<Span>(capacity)?,
            ))?;
            self.spans.try_reserve_exact(capacity - self.spans.len())?;
        }
        // The active key remains live until the next source boundary. Admit its
        // overlap with the immutable range copy before allocating.
        let key = if let Some(key) = self.key.take() {
            let copy = self.copy_key(&key);
            self.key = Some(key);
            Some(copy?)
        } else {
            None
        };
        self.spans.push(Span {
            key,
            role: self.role,
            range: self.start..self.values.len(),
        });
        self.start = self.values.len();
        Ok(())
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
    fn source(&mut self, key: Option<&str>, role: Role) {
        if !self.tracking || self.failure.is_some() {
            return;
        }
        let result = (|| -> anyhow::Result<()> {
            self.flush_source()?;
            self.key = match key {
                Some(key) => Some(self.copy_key(key)?),
                None => None,
            };
            self.role = role;
            Ok(())
        })();
        self.failure = result.err();
    }
    fn role(&mut self, role: Role) {
        if !self.tracking || self.failure.is_some() || self.role == role {
            return;
        }
        match self.flush_source() {
            Ok(()) => self.role = role,
            Err(error) => self.failure = Some(error),
        }
    }

    fn admit_temporary(&mut self, bytes: usize) -> bool {
        if self.failure.is_none() {
            self.failure = (self.admit)(bytes).err();
        }
        self.failure.is_none()
    }
    fn allocation_failed(&mut self, error: anyhow::Error) {
        if self.failure.is_none() {
            self.failure = Some(error);
        }
    }

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
    fn tracked_emission_keeps_geometry_and_refuses_source_allocation_atomically() {
        let mut values = Admitted::new(|_| Ok(())).track_sources();
        values.source(Some("pad"), Role::Material);
        values.push(1u32);
        values.role(Role::Void);
        values.push(2);
        values.role(Role::Material);
        values.push(3);
        values.source(None, Role::Material);
        values.push(4);
        let (geometry, spans) = values.finish_with_sources().unwrap();
        assert_eq!(geometry, [1, 2, 3, 4]);
        assert_eq!(
            spans
                .iter()
                .map(|span| span.range.clone())
                .collect::<Vec<_>>(),
            [0..1, 1..2, 2..3, 3..4]
        );
        assert_eq!(spans[1].role, Role::Void);
        assert_eq!(spans[1].key.as_deref(), Some("pad"));
        assert_eq!(spans[3].key, None);
        let mut refuse =
            Admitted::new(|_| anyhow::bail!("source allocation refused")).track_sources();
        refuse.source(Some("pad"), Role::Material);
        refuse.push(1u32);
        assert!(refuse.finish_with_sources().is_err());
        let mut flushed = Admitted::new(|bytes| {
            anyhow::ensure!(bytes < 100, "range allocation refused");
            Ok(())
        })
        .track_sources();
        flushed.source(Some("pad"), Role::Material);
        flushed.push(1u32);
        assert!(
            flushed.finish_with_sources().is_err(),
            "final range admission cannot publish partial geometry"
        );
    }

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
