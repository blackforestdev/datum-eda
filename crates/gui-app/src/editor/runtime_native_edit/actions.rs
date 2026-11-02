//! Native menu dispatch and history admission share the canonical edit queue.
use super::*;
impl Runtime {
    pub(crate) fn activate_native_edit_menu(&mut self, verb: Option<&str>) -> Option<bool> {
        Some(match verb? {
            "datum.schematic.create_sheet" => self.begin_schematic_creation(),
            "datum.schematic.place_symbol" => self.begin_native_schematic(false),
            "datum.schematic.draw_wire" => self.begin_native_schematic(true),
            "datum.schematic.set_symbol_reference" | "datum.schematic.set_symbol_value" => {
                self.begin_selected_symbol_edit()
            }
            verb @ ("datum.schematic.delete_symbol"
            | "datum.schematic.delete_wire"
            | "datum.schematic.delete_label") => self.delete_selected_schematic_verb(verb),
            "datum.schematic.place_label" => {
                self.begin_native_label(eda_engine::schematic::LabelKind::Local)
            }
            "datum.journal.undo" => self.queue_native_history(false),
            "datum.journal.redo" => self.queue_native_history(true),
            "datum.pcb.move_component" => self.set_workspace_tool(WorkspaceTool::Move),
            _ => return None,
        })
    }
    pub(crate) fn queue_native_history(&mut self, redo: bool) -> bool {
        let tip = self
            .workspace()
            .supervision
            .journal
            .accepted_transaction_tip
            .clone();
        let Some(tip) = tip.and_then(|t| t.parse().ok()) else {
            self.log_console_refusal(
                ConsoleFeedbackSource::Tool,
                "No journal transaction is available",
            );
            return true;
        };
        self.queue_native_edit(if redo {
            NativeEditAction::Redo { expected_tip: tip }
        } else {
            NativeEditAction::Undo { expected_tip: tip }
        })
    }
}
