//! Acquire presentation storage only after every upload-only exit has passed.
use super::*;

pub(crate) trait Target {
    fn acquire(&mut self) -> anyhow::Result<Option<wgpu::TextureView>>;
    fn submitted(&mut self, submission: wgpu::SubmissionIndex);
}

struct Callbacks<'a, C, A, S> {
    context: &'a mut C,
    acquire: &'a mut A,
    submitted: &'a mut S,
}
impl<C, A, S> Target for Callbacks<'_, C, A, S>
where
    A: FnMut(&mut C) -> anyhow::Result<Option<wgpu::TextureView>>,
    S: FnMut(&mut C, wgpu::SubmissionIndex),
{
    fn acquire(&mut self) -> anyhow::Result<Option<wgpu::TextureView>> {
        (self.acquire)(self.context)
    }
    fn submitted(&mut self, submission: wgpu::SubmissionIndex) {
        (self.submitted)(self.context, submission);
    }
}

impl Renderer {
    /// Native callers admit a queue turn before entering this method. Upload-only
    /// submissions notify `submitted` without calling `acquire`; each returns
    /// false so the coordinator can retain damage and yield until completion.
    /// Acquisition may itself defer (None), also retaining pending damage.
    /// One context serializes both callbacks without interior mutability.
    #[allow(clippy::too_many_arguments)]
    pub fn render_with_acquisition<C>(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        prepared: &PreparedScene,
        retained: &RetainedScene,
        schematic_retained: Option<&RetainedScene>,
        width: u32,
        height: u32,
        context: &mut C,
        acquire: &mut impl FnMut(&mut C) -> anyhow::Result<Option<wgpu::TextureView>>,
        submitted: &mut impl FnMut(&mut C, wgpu::SubmissionIndex),
    ) -> anyhow::Result<bool> {
        self.frame_consumers = prepared.consumer_incidence();
        let _resource_scope = self.resource_host.enter_for(self.frame_consumers.all());
        self.atlas.owner.begin_upload_frame();
        let result = self.render_submission_inner(
            device,
            queue,
            &mut Callbacks {
                context,
                acquire,
                submitted,
            },
            prepared,
            retained,
            schematic_retained,
            width,
            height,
        );
        self.atlas
            .owner
            .finish_upload_frame(result.as_ref().ok().copied());
        if result.is_err() && !self.text_preparation.is_continuing() {
            self.text_buffers.finish_frame();
            self.text_preparation.cancel();
            self.text_renderer.cancel_preparation();
            self.menu_overlay_text_renderer.cancel_preparation();
        }
        result
    }
}
