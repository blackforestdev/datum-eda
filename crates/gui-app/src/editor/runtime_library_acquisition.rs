//! Background local-file reads with disposable, origin-qualified publication.
use super::*;
use datum_gui_protocol::library_browser::acquisition::{
    LibraryAcquisition, LibraryAcquisitionPreview, LibrarySourceSymbol,
    catalog::{self, SourceCatalog},
};
use std::sync::mpsc;

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
    pending: Option<mpsc::Receiver<Answer>>,
    wake: winit::event_loop::EventLoopProxy<()>,
}
impl AcquisitionWork {
    pub(crate) fn new(wake: winit::event_loop::EventLoopProxy<()>) -> Self {
        Self {
            pending: None,
            wake,
        }
    }
    pub(crate) fn cancel(&mut self) {
        self.pending = None;
    }
    fn start(&mut self, acquisition: &LibraryAcquisition, source: Option<LibrarySourceSymbol>) {
        let (sender, receiver) = mpsc::channel();
        self.pending = Some(receiver);
        let session = acquisition.session;
        let revision = acquisition.revision.clone();
        let path = std::path::PathBuf::from(&acquisition.path);
        let bundled = acquisition.bundled;
        let wake = self.wake.clone();
        std::thread::spawn(move || {
            let result = (|| {
                if let Some(source) = source {
                    let converted = if bundled {
                        eda_engine::import::library_bundle::preview(&source.path, &source.name)
                    } else {
                        eda_engine::import::kicad_library::convert(&source.path, &source.name)
                    }
                    .map_err(|error| error.to_string())?;
                    Ok(ReadResult::Preview(LibraryAcquisitionPreview {
                        name: source.name,
                        source_digest: converted.source_hash,
                        source_path: converted.source_path,
                        unit: serde_json::to_value(converted.unit).map_err(|e| e.to_string())?,
                        symbol: serde_json::to_value(converted.symbol)
                            .map_err(|e| e.to_string())?,
                        notices: converted.warnings,
                    }))
                } else if bundled {
                    catalog::bundled().map(ReadResult::Names)
                } else {
                    catalog::discover(&path).map(ReadResult::Names)
                }
            })();
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
            a.refusal =
                Some("Project changed during the source read; cancel and acquire again".into());
            self.invalidate_frame();
            return true;
        }
        a.loading = false;
        match answer.result {
            Ok(ReadResult::Names(catalog)) => {
                a.names = catalog.symbols.iter().map(|s| s.label.clone()).collect();
                a.sources = catalog.symbols;
                a.catalog_notices = catalog.notices;
                a.loaded = true;
            }
            Ok(ReadResult::Preview(preview)) => a.preview = Some(preview),
            Err(reason) => a.refusal = Some(reason),
        }
        self.preview_existing_project_symbol();
        self.invalidate_frame();
        true
    }

    pub(super) fn activate_acquisition(&mut self, control: &C) -> bool {
        match control {
            C::Acquire | C::BrowseBundled => {
                if !self.native_edit_available() || self.check_work.pending() {
                    self.log_console_refusal(ConsoleFeedbackSource::Tool, "Open a native Project and finish its check job before acquiring a library symbol");
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
                acquisition.bundled = *control == C::BrowseBundled;
                if acquisition.bundled {
                    acquisition.path.clear();
                }
                acquisition.loading = true;
                self.library_acquisition_work.start(&acquisition, None);
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
                    Some(source)
                } else {
                    a.source_changed();
                    None
                };
                a.session = eda_engine::substrate::ObjectId::new_v4();
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
            C::AcquireImport => {
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
