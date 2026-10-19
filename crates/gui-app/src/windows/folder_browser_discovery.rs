//! Shared browser discovery lifecycle, latest-query admission and worker submission.
use super::*;

impl FolderBrowser {
    pub(super) fn invalidate_listing(&mut self) {
        self.select_after_scan = None;
        if let Some(scanner) = &self.scanner {
            scanner.cancel();
        }
        self.pending = None;
        self.last_click = None;
        if let Some(state) = &mut self.state {
            state.directory.clear();
            state.suggestions = None;
            state.entries.clear();
            state.parent_identity = None;
            state.occupied.clear();
            state.visible_rows.clear();
            state.selected = None;
            state.loading = false;
            state.error = None;
            state.notice = None;
        }
    }

    fn navigate(&mut self, path: PathBuf, replace_input: bool, assisted: bool) {
        self.last_click = None;
        self.select_after_scan = None;
        let Some(state) = &mut self.state else {
            return;
        };
        let Some(text) = path.to_str() else {
            state.error = Some("This folder path has an unsupported filename encoding.".into());
            return;
        };
        if replace_input {
            state.path_input = text.into();
            state.path_editor =
                datum_gui_protocol::single_line_editor::SingleLineEditor::at_end(&state.path_input);
        }
        state.suggestions = None;
        state.loading = true;
        state.error = None;
        state.notice = None;
        state.entries.clear();
        state.parent_identity = None;
        state.occupied.clear();
        state.selected = None;
        state.visible_rows.clear();
        let result = self
            .scanner
            .as_ref()
            .ok_or_else(|| "Folder reader unavailable. Reopen the dialog to retry.".into())
            .and_then(|s| s.request_input(path, Some(state.path_input.clone()), assisted));
        match result {
            Ok(ticket) => self.pending = Some(ticket),
            Err(error) => {
                state.loading = false;
                state.error = Some(error);
                self.pending = None;
            }
        }
    }

    pub(super) fn poll(&mut self) -> bool {
        let mut changed = false;
        while let Some(answer) = self.scanner.as_ref().and_then(scan::Scanner::receive) {
            changed |= admit_answer(&mut self.state, &mut self.pending, answer);
        }
        changed
    }
}

impl Runtime {
    pub(super) fn read_folder_navigation(&mut self, directory: PathBuf, replace_input: bool) {
        self.read_folder_request(directory, replace_input, false);
    }
    pub(super) fn read_folder_request(
        &mut self,
        directory: PathBuf,
        replace_input: bool,
        assisted: bool,
    ) {
        if self.folder_browser.scanner.is_none() {
            let wake = self.folder_browser.wake.clone();
            match scan::Scanner::new(move || {
                let _ = wake.send_event(());
            }) {
                Ok(scanner) => self.folder_browser.scanner = Some(scanner),
                Err(error) => {
                    let state = self.folder_browser.state.as_mut().unwrap();
                    state.loading = false;
                    state.error = Some(format!(
                        "Folder reader unavailable: {error}. Reopen the dialog to retry."
                    ));
                    self.refresh_folder_browser();
                    return;
                }
            }
        }
        self.folder_browser
            .navigate(directory, replace_input, assisted);
        self.refresh_folder_browser();
    }
}

pub(super) fn admit_answer(
    state: &mut Option<FolderBrowserState>,
    pending: &mut Option<u64>,
    answer: scan::ScanResult,
) -> bool {
    if *pending != Some(answer.ticket) {
        return false;
    }
    let Some(state) = state else {
        return false;
    };
    if answer
        .input
        .as_deref()
        .is_some_and(|input| input != state.path_input)
    {
        return false;
    }
    if entered_path(&state.path_input).ok().as_ref() != Some(&answer.directory) {
        return false;
    }
    *pending = None;
    state.loading = false;
    match answer.result {
        Ok(listing) => {
            // Bind the listing to the exact displayed spelling, including ~ and spaces.
            state.suggestions = listing.suggestions.map(|mut q| {
                q.input = state.path_input.clone();
                q
            });
            state.directory = state
                .suggestions
                .as_ref()
                .map(|q| q.parent.clone())
                .unwrap_or_else(|| state.path_input.clone());
            if state.suggestions.is_some() {
                state.expanded = true;
            }
            state.entries = listing.entries;
            state.parent_identity = listing.parent_identity;
            state.occupied = listing.occupied;
            state.notice = listing.notice;
            state.error = None;
        }
        Err(error) => {
            state.directory.clear();
            state.error = Some(error);
        }
    }
    true
}
