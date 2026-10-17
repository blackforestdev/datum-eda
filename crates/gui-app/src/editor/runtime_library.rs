//! Library persona over shared pane ownership, selection, and native edit worker.
use super::*;
use datum_gui_protocol::{
    library_authoring::LibraryDraft,
    library_browser::{KINDS, LibraryControl as C},
};
impl Runtime {
    pub(super) fn open_library(&mut self) {
        let id = self.session.workspace_mut().ui.layout.open_beside(
            PaneContent::Library,
            datum_gui_protocol::SplitOrientation::Vertical,
            true,
        );
        self.set_application_focus(datum_gui_protocol::ApplicationFocus::Editor(id));
        self.invalidate_frame();
    }
    pub(super) fn activate_library(&mut self, control: C) -> bool {
        if self.native_edit.pending() {
            return true;
        }
        self.session.workspace_mut().library.focus = Some(control.clone());
        self.session.workspace_mut().library.reveal_focus = true;
        match control {
            C::Scroll { .. } => {}
            C::Relationship(i) => {
                let s = self.workspace();
                let target = s.library.selected(&s.selection).and_then(|e| {
                    s.library
                        .links(e)
                        .get(i)
                        .filter(|l| l.targets.len() == 1)
                        .map(|l| l.targets[0])
                });
                if let Some(target) = target {
                    return self.activate_library(C::Result(target));
                }
            }
            C::PreviewPad(i) => self.session.workspace_mut().library.preview_pad = i,
            C::Notice => {
                let s = self.workspace();
                let source = s
                    .library
                    .draft
                    .as_ref()
                    .and_then(|d| d.preview.as_ref().map(|raw| (d.kind.as_str(), raw)))
                    .or_else(|| {
                        s.library
                            .selected(&s.selection)
                            .map(|e| (e.kind.as_str(), &e.raw))
                    });
                let reason = s
                    .library
                    .draft
                    .as_ref()
                    .and_then(|d| d.refusal.clone())
                    .or_else(|| s.library.refusal.clone())
                    .or_else(|| {
                        source
                            .filter(|(kind, _)| matches!(*kind, "footprints" | "padstacks"))
                            .and_then(|(kind, raw)| {
                                s.library
                                    .process_inspection(kind, raw, s.library.preview_pad)
                                    .err()
                            })
                    });
                if let Some(reason) = reason {
                    self.log_console_refusal(ConsoleFeedbackSource::Tool, reason);
                }
            }
            C::Kind(index) => {
                let browser = &mut self.session.workspace_mut().library;
                if browser.draft.is_none() {
                    browser.kind = index.min(7);
                    browser.result_offset = 0;
                    browser.detail_offset = 0;
                    browser.search.clear();
                }
            }
            C::Search => self.session.workspace_mut().library.search_focus = true,
            C::Result(index) => {
                let state = self.session.workspace_mut();
                if state.library.draft.is_none() {
                    if let Some(e) = state.library.entries.get(index) {
                        state.selection =
                            datum_gui_protocol::SelectionTarget::AuthoredObject(e.key());
                        state.library.detail_offset = 0;
                        state.library.preview_pad = 0;
                        state.library.scroll_offset = 0;
                    }
                }
            }
            C::ResultsPage(next) => {
                let b = &mut self.session.workspace_mut().library;
                let count = b.results().len();
                b.result_offset = if next {
                    (b.result_offset + 4).min(count.saturating_sub(1))
                } else {
                    b.result_offset.saturating_sub(4)
                };
            }
            C::DetailPage(next) => {
                let b = &mut self.session.workspace_mut().library;
                b.detail_offset = if next {
                    b.detail_offset + 1
                } else {
                    b.detail_offset.saturating_sub(1)
                };
            }
            C::New | C::Edit => {
                if !self.native_edit_available() {
                    self.log_console_refusal(
                        ConsoleFeedbackSource::Tool,
                        "Library authoring requires a persistent native Project",
                    );
                    return true;
                }
                let s = self.workspace();
                let entry = if control == C::Edit {
                    s.library.selected(&s.selection)
                } else {
                    None
                };
                if control == C::Edit && entry.is_none() {
                    return true;
                }
                let kind = entry.map_or(KINDS[s.library.kind].0, |e| e.kind.as_str());
                let draft = LibraryDraft::begin(
                    &s.library,
                    entry,
                    kind,
                    s.scene.source_revision.clone(),
                    s.supervision
                        .journal
                        .accepted_transaction_tip
                        .as_ref()
                        .and_then(|s| s.parse().ok()),
                );
                match draft {
                    Ok(d) => {
                        let b = &mut self.session.workspace_mut().library;
                        b.draft = Some(d);
                        b.refusal = None;
                        b.focus = Some(C::Field(0));
                    }
                    Err(e) => self.session.workspace_mut().library.refusal = Some(e),
                }
            }
            C::Field(index) => {
                if let Some(d) = &mut self.session.workspace_mut().library.draft {
                    d.focused = index.min(d.fields.len().saturating_sub(1));
                    d.offset = d.focused;
                }
            }
            C::Preview => {
                let root = self
                    .workspace()
                    .backing
                    .as_ref()
                    .map(|b| b.request.project_root.clone());
                let mut draft = self.session.workspace_mut().library.draft.take();
                if let (Some(d), Some(root)) = (&mut draft, root) {
                    if let Err(e) = d.validate_preview(&self.workspace().library, &root) {
                        d.preview = None;
                        d.refusal = Some(e);
                    }
                }
                self.session.workspace_mut().library.draft = draft;
            }
            C::Commit => {
                if self
                    .workspace()
                    .library
                    .draft
                    .as_ref()
                    .is_some_and(|d| d.preview.is_some())
                {
                    return self.queue_native_edit(
                        datum_gui_protocol::native_edit_client::NativeEditAction::Library,
                    );
                }
                self.session.workspace_mut().library.refusal =
                    Some("Preview the current fields before committing".into());
            }
            C::Cancel => {
                let b = &mut self.session.workspace_mut().library;
                b.draft = None;
                b.refusal = None;
                b.search_focus = false;
            }
            C::Pad(_) | C::AddPad | C::RemovePad => {
                let index = if let C::Pad(i) = control {
                    self.session.workspace_mut().library.preview_pad = i;
                    Some(i)
                } else {
                    None
                };
                if let Some(d) = &mut self.session.workspace_mut().library.draft {
                    if let Err(e) = d.switch_pad(index, control == C::RemovePad) {
                        d.refusal = Some(e);
                    }
                }
            }
            C::Terminal(_) | C::AddTerminal | C::RemoveTerminal => {
                let index = if let C::Terminal(i) = control {
                    Some(i)
                } else {
                    None
                };
                if let Some(d) = &mut self.session.workspace_mut().library.draft {
                    if let Err(e) = d.switch_terminal(index, control == C::RemoveTerminal) {
                        d.refusal = Some(e);
                    }
                }
            }
        }
        self.invalidate_frame();
        true
    }
    pub(super) fn handle_library_key(&mut self, event: &winit::event::KeyEvent) -> bool {
        self.handle_library_input(&event.logical_key, event.state)
    }
    /// Shared logical-key consumer, also exercised by programmatic App proof.
    pub(super) fn handle_library_input(&mut self, key: &Key, state: ElementState) -> bool {
        if self.workspace().ui.layout.focused_content() != PaneContent::Library
            || !matches!(
                self.application_focus(),
                datum_gui_protocol::ApplicationFocus::Editor(_)
            )
            || state != ElementState::Pressed
            || self.modifiers.control_key()
            || self.modifiers.super_key()
        {
            return false;
        }
        if self.native_edit.pending() {
            return true;
        }
        let focus = self.workspace().library.focus.clone();
        match key {
            Key::Named(NamedKey::Escape) => return self.activate_library(C::Cancel),
            Key::Named(NamedKey::Tab) => {
                let s = self.workspace();
                let mut controls = Vec::new();
                if let Some(d) = &s.library.draft {
                    if d.kind == "footprints" {
                        controls.extend((0..d.pads.len()).map(C::Pad));
                        controls.extend([C::AddPad, C::RemovePad]);
                    }
                    if d.kind == "packages" {
                        controls.extend((0..d.terminals.len()).map(C::Terminal));
                        controls.extend([C::AddTerminal, C::RemoveTerminal]);
                    }
                    controls.extend((0..d.fields.len()).map(C::Field));
                    if let Some(raw) = &d.preview {
                        controls.extend(
                            (0..raw
                                .get("pads")
                                .and_then(|p| p.as_object())
                                .map_or(0, |p| p.len()))
                                .map(C::PreviewPad),
                        );
                    }
                    controls.extend([C::Notice, C::Preview, C::Commit, C::Cancel]);
                } else {
                    controls.extend((0..8).map(C::Kind));
                    controls.push(C::Search);
                    controls.extend(s.library.results().into_iter().map(C::Result));
                    if matches!(
                        KINDS[s.library.kind].0,
                        "footprints" | "padstacks" | "packages"
                    ) {
                        controls.push(C::New);
                    }
                    if s.library
                        .selected(&s.selection)
                        .is_some_and(|e| e.editable())
                    {
                        controls.push(C::Edit);
                    }
                    if let Some(entry) = s.library.selected(&s.selection) {
                        for (i, link) in s.library.links(entry).into_iter().enumerate() {
                            if link.targets.len() == 1 {
                                let control = C::Relationship(i);
                                if !controls.contains(&control) {
                                    controls.push(control);
                                }
                            }
                        }
                    }
                    if let Some(e) = s
                        .library
                        .selected(&s.selection)
                        .filter(|e| matches!(e.kind.as_str(), "footprints" | "padstacks"))
                    {
                        controls.extend(
                            (0..e
                                .raw
                                .get("pads")
                                .and_then(|p| p.as_object())
                                .map_or(0, |p| p.len()))
                                .map(C::PreviewPad),
                        );
                    }
                    if s.library
                        .selected(&s.selection)
                        .is_some_and(|e| matches!(e.kind.as_str(), "footprints" | "padstacks"))
                    {
                        controls.push(C::Notice);
                    }
                    controls.extend([C::DetailPage(false), C::DetailPage(true)]);
                }
                let n = controls.len();
                if n == 0 {
                    return true;
                }
                let index = focus
                    .as_ref()
                    .and_then(|f| controls.iter().position(|c| c == f));
                let next = index.map_or(0, |i| {
                    if self.modifiers.shift_key() {
                        (i + n - 1) % n
                    } else {
                        (i + 1) % n
                    }
                });
                let control = controls[next].clone();
                let selection = self.workspace().selection.clone();
                let b = &mut self.session.workspace_mut().library;
                if let C::Field(i) = control {
                    if let Some(d) = &mut b.draft {
                        d.focused = i;
                        d.offset = i;
                    }
                }
                if let C::Result(i) = control {
                    if let Some(p) = b.results().iter().position(|j| *j == i) {
                        b.result_offset = p.saturating_sub(2);
                    }
                    if let Some(entry) = b.selected(&selection) {
                        if let Some(position) = b
                            .links(entry)
                            .iter()
                            .position(|link| link.targets.as_slice() == [i])
                        {
                            b.detail_offset = position;
                        }
                    }
                }
                b.search_focus = control == C::Search;
                b.focus = Some(control);
                b.reveal_focus = true;
            }
            Key::Named(NamedKey::Enter) => {
                if let Some(control) = focus {
                    return self.activate_library(control);
                }
            }
            Key::Named(NamedKey::ArrowLeft | NamedKey::ArrowRight) => {
                if matches!(focus, Some(C::Field(_))) {
                    if let Some(d) = &mut self.session.workspace_mut().library.draft {
                        d.cycle(matches!(key, Key::Named(NamedKey::ArrowRight)));
                    }
                }
            }
            Key::Named(NamedKey::Backspace) => self.library_text(None),
            Key::Character(text) => self.library_text(Some(text)),
            _ => return false,
        }
        self.invalidate_frame();
        true
    }
    pub(super) fn handle_library_scroll(
        &mut self,
        wheel: crate::app_native_events::scroll_input::VerticalWheel,
    ) -> Option<bool> {
        let (x, y) = self.last_cursor_pos?;
        let (content, height, offset) = self.presented_hits.regions().iter().find_map(|r| {
            if r.rect.contains(x, y) {
                if let HitTarget::Library(C::Scroll {
                    content,
                    height,
                    offset,
                }) = r.target
                {
                    Some((content, height, offset))
                } else {
                    None
                }
            } else {
                None
            }
        })?;
        let mut scroll = datum_gui_viewport::scroll::ScrollViewport::default();
        scroll.layout(
            datum_gui_viewport::ScreenRectPx {
                x: 0.,
                y: 0.,
                width: 1.,
                height: height as f32,
            },
            content as f32,
        );
        scroll.set_offset(offset as f32);
        let changed = scroll.wheel(wheel.pixels(24.));
        let b = &mut self.session.workspace_mut().library;
        b.scroll_offset = scroll.offset() as usize;
        b.reveal_focus = false;
        if changed {
            self.invalidate_frame();
        }
        Some(changed)
    }
    fn library_text(&mut self, text: Option<&str>) {
        let b = &mut self.session.workspace_mut().library;
        match b.focus {
            Some(C::Search) => {
                if let Some(text) = text {
                    b.search.push_str(text);
                } else {
                    b.search.pop();
                }
                b.result_offset = 0;
            }
            Some(C::Field(_)) => {
                if let Some(d) = &mut b.draft {
                    if !d.fields[d.focused].selector {
                        let mut value = d.fields[d.focused].text.clone();
                        if let Some(text) = text {
                            value.push_str(text);
                        } else {
                            value.pop();
                        }
                        d.change(value);
                    }
                }
            }
            _ => {}
        }
    }
}
