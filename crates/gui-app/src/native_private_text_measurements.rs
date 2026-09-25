//! Optional bounded PM047 call delivery. Disabled runs allocate no event buffers,
//! timer, redraw or file output. These receipts alone are not memory qualification.
use anyhow::{Context, Result};
use datum_gui_render::cpu_alloc::calls::observation::{self, Batch, Phase};
use serde_json::json;
use std::{
    fs::File,
    io::Write,
    sync::{
        Mutex,
        atomic::{AtomicBool, Ordering},
    },
};

static ENABLED: AtomicBool = AtomicBool::new(false);
static WRITER: Mutex<Option<Writer>> = Mutex::new(None);
struct Writer {
    file: File,
    id: u64,
    incomplete: bool,
}

pub(crate) fn start() -> Result<()> {
    let Some(path) = std::env::var_os("DATUM_PRIVATE_TEXT_TRACE") else {
        return Ok(());
    };
    let capacity = match std::env::var("DATUM_PRIVATE_TEXT_TRACE_CAPACITY") {
        Ok(value) => value
            .parse::<usize>()
            .context("invalid private-text trace capacity")?,
        Err(std::env::VarError::NotPresent) => 8192,
        Err(error) => return Err(error.into()),
    };
    let file = File::options()
        .write(true)
        .create_new(true)
        .open(path)
        .context("create private-text observation file")?;
    let id = observation::start(capacity)?;
    let mut writer = Writer {
        file,
        id,
        incomplete: false,
    };
    writer.line(json!({"phase":"start", "observation_id":id, "pid":std::process::id(),
        "capacity":capacity, "scope":"private call events; not complete CPU/GPU/RSS accounting",
        "storage":"at most two event buffers during drain; allocator/JSON/output overhead and RSS separate"}))?;
    *WRITER.lock().unwrap_or_else(|e| e.into_inner()) = Some(writer);
    ENABLED.store(true, Ordering::Release);
    Ok(())
}

impl Writer {
    fn line(&mut self, value: serde_json::Value) -> Result<()> {
        let result = serde_json::to_writer(&mut self.file, &value)
            .map_err(anyhow::Error::from)
            .and_then(|()| self.file.write_all(b"\n").map_err(Into::into));
        if result.is_err() {
            self.incomplete = true;
        }
        result
    }
    fn batch(&mut self, batch: Batch) -> Result<()> {
        self.incomplete |= batch.dropped_events != 0 || batch.first_call_id != 1;
        for event in batch.events {
            self.incomplete |= event.phase == Phase::Abandoned;
            let r = event.report;
            self.line(json!({
                "phase":"call", "observation_id":batch.observation_id,
                "sequence":event.sequence, "elapsed_ns":event.elapsed_ns,
                "transition":format!("{:?}", event.phase),
                "renderer_origin":event.renderer_id, "owner_label":event.owner_label,
                "host_limit_bytes":event.host_limit_bytes, "process_limit_bytes":event.process_limit_bytes,
                "excluded_bytes":event.excluded_bytes, "credited_bytes":event.credited_bytes,
                "requested_retained_bytes":event.requested_retained_bytes,
                "report": {"call_id":r.call_id,"owner_id":r.owner_id,"allocator_installed":r.allocator_installed,
                    "host_initial_bytes":r.host_initial_bytes,"host_final_bytes":r.host_final_bytes,
                    "process_initial_bytes":r.process_initial_bytes,"process_final_bytes":r.process_final_bytes,
                    "initial_bytes":r.initial_bytes,"final_bytes":r.final_bytes,"peak_bytes":r.peak_bytes,
                    "host_peak_bytes":r.host_peak_bytes,"process_peak_bytes":r.process_peak_bytes,"exceeded":r.exceeded}
            }))?;
        }
        self.line(
            json!({"phase":"batch", "observation_id":batch.observation_id,
            "first_call_id":batch.first_call_id,"total_events":batch.total_events,"dropped_events":batch.dropped_events,
            "active_calls":batch.active_calls,"capacity":batch.capacity,
            "buffer_capacity_bytes":batch.buffer_capacity_bytes,"incomplete":self.incomplete}),
        )
    }
}

pub(crate) fn poll() -> Result<()> {
    if !ENABLED.load(Ordering::Acquire) {
        return Ok(());
    }
    let mut guard = WRITER.lock().unwrap_or_else(|e| e.into_inner());
    let writer = guard.as_mut().context("private-text writer missing")?;
    if !observation::has_pending(writer.id)? {
        return Ok(());
    }
    writer.batch(observation::drain(writer.id)?)?;
    anyhow::ensure!(!writer.incomplete, "private-text observation incomplete");
    Ok(())
}

pub(crate) fn finish(event_loop_ok: bool) -> Result<()> {
    if !ENABLED.swap(false, Ordering::AcqRel) {
        return Ok(());
    }
    let mut writer = WRITER
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .take()
        .context("private-text writer missing")?;
    writer.batch(observation::stop(writer.id)?)?;
    writer.incomplete |= !event_loop_ok;
    writer.line(json!({"phase":"end", "observation_id":writer.id,
        "event_loop_ok":event_loop_ok,"complete_delivery":!writer.incomplete}))?;
    writer.file.flush()?;
    anyhow::ensure!(!writer.incomplete, "private-text observation incomplete");
    Ok(())
}
