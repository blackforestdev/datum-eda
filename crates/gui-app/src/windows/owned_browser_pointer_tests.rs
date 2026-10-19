//! Published browser rows/blank space cannot target a newer parent or listing.
use super::*;
use winit::platform::wayland::EventLoopBuilderExtWayland;

fn read(runtime: &mut Runtime, path: &std::path::Path) {
    let state = runtime.folder_browser.state.as_mut().unwrap();
    state.expanded = true;
    state.path_input = path.display().to_string();
    runtime.activate_folder_browser_hit(&HitTarget::FolderBrowser(
        datum_gui_protocol::FolderBrowserFocus::Browse,
    ));
    let deadline = Instant::now() + Duration::from_secs(5);
    while !runtime.folder_browser.state.as_ref().unwrap().ready() {
        runtime.poll_folder_browser();
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(5));
    }
}
fn right() -> WindowEvent {
    WindowEvent::MouseInput {
        device_id: winit::event::DeviceId::dummy(),
        state: ElementState::Pressed,
        button: MouseButton::Right,
    }
}
#[test]
#[ignore = "requires owned Wayland/Vulkan and isolated config; run serially"]
#[allow(deprecated)]
fn folder_actions_old_presented_a_cannot_target_new_b_or_blank_parent() {
    let args = GuiArgs::try_parse_from(["datum-gui", "--open-new-project"]).unwrap();
    let mut events = winit::event_loop::EventLoop::<()>::with_user_event()
        .with_wayland()
        .with_any_thread(true)
        .build()
        .unwrap();
    let wake = events.create_proxy();
    let main = Arc::new(
        events
            .create_window(
                Window::default_attributes()
                    .with_visible(false)
                    .with_inner_size(winit::dpi::PhysicalSize::new(1280, 800)),
            )
            .unwrap(),
    );
    let launch = args.load_launch_state(Some(wake.clone())).unwrap();
    let mut runtime =
        pollster::block_on(Runtime::new(main.clone(), launch, Some(1.0), wake.clone())).unwrap();
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/qa")
        .join(format!(
            "browser-presented-{}",
            eda_engine::preferences::new_product_id()
        ));
    let old = root.join("old");
    let new = root.join("new");
    for (parent, name) in [(&old, "A.txt"), (&new, "B.txt")] {
        std::fs::create_dir_all(parent).unwrap();
        std::fs::write(parent.join(name), name).unwrap();
    }
    for index in 0..40 {
        std::fs::write(new.join(format!("item-{index:03}.txt")), b"scroll entry").unwrap();
    }
    read(&mut runtime, &old);
    let window = Arc::new(
        events
            .create_window(
                Window::default_attributes()
                    .with_visible(true)
                    .with_inner_size(winit::dpi::PhysicalSize::new(760, 720)),
            )
            .unwrap(),
    );
    let owned = GlobalPreferencesWindowSurface::new(&runtime, window.clone(), Some(1.0)).unwrap();
    let mut app = App::new(args, wake);
    app.window = Some(main);
    app.runtime = Some(runtime);
    app.frames.register(
        window.id(),
        owned.measurements.epoch(),
        window.inner_size(),
        owned.surface_transaction.recovery.clone(),
    );
    app.new_project_window = Some(window);
    app.new_project_surface = Some(owned);
    present_changed_content(&mut events, &mut app, 2);
    let row = surface(&app, 2)
        .presented_hits
        .regions()
        .iter()
        .find(|r| r.target == HitTarget::FolderBrowserRow(0))
        .unwrap()
        .rect;
    let list = surface(&app, 2)
        .presented_hits
        .regions()
        .iter()
        .find(|r| {
            r.target == HitTarget::FolderBrowser(datum_gui_protocol::FolderBrowserFocus::Folders)
        })
        .unwrap()
        .rect;
    assert!(surface(&app, 2).browser_hits_current(app.runtime.as_ref().unwrap()));
    // Complete the real scanner without pumping native redraw: A is still
    // presented while the current same-index row and parent now identify B.
    read(app.runtime.as_mut().unwrap(), &new);
    assert!(!surface(&app, 2).browser_hits_current(app.runtime.as_ref().unwrap()));
    assert_eq!(
        app.runtime
            .as_ref()
            .unwrap()
            .folder_browser
            .state
            .as_ref()
            .unwrap()
            .entries[0]
            .name,
        "B.txt"
    );
    for (x, y) in [
        (row.x + 10.0, row.y + 10.0),
        (list.x + 10.0, list.y + list.height - 10.0),
    ] {
        // Exercise the exact production dispatcher without giving a queued
        // redraw the opportunity to replace the old pixels before the click.
        app.handle_owned_dialog_pointer(OwnedHost::New, &motion(x, y));
        app.handle_owned_dialog_pointer(OwnedHost::New, &right());
        let state = app
            .runtime
            .as_ref()
            .unwrap()
            .folder_browser
            .state
            .as_ref()
            .unwrap();
        assert!(
            state.menu.is_none() && state.naming.is_none(),
            "old row or blank-space pixels must not authorize B/new parent"
        );
    }
    app.handle_owned_dialog_pointer(OwnedHost::New, &motion(row.x + 10.0, row.y + 10.0));
    app.handle_owned_dialog_pointer(OwnedHost::New, &primary(ElementState::Pressed));
    app.handle_owned_dialog_pointer(OwnedHost::New, &primary(ElementState::Released));
    assert!(
        app.runtime
            .as_ref()
            .unwrap()
            .folder_browser
            .state
            .as_ref()
            .unwrap()
            .selected
            .is_none()
    );
    // A subsequent matching presentation admits the same ordinary pointer route.
    app.new_project_surface.as_mut().unwrap().invalidate();
    present_changed_content(&mut events, &mut app, 2);
    assert!(surface(&app, 2).browser_hits_current(app.runtime.as_ref().unwrap()));
    let row = surface(&app, 2)
        .presented_hits
        .regions()
        .iter()
        .find(|r| r.target == HitTarget::FolderBrowserRow(0))
        .unwrap()
        .rect;
    app.handle_owned_dialog_pointer(OwnedHost::New, &motion(row.x + 10.0, row.y + 10.0));
    app.handle_owned_dialog_pointer(OwnedHost::New, &right());
    assert_eq!(
        app.runtime
            .as_ref()
            .unwrap()
            .folder_browser
            .state
            .as_ref()
            .unwrap()
            .context_entry
            .as_ref()
            .unwrap()
            .name,
        "B.txt"
    );
    for outside in [
        datum_gui_protocol::FolderBrowserFocus::Folders,
        datum_gui_protocol::FolderBrowserFocus::Path,
    ] {
        if app
            .runtime
            .as_ref()
            .unwrap()
            .folder_browser
            .state
            .as_ref()
            .unwrap()
            .menu
            .is_none()
        {
            app.handle_owned_dialog_pointer(OwnedHost::New, &motion(row.x + 10.0, row.y + 10.0));
            app.handle_owned_dialog_pointer(OwnedHost::New, &right());
        }
        let deadline = Instant::now() + Duration::from_secs(5);
        while !app
            .runtime
            .as_ref()
            .unwrap()
            .folder_browser
            .state
            .as_ref()
            .unwrap()
            .menu
            .as_ref()
            .unwrap()
            .items[0]
            .unavailable
            .is_none()
        {
            app.runtime.as_mut().unwrap().poll_folder_browser();
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(5));
        }
        app.new_project_surface.as_mut().unwrap().invalidate();
        present_changed_content(&mut events, &mut app, 2);
        let rect = surface(&app, 2)
            .presented_hits
            .regions()
            .iter()
            .find(|r| r.target == HitTarget::FolderBrowser(outside))
            .unwrap()
            .rect;
        let point = if outside == datum_gui_protocol::FolderBrowserFocus::Folders {
            (rect.x + rect.width - 20.0, rect.y + rect.height - 10.0)
        } else {
            (rect.x + 10.0, rect.y + 10.0)
        };
        app.handle_owned_dialog_pointer(OwnedHost::New, &motion(point.0, point.1));
        app.handle_owned_dialog_pointer(OwnedHost::New, &primary(ElementState::Pressed));
        // No intervening presentation: release still sees the old menu pixels.
        app.handle_owned_dialog_pointer(OwnedHost::New, &primary(ElementState::Released));
        assert!(
            app.runtime
                .as_ref()
                .unwrap()
                .folder_browser
                .state
                .as_ref()
                .unwrap()
                .menu
                .is_none()
        );
        app.new_project_surface.as_mut().unwrap().invalidate();
        present_changed_content(&mut events, &mut app, 2);
        let fresh = surface(&app, 2)
            .presented_hits
            .regions()
            .iter()
            .find(|r| r.target == HitTarget::FolderBrowserRow(1))
            .unwrap()
            .rect;
        let before = app
            .runtime
            .as_ref()
            .unwrap()
            .folder_browser
            .state
            .as_ref()
            .unwrap()
            .selected;
        assert_ne!(before, Some(1));
        app.handle_owned_dialog_pointer(OwnedHost::New, &motion(fresh.x + 10.0, fresh.y + 10.0));
        app.handle_owned_dialog_pointer(OwnedHost::New, &primary(ElementState::Pressed));
        assert_eq!(
            app.runtime
                .as_ref()
                .unwrap()
                .folder_browser
                .state
                .as_ref()
                .unwrap()
                .selected,
            before,
            "row selection must not take effect on press"
        );
        app.handle_owned_dialog_pointer(OwnedHost::New, &primary(ElementState::Released));
        assert_eq!(
            app.runtime
                .as_ref()
                .unwrap()
                .folder_browser
                .state
                .as_ref()
                .unwrap()
                .selected,
            Some(1),
            "first fresh release after popup dismissal must select the different visible row"
        );
        let path = surface(&app, 2)
            .presented_hits
            .regions()
            .iter()
            .find(|r| {
                r.target == HitTarget::FolderBrowser(datum_gui_protocol::FolderBrowserFocus::Path)
            })
            .unwrap()
            .rect;
        app.handle_owned_dialog_pointer(OwnedHost::New, &motion(path.x + 10.0, path.y + 10.0));
        app.handle_owned_dialog_pointer(OwnedHost::New, &primary(ElementState::Pressed));
        app.handle_owned_dialog_pointer(OwnedHost::New, &primary(ElementState::Released));
        assert_eq!(
            app.runtime.as_ref().unwrap().folder_navigation_focus(),
            Some(datum_gui_protocol::FolderBrowserFocus::Path),
            "fresh click after dismissal must work immediately"
        );
    }
    let thumb = surface(&app, 2)
        .navigation_scroll
        .thumb()
        .expect("real overflowing file list");
    app.handle_owned_dialog_pointer(OwnedHost::New, &motion(thumb.x + 1.0, thumb.y + 3.0));
    app.handle_owned_dialog_pointer(OwnedHost::New, &primary(ElementState::Pressed));
    assert!(surface(&app, 2).scrollbar_pressed);
    app.handle_owned_dialog_pointer(OwnedHost::New, &motion(thumb.x + 1.0, thumb.y + 100.0));
    assert!(surface(&app, 2).navigation_scroll.offset() > 0.0);
    // The current pointer now hits another old row. Its release still belongs
    // to the existing scrollbar gesture and cannot remain captured or click it.
    app.handle_owned_dialog_pointer(OwnedHost::New, &primary(ElementState::Released));
    assert!(!surface(&app, 2).scrollbar_pressed);
    let offset = surface(&app, 2).navigation_scroll.offset();
    app.handle_owned_dialog_pointer(OwnedHost::New, &motion(thumb.x + 1.0, thumb.y + 10.0));
    assert_eq!(
        surface(&app, 2).navigation_scroll.offset(),
        offset,
        "released scroll grab cannot move again"
    );
    assert_eq!(std::fs::read(old.join("A.txt")).unwrap(), b"A.txt");
    assert_eq!(std::fs::read(new.join("B.txt")).unwrap(), b"B.txt");
    app.runtime.as_mut().unwrap().close_new_project();
    std::fs::remove_dir_all(root).unwrap();
}
