//! Finite opt-in observation of ordinary input phases and actual submitted pixels.
//! No state-dependent frame search, alternate renderer, or presentation delay.
use super::*;
#[cfg(feature = "visual")]
use datum_gui_protocol::SelectionTarget;
#[cfg(feature = "visual")]
use serde_json::json;

#[cfg(any(feature = "visual", test))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Phase {
    Zoom(u8),
    BaselineArmed,
    BaselinePending,
    ReleasedReady,
    ReleasedArmed,
    ReleasedPending,
    Done,
    Failed,
}

#[cfg(any(feature = "visual", test))]
impl Phase {
    pub(super) fn wheel(&mut self, prescribed: bool) -> Result<()> {
        if *self == Self::Done {
            return Ok(());
        }
        anyhow::ensure!(prescribed, "unexpected native content wheel");
        let Self::Zoom(count) = *self else {
            anyhow::bail!("wheel outside prescribed zoom phase");
        };
        *self = if count == 5 {
            Self::BaselineArmed
        } else {
            Self::Zoom(count + 1)
        };
        Ok(())
    }
    pub(super) fn primary(&mut self, pressed: bool, shift: bool, held: &mut bool) -> Result<()> {
        if matches!(*self, Self::Zoom(0) | Self::Done) {
            return Ok(());
        }
        anyhow::ensure!(
            *self == Self::ReleasedReady && shift,
            "primary input outside admitted diagnostic phase"
        );
        if pressed {
            anyhow::ensure!(!*held, "duplicate primary press");
            *held = true;
        } else {
            anyhow::ensure!(*held, "primary release without press");
            *held = false;
            *self = Self::ReleasedArmed;
        }
        Ok(())
    }
    fn armed(self) -> Option<&'static str> {
        match self {
            Self::BaselineArmed => Some("baseline"),
            Self::ReleasedArmed => Some("released"),
            _ => None,
        }
    }
    fn observe_armed<T>(self, observe: impl FnOnce() -> T) -> Option<T> {
        self.armed()?;
        Some(observe())
    }
}

#[derive(Default)]
pub(super) struct Probe {
    #[cfg(feature = "visual")]
    state: Option<State>,
}

