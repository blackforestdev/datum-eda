//! Native edit worker lifecycle and shared client ownership.
use super::*;

pub(crate) struct EditWork {
    pub preview: Option<NativeMovePreview>,
    pub(super) previous_context_tool: Option<WorkspaceTool>,
    pub(super) preview_tip: Option<String>,
    pub(super) pending_position: Option<PointNm>,
    pub(super) exact_position: Option<PointNm>,
    pub(super) current_position: Option<PointNm>,
    pub(super) exact_cursor: Option<(f32, f32)>,
    pub(super) coordinate_escape_release: bool,
    pub(super) preview_awaiting_presentation: bool,
    pub(super) pending_move_base: Option<(bool, datum_gui_protocol::BoardReviewSceneV1)>,
    pub(super) pending: Option<mpsc::Receiver<Answer>>,
    pub(super) library_pending: Option<mpsc::Receiver<library_revisions::PreviewReply>>,
    /// Startup verb catalog from the warmup describe, fed back through the wake
    /// so native-edit menus reflect engine capability from cold open rather than
    /// only after the first edit populates it.
    pub(super) catalog_pending: Option<mpsc::Receiver<catalog::CatalogReply>>,
    pub(super) client: Arc<Mutex<Option<NativeProjectClient>>>,
    pub(super) wake: winit::event_loop::EventLoopProxy<()>,
    pub blocked: bool,
}
impl EditWork {
    pub fn new(wake: winit::event_loop::EventLoopProxy<()>) -> Self {
        Self {
            preview: None,
            previous_context_tool: None,
            preview_tip: None,
            pending_position: None,
            exact_position: None,
            current_position: None,
            exact_cursor: None,
            coordinate_escape_release: false,
            preview_awaiting_presentation: false,
            pending_move_base: None,
            pending: None,
            library_pending: None,
            catalog_pending: None,
            client: Arc::new(Mutex::new(None)),
            wake,
            blocked: false,
        }
    }
    pub(crate) fn reset_project(&mut self) {
        debug_assert!(!self.pending());
        self.preview = None;
        self.previous_context_tool = None;
        self.preview_tip = None;
        self.pending_position = None;
        self.exact_position = None;
        self.current_position = None;
        self.exact_cursor = None;
        self.coordinate_escape_release = false;
        self.preview_awaiting_presentation = false;
        self.pending_move_base = None;
        self.blocked = false;
        self.catalog_pending = None;
        self.library_pending = None;
    }
    pub fn pending(&self) -> bool {
        self.pending.is_some() || self.library_pending.is_some()
    }
    pub(super) fn reconcile_again(
        &mut self,
        mut answer: Answer,
        inputs: native_edit_selection::Inputs,
    ) -> Result<()> {
        let (sender, receiver) = mpsc::sync_channel(1);
        let client = self.client.clone();
        let wake = self.wake.clone();
        std::thread::Builder::new()
            .name("datum-edit-selection".into())
            .spawn(move || {
                let result = (|| -> Result<()> {
                    let next = answer.next.as_mut().context("Edit scene unavailable")?;
                    // Read-only superseding projection. The committed mutation is
                    // never resent when input selection changed during preparation.
                    let prepared = answer
                        .selection
                        .take()
                        .context("Qualified selection basis unavailable")?;
                    let model = prepared.session.model.clone();
                    answer.selection = Some(native_edit_selection::prepare_from_model(
                        next,
                        inputs,
                        model,
                        prepared.cache,
                    )?);
                    let owner = client
                        .lock()
                        .map_err(|_| anyhow::anyhow!("Native writer owner unavailable"))?;
                    let current = owner
                        .as_ref()
                        .context("Native writer unavailable")?
                        .describe(&answer.intent.project_root)?;
                    anyhow::ensure!(
                        current
                            .get("model_revision")
                            .and_then(serde_json::Value::as_str)
                            == Some(next.scene.source_revision.as_str())
                            && current
                                .get("transaction_tip")
                                .and_then(serde_json::Value::as_str)
                                == next.supervision.journal.accepted_transaction_tip.as_deref(),
                        "Project changed during selection preparation; reopen before editing"
                    );
                    Ok(())
                })();
                if let Err(error) = result {
                    answer.next = None;
                    answer.selection = None;
                    answer.result = Err(format!("{error:#}"));
                }
                finish_owned_edit(client, sender, answer, || {
                    let _ = wake.send_event(());
                });
            })?;
        self.pending = Some(receiver);
        Ok(())
    }
}
