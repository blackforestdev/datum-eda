//! Output-only observation: retain inputs and semantic transitions, never GPU demand tags.
use super::*;

#[derive(Default)]
pub(super) struct Diagnostic {
    pub(super) pending: Option<Record>,
    pub(super) failure: Option<&'static str>,
    last: Option<State>,
    retain: bool,
}

pub(super) fn enabled() -> Result<bool> {
    let active = match std::env::var("DATUM_OUTPUT_DIAGNOSTIC").as_deref() {
        Err(std::env::VarError::NotPresent) | Ok("0") => false,
        Ok("1") => true,
        _ => anyhow::bail!("invalid output diagnostic mode"),
    };
    validate_options(
        active,
        std::env::var("DATUM_GPU_MEASUREMENTS").ok().as_deref(),
        std::env::var_os("DATUM_WORKLOAD_MANIFEST").is_some(),
    )?;
    Ok(active)
}
fn validate_options(active: bool, gpu: Option<&str>, workload: bool) -> Result<()> {
    ensure!(
        !active || (gpu == Some("0") && !workload),
        "output diagnostic requires GPU measurements=0 and no workload manifest"
    );
    Ok(())
}
fn kind(event: Option<&WindowEvent>) -> (&'static str, bool) {
    match event {
        Some(WindowEvent::CursorMoved { .. }) => ("pointer", true),
        Some(WindowEvent::MouseInput { .. }) => ("button", true),
        Some(WindowEvent::CursorEntered { .. }) => ("cursor_enter", true),
        Some(WindowEvent::CursorLeft { .. }) => ("cursor_leave", true),
        Some(WindowEvent::Focused(true)) => ("focus_gain", true),
        Some(WindowEvent::Focused(false)) => ("focus_loss", true),
        Some(WindowEvent::Occluded(true)) => ("occluded", true),
        Some(WindowEvent::Occluded(false)) => ("exposed", true),
        Some(WindowEvent::Resized(_)) => ("resize", true),
        Some(WindowEvent::ScaleFactorChanged { .. }) => ("scale", true),
        Some(WindowEvent::CloseRequested) => ("close", true),
        Some(WindowEvent::RedrawRequested) => ("redraw", false),
        Some(_) => ("native_event", false),
        None => ("native_round", false),
    }
}
impl Record {
    pub(super) fn value(&self, sequence: usize) -> Value {
        json!({"sequence":sequence,"workload":self.workload,"workload_ns":self.workload_ns,
            "demand_kind":self.demand_kind,"received_ns":self.received_ns,"completed_ns":self.completed_ns,
            "position":self.position,"button":self.button,"route":self.route,
            "before":self.before.value(),"after":self.after.map(|s|s.value())})
    }
}
impl InputObservation {
    pub(super) fn begin_diagnostic(&mut self, event: Option<&WindowEvent>, before: State) -> bool {
        let d = self.diagnostic.as_mut().unwrap();
        if d.failure.is_some() {
            return false;
        }
        if d.pending.is_some() {
            d.failure = Some("nested diagnostic dispatch");
            return false;
        }
        if d.last.is_some_and(|last| !last.semantic_eq(&before)) {
            d.failure = Some("unobserved semantic state transition");
        }
        let (demand_kind, retain) = kind(event);
        let position = match event {
            Some(WindowEvent::CursorMoved { position, .. }) => Some([position.x, position.y]),
            _ => None,
        };
        let button = match event {
            Some(WindowEvent::MouseInput { button, state, .. }) => {
                let id = match button {
                    MouseButton::Left => 1,
                    MouseButton::Middle => 2,
                    MouseButton::Right => 3,
                    MouseButton::Back => 4,
                    MouseButton::Forward => 5,
                    MouseButton::Other(n) => u32::from(*n) + 6,
                };
                Some((id, *state == ElementState::Pressed))
            }
            _ => None,
        };
        d.retain = retain;
        d.pending = Some(Record {
            workload: [0; 3],
            workload_ns: None,
            demand_kind,
            received_ns: self.started.elapsed().as_nanos(),
            completed_ns: None,
            position,
            button,
            route: "unhandled",
            before,
            after: None,
        });
        if before.truncated {
            d.failure.get_or_insert("truncated diagnostic state");
        }
        if retain && self.records.len() == LIMIT {
            self.overflow = true;
            d.failure
                .get_or_insert("diagnostic record capacity exhausted");
        }
        d.failure.is_none()
    }
    pub(super) fn complete_diagnostic(&mut self, after: Option<State>) {
        let d = self.diagnostic.as_mut().unwrap();
        let Some(mut record) = d.pending.take() else {
            return;
        };
        record.completed_ns = Some(self.started.elapsed().as_nanos());
        record.after = after;
        let Some(after) = after else {
            d.failure.get_or_insert("diagnostic runtime lost");
            d.pending = Some(record);
            return;
        };
        if after.truncated {
            d.failure.get_or_insert("truncated diagnostic state");
        }
        d.last = Some(after);
        if d.retain || !record.before.semantic_eq(&after) {
            if self.records.len() == LIMIT {
                self.overflow = true;
                d.failure
                    .get_or_insert("diagnostic record capacity exhausted");
                d.pending = Some(record);
            } else {
                self.records.push(record);
            }
        }
    }
    fn diagnostic_report(&self, final_state: Option<State>) -> Value {
        let d = self.diagnostic.as_ref().unwrap();
        let final_gap = final_state
            .zip(d.last)
            .is_some_and(|(a, b)| !a.semantic_eq(&b));
        json!({"schema":"datum.input-receipt/v1","mode":"output-diagnostic","pid":std::process::id(),
            "complete":false,"coverage_complete":self.complete_receipt() && final_state.is_some_and(|s|!s.truncated) && !final_gap,
            "first_error":d.failure.or(if final_gap {Some("unobserved final state transition")}else{None}),
            "gpu_drained":null,"drain_status":"not attempted: pre-drain diagnostic snapshot",
            "monotonic_origin_ns":self.monotonic_origin_ns,"snapshot_ns":self.started.elapsed().as_nanos(),
            "overflow":self.overflow,"record_limit":LIMIT,"record_count":self.records.len(),
            "record_storage_bytes":self.records.capacity()*std::mem::size_of::<Record>(),
            "provisional_storage_bytes":std::mem::size_of::<Record>(),
            "records":self.records.iter().enumerate().map(|(i,r)|r.value(i)).collect::<Vec<_>>(),
            "pending":d.pending.as_ref().map(|r|r.value(self.records.len())),"final_state":final_state.map(|s|s.value())})
    }
    fn write_diagnostic(&self, final_state: Option<State>) -> Result<()> {
        let report = self.diagnostic_report(final_state);
        std::fs::write(
            self.path.with_file_name("input-diagnostic.json"),
            serde_json::to_vec(&report)?,
        )
        .context("export pre-drain input diagnostic")
    }
}
impl App {
    pub(crate) fn export_output_diagnostic(&self) -> Result<()> {
        if let Some(o) = &self.input_observation
            && o.diagnostic.is_some()
        {
            o.write_diagnostic(self.runtime.as_ref().map(State::capture))?;
        }
        Ok(())
    }
    pub(crate) fn check_output_diagnostic(&self, event_loop: &ActiveEventLoop) {
        if let Some(error) = self
            .input_observation
            .as_ref()
            .and_then(|o| o.diagnostic.as_ref())
            .and_then(|d| d.failure)
        {
            if let Err(export) = self.export_output_diagnostic() {
                eprintln!("input diagnostic export failed: {export:#}");
            }
            crate::app_shell::fatal_gui_error(event_loop, "output diagnostic incomplete", error);
        }
    }
}
#[cfg(test)]
#[path = "native_output_diagnostic_tests.rs"]
mod tests;