impl Probe {
    #[cfg(feature = "visual")]
    pub(super) fn observe_armed<T>(&self, observe: impl FnOnce() -> T) -> Option<T> {
        self.state.as_ref()?.phase.observe_armed(observe)
    }
    fn active(&self) -> bool {
        #[cfg(feature = "visual")]
        {
            return self
                .state
                .as_ref()
                .is_some_and(|s| !matches!(s.phase, Phase::Done | Phase::Failed));
        }
        #[cfg(not(feature = "visual"))]
        {
            false
        }
    }
    pub(super) fn from_environment(
        caps: &wgpu::SurfaceCapabilities,
        config: &mut wgpu::SurfaceConfiguration,
        wake: winit::event_loop::EventLoopProxy<()>,
    ) -> Result<Self> {
        let Some(path) = std::env::var_os("DATUM_NATIVE_CONTENT_PROBE") else {
            return Ok(Self::default());
        };
        #[cfg(not(feature = "visual"))]
        {
            let _ = (caps, config, wake, path);
            anyhow::bail!("native content probe requires existing visual feature");
        }
        #[cfg(feature = "visual")]
        {
            gui_runtime_support::admit_main_content_readback(caps, config)?;
            let root = PathBuf::from(path);
            anyhow::ensure!(
                root.is_absolute() && !root.exists(),
                "native content output must be fresh and absolute"
            );
            fs::create_dir_all(&root).context("create native content output")?;
            Ok(Self {
                state: Some(State {
                    root,
                    phase: Phase::Zoom(0),
                    held: false,
                    extent: [config.width, config.height],
                    format: config.format,
                    target: None,
                    pending: None,
                    failure: None,
                    wake,
                }),
            })
        }
    }
    pub(super) fn wheel(&mut self, prescribed: bool, main_board: bool) {
        #[cfg(feature = "visual")]
        if let Some(state) = &mut self.state {
            if matches!(state.phase, Phase::Done | Phase::Failed) {
                return;
            }
            let result = if main_board {
                state.phase.wheel(prescribed)
            } else {
                Err(anyhow::anyhow!("wrong native input target"))
            };
            if let Err(error) = result {
                state.fail(error.to_string());
            }
        }
        #[cfg(not(feature = "visual"))]
        let _ = (prescribed, main_board);
    }
    pub(super) fn primary(&mut self, pressed: bool, shift: bool, main_board: bool) {
        #[cfg(feature = "visual")]
        if let Some(state) = &mut self.state {
            if matches!(state.phase, Phase::Done | Phase::Failed) {
                return;
            }
            let result = if main_board {
                state.phase.primary(pressed, shift, &mut state.held)
            } else {
                Err(anyhow::anyhow!("wrong native primary target"))
            };
            if let Err(error) = result {
                state.fail(error.to_string());
            }
        }
        #[cfg(not(feature = "visual"))]
        let _ = (pressed, shift, main_board);
    }
    #[cfg(feature = "visual")]
    pub(super) fn request(
        &mut self,
        config: &wgpu::SurfaceConfiguration,
        revision: u64,
        subject: serde_json::Value,
    ) -> Option<Arm> {
        let state = self.state.as_mut()?;
        if [config.width, config.height] != state.extent
            || config.format != state.format
            || !config.usage.contains(wgpu::TextureUsages::COPY_SRC)
        {
            state.fail("native content configuration changed".into());
            return None;
        }
        Some(Arm {
            phase: state.phase.armed()?,
            revision,
            subject,
        })
    }
    #[cfg(feature = "visual")]
    pub(super) fn submitted(
        &mut self,
        work: Option<Work>,
        device: &wgpu::Device,
    ) -> Option<&'static str> {
        let work = work?;
        let state = self.state.as_mut()?;
        if let Err(error) = state.submit(work, device) {
            state.fail(error.to_string());
            return None;
        }
        state.pending.as_ref().map(|p| p.work.arm.phase)
    }
    #[cfg(feature = "visual")]
    pub(super) fn present(
        &mut self,
        phase: Option<&'static str>,
        success: bool,
        error: Option<String>,
    ) {
        let Some(phase) = phase else {
            return;
        };
        if let Some(state) = &mut self.state {
            if let Some(pending) = &mut state.pending {
                if pending.work.arm.phase == phase {
                    pending.presented = Some(success);
                    pending.present_error = error;
                    pending.present_time = Some(format!("{:?}", std::time::SystemTime::now()));
                } else {
                    state.fail("native presentation ticket mismatch".into());
                }
            }
        }
    }
    #[cfg(feature = "visual")]
    pub(super) fn render_failed(&mut self, work: Option<Work>, error: &str) {
        if work.is_some() {
            if let Some(state) = &mut self.state {
                state.fail(format!("native encoding failed: {error}"));
            }
        }
    }
    pub(super) fn service(
        &mut self,
        device: &wgpu::Device,
        failed: bool,
    ) -> Result<Option<std::time::Instant>> {
        #[cfg(feature = "visual")]
        if let Some(state) = &mut self.state {
            return state.service(device, failed);
        }
        let _ = (device, failed);
        Ok(None)
    }
}

#[cfg(feature = "visual")]
#[derive(Clone)]
pub(crate) struct Arm {
    pub phase: &'static str,
    pub revision: u64,
    pub subject: serde_json::Value,
}

#[cfg(feature = "visual")]
pub(crate) struct Work {
    pub arm: Arm,
    pub readback: datum_gui_render::render_input::NativeReadback,
    pub identity: serde_json::Value,
    pub target: [u64; 3],
}

#[cfg(feature = "visual")]
struct Pending {
    work: Work,
    receiver: std::sync::mpsc::Receiver<std::result::Result<(), String>>,
    mapped: Option<std::result::Result<(), String>>,
    presented: Option<bool>,
    present_error: Option<String>,
    present_time: Option<String>,
    deadline: std::time::Instant,
}

#[cfg(feature = "visual")]
fn delivery_ready(
    mapped: &Option<std::result::Result<(), String>>,
    presented: Option<bool>,
) -> bool {
    mapped.is_some() && presented.is_some()
}

#[cfg(feature = "visual")]
fn delivery_valid(
    mapped: &Option<std::result::Result<(), String>>,
    presented: Option<bool>,
    failed: bool,
) -> bool {
    !failed && presented == Some(true) && mapped.as_ref().is_some_and(|value| value.is_ok())
}

