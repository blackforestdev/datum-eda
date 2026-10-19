use super::*;
#[test]
fn folder_actions_pending_receipt_explains_both_outer_actions_without_a_naming_bar() {
    for (w, h) in [(760, 720), (620, 620)] {
        for is_new in [true, false] {
            let (mut dialog, nav) = fixture();
            dialog.host_action_pending = true;
            let mut open = datum_gui_protocol::ProjectOpenDialogState {
                open: true,
                directory: "/parent".into(),
                host_action_pending: true,
                ..Default::default()
            };
            for pending in [true, false] {
                dialog.host_action_pending = pending;
                open.host_action_pending = pending;
                let form = if is_new {
                    FormData::New(&dialog)
                } else {
                    FormData::Open(&open)
                };
                let (text, _) = paint(
                    form,
                    &nav,
                    w,
                    h,
                    1.0,
                    &mut ScrollViewport::default(),
                    &mut ScrollViewport::default(),
                    &mut ScrollViewport::default(),
                    false,
                );
                assert_eq!(
                    text.iter()
                        .any(|r| r.text.contains("becomes available") && r.y >= h as f32 - 56.0),
                    pending,
                    "explanation belongs to fixed footer even after dialog reopen"
                );
                let nodes = if is_new {
                    dialog.accessibility_nodes()
                } else {
                    open.accessibility_nodes()
                };
                assert_eq!(
                    nodes.iter().any(|n| n.name == "Folder action pending"),
                    pending
                );
                let action = nodes
                    .iter()
                    .find(|n| n.name == if is_new { "Create" } else { "Open" })
                    .unwrap();
                assert_eq!(action.available, !pending);
                assert_eq!(
                    action.description.contains("folder action finishes"),
                    pending
                );
                assert!(nodes.iter().find(|n| n.name == "Cancel").unwrap().available);
            }
        }
    }
}
#[test]
fn folder_actions_naming_is_reachable_above_fixed_project_footer_at_supported_scales() {
    use datum_gui_protocol::{NamingDraft, ProjectOpenDialogState, ProjectOpenFocus};
    for (w, h) in [(760, 720), (620, 620)] {
        for scale in [1.0, 1.5, 2.0] {
            for is_new in [true, false] {
                let (mut dialog, mut nav) = fixture();
                nav.expanded = true;
                nav.naming = Some(NamingDraft::new("/parent".into(), (1, 1), 1, None));
                let draft = nav.naming.as_mut().unwrap();
                draft.value = " exact λ ".into();
                draft.refusal = Some(
                    "No overwrite: this exact name already exists. The naming draft is retained. "
                        .repeat(5),
                );
                dialog.focus = NewProjectFocus::Navigation(FolderBrowserFocus::NameEditor);
                let open = ProjectOpenDialogState {
                    open: true,
                    directory: "/parent".into(),
                    focus: ProjectOpenFocus::Navigation(FolderBrowserFocus::NameEditor),
                    ..Default::default()
                };
                if !is_new {
                    nav.mode = FolderBrowserMode::Open;
                }
                let form = if is_new {
                    FormData::New(&dialog)
                } else {
                    FormData::Open(&open)
                };
                let (mut body, mut list, mut preview) = (
                    ScrollViewport::default(),
                    ScrollViewport::default(),
                    ScrollViewport::default(),
                );
                let (_, hits) = paint(
                    form,
                    &nav,
                    w,
                    h,
                    scale,
                    &mut body,
                    &mut list,
                    &mut preview,
                    true,
                );
                let editor = hits
                    .iter()
                    .find(|h| h.target == HitTarget::FolderBrowser(FolderBrowserFocus::NameEditor))
                    .expect("naming editor revealed even with wrapped refusal");
                assert!(
                    editor.rect.width > 1.0
                        && editor.rect.y >= body.viewport.y
                        && editor.rect.y + editor.rect.height
                            <= body.viewport.y + body.viewport.height
                );
                for target in if is_new {
                    [HitTarget::NewProjectCancel, HitTarget::NewProjectCreate]
                } else {
                    [HitTarget::ProjectOpenCancel, HitTarget::ProjectOpenConfirm]
                } {
                    let footer = hits.iter().find(|h| h.target == target).unwrap();
                    assert!(
                        footer.rect.y >= h as f32 - 56.0
                            && footer.rect.y + footer.rect.height <= h as f32
                    );
                    assert!(editor.rect.y + editor.rect.height <= footer.rect.y);
                }
                nav.naming = None;
                let (_, hits) = paint(
                    form,
                    &nav,
                    w,
                    h,
                    scale,
                    &mut body,
                    &mut list,
                    &mut preview,
                    false,
                );
                assert!(!hits.iter().any(|h| matches!(
                    h.target,
                    HitTarget::FolderBrowser(
                        FolderBrowserFocus::NameEditor
                            | FolderBrowserFocus::NameCancel
                            | FolderBrowserFocus::NameSubmit
                    )
                )));
            }
        }
    }
}
#[test]
fn shared_editor_pointer_placement_matches_scrolled_unicode_and_never_reuses_previous_text_run() {
    use datum_gui_protocol::single_line_editor::SingleLineEditor;
    let value = "prefix λ suffix with a deliberately long exact path";
    let end = SingleLineEditor::at_end(value);
    assert_eq!(
        crate::single_line_control::single_line_caret_at(value, end, 100.0, 99.0).unwrap(),
        value.len()
    );
    for x in [0.0, 9.0, 30.0, 60.0, 100.0] {
        let byte = crate::single_line_control::single_line_caret_at(value, end, 100.0, x).unwrap();
        assert!(value.is_char_boundary(byte));
    }
    let (mut quads, mut text, mut hits) = (vec![], vec![], vec![]);
    draw_text(
        "prior owner",
        0.0,
        0.0,
        11.0,
        TEXT_PRIMARY,
        TextFace::Ui,
        &mut text,
    );
    let before = text[0].layout_size;
    let mut controls = ControlMeshCache::default();
    let mut painter = ControlPainter::new(&mut quads, &mut controls, 1.0);
    crate::single_line_control::paint(
        "",
        "",
        Default::default(),
        false,
        true,
        RectPx {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 34.0,
        },
        &mut painter,
        &mut text,
        &mut hits,
        HitTarget::NewProjectName,
    )
    .unwrap();
    assert_eq!(
        text[0].layout_size, before,
        "empty editor cannot change another control's layout"
    );
}
fn fixture() -> (
    datum_gui_protocol::NewProjectDialogState,
    FolderBrowserState,
) {
    let dialog = datum_gui_protocol::NewProjectDialogState {
        open: true,
        project_name: "Exact name".into(),
        destination: "/parent/Exact name".into(),
        location_ready: true,
        global_units_ready: true,
        ..Default::default()
    };
    let mut nav = FolderBrowserState::new(FolderBrowserMode::New, "/parent".into());
    nav.directory = "/parent".into();
    nav.loading = false;
    nav.entries = vec!["folder".into()];
    (dialog, nav)
}
#[allow(clippy::too_many_arguments)]
fn paint(
    form: FormData<'_>,
    nav: &FolderBrowserState,
    w: u32,
    h: u32,
    scale: f32,
    body: &mut ScrollViewport,
    list: &mut ScrollViewport,
    preview: &mut ScrollViewport,
    reveal: bool,
) -> (Vec<TextRun>, Vec<HitRegion>) {
    let layout = PreparedScene::native_dialog_layout(
        (w as f32 * scale) as u32,
        (h as f32 * scale) as u32,
        scale,
    );
    let (mut quads, mut text, mut hits) = (vec![], vec![], vec![]);
    paint_form(
        form,
        Some(nav),
        &layout,
        &mut ControlMeshCache::default(),
        scale,
        &mut quads,
        &mut text,
        &mut hits,
        body,
        list,
        preview,
        reveal,
    )
    .unwrap();
    (text, hits)
}
#[test]
fn folder_browser_inline_project_forms_keep_one_fixed_action_and_no_units_controls_at_supported_scales()
 {
    for (w, h) in [(760, 720), (620, 620)] {
        for scale in [1.0, 1.5, 2.0] {
            let (mut dialog, mut nav) = fixture();
            nav.expanded = true;
            dialog.focus = NewProjectFocus::Create;
            let (mut body, mut list, mut preview) = (
                ScrollViewport::default(),
                ScrollViewport::default(),
                ScrollViewport::default(),
            );
            let (text, hits) = paint(
                FormData::New(&dialog),
                &nav,
                w,
                h,
                scale,
                &mut body,
                &mut list,
                &mut preview,
                true,
            );
            for target in [HitTarget::NewProjectCreate, HitTarget::NewProjectCancel] {
                let items = hits
                    .iter()
                    .filter(|h| h.target == target)
                    .collect::<Vec<_>>();
                assert_eq!(items.len(), 1);
                assert!(
                    items[0].rect.y >= h as f32 - 56.0
                        && items[0].rect.y + items[0].rect.height <= h as f32
                );
            }
            assert!(!text.iter().any(|r| r.text.contains("Working units")
                || r.text.contains("Use my Global")
                || r.text.contains("Datum factory")));
            let list_hit = hits
                .iter()
                .find(|h| h.target == HitTarget::FolderBrowser(FolderBrowserFocus::Folders))
                .unwrap();
            assert_eq!(list_hit.rect.x, 180.0);
            assert_eq!(list_hit.rect.width, w as f32 - 198.0);
            let preview_hit = hits
                .iter()
                .find(|h| h.target == HitTarget::FolderBrowser(FolderBrowserFocus::Preview))
                .unwrap();
            assert_eq!(preview_hit.rect.x, 18.0);
            assert!(preview_hit.rect.y >= list_hit.rect.y + list_hit.rect.height);
            assert_eq!(list_hit.rect.height, if w == 760 { 290.0 } else { 220.0 });
            if w == 760 {
                assert_eq!(
                    body.maximum(),
                    0.0,
                    "ordinary short-path New fits without body scrolling"
                );
            }
            assert!(
                !text
                    .iter()
                    .any(|r| matches!(r.text.as_str(), "Use Location" | "Select"))
            );
        }
    }
}
#[test]
fn folder_browser_inline_preview_scrolls_independently_and_open_keeps_left_footer() {
    let (mut dialog, mut nav) = fixture();
    dialog.focus = NewProjectFocus::Navigation(FolderBrowserFocus::Preview);
    nav.path_input = format!("/parent/{}", "full-path-segment/".repeat(70));
    nav.directory = nav.path_input.clone();
    let (mut body, mut list, mut preview) = (
        ScrollViewport::default(),
        ScrollViewport::default(),
        ScrollViewport::default(),
    );
    paint(
        FormData::New(&dialog),
        &nav,
        620,
        620,
        1.0,
        &mut body,
        &mut list,
        &mut preview,
        true,
    );
    assert!(preview.maximum() > preview.viewport.height);
    preview.set_offset(preview.maximum());
    let before_body = body.offset();
    let before_list = list.offset();
    paint(
        FormData::New(&dialog),
        &nav,
        620,
        620,
        1.0,
        &mut body,
        &mut list,
        &mut preview,
        false,
    );
    assert_eq!(preview.offset(), preview.maximum());
    assert_eq!(body.offset(), before_body);
    assert_eq!(list.offset(), before_list);
    nav.mode = FolderBrowserMode::Open;
    let open = datum_gui_protocol::ProjectOpenDialogState {
        open: true,
        directory: nav.directory.clone(),
        ..Default::default()
    };
    let (_, hits) = paint(
        FormData::Open(&open),
        &nav,
        760,
        720,
        1.0,
        &mut body,
        &mut list,
        &mut preview,
        true,
    );
    let cancel = hits
        .iter()
        .find(|h| h.target == HitTarget::ProjectOpenCancel)
        .unwrap();
    let action = hits
        .iter()
        .find(|h| h.target == HitTarget::ProjectOpenConfirm)
        .unwrap();
    assert_eq!(cancel.rect.x, 18.0);
    assert!(action.rect.x < 220.0);
    assert!(action.rect.y >= 664.0);
}

