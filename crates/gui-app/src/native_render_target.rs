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
    pub(crate) fn acquire(&mut self) -> anyhow::Result<Option<wgpu::TextureView>> {
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
        Ok(self.frame.as_ref().map(NativeSurfaceFrame::view))
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
    pub(crate) fn begin_render_receipt(
        &self,
        renderer: &mut datum_gui_render::Renderer,
    ) -> impl FnOnce(&mut datum_gui_render::Renderer, u64, u64, u64, bool) -> bool + use<> {
        let (device, _, _) = self.queue_owner.snapshot();
        let receipt = renderer.render_session_mut().begin_frame(
            self.queue_host,
            device,
            self.configuration_generation,
            true,
        );
        move |renderer, host, device, configuration, presented| {
            renderer.render_session_mut().complete_frame(
                receipt,
                host,
                device,
                configuration,
                presented,
            )
        }
    }

    pub(crate) fn present_rendered(
        &mut self,
        frame: NativeSurfaceFrame,
        window: &winit::window::Window,
        renderer: &mut datum_gui_render::Renderer,
        complete: impl FnOnce(&mut datum_gui_render::Renderer, u64, u64, u64, bool) -> bool,
    ) -> anyhow::Result<()> {
        let (device, _, _) = self.queue_owner.snapshot();
        let result = self.present(frame, window);
        let accepted = complete(
            renderer,
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