#[cfg(feature = "visual")]
impl Drop for Pending {
    fn drop(&mut self) {
        self.work.readback.buffer().unmap();
    }
}

#[cfg(feature = "visual")]
struct State {
    root: PathBuf,
    phase: Phase,
    held: bool,
    extent: [u32; 2],
    format: wgpu::TextureFormat,
    target: Option<[u64; 3]>,
    pending: Option<Pending>,
    failure: Option<String>,
    wake: winit::event_loop::EventLoopProxy<()>,
}

#[cfg(feature = "visual")]
impl State {
    fn fail(&mut self, reason: String) {
        if self.failure.is_some() {
            return;
        }
        self.phase = Phase::Failed;
        self.failure = Some(reason.clone());
        let value = json!({"status":"FAIL","reason":reason,"pid":std::process::id(),
            "time":format!("{:?}",std::time::SystemTime::now())});
        if let Err(error) = fs::write(self.root.join("invalid.json"), value.to_string()) {
            append_gui_diagnostic_line(format!(
                "native content failure persistence: {error}; {value}"
            ));
        }
    }
    fn submit(&mut self, work: Work, _device: &wgpu::Device) -> Result<()> {
        anyhow::ensure!(
            self.failure.is_none() && self.pending.is_none(),
            "native content phase already pending/failed"
        );
        anyhow::ensure!(
            self.phase.armed() == Some(work.arm.phase),
            "native content phase mismatch"
        );
        let snapshot = work
            .readback
            .snapshot()
            .context("actual consumed native CPU payload absent")?;
        anyhow::ensure!(
            snapshot.attempt[2] == work.arm.revision,
            "native frame revision changed during encoding"
        );
        if let Some(target) = self.target {
            anyhow::ensure!(target == work.target, "native device/configuration changed");
        }
        self.target = Some(work.target);
        let (sender, receiver) = std::sync::mpsc::channel();
        let wake = self.wake.clone();
        work.readback
            .buffer()
            .slice(..)
            .map_async(wgpu::MapMode::Read, move |result| {
                let _ = sender.send(result.map_err(|error| error.to_string()));
                let _ = wake.send_event(());
            });
        self.phase = if work.arm.phase == "baseline" {
            Phase::BaselinePending
        } else {
            Phase::ReleasedPending
        };
        self.pending = Some(Pending {
            work,
            receiver,
            mapped: None,
            presented: None,
            present_error: None,
            present_time: None,
            // Operational stop bound inherited from the finite diagnostic client,
            // never a latency criterion or a delay before painting.
            deadline: std::time::Instant::now() + std::time::Duration::from_secs(10),
        });
        Ok(())
    }
    fn service(
        &mut self,
        device: &wgpu::Device,
        failed: bool,
    ) -> Result<Option<std::time::Instant>> {
        let Some(pending) = &mut self.pending else {
            return Ok(None);
        };
        if failed {
            self.fail("device failed during native content observation".into());
            self.pending.take();
            return Ok(None);
        }
        // Map completion has its own service even when ordinary queue work is idle.
        device
            .poll(wgpu::PollType::Poll)
            .context("service native content map")?;
        if pending.mapped.is_none() {
            match pending.receiver.try_recv() {
                Ok(value) => pending.mapped = Some(value),
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    pending.mapped = Some(Err("map callback disconnected".into()))
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {}
            }
        }
        if delivery_ready(&pending.mapped, pending.presented) {
            let pending = self.pending.take().unwrap();
            if let Err(error) = self.deliver(&pending) {
                self.fail(error.to_string());
            } else {
                self.phase = if pending.work.arm.phase == "baseline" {
                    Phase::ReleasedReady
                } else {
                    Phase::Done
                };
            }
            return Ok(None);
        }
        let now = std::time::Instant::now();
        if now >= pending.deadline {
            self.fail(
                "native map/presentation observation incomplete at operational stop bound".into(),
            );
            self.pending.take();
            return Ok(None);
        }
        // Bounded nonblocking I/O servicing, not a repaint or semantic retry.
        Ok(Some(now + std::time::Duration::from_millis(10)))
    }
    fn deliver(&self, pending: &Pending) -> Result<()> {
        let snapshot = pending
            .work
            .readback
            .snapshot()
            .context("native CPU snapshot lost")?;
        let text: Vec<_> = snapshot.text.iter().map(|r| json!({"region":r.region,"text":r.text,
            "spans":r.spans.iter().map(|s|json!({"text":s.text,"color":s.color,"bold":s.bold,"italic":s.italic})).collect::<Vec<_>>(),
            "position":r.position,"size":r.size,"face":r.face,"color":r.color,"clip":r.clip,"layout_size":r.layout_size})).collect();
        let valid = delivery_valid(&pending.mapped, pending.presented, self.failure.is_some());
        let record = json!({"status":if valid {"CAPTURE_ONLY"} else {"FAIL"},
            "phase":pending.work.arm.phase,"pid":std::process::id(),"subject":pending.work.arm.subject,
            "consumed_frame_attempt":snapshot.attempt,"native_identity":pending.work.identity,
            "cpu_host_text":text,"mapped":pending.mapped,"presented":pending.presented,
            "present_error":pending.present_error,"present_time":pending.present_time,
            "failure":self.failure,"source":"actual acquired texture copied after composition in the same encoder/sole submission",
            "limits":"Not offscreen rerender; no compositor or physical-display assertion, wire identity requires independent unambiguous pairing."});
        if !valid {
            fs::write(
                self.root.join(format!("{}.json", pending.work.arm.phase)),
                serde_json::to_vec_pretty(&record)?,
            )?;
            anyhow::bail!("native map and presentation must both succeed");
        }
        let readback = &pending.work.readback;
        let [width, height] = readback.extent();
        let mapped = readback.buffer().slice(..).get_mapped_range();
        let mut pixels = vec![0; width as usize * height as usize * 4];
        for (row, out) in pixels.chunks_exact_mut(width as usize * 4).enumerate() {
            let offset = row * readback.padded_row() as usize;
            out.copy_from_slice(&mapped[offset..offset + out.len()]);
        }
        drop(mapped);
        gui_runtime_support::convert_texture_pixels_to_rgba(&mut pixels, readback.format())?;
        image::RgbaImage::from_raw(width, height, pixels)
            .context("native image packing")?
            .save(self.root.join(format!("{}.png", pending.work.arm.phase)))?;
        let temporary = self
            .root
            .join(format!("{}.json.tmp", pending.work.arm.phase));
        fs::write(&temporary, serde_json::to_vec_pretty(&record)?)?;
        fs::rename(
            temporary,
            self.root.join(format!("{}.json", pending.work.arm.phase)),
        )?;
        Ok(())
    }
}

