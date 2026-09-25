//! Optional allocation-lifetime delivery. No timer or redraw is introduced.
//! Registration capacity, reserved peaks, driver residency and RSS are distinct.
use anyhow::{Context, Result};
use datum_gui_render::gpu_allocation_observation::{self as observation, Batch};
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
    let Some(path) = std::env::var_os("DATUM_GPU_ALLOCATION_TRACE") else {
        return Ok(());
    };
    let capacity = match std::env::var("DATUM_GPU_ALLOCATION_TRACE_CAPACITY") {
        Ok(value) => value
            .parse::<usize>()
            .context("invalid GPU-allocation trace capacity")?,
        Err(std::env::VarError::NotPresent) => 8192,
        Err(error) => return Err(error.into()),
    };
    let file = File::options()
        .write(true)
        .create_new(true)
        .open(path)
        .context("create GPU-allocation observation file")?;
    let id = observation::start(capacity)?;
    let mut writer = Writer {
        file,
        id,
        incomplete: false,
    };
    writer.line(json!({"phase":"start", "observation_id":id, "pid":std::process::id(),
        "capacity":capacity, "scope":"tracked allocation history; API registration follows creation; not driver residency or complete memory qualification",
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
        self.incomplete |=
            batch.dropped_events != 0 || batch.first_identity_id != 1 || batch.invalid_accounting;
        for event in batch.events {
            let r = event.allocation;
            let upload = r.last_upload.map(|u| {
                json!({
                    "owner":u.owner, "attempt":u.attempt,
                    "source_bytes":u.source_bytes, "transfer_bytes":u.transfer_bytes,
                    "consumers":u.consumers.iter().map(|c| c.adoption_id()).collect::<Vec<_>>()
                })
            });
            self.line(json!({
                "phase":"allocation", "observation_id":batch.observation_id,
                "sequence":event.sequence,"elapsed_ns":event.elapsed_ns,
                "transition":format!("{:?}",event.transition),
                "record":{"id":r.id,"owner":r.owner,"renderer_origin":r.renderer_id,
                    "generation":r.generation,"kind":format!("{:?}",r.kind),
                    "capacity_bytes":r.bytes,"requested_bytes":r.requested_bytes,
                    "consumers":r.consumers.iter().map(|c| c.adoption_id()).collect::<Vec<_>>(),"prepared_consumers":r.prepared_consumers.iter().map(|c| c.adoption_id()).collect::<Vec<_>>(),
                    "submitted_consumers":r.submitted_consumers.iter().map(|c| c.adoption_id()).collect::<Vec<_>>(),
                    "prepared_references":r.prepared_references,"submission_references":r.submission_references,
                    "retiring":r.retiring,"retirement_reason":r.retirement_reason.map(|reason|format!("{reason:?}")),
                    "submitted_source_bytes":r.submitted_source_bytes,
                    "submitted_transfer_bytes":r.submitted_transfer_bytes,"last_upload":upload}
            }))?;
        }
        let peaks = observation::reservation_peaks();
        self.line(json!({"phase":"batch","observation_id":batch.observation_id,
            "first_identity_id":batch.first_identity_id,"total_events":batch.total_events,
            "dropped_events":batch.dropped_events,"invalid_accounting":batch.invalid_accounting,
            "active_allocations":batch.active_allocations,"tracked_capacity_bytes":batch.tracked_capacity_bytes,
            "tracked_capacity_peak_bytes":batch.tracked_capacity_peak_bytes,
            "reservation_lifetime_peaks":{"gpu_process":peaks.gpu_process,"atlas_process":peaks.atlas_process,
                "staging_process":peaks.staging_process,"terminal_process":peaks.terminal_process},
            "capacity":batch.capacity,"buffer_capacity_bytes":batch.buffer_capacity_bytes,"incomplete":self.incomplete}))
    }
}

pub(crate) fn poll() -> Result<()> {
    if !ENABLED.load(Ordering::Acquire) {
        return Ok(());
    }
    let mut guard = WRITER.lock().unwrap_or_else(|e| e.into_inner());
    let writer = guard.as_mut().context("GPU-allocation writer missing")?;
    if !observation::has_pending(writer.id)? {
        return Ok(());
    }
    writer.batch(observation::drain(writer.id)?)?;
    anyhow::ensure!(!writer.incomplete, "GPU-allocation observation incomplete");
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
        .context("GPU-allocation writer missing")?;
    writer.batch(observation::stop(writer.id)?)?;
    writer.incomplete |= !event_loop_ok;
    writer.line(json!({"phase":"end", "observation_id":writer.id,
        "event_loop_ok":event_loop_ok,"complete_delivery":!writer.incomplete}))?;
    writer.file.flush()?;
    anyhow::ensure!(!writer.incomplete, "GPU-allocation observation incomplete");
    Ok(())
}