#[test]
fn folder_browser_inline_fixed_refusal_survives_body_scroll_and_create_focus() {
    let (mut dialog, mut nav) = fixture();
    nav.expanded = true;
    dialog.focus = NewProjectFocus::Create;
    dialog.refusal = Some(format!(
        "Permission denied. {} FULL-REASON-TAIL",
        "/long/parent/path".repeat(70)
    ));
    let (mut body, mut list, mut preview) = (
        ScrollViewport::default(),
        ScrollViewport::default(),
        ScrollViewport::default(),
    );
    paint(
        FormData::New(&dialog),
        &nav,
        620,
        620,
        1.0,
        &mut body,
        &mut list,
        &mut preview,
        true,
    );
    body.set_offset(body.maximum());
    assert!(body.offset() > 0.0);
    let (text, hits) = paint(
        FormData::New(&dialog),
        &nav,
        620,
        620,
        1.0,
        &mut body,
        &mut list,
        &mut preview,
        false,
    );
    assert!(body.offset() > 0.0 && body.viewport.y > 42.0);
    let cue = text
        .iter()
        .find(|r| r.text.contains("Full refusal details continue"))
        .unwrap();
    assert!(cue.y >= 42.0 && cue.y < body.viewport.y);
    assert!(
        text.iter()
            .any(|r| r.text.contains("Permission denied") && r.y < body.viewport.y)
    );
    assert!(
        hits.iter()
            .any(|h| h.target == HitTarget::NewProjectCreate && h.rect.y >= 564.0)
    );
    let cue_y = cue.y;
    for offset in [0.0, body.maximum()] {
        body.set_offset(offset);
        let (text, _) = paint(
            FormData::New(&dialog),
            &nav,
            620,
            620,
            1.0,
            &mut body,
            &mut list,
            &mut preview,
            false,
        );
        assert_eq!(
            text.iter()
                .find(|r| r.text.contains("Full refusal details continue"))
                .unwrap()
                .y,
            cue_y
        );
    }
    dialog.focus = NewProjectFocus::Navigation(FolderBrowserFocus::Preview);
    paint(
        FormData::New(&dialog),
        &nav,
        620,
        620,
        1.0,
        &mut body,
        &mut list,
        &mut preview,
        true,
    );
    preview.set_offset(preview.maximum());
    let (text, _) = paint(
        FormData::New(&dialog),
        &nav,
        620,
        620,
        1.0,
        &mut body,
        &mut list,
        &mut preview,
        false,
    );
    assert!(text.iter().any(|r| r.text.contains("FULL-REASON-TAIL")));
    assert_eq!(
        dialog
            .accessibility_nodes()
            .iter()
            .find(|n| n.id == "new-project-refusal")
            .unwrap()
            .value,
        dialog.refusal
    );
}

