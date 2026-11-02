//! Bus entries configure the existing name/choice gesture with qualified Bus and wire choices.
use super::*;
use datum_gui_protocol::schematic_authoring::{BusEntryDraft, LabelDraft, selected};
use eda_engine::api::native_write::schematic_bus::WireEnd;

impl Runtime {
    pub(crate) fn begin_native_bus_entry(&mut self) -> bool {
        let prepare = (|| {
            self.selected_schematic_idle()?;
            let (basis, wire) = selected::target(self.workspace())?;
            if wire.class != datum_gui_protocol::AuthoredSelectionClass::Wire {
                return Err("Select an existing scalar member Wire first".into());
            }
            let sheet = self
                .workspace()
                .schematic_authoring
                .source
                .as_ref()
                .and_then(|s| s.sheets.get(&basis.source))
                .ok_or("Member Wire source unavailable")?;
            let original = sheet
                .wires
                .get(&wire.id)
                .cloned()
                .ok_or("Member Wire source unavailable")?;
            let mut buses: Vec<_> = sheet.buses.values().cloned().collect();
            buses.sort_by(|a, b| a.name.cmp(&b.name).then_with(|| a.uuid.cmp(&b.uuid)));
            if buses.is_empty() {
                return Err("Draw a Bus with this scalar net declared as a member first".into());
            }
            let tip = self
                .workspace()
                .supervision
                .journal
                .accepted_transaction_tip
                .as_ref()
                .and_then(|t| t.parse().ok());
            Ok::<_, String>(BusEntryDraft {
                wire,
                basis,
                tip,
                original,
                buses,
                choice: 0,
                wire_end: WireEnd::From,
            })
        })();
        let draft = match prepare {
            Ok(draft) => draft,
            Err(reason) => {
                self.log_console_refusal(ConsoleFeedbackSource::Tool, reason);
                return true;
            }
        };
        if !self.prepare_native_schematic(false) {
            return true;
        }
        let e = &mut self.session.workspace_mut().schematic_authoring;
        e.chosen = None;
        let mut form = LabelDraft::new(eda_engine::schematic::LabelKind::Local);
        form.name = draft.bus().unwrap().name.clone();
        e.label = Some(form);
        e.bus_entry = Some(draft);
        e.refusal = None;
        self.invalidate_frame();
        true
    }
    pub(super) fn cycle_bus_entry_choice(&mut self, endpoint: bool) -> bool {
        let e = &mut self.session.workspace_mut().schematic_authoring;
        let Some(entry) = &mut e.bus_entry else {
            return false;
        };
        if endpoint {
            entry.cycle_end();
        } else {
            entry.choice = (entry.choice + 1) % entry.buses.len();
        }
        if let Some(form) = &mut e.label {
            form.name = entry.bus().unwrap().name.clone();
            form.focus = usize::from(endpoint);
        }
        e.refusal = None;
        e.preview_dirty = true;
        self.invalidate_frame();
        true
    }
    pub(super) fn place_native_bus_entry(&mut self, at: PointNm) -> bool {
        let request = (|| {
            let e = &self.workspace().schematic_authoring;
            let draft = e.bus_entry.as_ref().ok_or("No Bus Entry is open")?;
            let (basis, wire) = selected::target(self.workspace())?;
            let tip = self
                .workspace()
                .supervision
                .journal
                .accepted_transaction_tip
                .as_ref()
                .and_then(|t| t.parse().ok());
            if basis != draft.basis || wire != draft.wire || tip != draft.tip {
                return Err("Member Wire, source or history changed; reopen Bus Entry".into());
            }
            let sheet = e
                .source
                .as_ref()
                .and_then(|s| s.sheets.get(&basis.source))
                .ok_or("Member Wire source unavailable")?;
            draft.request(at, sheet)
        })();
        match request {
            Ok(request) => {
                let e = &mut self.session.workspace_mut().schematic_authoring;
                e.position = Some(at);
                e.preview_dirty = true;
                e.pending_request = Some(request);
                self.prepare_native_schematic_frame();
                self.queue_native_edit(NativeEditAction::Schematic)
            }
            Err(reason) => {
                self.session.workspace_mut().schematic_authoring.refusal = Some(reason);
                self.invalidate_frame();
                true
            }
        }
    }
}
