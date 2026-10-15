//! Eligible members cannot turn a read-only Compound into a partial Move preview.
#[path = "../../../../test-harness/src/development_workspace.rs"]
mod development_workspace;
use super::*;
use clap::Parser;
use datum_gui_protocol::{
    AuthoredSelectionClass, SelectionSubject, SelectionTarget, WorkspaceTool,
};
use std::path::{Path, PathBuf};
use winit::platform::x11::EventLoopBuilderExtX11;
const A: &str = "ca340e78-04a5-5307-a024-9c5720a807d2";
const B: &str = "c270585d-8466-53f3-9c19-da87fb527602";
#[test]
#[ignore = "requires X11/Vulkan, scoped existing fixture and pinned CLI; run serially"]
#[allow(deprecated)] // Same hidden integration host as existing Runtime controls.
fn eligible_compound_move_does_not_preview_a_member_or_write_history() {
    assert!(
        std::env::var_os("EDA_CLI_BIN").is_some(),
        "pinned CLI required"
    );
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    safe_directory(&repo);
    let setup_path = development_workspace::path(
        "target/s5a-c04-inventory-keyboard-20261011/n13-compound-move-setup.json",
    );
    safe_file(&setup_path);
    let setup: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&setup_path).unwrap()).unwrap();
    let root = development_workspace::path(setup["root"].as_str().unwrap());
    assert_eq!(
        root,
        development_workspace::path("target/user-edit-selection/journal-read-project")
    );
    safe_directory(&root);
    for name in setup["absent_source_roles"].as_array().unwrap() {
        let name = name.as_str().unwrap();
        assert!(name.starts_with(".datum/") && !name.contains(".."));
        let path = root.join(name);
        safe_directory(path.parent().unwrap());
        assert!(
            matches!(std::fs::symlink_metadata(path), Err(e) if e.kind() == std::io::ErrorKind::NotFound),
            "unlisted canonical source role present"
        );
    }
    let allowed = [
        ".datum/import_map/kicad-board-b5685273-4801-5bf7-956f-ae8c3477c712.json",
        ".datum/journal/cursor.json",
        ".datum/journal/transactions.jsonl",
        ".datum/revision/v1/head.json",
        ".datum/zone_fills/0aa7c34d-bae9-55b9-b951-538ad0bb7107.json",
        ".datum/zone_fills/178dda32-54b1-5b59-bf07-7e25c1fdc361.json",
        ".datum/zone_fills/21e58b4e-c65b-56d8-9500-28cab6667b2f.json",
        ".datum/zone_fills/2e9e8834-cb56-5c73-be87-8f794244fb56.json",
        ".datum/zone_fills/3717c49f-c576-5c75-8fca-81a69200e390.json",
        ".datum/zone_fills/558040cb-94b0-5fcf-9382-3aa928dd320f.json",
        ".datum/zone_fills/56c27f1a-feeb-5da1-a7cb-6e35bf6936d3.json",
        ".datum/zone_fills/5de85200-465e-5359-8af3-2529d613b047.json",
        ".datum/zone_fills/7590676a-13d9-5fab-9d81-2305bb76a38e.json",
        ".datum/zone_fills/8533b53a-45f2-5e34-bcda-ccb494d223d3.json",
        ".datum/zone_fills/a6fd42e7-0cbf-501a-bbfc-928e99160365.json",
        ".datum/zone_fills/be222787-42da-5238-a22b-1895973bfe55.json",
        ".datum/zone_fills/bea45499-8f67-5f6d-8425-2837c0817422.json",
        ".datum/zone_fills/cd6be3fa-7413-50d1-a407-320a472cca10.json",
        ".datum/zone_fills/d009a7f1-b6ae-59c1-a364-87572fcd43ef.json",
        ".datum/zone_fills/df56c5b9-f5e2-5853-a6cc-d5fac3c49222.json",
        ".datum/zone_fills/e6882b3b-5d7f-5637-9469-48874af939df.json",
        ".datum/zone_fills/eb0a9908-33b6-53ae-8f31-a6984b193d99.json",
        "board/board.json",
        "project.json",
        "rules/rules.json",
        "schematic/schematic.json",
    ];
    let maps = setup["canonical_maps"].as_object().unwrap();
    assert_eq!(maps.len(), allowed.len());
    assert!(
        maps.keys().all(|rel| allowed.contains(&rel.as_str())),
        "map must match the exact reviewed source allowlist"
    );
    let canonical: Vec<_> = maps
        .keys()
        .map(|rel| {
            let path = root.join(rel);
            safe_file(&path);
            let digest = std::process::Command::new("sha256sum")
                .arg(&path)
                .output()
                .unwrap();
            assert!(digest.status.success(), "source digest failed");
            let actual = std::str::from_utf8(&digest.stdout)
                .unwrap()
                .split_whitespace()
                .next()
                .unwrap();
            assert!(
                actual == maps[rel].as_str().unwrap(),
                "fixture differs from frozen scoped source map"
            );
            (path.clone(), std::fs::read(path).unwrap())
        })
        .collect();
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
                    .with_inner_size(winit::dpi::PhysicalSize::new(1280, 800)),
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
    let mut runtime = pollster::block_on(Runtime::new(window, launch, Some(1.0), wake)).unwrap();
    runtime.terminal_accessibility = LinuxTerminalAccessibilityBridge::without_platform();
    unchanged(&canonical); // Loading must not silently establish a changed baseline.
    assert_eq!(
        runtime.workspace().scene.project_uuid,
        setup["project_id"].as_str().unwrap()
    );
    let revision = runtime.workspace().scene.source_revision.clone();
    let scene = runtime.workspace().scene.clone();
    let a = scene
        .components
        .iter()
        .find(|c| c.component_uuid == A)
        .unwrap()
        .clone();
    let b = scene
        .components
        .iter()
        .find(|c| c.component_uuid == B)
        .unwrap()
        .clone();
    assert!(!a.locked && !b.locked);
    assert_eq!(scene.pads.len(), 81);
    assert_eq!(scene.zones.len(), 18);
    // Current positive contrast: this exact eligible Object creates a preview,
    // motion materializes it, and Select cancels it without a commit.
    acquire(&mut runtime, &b.object_id, b.position, false);
    assert!(
        matches!(&runtime.workspace().selection, SelectionTarget::Native(v)
        if matches!(&v.subject, SelectionSubject::Object(id) if id.id.to_string() == B))
    );
    let object = runtime.workspace().selection.clone();
    let cursor = screen(&mut runtime, b.position);
    runtime.last_cursor_pos = Some(cursor);
    key(&mut runtime, "m");
    assert_eq!(runtime.workspace().tool, WorkspaceTool::Move);
    assert!(runtime.native_edit.preview.is_some());
    assert!(runtime.workspace().authoring.gesture.is_active());
    assert!(!runtime.native_edit.pending());
    let destination = datum_gui_protocol::PointNm {
        x: b.position.x + 1_000_000,
        y: b.position.y + 1_000_000,
    };
    let moved = screen(&mut runtime, destination);
    runtime.last_cursor_pos = Some(moved);
    assert!(runtime.handle_authoring_pointer_move(moved));
    runtime.prepare_native_move_frame();
    assert!(
        runtime.workspace().scene != scene,
        "eligible Object motion must materialize a different scene"
    );
    key(&mut runtime, "s");
    assert!(runtime.native_edit.preview.is_none());
    assert!(!runtime.workspace().authoring.gesture.is_active());
    assert!(
        runtime.workspace().scene == scene,
        "cancel must restore the complete source scene"
    );
    assert!(
        runtime.workspace().selection == object,
        "positive cancel must preserve Object identity"
    );
    unchanged(&canonical);
    // Actual typed acquisition, including a real focused eligible member.
    acquire(&mut runtime, &a.object_id, a.position, false);
    acquire(&mut runtime, &b.object_id, b.position, true);
    let whole = runtime.workspace().selection.clone();
    let SelectionTarget::Native(view) = &whole else {
        panic!("native Compound required")
    };
    let SelectionSubject::Compound(compound) = &view.subject else {
        panic!("Compound required")
    };
    assert_eq!(compound.members.len(), 2);
    assert!(
        compound
            .members
            .iter()
            .all(|id| id.class == AuthoredSelectionClass::Footprint
                && id.instance_path.is_empty()
                && [A, B].contains(&id.id.to_string().as_str()))
    );
    assert_eq!(compound.focus.as_ref().unwrap().id.to_string(), B);
    assert_eq!(view.members, compound.members);
    assert_eq!(view.scope, None);
    assert_eq!(view.revision.0, revision);
    assert!(!view.stale);
    let cursor = screen(&mut runtime, b.position);
    runtime.last_cursor_pos = Some(cursor);
    key(&mut runtime, "m");
    assert_eq!(runtime.workspace().tool, WorkspaceTool::Move);
    assert_eq!(
        runtime
            .workspace()
            .ui
            .console
            .records()
            .next_back()
            .unwrap()
            .message,
        "move tool active"
    );
    compound_unchanged(&runtime, &whole, &scene, &revision);
    // No primary click: idle Move clicks lawfully reacquire an Object.
    for world in [destination, a.position] {
        let cursor = screen(&mut runtime, world);
        runtime.last_cursor_pos = Some(cursor);
        assert!(!runtime.handle_authoring_pointer_move(cursor));
        runtime.prepare_native_move_frame();
        compound_unchanged(&runtime, &whole, &scene, &revision);
        unchanged(&canonical);
    }
    key(&mut runtime, "s");
    assert_eq!(runtime.workspace().tool, WorkspaceTool::Select);
    compound_unchanged(&runtime, &whole, &scene, &revision);
    unchanged(&canonical);
    println!(
        "N13 scoped current Project={} revision={} full Compound2/focusB; positive Object preview/cancel and negative M/motion/S verified",
        runtime.workspace().scene.project_uuid,
        revision
    );
    drop(runtime);
    unchanged(&canonical);
}
fn key(runtime: &mut Runtime, text: &str) {
    let action = crate::workspace_keyboard::character_action(text, false).unwrap();
    assert!(crate::workspace_keyboard::apply(runtime, action));
}
fn acquire(runtime: &mut Runtime, object: &str, world: datum_gui_protocol::PointNm, add: bool) {
    runtime.modifiers = if add {
        winit::keyboard::ModifiersState::SHIFT
    } else {
        winit::keyboard::ModifiersState::empty()
    };
    assert_eq!(
        runtime.acquire_native_key(object, crate::SceneSurface::Board, Some(world)),
        Some(true)
    );
    runtime.modifiers = winit::keyboard::ModifiersState::empty();
}
fn compound_unchanged(
    runtime: &Runtime,
    selection: &SelectionTarget,
    scene: &datum_gui_protocol::BoardReviewSceneV1,
    revision: &str,
) {
    assert!(
        runtime.native_edit.preview.is_none(),
        "Compound must not preview a member"
    );
    assert!(
        !runtime.native_edit.pending(),
        "Compound must not queue an edit"
    );
    assert!(
        !runtime.workspace().authoring.gesture.is_active(),
        "Compound must not start authoring"
    );
    assert!(
        runtime.workspace().selection == *selection,
        "complete selection and focus must survive"
    );
    assert!(
        runtime.workspace().scene == *scene,
        "complete scene must survive"
    );
    assert_eq!(runtime.workspace().scene.source_revision, revision);
    let SelectionTarget::Native(view) = selection else {
        panic!("native required")
    };
    let accessible = runtime.terminal_accessibility.selection.as_ref().unwrap();
    assert!(
        accessible.view.as_ref() == view.as_ref(),
        "current accessibility must preserve the complete typed view"
    );
    assert_eq!(accessible.pane, Some(runtime.workspace().ui.layout.focused));
    assert_eq!(accessible.evaluating, None);
}
fn screen(runtime: &mut Runtime, world: datum_gui_protocol::PointNm) -> (f32, f32) {
    let prepared = runtime.prepared_scene().unwrap();
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
fn safe_directory(path: &Path) {
    assert!(path.is_absolute());
    for ancestor in path.ancestors() {
        let meta = std::fs::symlink_metadata(ancestor).unwrap();
        assert!(
            !meta.file_type().is_symlink() && meta.is_dir(),
            "source ancestor must be a real directory"
        );
    }
}
fn safe_file(path: &Path) {
    safe_directory(path.parent().unwrap());
    let meta = std::fs::symlink_metadata(path).unwrap();
    assert!(
        !meta.file_type().is_symlink() && meta.is_file(),
        "source must be a regular nonsymlink file"
    );
}
fn unchanged(files: &[(PathBuf, Vec<u8>)]) {
    for (path, bytes) in files {
        safe_file(path);
        assert!(
            std::fs::read(path).unwrap() == *bytes,
            "scoped canonical/source/history bytes changed"
        );
    }
}
