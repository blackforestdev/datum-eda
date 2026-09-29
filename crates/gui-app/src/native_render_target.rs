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
        self.frame
            .as_ref()
            .map(|frame| {
                datum_gui_render::render_input::FrameTarget::full_texture(
                    &frame
                        .texture
                        .as_ref()
                        .expect("unconsumed native frame")
                        .texture,
                )
            })
            .transpose()
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
