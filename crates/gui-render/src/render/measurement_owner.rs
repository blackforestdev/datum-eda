//! Synchronous layout borrows the native renderer's measurement owner. Returning
//! from preparation restores it even on failure, without sharing host permits.
use crate::text_gpu::budget::Budget;
use crate::text_layout::fonts::Fonts;
use std::{cell::RefCell, sync::Arc};

pub(crate) struct Owner {
    host: Arc<Budget>,
    fonts: Option<Fonts>,
}
impl Owner {
    pub(crate) fn new(host: Arc<Budget>) -> Self {
        Self { host, fonts: None }
    }
    pub(crate) fn usage(&self) -> Option<crate::text_layout::fonts::Usage> {
        self.fonts.as_ref().map(Fonts::usage)
    }
    pub(crate) fn release_for(&mut self, bytes: u64) {
        use crate::text_layout::fonts::Source;
        if let Some(fonts) = &mut self.fonts {
            fonts.release_for(bytes);
        }
    }
    fn measure(
        &mut self,
        text: &str,
        size: f32,
        face: crate::TextFace,
        width: Option<f32>,
    ) -> anyhow::Result<(f32, usize)> {
        if self.fonts.is_none() {
            self.fonts = Some(Fonts::measurement(self.host.clone())?);
        }
        self.fonts
            .as_mut()
            .expect("initialized measurement owner")
            .measure(text, &crate::text_attrs(face), size, width)
    }
}

thread_local! {
    static CURRENT: RefCell<Option<Owner>> = const { RefCell::new(None) };
}

/// Native preparation supplies its existing staging budget; stand-alone CPU
/// scene clients retain an independent owner under the same process ceiling.
pub(crate) fn with_owner<T>(owner: &mut Option<Owner>, work: impl FnOnce() -> T) -> T {
    struct Restore<'a> {
        destination: &'a mut Option<Owner>,
        previous: Option<Owner>,
    }
    impl Drop for Restore<'_> {
        fn drop(&mut self) {
            CURRENT.with(|current| {
                *self.destination = current.replace(self.previous.take());
            });
        }
    }
    let previous = CURRENT.with(|current| current.replace(owner.take()));
    let _restore = Restore {
        destination: owner,
        previous,
    };
    work()
}

pub(super) fn measure(
    text: &str,
    size: f32,
    face: crate::TextFace,
    width: Option<f32>,
) -> anyhow::Result<(f32, usize)> {
    CURRENT.with(|current| {
        let mut current = current.borrow_mut();
        let owner = current.get_or_insert_with(|| Owner::new(Budget::new(16 * 1024 * 1024)));
        owner.measure(text, size, face, width)
    })
}

impl crate::Renderer {
    pub fn measurement_cpu_usage(&self) -> Option<crate::text_layout::fonts::Usage> {
        self.measurement_fonts.as_ref().and_then(Owner::usage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "visual")]
    #[test]
    #[ignore = "requires local GPU; shared-device native preparation adapters"]
    fn default_native_preparation_adapters_refuse_host_pressure_and_retry() {
        let instance = wgpu::Instance::default();
        let adapter = pollster::block_on(instance.request_adapter(&Default::default())).unwrap();
        let (device, queue) =
            pollster::block_on(adapter.request_device(&Default::default())).unwrap();
        for consumer in 0..4 {
            let mut renderer =
                crate::Renderer::new(&device, &queue, wgpu::TextureFormat::Rgba8UnormSrgb, 4)
                    .unwrap();
            let mut state = crate::global_preferences_dialog_tests::state_with_preferences_open();
            state.ui.project_preferences = state.ui.global_preferences.clone();
            state.ui.project_preferences.title = "Project Preferences".into();
            state.ui.global_preferences.open = consumer == 1;
            state.ui.project_preferences.open = consumer == 2;
            state.ui.new_project.open = consumer == 3;
            let retained = crate::RetainedScene::empty();
            let camera = crate::CameraState::fit_to_bounds(&state.scene.bounds);
            let mut scroll = datum_gui_viewport::scroll::ScrollViewport::default();
            super::super::MEASUREMENTS.with(|cache| *cache.borrow_mut() = Default::default());
            let host = renderer.atlas.staging_budget.clone();
            let pressure = host.reserve(host.available()).unwrap();
            let mut prepare = |renderer: &mut crate::Renderer| match consumer {
                0 => renderer.prepare_workspace_with_terminal_renderer(
                    &state,
                    960,
                    720,
                    1.0,
                    camera,
                    &retained,
                    &[],
                    None,
                    false,
                ),
                1 | 2 => renderer.prepare_native_preferences_scrolled(
                    if consumer == 1 {
                        &state.ui.global_preferences
                    } else {
                        &state.ui.project_preferences
                    },
                    960,
                    720,
                    1.0,
                    &mut scroll,
                    Some(0),
                ),
                _ => renderer.prepare_native_new_project_scrolled(
                    &state.ui.new_project,
                    960,
                    720,
                    1.0,
                    &mut scroll,
                    true,
                ),
            };
            assert!(prepare(&mut renderer).is_err(), "consumer {consumer}");
            drop(pressure);
            let prepared = prepare(&mut renderer).unwrap();
            assert!(!prepared.hit_regions.is_empty(), "consumer {consumer}");
            assert!(
                !renderer
                    .measurement_cpu_usage()
                    .unwrap()
                    .last_call
                    .unwrap()
                    .exceeded
            );
            renderer.release_text_scratch_for(host.available() + 1);
            assert_eq!(
                renderer
                    .measurement_cpu_usage()
                    .unwrap()
                    .cache_reserved_bytes,
                0
            );
            assert_eq!(prepare(&mut renderer).unwrap(), prepared);
            drop(renderer);
            assert_eq!(host.used(), 0);
        }
    }

    #[test]
    fn native_layout_refuses_cold_measurement_and_retries_without_publishing() {
        super::super::MEASUREMENTS.with(|cache| *cache.borrow_mut() = Default::default());
        let state = crate::global_preferences_dialog_tests::state_with_preferences_open();
        let host = Budget::new(16 * 1024 * 1024);
        let mut owner = Some(Owner::new(host.clone()));
        let held = host.reserve(host.available()).unwrap();
        let prepare = || {
            crate::PreparedScene::from_native_preferences(
                &state.ui.global_preferences,
                960,
                720,
                1.0,
            )
        };
        assert!(with_owner(&mut owner, prepare).is_err());
        assert!(owner.as_ref().unwrap().fonts.is_none());
        assert!(super::super::MEASUREMENTS.with(|cache| cache.borrow().misses) > 0);
        drop(held);
        let scene = with_owner(&mut owner, prepare).unwrap();
        assert!(!scene.hit_regions.is_empty());
        assert!(owner.as_ref().unwrap().usage().unwrap().last_call.is_some());
        drop(owner);
        assert_eq!(host.used(), 0);
    }
}
