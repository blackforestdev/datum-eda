//! Reuse shell geometry across pointer hit tests without coupling invalidation
//! to every runtime mutation. The complete solver input is the cache key.

use datum_gui_render::ShellLayout;
use std::cell::RefCell;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct LayoutKey {
    width: u32,
    height: u32,
    scale_bits: u32,
    dock_height: Option<u32>,
}

#[derive(Default)]
struct LayoutCache {
    entries: [Option<(LayoutKey, ShellLayout)>; 2],
    next: usize,
}

impl LayoutCache {
    fn resolve(&mut self, key: LayoutKey, solve: impl FnOnce() -> ShellLayout) -> ShellLayout {
        for (previous, layout) in self.entries.iter().flatten() {
            if *previous == key {
                return layout.clone();
            }
        }
        let layout = solve();
        self.entries[self.next] = Some((key, layout.clone()));
        self.next = (self.next + 1) % self.entries.len();
        layout
    }
}

thread_local! {
    // The solver is a pure function of the key: identical geometry can be shared
    // by runtimes on the UI thread. Two entries retain the visible shell and
    // terminal geometry (which also gets queried while the dock is closed).
    // Memory stays bounded during resize; no global lock or stale geometry.
    static CACHE: RefCell<LayoutCache> = RefCell::new(LayoutCache::default());
}

pub(super) fn for_surface(
    width: u32,
    height: u32,
    scale: f32,
    dock_height: Option<u32>,
) -> ShellLayout {
    let key = LayoutKey {
        width,
        height,
        scale_bits: scale.to_bits(),
        dock_height,
    };
    CACHE.with(|cache| {
        cache.borrow_mut().resolve(key, || {
            ShellLayout::for_surface(width, height, scale, dock_height)
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key() -> LayoutKey {
        LayoutKey {
            width: 1280,
            height: 800,
            scale_bits: 1.0_f32.to_bits(),
            dock_height: None,
        }
    }

    fn solve(key: LayoutKey) -> ShellLayout {
        ShellLayout::for_surface(
            key.width,
            key.height,
            f32::from_bits(key.scale_bits),
            key.dock_height,
        )
    }

    #[test]
    fn repeated_pointer_queries_solve_only_once() {
        let mut cache = LayoutCache::default();
        let key = key();
        let expected = solve(key);
        let mut solves = 0;
        for _ in 0..1000 {
            assert_eq!(
                cache.resolve(key, || {
                    solves += 1;
                    solve(key)
                }),
                expected
            );
        }
        assert_eq!(solves, 1);
    }

    #[test]
    fn every_geometry_input_selects_correct_layout() {
        let base = key();
        let variants = [
            LayoutKey {
                width: 1024,
                ..base
            },
            LayoutKey {
                height: 600,
                ..base
            },
            LayoutKey {
                scale_bits: 2.0_f32.to_bits(),
                ..base
            },
            LayoutKey {
                dock_height: Some(240),
                ..base
            },
            LayoutKey {
                dock_height: Some(360),
                ..base
            },
        ];
        for changed in variants {
            let mut cache = LayoutCache::default();
            let mut solves = 0;
            for next in [base, changed, changed, base] {
                let actual = cache.resolve(next, || {
                    solves += 1;
                    solve(next)
                });
                assert_eq!(actual, solve(next));
            }
            assert_eq!(solves, 2, "input {changed:?}");
        }
    }

    #[test]
    fn alternating_shell_and_terminal_queries_stay_warm_and_eviction_is_bounded() {
        let base = key();
        let terminal = LayoutKey {
            dock_height: Some(240),
            ..base
        };
        let resized = LayoutKey { width: 960, ..base };
        let mut cache = LayoutCache::default();
        let mut solves = 0;
        for next in [base, terminal]
            .into_iter()
            .cycle()
            .take(1000)
            .chain([resized, base])
        {
            assert_eq!(
                cache.resolve(next, || {
                    solves += 1;
                    solve(next)
                }),
                solve(next)
            );
        }
        assert_eq!(solves, 4);
    }

    #[test]
    fn public_cached_path_matches_uncached_geometry_across_scales_and_docks() {
        for (width, height) in [(1, 1), (800, 600), (1280, 800), (2560, 1440)] {
            for scale in [0.0, 1.0, 1.25, 1.5, 2.0] {
                for dock in [None, Some(0), Some(240), Some(1000)] {
                    for _ in 0..2 {
                        assert_eq!(
                            for_surface(width, height, scale, dock),
                            ShellLayout::for_surface(width, height, scale, dock)
                        );
                    }
                }
            }
        }
    }
}
