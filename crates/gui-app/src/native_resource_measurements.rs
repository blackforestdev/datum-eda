//! Opt-in sampled ownership views, with reservation observers surviving close.
//! No timer/redraw is added. These overlapping views are not a memory grand total.
use super::App;
use anyhow::{Context, Result};
use datum_gui_render::{
    Renderer, cpu_alloc,
    resource_observation::{
        self, LocalReservationObserver, LocalReservationUsage, ReservationUsage,
    },
};
use serde_json::{Value, json};
use std::{
    fs::File,
    io::Write,
    sync::{
        Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

static ENABLED: AtomicBool = AtomicBool::new(false);
static WRITER: Mutex<Option<Writer>> = Mutex::new(None);
const HOST_LIMIT: usize = 1024;
struct Host {
    label: &'static str,
    window: String,
    observer: LocalReservationObserver,
    present: bool,
    was_present: bool,
    released_last_snapshot: bool,
}
struct Writer {
    file: File,
    hosts: Vec<Host>,
    started: Instant,
    last: Instant,
    interval: Duration,
    sequence: u64,
    limit: u64,
}

pub(crate) fn start() -> Result<()> {
    let Some(path) = std::env::var_os("DATUM_RESOURCE_TRACE") else {
        return Ok(());
    };
    let limit = parameter("DATUM_RESOURCE_TRACE_LIMIT", 65_536, 1_000_000)?;
    let interval =
        Duration::from_millis(parameter("DATUM_RESOURCE_TRACE_INTERVAL_MS", 1000, 60_000)?);
    let file = File::options()
        .write(true)
        .create_new(true)
        .open(path)
        .context("create resource observation file")?;
    let mut hosts = Vec::new();
    hosts.try_reserve_exact(HOST_LIMIT)?;
    let now = Instant::now();
    let mut writer = Writer {
        file,
        hosts,
        started: now,
        last: now,
        interval,
        sequence: 0,
        limit,
    };
    writer.line(json!({"phase":"start","pid":std::process::id(),"snapshot_limit":limit,
        "minimum_interval_ms":interval.as_millis(),"host_limit":HOST_LIMIT,
        "host_slot_capacity_bytes":writer.hosts.capacity()*std::mem::size_of::<Host>(),
        "budget_metadata_bytes_each":LocalReservationObserver::budget_metadata_bytes_each(),
        "semantics":"opportunistic sampled counters on existing event-loop turns, plus lifecycle transitions; no atomic cross-owner or exact API-live peak claim",
        "overlap":"cache, document, font, scoped heap and reservation views overlap; do not sum them; deduplicate budget IDs shared across renderer replacement",
        "observer":"host slots, copied window strings, retained budget metadata, snapshot/JSON/I/O allocations and RSS are separate measurement overhead"}))?;
    *WRITER.lock().unwrap_or_else(|e| e.into_inner()) = Some(writer);
    ENABLED.store(true, Ordering::Release);
    Ok(())
}
fn parameter(name: &str, default: u64, maximum: u64) -> Result<u64> {
    let value = match std::env::var(name) {
        Ok(value) => value
            .parse::<u64>()
            .with_context(|| format!("invalid {name}"))?,
        Err(std::env::VarError::NotPresent) => default,
        Err(error) => return Err(error.into()),
    };
    anyhow::ensure!((1..=maximum).contains(&value), "invalid {name} range");
    Ok(value)
}

fn renderers(
    app: &App,
) -> [(
    &'static str,
    Option<&Renderer>,
    Option<winit::window::WindowId>,
); 4] {
    [
        (
            "MAIN",
            app.runtime.as_ref().map(|r| &r.renderer),
            app.window.as_ref().map(|w| w.id()),
        ),
        (
            "GLOBAL",
            app.global_preferences_surface.as_ref().map(|s| &s.renderer),
            app.global_preferences_window.as_ref().map(|w| w.id()),
        ),
        (
            "PROJECT",
            app.project_preferences_surface
                .as_ref()
                .map(|s| &s.renderer),
            app.project_preferences_window.as_ref().map(|w| w.id()),
        ),
        (
            "NEW",
            app.new_project_surface.as_ref().map(|s| &s.renderer),
            app.new_project_window.as_ref().map(|w| w.id()),
        ),
    ]
}

impl Writer {
    fn line(&mut self, value: Value) -> Result<()> {
        serde_json::to_writer(&mut self.file, &value)?;
        self.file.write_all(b"\n")?;
        Ok(())
    }
    fn update_hosts(&mut self, app: Option<&App>) -> Result<bool> {
        for host in &mut self.hosts {
            host.was_present = host.present;
            host.present = false;
        }
        let mut added = false;
        if let Some(app) = app {
            for (label, renderer, window) in renderers(app) {
                let Some(renderer) = renderer else { continue };
                if let Some(host) = self
                    .hosts
                    .iter_mut()
                    .find(|h| h.observer.renderer_id() == renderer.resource_owner_id())
                {
                    host.present = true;
                } else {
                    anyhow::ensure!(
                        self.hosts.len() < HOST_LIMIT,
                        "resource observation host capacity exhausted"
                    );
                    self.hosts.push(Host {
                        label,
                        window: format!("{:?}", window.context("observed renderer has no window")?),
                        observer: renderer.local_reservation_observer(),
                        present: true,
                        was_present: false,
                        released_last_snapshot: false,
                    });
                    added = true;
                }
            }
        }
        Ok(added || self.hosts.iter().any(|h| h.present != h.was_present))
    }
    fn snapshot(&mut self, app: Option<&App>, final_snapshot: bool) -> Result<()> {
        if self.sequence == self.limit {
            self.line(json!({"phase":"invalid","reason":"snapshot capacity exhausted"}))?;
            anyhow::bail!("resource observation snapshot capacity exhausted");
        }
        let started_ns = self.started.elapsed().as_nanos();
        let mut hosts = Vec::new();
        for host in &mut self.hosts {
            let reservations = host.observer.usage();
            host.released_last_snapshot = reservations.released();
            let renderer = app.and_then(|app| {
                renderers(app).into_iter().find_map(|(_, r, _)| {
                    r.filter(|r| r.resource_owner_id() == reservations.renderer_id)
                })
            });
            hosts.push(json!({"host":host.label,"window":host.window,"present":host.present,
                "reservations":local_reservations(reservations),"renderer":renderer.map(renderer_view)}));
        }
        let scopes: Vec<_> = cpu_alloc::usage().iter().map(scope_view).collect();
        let registry = cpu_alloc::registry_metadata_bytes();
        let text: Vec<_> = Renderer::text_cache_process_usage()
            .iter()
            .map(|u| {
                json!({
            "owner_id":u.owner_id,"bytes":u.bytes,"constructing_bytes":u.constructing_bytes,
            "preparing":u.preparing,"retention_overflow":u.retention_overflow})
            })
            .collect();
        let widths: Vec<_> = Renderer::width_measurement_cache_usage()
            .iter()
            .map(|u| {
                json!({
            "owner_id":u.owner_id,"thread":format!("{:?}",u.thread),"entries":u.entries,
            "key_bytes":u.key_bytes,"retained_bytes":u.retained_bytes})
            })
            .collect();
        let documents_cpu: Vec<_> = resource_observation::document_cpu_usage()
            .iter()
            .map(|u| {
                json!({
            "scene_id":u.scene_id,"budget_id":u.budget_id,"retained_bytes":u.retained_bytes,
            "history_entries":u.history_entries,"limit_bytes":u.limit_bytes})
            })
            .collect();
        let documents_gpu: Vec<_> = Renderer::world_document_gpu_usage().iter().map(|u| json!({
            "scene_id":u.scene_id,"budget_id":u.budget_id,"reserved_bytes":u.reserved_bytes,
            "lifetime_peak_reserved_bytes":u.lifetime_peak_reserved_bytes,"limit_bytes":u.limit_bytes})).collect();
        let text_registry_bytes = Renderer::text_cache_registry_bytes();
        let rss = linux_rss()?;
        let finished_ns = self.started.elapsed().as_nanos();
        self.sequence += 1;
        self.line(json!({"phase":"snapshot","sequence":self.sequence,"started_ns":started_ns,
            "finished_ns":finished_ns,"final":final_snapshot,"hosts":hosts,"scoped_heap":scopes,
            "scope_registry":{"registry_heap_bytes":registry.registry_heap_bytes,"scope_owner_bytes":registry.scope_owner_bytes,"call_slots_static_bytes":registry.call_slots_static_bytes},
            "text_cache_owners":text,"text_cache_registry_bytes":text_registry_bytes,
            "thread_measurements":widths,"documents_cpu":documents_cpu,"documents_gpu":documents_gpu,"gui_rss":rss,
            "observer_storage":{"host_slots_capacity_bytes":self.hosts.capacity()*std::mem::size_of::<Host>(),
                "window_strings_capacity_bytes":self.hosts.iter().map(|h|h.window.capacity()).sum::<usize>()}}))?;
        self.last = Instant::now();
        self.hosts
            .retain(|host| host.present || !host.released_last_snapshot);
        Ok(())
    }
}

pub(crate) fn poll(app: &App) -> Result<()> {
    if !ENABLED.load(Ordering::Acquire) {
        return Ok(());
    }
    let mut writer = WRITER.lock().unwrap_or_else(|e| e.into_inner());
    let writer = writer.as_mut().context("resource writer missing")?;
    let changed = writer.update_hosts(Some(app))?;
    if changed || writer.sequence == 0 || writer.last.elapsed() >= writer.interval {
        writer.snapshot(Some(app), false)?;
    }
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
        .context("resource writer missing")?;
    writer.update_hosts(None)?;
    writer.snapshot(None, true)?;
    let complete = event_loop_ok && writer.hosts.is_empty();
    writer.line(json!({"phase":"end","complete_delivery":complete,"event_loop_ok":event_loop_ok,"snapshots":writer.sequence,"unreleased_hosts":writer.hosts.len()}))?;
    writer.file.flush()?;
    anyhow::ensure!(complete, "resource observation incomplete");
    Ok(())
}

fn reservation(r: ReservationUsage) -> Value {
    json!({"budget_id":r.budget_id,"reserved_bytes":r.reserved_bytes,
        "lifetime_peak_reserved_bytes":r.lifetime_peak_reserved_bytes,"limit_bytes":r.limit_bytes})
}
fn local_reservations(r: LocalReservationUsage) -> Value {
    json!({"renderer_id":r.renderer_id,"screen":reservation(r.screen),"control_mesh":reservation(r.control_mesh),
        "atlas":reservation(r.atlas),"staging":reservation(r.staging),"released":r.released()})
}
fn scope_view(u: &cpu_alloc::Usage) -> Value {
    json!({"owner_id":u.owner_id,"label":u.label,"allocator_installed":u.allocator_installed,
        "payload_bytes":u.payload_bytes,"tracking_bytes":u.tracking_bytes,"allocations":u.allocations,
        "peak_payload_bytes":u.peak_payload_bytes,"owner_metadata_bytes":u.owner_metadata_bytes})
}
fn font_view(u: resource_observation::FontCpuUsage) -> Value {
    json!({"allocation":scope_view(&u.allocation),"returned_shape_bytes":u.returned_shape_bytes,
        "private_bytes":u.private_bytes,"fixed_font_bytes":u.fixed_font_bytes,"cache_reserved_bytes":u.cache_reserved_bytes})
}
fn renderer_view(renderer: &Renderer) -> Value {
    let control = renderer.control_mesh_usage();
    let text = renderer.text_cache_key_usage();
    json!({"text_scope":scope_view(&renderer.text_cpu_usage()),"font":font_view(renderer.font_cpu_usage()),
        "measurement_font":renderer.measurement_cpu_usage().map(font_view),
        "control_mesh":{"entries":control.entries,"entry_capacity":control.entry_capacity,
            "key_capacity_bytes":control.key_capacity_bytes,"entry_storage_bytes":control.entry_storage_bytes,
            "mesh_storage_bytes":control.mesh_storage_bytes,"total_bytes":control.total_bytes},
        "text_keys":{"owner_id":text.owner_id,"label_entries":text.label_entries,"label_key_text_bytes":text.label_key_text_bytes,
            "entries":text.entries,"key_text_bytes":text.key_text_bytes,"entry_storage_bytes":text.entry_storage_bytes,"shaped_payload_bytes":text.shaped_payload_bytes}})
}
fn linux_rss() -> Result<Value> {
    #[cfg(target_os = "linux")]
    {
        let status = std::fs::read_to_string("/proc/self/status")?;
        let read = |name: &str| -> Result<u64> {
            let line = status
                .lines()
                .find(|line| line.starts_with(name))
                .context("missing RSS field")?;
            let mut fields = line.split_whitespace().skip(1);
            let kib = fields
                .next()
                .context("missing RSS amount")?
                .parse::<u64>()?;
            anyhow::ensure!(fields.next() == Some("kB"), "unexpected RSS unit");
            kib.checked_mul(1024).context("RSS amount overflow")
        };
        Ok(
            json!({"rss_bytes":read("VmRSS:")?,"high_water_bytes":read("VmHWM:")?,"scope":"GUI PID only; native/driver mappings included, cooperating engine processes separate"}),
        )
    }
    #[cfg(not(target_os = "linux"))]
    Ok(json!({"unsupported":"Linux /proc RSS method unavailable"}))
}
