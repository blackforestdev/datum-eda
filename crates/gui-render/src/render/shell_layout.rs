//! Shared bounded shell solve ownership for frame preparation and input geometry.
use crate::{RectPx, ShellLayout, design_tokens};
use std::cell::RefCell;
use taffy::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct LayoutKey {
    width: u32,
    height: u32,
    scale_bits: u32,
    dock_height: Option<u32>,
}

#[path = "shell_layout/cache.rs"]
pub(crate) mod cache;
type LayoutCache = cache::LayoutCache<LayoutKey, ShellLayout>;

thread_local! {
    // The solver is a pure function of the key: identical geometry can be shared
    // by runtimes on the UI thread. Two entries retain the visible shell and
    // terminal geometry (which also gets queried while the dock is closed).
    // Memory stays bounded during resize; no global lock or stale geometry.
    static CACHE: RefCell<LayoutCache> = RefCell::new(LayoutCache::default());
}

impl ShellLayout {
    pub fn for_surface(
        physical_width: u32,
        physical_height: u32,
        scale_factor: f32,
        dock_height_px: Option<u32>,
    ) -> Self {
        let key = LayoutKey {
            width: physical_width,
            height: physical_height,
            scale_bits: scale_factor.to_bits(),
            dock_height: dock_height_px,
        };
        CACHE.with(|cache| {
            cache
                .borrow_mut()
                .resolve(key, || Self::uncached_surface(key))
        })
    }

    fn uncached_surface(key: LayoutKey) -> Self {
        record_solve();
        let scale = f32::from_bits(key.scale_bits).max(0.01);
        let logical_width = ((key.width as f32) / scale).round().max(1.0) as u32;
        let logical_height = ((key.height as f32) / scale).round().max(1.0) as u32;
        Self::for_window(logical_width, logical_height, key.dock_height).scale_by(scale)
    }

    pub fn for_window(width: u32, height: u32, dock_height_px: Option<u32>) -> Self {
        let width = width as f32;
        let height = height as f32;
        let menu_height = design_tokens::spacing::SP_07 + 1.0;
        let status_height = design_tokens::spacing::SP_06 + design_tokens::spacing::SP_01;
        let left_width = 224.0_f32.min(width * 0.3);
        let right_width = 296.0_f32.min(width * 0.35);
        let bottom_height = match dock_height_px {
            Some(h) => (h as f32).clamp(
                design_tokens::spacing::SP_07,
                (height - menu_height - status_height).max(design_tokens::spacing::SP_07),
            ),
            None => design_tokens::spacing::SP_07.min(height * 0.25),
        };
        if let Some(layout) = solve_shell_layout_with_taffy(
            width,
            height,
            menu_height,
            left_width,
            right_width,
            bottom_height,
            status_height,
        ) {
            return layout;
        }
        // Taffy is the adopted shell solver; keep a manual fallback so a
        // malformed runtime input cannot prevent the GUI from opening.
        Self {
            top_menu_bar: RectPx {
                x: 0.0,
                y: 0.0,
                width,
                height: menu_height,
            },
            left_sidebar: RectPx {
                x: 0.0,
                y: menu_height,
                width: left_width,
                height: (height - menu_height - bottom_height - status_height).max(0.0),
            },
            viewport: RectPx {
                x: left_width,
                y: menu_height,
                width: (width - left_width - right_width).max(0.0),
                height: (height - menu_height - bottom_height - status_height).max(0.0),
            },
            right_sidebar: RectPx {
                x: (width - right_width).max(0.0),
                y: menu_height,
                width: right_width,
                height: (height - menu_height - bottom_height - status_height).max(0.0),
            },
            bottom_strip: RectPx {
                x: 0.0,
                y: height - bottom_height - status_height,
                width,
                height: bottom_height,
            },
            status_bar: RectPx {
                x: 0.0,
                y: height - status_height,
                width,
                height: status_height,
            },
        }
    }
}

