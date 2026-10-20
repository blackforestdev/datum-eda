//! Shared late-acquisition adapter for main and owned native windows.
use super::*;

pub(crate) struct NativeRenderTarget<'a, 'window> {
    transaction: &'a mut SurfaceTransaction,
    surface: &'a wgpu::Surface<'window>,
    device: &'a wgpu::Device,
    queue: &'a wgpu::Queue,
    config: &'a wgpu::SurfaceConfiguration,
    health: &'a crate::native_device_recovery::DeviceHealth,
    frame: Option<NativeSurfaceFrame>,
    acquire_elapsed: std::time::Duration,
    upload_submitted: bool,
    #[cfg(feature = "visual")]
    content_arm: Option<crate::native_content_probe::Arm>,
    #[cfg(feature = "visual")]
    content_work: Option<crate::native_content_probe::Work>,
}
impl<'a, 'window> NativeRenderTarget<'a, 'window> {
    pub(crate) fn new(
        transaction: &'a mut SurfaceTransaction,
        surface: &'a wgpu::Surface<'window>,
        device: &'a wgpu::Device,
        queue: &'a wgpu::Queue,
        config: &'a wgpu::SurfaceConfiguration,
        health: &'a crate::native_device_recovery::DeviceHealth,
    ) -> Self {
        Self {
            transaction,
            surface,
            device,
            queue,
            config,
            health,
            frame: None,
            acquire_elapsed: std::time::Duration::ZERO,
            upload_submitted: false,
            #[cfg(feature = "visual")]
            content_arm: None,
            #[cfg(feature = "visual")]
            content_work: None,
        }
    }
    pub(crate) fn acquire(
        &mut self,
    ) -> anyhow::Result<Option<datum_gui_render::render_input::FrameTarget>> {
        assert!(
            self.frame.is_none(),
            "one acquisition per rendering attempt"
        );
        let start = Instant::now();
        let result = self
            .transaction
            .acquire(self.surface, self.device, self.config, self.health);
        self.acquire_elapsed = start.elapsed();
        self.frame = result?;
        let target = self
            .frame
            .as_ref()
            .map(|frame| {
                let texture = &frame
                    .texture
                    .as_ref()
                    .expect("unconsumed native frame")
                    .texture;
                let extent = self.transaction.render_extent(self.config);
                anyhow::ensure!(
                    [texture.width(), texture.height()] == extent,
                    "acquired extent differs from configured diagnostic extent"
                );
                if extent != [self.config.width, self.config.height] {
                    datum_gui_render::render_input::FrameTarget::diagnostic_resampled_texture(
                        texture,
                    )
                } else {
                    datum_gui_render::render_input::FrameTarget::full_texture(texture)
                }
            })
            .transpose()?;
        #[cfg(feature = "visual")]
        if let (Some(arm), Some(target)) = (self.content_arm.take(), target.as_ref()) {
            let readback = datum_gui_render::render_input::NativeReadback::new(
                self.device,
                [self.config.width, self.config.height],
                self.config.format,
            )?;
            let (epoch, _, _) = self.transaction.queue_owner.snapshot();
            let identity = serde_json::json!({"window":format!("{:?}",self.transaction.window),
                "host":self.transaction.queue_host,"queue_epoch":epoch,
                "configuration":self.transaction.configuration_generation,
                "acquisition":self.transaction.texture_active.acquired.get(),
                "extent":[self.config.width,self.config.height],"format":format!("{:?}",self.config.format)});
            self.content_work = Some(crate::native_content_probe::Work {
                arm,
                readback: readback.clone(),
                identity,
                target: [
                    self.transaction.queue_host,
                    epoch,
                    self.transaction.configuration_generation,
                ],
            });
            return Ok(Some(target.clone().with_native_readback(readback)?));
        }
        Ok(target)
    }
    #[cfg(feature = "visual")]
    pub(crate) fn arm_content(&mut self, arm: Option<crate::native_content_probe::Arm>) {
        self.content_arm = arm;
    }
    #[cfg(feature = "visual")]
    pub(crate) fn take_content_work(&mut self) -> Option<crate::native_content_probe::Work> {
        let mut work = self.content_work.take()?;
        work.identity["submission_receipt"] =
            serde_json::json!(self.frame.as_ref().and_then(|f| f.lease.submission_receipt));
        Some(work)
    }
    pub(crate) fn submitted(&mut self, submission: wgpu::SubmissionIndex) {
        assert!(
            !self.upload_submitted,
            "one native submission per admitted turn"
        );
        if let Some(frame) = &mut self.frame {
            self.transaction.submitted(frame, self.queue, submission);
        } else {
            self.transaction.submitted_upload(self.queue, submission);
            self.upload_submitted = true;
        }
    }
    pub(crate) fn finish(
        self,
        renderer: &datum_gui_render::Renderer,
    ) -> (Option<NativeSurfaceFrame>, std::time::Duration) {
        self.transaction
            .observe_attachment(renderer, self.frame.as_ref(), self.upload_submitted);
        (self.frame, self.acquire_elapsed)
    }
}

// Shared presentation bridge for Main and every owned dialog kind. The receipt
// captures the platform host, queue epoch and configured surface generation.
impl SurfaceTransaction {
    /// Read-only correlation of this retained acquisition and its submission.
    /// This is not GPU completion, compositor sampling, or displayed-content proof.
    #[cfg(feature = "visual")]
    pub(crate) fn frame_diagnostic_identity(
        &self,
        frame: &NativeSurfaceFrame,
    ) -> serde_json::Value {
        let (epoch, _, _) = self.queue_owner.snapshot();
        serde_json::json!({"window":format!("{:?}",self.window),"host":self.queue_host,
            "queue_epoch":epoch,"configuration":frame.lease.generation,
            "acquisition":frame.lease.active.acquired.get(),
            "submission_receipt":frame.lease.submission_receipt})
    }

    pub(crate) fn prepare_render_plan(
        &self,
        renderer: &mut datum_gui_render::Renderer,
        width: u32,
        height: u32,
    ) -> anyhow::Result<datum_gui_render::FramePlan> {
        let (device, _, _) = self.queue_owner.snapshot();
        renderer.render_session_mut().prepare_frame(
            self.queue_host,
            device,
            self.configuration_generation,
            true,
            width,
            height,
        )
    }

    pub(crate) fn present_rendered(
        &mut self,
        frame: NativeSurfaceFrame,
        window: &winit::window::Window,
        renderer: &mut datum_gui_render::Renderer,
        submitted: datum_gui_render::SubmittedFrame,
    ) -> anyhow::Result<()> {
        let (device, _, _) = self.queue_owner.snapshot();
        let result = self.present(frame, window);
        let accepted = renderer.render_session_mut().complete_submitted(
            submitted,
            self.queue_host,
            device,
            self.configuration_generation,
            result.is_ok(),
        );
        result?;
        anyhow::ensure!(
            accepted,
            "presented frame rejected by shared render session"
        );
        Ok(())
    }
}