#[cfg(feature = "visual")]
impl Drop for State {
    fn drop(&mut self) {
        if !matches!(self.phase, Phase::Done | Phase::Failed) {
            self.fail("native content observation abandoned before both slots completed".into());
        }
    }
}

impl Runtime {
    pub(super) fn native_content_wheel(&mut self, delta: winit::event::MouseScrollDelta) {
        if !self.native_content.active() {
            return;
        }
        let main = matches!(
            self.application_focus(),
            ApplicationFocus::Editor(datum_gui_protocol::PaneId(0))
        ) && self.workspace().ui.layout.focused_content() == PaneContent::Board;
        self.native_content.wheel(
            matches!(delta,winit::event::MouseScrollDelta::LineDelta(x,y) if x==0.0 && y==1.0),
            main,
        );
    }
    pub(super) fn native_content_primary(&mut self, pressed: bool) {
        if !self.native_content.active() {
            return;
        }
        let main = matches!(
            self.application_focus(),
            ApplicationFocus::Editor(datum_gui_protocol::PaneId(0))
        ) && self.workspace().ui.layout.focused_content() == PaneContent::Board;
        self.native_content
            .primary(pressed, self.modifiers.shift_key(), main);
    }
    #[cfg(feature = "visual")]
    pub(super) fn native_content_subject(&self) -> serde_json::Value {
        match &self.workspace().selection {
            SelectionTarget::Native(v) => json!({"kind":v.subject.kind(),"project":v.project,
                "revision":v.revision,"subject":v.subject,"members":v.members,"scope":v.scope,"focus":v.focus(),"stale":v.stale}),
            SelectionTarget::None => json!({"kind":"none"}),
            other => json!({"kind":"legacy","selection":format!("{other:?}")}),
        }
    }
}

#[cfg(test)]
#[path = "native_content_probe_tests.rs"]
mod tests;