fn solve_shell_layout_with_taffy(
    width: f32,
    height: f32,
    menu_height: f32,
    left_width: f32,
    right_width: f32,
    bottom_height: f32,
    status_height: f32,
) -> Option<ShellLayout> {
    let mut taffy: TaffyTree<()> = TaffyTree::new();
    let top_menu_bar = taffy
        .new_leaf(Style {
            grid_row: line(1),
            grid_column: span(3),
            ..Default::default()
        })
        .ok()?;
    let left_sidebar = taffy
        .new_leaf(Style {
            grid_row: line(2),
            grid_column: line(1),
            ..Default::default()
        })
        .ok()?;
    let viewport = taffy
        .new_leaf(Style {
            grid_row: line(2),
            grid_column: line(2),
            ..Default::default()
        })
        .ok()?;
    let right_sidebar = taffy
        .new_leaf(Style {
            grid_row: line(2),
            grid_column: line(3),
            ..Default::default()
        })
        .ok()?;
    let bottom_strip = taffy
        .new_leaf(Style {
            grid_row: line(3),
            grid_column: span(3),
            ..Default::default()
        })
        .ok()?;
    let status_bar = taffy
        .new_leaf(Style {
            grid_row: line(4),
            grid_column: span(3),
            ..Default::default()
        })
        .ok()?;
    let root = taffy
        .new_with_children(
            Style {
                display: Display::Grid,
                size: Size {
                    width: length(width),
                    height: length(height),
                },
                grid_template_columns: vec![length(left_width), fr(1.0), length(right_width)],
                grid_template_rows: vec![
                    length(menu_height),
                    fr(1.0),
                    length(bottom_height),
                    length(status_height),
                ],
                ..Default::default()
            },
            &[
                top_menu_bar,
                left_sidebar,
                viewport,
                right_sidebar,
                bottom_strip,
                status_bar,
            ],
        )
        .ok()?;
    taffy
        .compute_layout(
            root,
            Size {
                width: AvailableSpace::Definite(width),
                height: AvailableSpace::Definite(height),
            },
        )
        .ok()?;

    let rect_for = |tree: &TaffyTree<()>, node| -> Option<RectPx> {
        let layout = tree.layout(node).ok()?;
        Some(RectPx {
            x: layout.location.x,
            y: layout.location.y,
            width: layout.size.width,
            height: layout.size.height,
        })
    };
    Some(ShellLayout {
        top_menu_bar: rect_for(&taffy, top_menu_bar)?,
        left_sidebar: rect_for(&taffy, left_sidebar)?,
        viewport: rect_for(&taffy, viewport)?,
        right_sidebar: rect_for(&taffy, right_sidebar)?,
        bottom_strip: rect_for(&taffy, bottom_strip)?,
        status_bar: rect_for(&taffy, status_bar)?,
    })
}

// Optimizes away outside test builds; counts misses through actual consumers.
#[inline]
fn record_solve() {
    #[cfg(test)]
    tests::SOLVES.with(|count| count.set(count.get() + 1));
}

#[cfg(test)]
mod tests {
    use super::*;
    thread_local! { pub(super) static SOLVES: std::cell::Cell<usize> = const { std::cell::Cell::new(0) }; }

    fn key() -> LayoutKey {
        LayoutKey {
            width: 1280,
            height: 800,
            scale_bits: 1.0_f32.to_bits(),
            dock_height: None,
        }
    }

    fn solve(key: LayoutKey) -> ShellLayout {
        ShellLayout::uncached_surface(key)
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
                            ShellLayout::for_surface(width, height, scale, dock),
                            ShellLayout::uncached_surface(LayoutKey {
                                width,
                                height,
                                scale_bits: scale.to_bits(),
                                dock_height: dock
                            })
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn retained_preparation_frame_preparation_and_input_queries_share_one_solve() {
        let state = datum_gui_protocol::load_fixture_workspace_state();
        CACHE.with(|cache| *cache.borrow_mut() = LayoutCache::default());
        SOLVES.with(|count| count.set(0));
        let retained = crate::RetainedScene::from_workspace(&state, 1280, 800);
        assert_eq!(SOLVES.with(|count| count.get()), 1);
        let camera = crate::CameraState::fit_to_bounds(&state.scene.bounds);
        for _ in 0..3 {
            let prepared =
                crate::PreparedScene::from_workspace(&state, 1280, 800, camera, &retained);
            let input =
                ShellLayout::for_surface(1280, 800, 1.0, crate::dock_height_for_state(&state));
            assert_eq!(prepared.layout, input);
        }
        assert_eq!(
            SOLVES.with(|count| count.get()),
            1,
            "paint and input must not maintain competing shell solves"
        );
    }
}
