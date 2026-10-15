//! Native release uses the presented menu surface, never underlying acquisition.
#[path = "../../../../test-harness/src/development_workspace.rs"]
mod development_workspace;
use super::*;
use clap::Parser;
use datum_gui_protocol::{PointNm, SelectionSubject, SelectionTarget};
use datum_gui_render::{HitRegion, HitTarget};
use std::path::{Path, PathBuf};
use winit::{event::ElementState, platform::x11::EventLoopBuilderExtX11};
const T1: &str = "40000000-0000-4000-8000-000000000001";
#[test]
#[ignore = "requires X11/Vulkan, existing scoped fixture and pinned CLI; run serially"]
#[allow(deprecated)] // Existing hidden Runtime integration host.
fn native_outside_right_release_dismisses_without_acquisition_and_preserves_inside_menu() {
    assert!(
        std::env::var_os("EDA_CLI_BIN").is_some(),
        "pinned CLI required"
    );
    let setup_path = development_workspace::path(
        "target/s5a-c04-inventory-keyboard-20261011/n14-two-project-setup.json",
    );
    safe_file(&setup_path);
    digest(
        &setup_path,
        "784b724650b158c1de19426b209bcac0b5a653fbaf8217567f174e283f6c56f6",
    );
    let setup: serde_json::Value =
        serde_json::from_slice(&std::fs::read(setup_path).unwrap()).unwrap();
    let root = development_workspace::path("target/s5a-c03-u5-scope-v2");
    safe_dir(&root);
    let mut canonical = Vec::new();
    for key in ["A", "B"] {
        let path = development_workspace::path(setup["projects"][key]["path"].as_str().unwrap());
        assert_eq!(
            path,
            if key == "A" {
                root.clone()
            } else {
                root.join("variants/second-project")
            }
        );
        for (rel, expected) in setup["canonical_maps"][key].as_object().unwrap() {
            assert!(!Path::new(rel).is_absolute() && !rel.split('/').any(|s| s == ".."));
            assert!(
                !["tool-sessions", "terminal", "credential", "writer.lock"]
                    .iter()
                    .any(|s| rel.contains(s))
            );
            let path = path.join(rel);
            safe_file(&path);
            digest(&path, expected.as_str().unwrap());
            canonical.push((path.clone(), std::fs::read(path).unwrap()));
        }
    }
    assert_eq!(canonical.len(), 69);
    let event_loop = winit::event_loop::EventLoop::<()>::with_user_event()
        .with_x11()
        .with_any_thread(true)
        .build()
        .unwrap();
    let wake = event_loop.create_proxy();
    let window = Arc::new(
        event_loop
            .create_window(
                winit::window::Window::default_attributes()
                    .with_visible(false)
                    .with_inner_size(winit::dpi::PhysicalSize::new(1333, 800)),
            )
            .unwrap(),
    );
    let args = crate::GuiArgs::try_parse_from([
        "datum-gui",
        "--project-root",
        root.to_str().unwrap(),
        "--focus-pane",
        "board",
    ])
    .unwrap();
    let launch = args.load_launch_state(Some(wake.clone())).unwrap();
    let mut r = pollster::block_on(Runtime::new(window, launch, Some(1.0), wake)).unwrap();
    r.terminal_accessibility = LinuxTerminalAccessibilityBridge::without_platform();
    unchanged(&canonical);
    assert_eq!(
        r.workspace().scene.project_uuid,
        setup["projects"]["A"]["project_id"].as_str().unwrap()
    );
    assert_eq!(
        r.workspace().scene.source_revision,
        setup["projects"]["A"]["revision"].as_str().unwrap()
    );
    for text in ["z", "f"] {
        assert!(crate::workspace_keyboard::apply(
            &mut r,
            crate::workspace_keyboard::character_action(text, false).unwrap()
        ));
    }
    if !r
        .workspace()
        .ui
        .filters
        .layer_visibility
        .get("L1")
        .copied()
        .unwrap_or(true)
    {
        assert!(r.select_hit_target(&HitTarget::ToggleLayer("L1".into())));
    }
    present(&mut r);
    let object = r
        .workspace()
        .scene
        .tracks
        .iter()
        .find(|t| t.object_id.contains(T1))
        .unwrap()
        .object_id
        .clone();
    assert_eq!(
        r.acquire_native_key(
            &object,
            crate::SceneSurface::Board,
            Some(PointNm { x: 5_000_000, y: 0 })
        ),
        Some(true)
    );
    let whole = r.workspace().selection.clone();
    let SelectionTarget::Native(view) = &whole else {
        panic!("native Object required")
    };
    assert!(matches!(&view.subject, SelectionSubject::Object(id) if id.id.to_string() == T1));
    let scene = r.workspace().scene.clone();
    let camera = r.camera;
    present(&mut r);
    let anchor = screen(
        &mut r,
        PointNm {
            x: 11_000_000,
            y: 0,
        },
    );
    // These positives consume real renderer-generated, successfully presented hits.
    open(&mut r, anchor);
    release(&mut r, anchor, true);
    r.dismiss_marking_menu();
    preserved(&r, &whole, &scene, camera);
    open(&mut r, anchor);
    let card = region(
        &r,
        |t| matches!(t, HitTarget::MarkingMenuItem { slot, .. } if slot == "SELECT"),
    );
    let card_point = center(&card);
    release(&mut r, card_point, true);
    r.dismiss_marking_menu();
    preserved(&r, &whole, &scene, camera);
    // Clamping makes the painted center distinct from the original stationary deadzone.
    let viewport = r.prepared_scene().unwrap().scene_viewport;
    let edge_anchor = (viewport.x + 20.0, anchor.1);
    open(&mut r, edge_anchor);
    let surface = region(&r, |t| matches!(t, HitTarget::MarkingMenuSurface { .. }));
    let center_point = center(&surface);
    let menu = r.workspace().ui.marking_menu.as_ref().unwrap();
    assert!(
        crate::runtime_geometry_helpers::marking_slot_for_delta(
            (center_point.0 - menu.anchor_x_px as f32).round() as i32,
            (center_point.1 - menu.anchor_y_px as f32).round() as i32
        )
        .is_some()
    );
    release(&mut r, center_point, true);
    r.dismiss_marking_menu();
    preserved(&r, &whole, &scene, camera);
    for location in ["header", "gutter", "item"] {
        open(&mut r, anchor);
        let select = region(
            &r,
            |t| matches!(t, HitTarget::MarkingMenuItem { slot, .. } if slot == "SELECT"),
        );
        assert!(r.select_hit_target(&select.target));
        present(&mut r);
        let panel = region(&r, |t| matches!(t, HitTarget::MarkingMenuSurface { .. }));
        let point = match location {
            "header" => (panel.rect.x + 1.0, panel.rect.y + 10.0),
            "gutter" => (panel.rect.x + 1.0, panel.rect.y + panel.rect.height * 0.5),
            _ => center(&region(
                &r,
                |t| matches!(t, HitTarget::MarkingMenuItem { slot, .. } if slot.starts_with("SELECT:")),
            )),
        };
        if location == "item" {
            assert!(matches!(
                r.presented_hits.hit_test(point.0, point.1),
                Some(HitTarget::MarkingMenuItem { .. })
            ));
        } else {
            assert!(matches!(
                r.presented_hits.hit_test(point.0, point.1),
                Some(HitTarget::MarkingMenuSurface { .. })
            ));
        }
        assert!(
            !r.select_hit_target(&panel.target),
            "passive surface must not activate any action"
        );
        assert!(r.workspace().selection == whole);
        release(&mut r, point, true);
        r.dismiss_marking_menu();
        preserved(&r, &whole, &scene, camera);
    }
    // Controlled negative tags are explicit injections, not renderer/presentation evidence.
    for invalid in ["missing", "foreign", "stale-anchor"] {
        open(&mut r, anchor);
        let item = region(
            &r,
            |t| matches!(t, HitTarget::MarkingMenuItem { slot, .. } if slot == "SELECT"),
        );
        let mut hits = r.presented_hits.regions().to_vec();
        if invalid == "missing" {
            hits.retain(|h| !matches!(h.target, HitTarget::MarkingMenuSurface { .. }));
        } else {
            for hit in &mut hits {
                if let HitTarget::MarkingMenuSurface {
                    menu_key,
                    anchor_x_px,
                    ..
                } = &mut hit.target
                {
                    if invalid == "foreign" {
                        *menu_key = "foreign-menu".into();
                    } else {
                        *anchor_x_px += 1;
                    }
                }
            }
        }
        r.presented_hits.replace(hits);
        release(&mut r, center(&item), false);
        preserved(&r, &whole, &scene, camera);
    }
    open(&mut r, anchor);
    let outside = screen(
        &mut r,
        PointNm {
            x: 45_000_000,
            y: 0,
        },
    );
    let prepared = r.prepared_scene().unwrap();
    let (inverse, surface) = prepared
        .world_point_at_screen(outside.0, outside.1)
        .unwrap();
    assert_eq!(surface, crate::SceneSurface::Board);
    let viewport = prepared.scene_viewport;
    assert!(
        outside.0 > viewport.x + 24.0
            && outside.0 < viewport.x + viewport.width - 24.0
            && outside.1 > viewport.y + 24.0
            && outside.1 < viewport.y + viewport.height - 24.0
    );
    // Independent canonical T3 capsule, with other three Tracks well outside this probe.
    let board_bytes = &canonical
        .iter()
        .find(|(path, _)| path == &root.join("board/board.json"))
        .unwrap()
        .1;
    let board: serde_json::Value = serde_json::from_slice(board_bytes).unwrap();
    let t3 = &board["tracks"]["40000000-0000-4000-8000-000000000003"];
    assert_eq!(t3["from"], serde_json::json!({"x":40_000_000,"y":0}));
    assert_eq!(t3["to"], serde_json::json!({"x":50_000_000,"y":0}));
    assert_eq!(t3["width"], 5_080_000);
    assert!(inverse.x > 40_000_000 && inverse.x < 50_000_000 && inverse.y.abs() < 2_540_000);
    assert!(r.presented_hits.regions().iter().all(|h| !matches!(
        h.target,
        HitTarget::MarkingMenuItem { .. } | HitTarget::MarkingMenuSurface { .. }
    ) || !h.rect.contains(outside.0, outside.1)));
    release(&mut r, outside, false);
    preserved(&r, &whole, &scene, camera);
    unchanged(&canonical);
    drop(r);
    unchanged(&canonical);
}
#[track_caller]
fn present(r: &mut Runtime) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    loop {
        assert!(!r.device_health.failed(), "native device health failed");
        assert!(
            !r.surface_transaction.recovery.borrow().failed(),
            "native surface recovery failed"
        );
        if r.render().unwrap() {
            return;
        }
        assert!(!r.device_health.failed(), "native device health failed");
        assert!(
            !r.surface_transaction.recovery.borrow().failed(),
            "native surface recovery failed"
        );
        assert!(
            std::time::Instant::now() < deadline,
            "real Runtime frame must present"
        );
        r.device.poll(wgpu::PollType::Poll).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}
