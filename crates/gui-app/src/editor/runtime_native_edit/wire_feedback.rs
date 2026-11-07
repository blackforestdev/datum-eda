//! Successful Move plans publish source-qualified, transient handle feedback.
use super::*;
use datum_gui_protocol::{ConsoleFeedbackCategory, ConsoleFeedbackDraft, ConsoleFeedbackSeverity};
use datum_gui_protocol::{MoveHandleFeedback, MoveHandleKind, SelectionTarget};

impl Runtime {
    pub(crate) fn clear_wire_handle_hint(&mut self) -> bool {
        if !self.workspace().ui.console.latest().is_some_and(|last| {
            last.category == ConsoleFeedbackCategory::ToolPrompt
                && last.message.starts_with("Handle: ")
        }) {
            return false;
        }
        self.publish_console_feedback(ConsoleFeedbackDraft::tool_prompt(
            ConsoleFeedbackSource::Tool,
            crate::console_feedback::occurred_unix_ms(),
            format!("{} · choose an object", self.workspace().tool.label()),
        ));
        true
    }

    /// Hover reads the renderer's prepared handle snapshot, never the resolver.
    pub(crate) fn refresh_wire_handle_hint(&mut self, pos: (f32, f32)) -> bool {
        if self
            .workspace()
            .ui
            .console
            .visible_latest()
            .is_some_and(|last| {
                matches!(
                    last.severity,
                    ConsoleFeedbackSeverity::Error
                        | ConsoleFeedbackSeverity::Warning
                        | ConsoleFeedbackSeverity::Success
                )
            })
        {
            return false;
        }
        let kind = (|| {
            let prepared = self.renderer.render_session().prepared()?;
            let pane = self.pane_at_screen(pos.0, pos.1)?;
            if !prepared.has_edit_handles(self.workspace(), pane) {
                return None;
            }
            if let Some(kind) = prepared.edit_handle_at_screen(self.workspace(), pos.0, pos.1) {
                return Some(kind);
            }
            let state = self.workspace();
            let SelectionTarget::Native(view) = &state.selection else {
                return None;
            };
            if !matches!(&view.subject, datum_gui_protocol::SelectionSubject::Object(id)
                if id.class == datum_gui_protocol::AuthoredSelectionClass::Wire)
            {
                return None;
            }
            let hover = state.ui.hovered_object.as_ref()?;
            let scene = datum_gui_protocol::camera_scene_for_pane(state, pane)?;
            (hover.surface == PaneContent::Schematic
                && view.selected_key(scene, PaneContent::Schematic, &hover.object_id))
            .then_some(MoveHandleKind::Segment)
        })();
        let Some(kind) = kind else {
            return self.clear_wire_handle_hint();
        };
        let detail = match kind {
            MoveHandleKind::Endpoint => {
                "wire end · press to reconnect this end; opposite end stays fixed"
            }
            MoveHandleKind::Vertex => "wire vertex · press to reshape incident spans",
            MoveHandleKind::Junction => "junction node · press to preserve its connected spans",
            MoveHandleKind::Segment => "wire span · press to reshape with fixed outer contacts",
        };
        let message = format!("Handle: {detail}");
        if self
            .workspace()
            .ui
            .console
            .latest()
            .is_some_and(|last| last.message == message)
        {
            return false;
        }
        self.publish_console_feedback(ConsoleFeedbackDraft::tool_prompt(
            ConsoleFeedbackSource::Tool,
            crate::console_feedback::occurred_unix_ms(),
            message,
        ));
        true
    }

    pub(crate) fn publish_move_handle_feedback(&mut self, position: PointNm) {
        let feedback = (|| {
            let preview = self.native_edit.preview.as_ref()?;
            let (kind, source) = preview.wire_handle_source()?;
            let basis = self.workspace().authoring.gesture.view_basis.clone()?;
            if !basis.is_current(self.workspace(), basis.pane) {
                return None;
            }
            let snap_target = if kind == MoveHandleKind::Endpoint {
                self.schematic_pin_snap_point_in(position, basis.pane)
                    .ok()
                    .flatten()
                    .filter(|point| *point == position)
            } else {
                None
            };
            Some(MoveHandleFeedback {
                basis,
                kind,
                source,
                destination: position,
                snap_target,
            })
        })();
        let notice_changed = match (&self.workspace().authoring.handle_feedback, &feedback) {
            (Some(old), Some(new)) => {
                old.kind != new.kind
                    || old.source != new.source
                    || old.snap_target != new.snap_target
            }
            (None, Some(_)) => true,
            _ => false,
        };
        if notice_changed && let Some(feedback) = &feedback {
            let subject = match feedback.kind {
                MoveHandleKind::Endpoint => "wire end",
                MoveHandleKind::Vertex => "wire vertex",
                MoveHandleKind::Junction => "junction node",
                MoveHandleKind::Segment => "wire span",
            };
            self.log_console_echo(
                ConsoleFeedbackSource::Tool,
                format!(
                    "Move {subject}{} · click applies · Esc cancels",
                    if feedback.snap_target.is_some() {
                        " · exact pin contact"
                    } else {
                        ""
                    }
                ),
            );
        }
        self.session.workspace_mut().authoring.handle_feedback = feedback;
    }
}
