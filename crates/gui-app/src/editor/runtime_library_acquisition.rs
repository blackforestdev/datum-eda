//! Background local-file reads with disposable, origin-qualified publication.
use super::*;
use datum_gui_protocol::library_browser::acquisition::{
    LibraryAcquisition, LibraryAcquisitionPreview, LibrarySourceSymbol,
    catalog::{self, SourceCatalog},
};
use std::sync::mpsc;
#[path = "runtime_library_footprint_acquisition.rs"]
mod source_read;

enum ReadResult {
    Names(SourceCatalog),
    Preview(LibraryAcquisitionPreview),
}
struct Answer {
    session: eda_engine::substrate::ObjectId,
    revision: String,
    result: Result<ReadResult, String>,
}
pub(crate) struct AcquisitionWork {
    pub(crate) component_sources:
        std::collections::BTreeMap<eda_engine::substrate::ObjectId, String>,
    pending: Option<mpsc::Receiver<Answer>>,
    wake: winit::event_loop::EventLoopProxy<()>,
}
impl AcquisitionWork {
    pub(crate) fn new(wake: winit::event_loop::EventLoopProxy<()>) -> Self {
        Self {
            component_sources: Default::default(),
            pending: None,
            wake,
        }
    }
    pub(crate) fn cancel(&mut self) {
        self.pending = None;
    }
    pub(crate) fn start(
        &mut self,
        acquisition: &LibraryAcquisition,
        source: Option<LibrarySourceSymbol>,
    ) {
        if acquisition.component_picker
            && !acquisition.path.trim().is_empty()
            && let Some(document) = &acquisition.placement_document
        {
            self.component_sources
                .insert(document.project, acquisition.path.clone());
        }
        let (sender, receiver) = mpsc::channel();
        self.pending = Some(receiver);
        let session = acquisition.session;
        let revision = acquisition.revision.clone();
        let path = std::path::PathBuf::from(&acquisition.path);
        let bundled = acquisition.bundled();
        let footprints = acquisition.footprints();
        let native_parts = acquisition.is_native_part();
        let wake = self.wake.clone();
        std::thread::spawn(move || {
            let result = if native_parts {
                read_native_parts(&path, source)
            } else {
                source_read::read(&path, bundled, footprints, source)
            };
            let _ = sender.send(Answer {
                session,
                revision,
                result,
            });
            let _ = wake.send_event(());
        });
    }
}
impl Runtime {
    pub(crate) fn poll_library_acquisition(&mut self) -> bool {
        let Some(receiver) = &self.library_acquisition_work.pending else {
            return false;
        };
        let answer = match receiver.try_recv() {
            Ok(answer) => answer,
            Err(mpsc::TryRecvError::Empty) => return false,
            Err(mpsc::TryRecvError::Disconnected) => {
                self.library_acquisition_work.pending = None;
                if let Some(a) = &mut self.session.workspace_mut().library.acquisition {
                    a.loading = false;
                    a.placement_confirmation = None;
                    a.refusal = Some("Local source read stopped; load the source again".into());
                }
                self.invalidate_frame();
                return true;
            }
        };
        self.library_acquisition_work.pending = None;
        let current_revision = self.workspace().scene.source_revision.clone();
        let Some(a) = &mut self.session.workspace_mut().library.acquisition else {
            return false;
        };
        if a.session != answer.session || a.revision != answer.revision {
            return false;
        }
        if current_revision != answer.revision {
            a.loading = false;
            a.preview = None;
            a.placement_confirmation = None;
            a.refusal =
                Some("Project changed during the source read; cancel and acquire again".into());
            self.invalidate_frame();
            return true;
        }
        a.loading = false;
        let mut catalog_arrived = false;
        match answer.result {
            Ok(ReadResult::Names(catalog)) => {
                a.names = catalog.symbols.iter().map(|s| s.label.clone()).collect();
                a.sources = catalog.symbols;
                a.catalog_notices = catalog.notices;
                a.loaded = true;
                catalog_arrived = true;
            }
            Ok(ReadResult::Preview(preview)) => {
                if a.component_picker
                    && a.placement_confirmation == Some(a.session)
                    && a.preview
                        .as_ref()
                        .is_none_or(|p| p.source_digest != preview.source_digest)
                {
                    a.preview = None;
                    a.placement_confirmation = None;
                    a.refusal = Some("Component source changed since preview; refresh the source and choose again".into());
                } else {
                    a.preview = Some(preview);
                }
            }
            Err(reason) => {
                if a.component_picker {
                    a.preview = None;
                }
                a.placement_confirmation = None;
                a.refusal = Some(reason);
            }
        }
        let confirmed = a.take_placement_confirmation();
        let component = a.component_picker;
        let native_part = a.is_native_part();
        self.preview_existing_project_symbol();
        if confirmed {
            if component {
                self.finish_component_picker_placement();
            } else if catalog_arrived {
                self.confirm_acquisition_search();
            } else if !native_part {
                self.acquire_symbol_for_placement();
            }
        }
        self.invalidate_frame();
        true
    }