#[test]
fn folder_browser_inline_global_reread_remains_in_fixed_alert_band() {
    let (mut dialog, nav) = fixture();
    dialog.global_units_ready = false;
    dialog.refusal =
        Some("Global source unavailable. Repair Global defaults and explicitly re-read.".into());
    dialog.focus = NewProjectFocus::RetryGlobal;
    let (mut body, mut list, mut preview) = (
        ScrollViewport::default(),
        ScrollViewport::default(),
        ScrollViewport::default(),
    );
    for offset in [0.0, 1000.0] {
        body.set_offset(offset);
        let (_, hits) = paint(
            FormData::New(&dialog),
            &nav,
            620,
            620,
            1.0,
            &mut body,
            &mut list,
            &mut preview,
            true,
        );
        let retry = hits
            .iter()
            .find(|h| h.target == HitTarget::NewProjectRetryGlobal)
            .unwrap()
            .rect;
        assert!(retry.y >= 42.0 && retry.y + retry.height <= body.viewport.y);
        assert_eq!(
            hits.iter()
                .filter(|h| h.target == HitTarget::NewProjectRetryGlobal)
                .count(),
            1
        );
        assert!(!dialog.create_enabled());
    }
}

#[test]
fn folder_browser_compact_forms_show_location_and_browse_without_hidden_hits() {
    for (w, h) in [(760, 720), (620, 620)] {
        for scale in [1.0, 1.5, 2.0] {
            let (dialog, mut nav) = fixture();
            let open = datum_gui_protocol::ProjectOpenDialogState {
                open: true,
                directory: "/parent".into(),
                ..Default::default()
            };
            for is_new in [true, false] {
                nav.mode = if is_new {
                    FolderBrowserMode::New
                } else {
                    FolderBrowserMode::Open
                };
                let form = if is_new {
                    FormData::New(&dialog)
                } else {
                    FormData::Open(&open)
                };
                let (mut body, mut list, mut preview) = (
                    ScrollViewport::default(),
                    ScrollViewport::default(),
                    ScrollViewport::default(),
                );
                let (text, hits) = paint(
                    form,
                    &nav,
                    w,
                    h,
                    scale,
                    &mut body,
                    &mut list,
                    &mut preview,
                    true,
                );
                assert!(text.iter().any(|r| r.text == "Location"));
                assert!(text.iter().any(|r| r.text == "Browse…"));
                for focus in [
                    FolderBrowserFocus::Path,
                    FolderBrowserFocus::Browse,
                    FolderBrowserFocus::Preview,
                ] {
                    assert_eq!(
                        hits.iter()
                            .filter(|h| h.target == HitTarget::FolderBrowser(focus))
                            .count(),
                        1
                    );
                }
                for focus in [
                    FolderBrowserFocus::Up,
                    FolderBrowserFocus::Home,
                    FolderBrowserFocus::Folders,
                ] {
                    assert!(
                        !hits
                            .iter()
                            .any(|h| h.target == HitTarget::FolderBrowser(focus))
                    );
                }
                assert!(
                    !hits
                        .iter()
                        .any(|h| matches!(h.target, HitTarget::FolderBrowserRow(_)))
                );
                assert_eq!(body.maximum(), 0.0);
            }
        }
    }
}