fn open(r: &mut Runtime, point: (f32, f32)) {
    r.last_cursor_pos = Some(point);
    assert!(r.handle_context_menu_button(ElementState::Pressed));
    present(r);
    assert!(
        r.workspace()
            .ui
            .marking_menu
            .as_ref()
            .unwrap()
            .selection
            .is_some()
    );
}
fn release(r: &mut Runtime, point: (f32, f32), stays: bool) {
    r.last_cursor_pos = Some(point);
    r.update_marking_menu_preview(point);
    assert!(r.handle_context_menu_button(ElementState::Released));
    assert_eq!(r.workspace().ui.marking_menu.is_some(), stays);
    assert_eq!(
        r.application_focus(),
        if stays {
            datum_gui_protocol::ApplicationFocus::Overlay
        } else {
            datum_gui_protocol::ApplicationFocus::Editor(r.workspace().ui.layout.focused)
        }
    );
    present(r);
}
fn region(r: &Runtime, predicate: impl Fn(&HitTarget) -> bool) -> HitRegion {
    r.presented_hits
        .regions()
        .iter()
        .find(|h| predicate(&h.target))
        .unwrap()
        .clone()
}
fn center(h: &HitRegion) -> (f32, f32) {
    (
        h.rect.x + h.rect.width * 0.5,
        h.rect.y + h.rect.height * 0.5,
    )
}
fn preserved(
    r: &Runtime,
    whole: &SelectionTarget,
    scene: &datum_gui_protocol::BoardReviewSceneV1,
    camera: datum_gui_render::CameraState,
) {
    assert!(
        r.workspace().selection == *whole,
        "complete prior Object and focus must survive"
    );
    assert!(
        r.workspace().scene == *scene,
        "complete authored scene must survive"
    );
    assert_eq!(r.camera, camera);
    let SelectionTarget::Native(view) = whole else {
        panic!("native required")
    };
    let at = r.terminal_accessibility.selection.as_ref().unwrap();
    assert!(
        at.view.as_ref() == view.as_ref(),
        "current accessibility must preserve typed view"
    );
    assert_eq!(at.pane, Some(r.workspace().ui.layout.focused));
    assert_eq!(at.evaluating, None);
}
fn screen(r: &mut Runtime, world: PointNm) -> (f32, f32) {
    let prepared = r.prepared_scene().unwrap();
    let viewport = prepared.scene_viewport;
    let origin = (viewport.x + 20.0, viewport.y + 20.0);
    let a = prepared
        .world_point_at_screen(origin.0, origin.1)
        .unwrap()
        .0;
    let b = prepared
        .world_point_at_screen(origin.0 + 100.0, origin.1 + 100.0)
        .unwrap()
        .0;
    (
        origin.0 + (world.x - a.x) as f32 * 100.0 / (b.x - a.x) as f32,
        origin.1 + (world.y - a.y) as f32 * 100.0 / (b.y - a.y) as f32,
    )
}
fn safe_dir(path: &Path) {
    assert!(path.is_absolute());
    for ancestor in path.ancestors() {
        let meta = std::fs::symlink_metadata(ancestor).unwrap();
        assert!(
            meta.is_dir() && !meta.file_type().is_symlink(),
            "real source ancestor required"
        );
    }
}
fn safe_file(path: &Path) {
    safe_dir(path.parent().unwrap());
    let meta = std::fs::symlink_metadata(path).unwrap();
    assert!(
        meta.is_file() && !meta.file_type().is_symlink(),
        "regular source file required"
    );
}
fn digest(path: &Path, expected: &str) {
    let value = std::process::Command::new("sha256sum")
        .arg(path)
        .output()
        .unwrap();
    assert!(value.status.success(), "source digest failed");
    let actual = std::str::from_utf8(&value.stdout)
        .unwrap()
        .split_whitespace()
        .next()
        .unwrap();
    assert!(actual == expected, "source differs from frozen allowlist");
}
fn unchanged(canonical: &[(PathBuf, Vec<u8>)]) {
    for (path, bytes) in canonical {
        safe_file(path);
        assert!(
            std::fs::read(path).unwrap() == *bytes,
            "canonical source and history must remain unchanged"
        );
    }
}