    /// Enter confirms the first usable match. A slow catalog keeps this explicit
    /// intent only until its session is retired by a source/query edit or Cancel.
    pub(super) fn confirm_acquisition_search(&mut self) -> bool {
        let Some(a) = &mut self.session.workspace_mut().library.acquisition else {
            return false;
        };
        if a.placement_pane.is_none() {
            return false;
        }
        if a.loading && !a.loaded {
            a.placement_confirmation = Some(a.session);
            return true;
        }
        let first = a.results().into_iter().find(|index| {
            a.sources
                .get(*index)
                .is_some_and(|source| !source.is_unavailable())
        });
        if let Some(index) = first {
            return self.activate_acquisition(&C::AcquireResult(index));
        }
        a.refusal = Some("No usable symbol matches this search".into());
        self.invalidate_frame();
        true
    }

    pub(super) fn activate_acquisition(&mut self, control: &C) -> bool {
        match control {
            C::ProjectSymbols => {
                if self
                    .workspace()
                    .library
                    .acquisition
                    .as_ref()
                    .is_some_and(|a| a.is_native_part())
                {
                    return true;
                }
                return self.browse_project_symbols_for_placement();
            }
            C::Acquire | C::BrowseBundled => {
                if !self.native_edit_available() || self.check_work.pending() {
                    self.log_console_refusal(ConsoleFeedbackSource::Tool, "Open a native Project and finish its check job before acquiring library content");
                    return true;
                }
                let state = self.workspace();
                let mut acquisition = LibraryAcquisition::begin(
                    state.scene.source_revision.clone(),
                    state
                        .supervision
                        .journal
                        .accepted_transaction_tip
                        .as_ref()
                        .and_then(|s| s.parse().ok()),
                );
                if *control == C::Acquire
                    && datum_gui_protocol::library_browser::KINDS
                        .get(state.library.kind)
                        .is_some_and(|(kind, _)| *kind == "parts")
                {
                    acquisition.use_native_part_source();
                } else if *control == C::Acquire
                    && datum_gui_protocol::library_browser::KINDS
                        .get(state.library.kind)
                        .is_some_and(|(kind, _)| {
                            matches!(*kind, "footprints" | "packages" | "padstacks")
                        })
                {
                    acquisition.use_footprint_source();
                }
                if *control == C::BrowseBundled {
                    acquisition.source = datum_gui_protocol::library_browser::acquisition::AcquisitionSource::BundledSymbols;
                }
                if acquisition.bundled() {
                    acquisition.path.clear();
                }
                acquisition.loading = !acquisition.is_native_part();
                if acquisition.loading {
                    self.library_acquisition_work.start(&acquisition, None);
                }
                let browser = &mut self.session.workspace_mut().library;
                browser.acquisition = Some(acquisition);
                browser.focus = Some(C::AcquireSearch);
                browser.scroll_offset = 0;
            }
            C::AcquirePath | C::AcquireSearch => {
                if let Some(region) = self
                    .presented_hits
                    .regions()
                    .iter()
                    .find(|region| region.target == HitTarget::Library(control.clone()))
                    .cloned()
                    && let Some((x, _)) = self.last_cursor_pos
                {
                    self.start_dialog_editor_pointer(region, x, self.scale_factor);
                    self.finish_dialog_editor_pointer();
                }
            }
            C::AcquireLoad | C::AcquireResult(_) => {
                let Some(a) = &mut self.session.workspace_mut().library.acquisition else {
                    return true;
                };
                let source = if let C::AcquireResult(index) = control {
                    let Some(source) = a.sources.get(*index).cloned() else {
                        return true;
                    };
                    if !a.results().contains(index) {
                        return true;
                    }
                    if a.placement_pane.is_some() && source.is_unavailable() {
                        a.search_changed();
                        a.refusal = Some(if a.component_picker {
                            source.description.clone()
                        } else {
                            "This symbol is unavailable for placement".into()
                        });
                        self.invalidate_frame();
                        return true;
                    }
                    if a.loading
                        && a.selected.as_deref() == Some(source.name.as_str())
                        && a.selected_source.as_ref() == Some(&source.path)
                    {
                        if a.placement_pane.is_some() {
                            a.placement_confirmation = Some(a.session);
                        }
                        return true;
                    }
                    Some(source)
                } else {
                    a.source_changed();
                    None
                };
                a.session = eda_engine::substrate::ObjectId::new_v4();
                a.placement_confirmation =
                    (a.placement_pane.is_some() && source.is_some() && !a.component_picker)
                        .then_some(a.session);
                a.selected = source.as_ref().map(|s| s.name.clone());
                a.selected_source = source.as_ref().map(|s| s.path.clone());
                a.preview = None;
                a.refusal = None;
                a.loading = true;
                self.library_acquisition_work.start(a, source);
            }
            C::AcquirePage(next) => {
                if let Some(a) = &mut self.session.workspace_mut().library.acquisition {
                    a.offset = if *next {
                        (a.offset + 4).min(a.results().len().saturating_sub(1))
                    } else {
                        a.offset.saturating_sub(4)
                    };
                }
            }
            C::AcquireShowUnavailable => {
                if let Some(a) = &mut self.session.workspace_mut().library.acquisition {
                    a.include_unavailable = !a.include_unavailable;
                    a.search_changed();
                }
            }
            C::AcquireImport => {
                if self.component_picker_open()
                    && self
                        .workspace()
                        .library
                        .acquisition
                        .as_ref()
                        .and_then(|a| a.preview.as_ref())
                        .and_then(|p| p.native_part.as_ref())
                        .is_some_and(|part| part.in_project(&self.workspace().library))
                {
                    return self.confirm_component_picker();
                }
                if self
                    .workspace()
                    .library
                    .acquisition
                    .as_ref()
                    .is_some_and(|a| a.placement_pane.is_some())
                {
                    return self.acquire_symbol_for_placement();
                }
                return self.queue_native_edit(
                    datum_gui_protocol::native_edit_client::NativeEditAction::LibraryAcquire,
                );
            }
            _ => return false,
        }
        self.invalidate_frame();
        true
    }
}

fn read_native_parts(
    path: &std::path::Path,
    source: Option<LibrarySourceSymbol>,
) -> Result<ReadResult, String> {
    use eda_engine::api::native_write::library_acquisition::native_part;
    match source {
        None => Ok(ReadResult::Names(LibraryAcquisition::native_part_catalog(
            path,
            native_part::catalog(path).map_err(|e| e.to_string())?,
        ))),
        Some(source) => {
            let id = source
                .name
                .parse()
                .map_err(|_| "Native Part identity unavailable")?;
            let snapshot = native_part::preview(&source.path, id).map_err(|e| e.to_string())?;
            Ok(ReadResult::Preview(
                LibraryAcquisition::native_part_preview(&snapshot)?,
            ))
        }
    }
}
