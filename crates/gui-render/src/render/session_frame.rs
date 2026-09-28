//! Linear immutable frame snapshots and exact rendered-snapshot publication.
use super::*;
use crate::{PreparedScene, Renderer};

/// One immutable revision/target snapshot. No editor can mutate its scene or
/// replace a retained source between planning and encoding. Dropping fails closed.
pub struct FramePlan {
    receipt: Receipt,
    scene: PreparedScene,
    board: RetainedScene,
    schematic: Option<RetainedScene>,
    extent: [u32; 2],
    publish_hits: bool,
}

/// Only a successful full-frame encoder produces this presentation capability.
pub struct SubmittedFrame(FramePlan);

impl RenderSession {
    #[allow(clippy::too_many_arguments)]
    pub fn prepare_frame(
        &mut self,
        host: u64,
        device: u64,
        configuration: u64,
        native: bool,
        width: u32,
        height: u32,
    ) -> anyhow::Result<FramePlan> {
        let scene = self
            .prepared
            .take()
            .ok_or_else(|| anyhow::anyhow!("shared preparation required before frame planning"))?;
        let board = self
            .board
            .as_ref()
            .unwrap_or_else(|| self.empty.get_or_init(RetainedScene::empty))
            .clone();
        Ok(FramePlan {
            receipt: self.begin_frame(host, device, configuration, native),
            scene,
            board,
            schematic: self.schematic.clone(),
            extent: [width, height],
            publish_hits: self.prepared_hits_pending,
        })
    }

    pub(super) fn finish_snapshot(
        &mut self,
        mut plan: FramePlan,
        actual: Target,
        presented: bool,
    ) -> bool {
        let revision = plan.receipt.revision();
        let restore = self.revisions.matches(&plan.receipt)
            && self.revisions.current() == revision
            && self.prepared.is_none();
        let accepted = self.revisions.complete(plan.receipt, actual, presented);
        if accepted {
            self.terminal_damage.presented(revision);
            if plan.publish_hits {
                self.publication = Some((
                    std::mem::take(&mut plan.scene.hit_regions),
                    plan.scene.console_overlay_layout(),
                ));
                plan.publish_hits = false;
            }
        }
        if restore {
            self.prepared = Some(plan.scene);
            self.prepared_revision = revision;
            self.prepared_hits_pending = plan.publish_hits;
        }
        accepted
    }

    pub fn complete_submitted(
        &mut self,
        frame: SubmittedFrame,
        host: u64,
        device: u64,
        configuration: u64,
        presented: bool,
    ) -> bool {
        self.finish_snapshot(
            frame.0,
            Target {
                host,
                device,
                configuration,
            },
            presented,
        )
    }
}

impl Renderer {
    /// Late acquisition and queue submission remain native responsibilities.
    /// A deferred or failed encoder returns its snapshot to the shared owner and
    /// never issues a presentation capability or acknowledges terminal damage.
    #[allow(clippy::too_many_arguments)]
    pub fn encode_frame<C>(
        &mut self,
        plan: FramePlan,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        context: &mut C,
        acquire: &mut impl FnMut(&mut C) -> anyhow::Result<Option<wgpu::TextureView>>,
        submitted: &mut impl FnMut(&mut C, wgpu::SubmissionIndex),
    ) -> anyhow::Result<Option<SubmittedFrame>> {
        anyhow::ensure!(
            self.render_session.revisions.matches(&plan.receipt),
            "frame plan belongs to a retired or foreign rendering attempt"
        );
        let result = self.render_with_acquisition(
            device,
            queue,
            &plan.scene,
            &plan.board,
            plan.schematic.as_ref(),
            plan.extent[0],
            plan.extent[1],
            context,
            acquire,
            submitted,
        );
        match result {
            Ok(true) => Ok(Some(SubmittedFrame(plan))),
            other => {
                let actual = plan.receipt.target();
                self.render_session.finish_snapshot(plan, actual, false);
                other.map(|_| None)
            }
        }
    }

    #[cfg(feature = "visual")]
    pub fn encode_capture(
        &mut self,
        plan: FramePlan,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        target: &wgpu::TextureView,
    ) -> anyhow::Result<SubmittedFrame> {
        anyhow::ensure!(
            self.render_session.revisions.matches(&plan.receipt),
            "capture plan belongs to a retired or foreign rendering attempt"
        );
        let result = self.render(
            device,
            queue,
            target,
            &plan.scene,
            &plan.board,
            plan.schematic.as_ref(),
            plan.extent[0],
            plan.extent[1],
        );
        match result {
            Ok(()) => Ok(SubmittedFrame(plan)),
            Err(error) => {
                let actual = plan.receipt.target();
                self.render_session.finish_snapshot(plan, actual, false);
                Err(error)
            }
        }
    }
}

#[cfg(test)]
#[path = "session_frame_tests.rs"]
mod tests;
