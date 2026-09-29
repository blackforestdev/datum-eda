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
    prefix_key: Option<session_prefix::Key>,
    encoded_prefix: Option<session_prefix::Completed>,
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
        let preparation = self.preparation.ok_or_else(|| {
            anyhow::anyhow!("shared preparation profile required before frame planning")
        })?;
        anyhow::ensure!(
            preparation.extent == [width, height],
            "frame extent differs from prepared projection"
        );
        let scene = self
            .prepared
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("shared preparation required before frame planning"))?;
        let board = match preparation.profile {
            PreparedProfile::Workspace => {
                anyhow::ensure!(
                    !scene
                        .surface_passes
                        .iter()
                        .any(|pass| pass.surface == crate::SceneSurface::Schematic)
                        || self.schematic.is_some(),
                    "prepared schematic pane requires retained schematic content"
                );
                self.board
                    .as_ref()
                    .ok_or_else(|| {
                        anyhow::anyhow!("workspace frame requires retained board content")
                    })?
                    .clone()
            }
            PreparedProfile::Dialog => self.empty.get_or_init(RetainedScene::empty).clone(),
        };
        // Validate before moving the pending projection: refusal keeps retryable input.
        let scene = self.prepared.take().expect("validated preparation");
        let receipt = self.begin_frame(host, device, configuration, native);
        let prefix_key =
            (preparation.profile == PreparedProfile::Workspace).then(|| session_prefix::Key {
                preparation: self.preparation_generation,
                strong_revision: self.revisions.strong(),
                target: receipt.target(),
            });
        Ok(FramePlan {
            receipt,
            prefix_key,
            encoded_prefix: None,
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
        let image_success =
            self.revisions.matches(&plan.receipt) && plan.receipt.target() == actual && presented;
        if self.revisions.matches(&plan.receipt) {
            self.prefix.complete(
                plan.encoded_prefix,
                session_prefix::Key {
                    preparation: self.preparation_generation,
                    strong_revision: self.revisions.strong(),
                    target: actual,
                },
                image_success,
            );
        }
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
    /// Actual bundle executions in the last attempt; cached bundles are not draws.
    pub fn world_bundle_execution_count(&self) -> usize {
        self.render_session.prefix.world_executions()
    }
    /// Actual last encoded graph work. Logical read+write payload, not device bandwidth.
    pub fn prefix_copy_work(&self) -> (bool, u64) {
        (
            self.render_session.prefix.reused(),
            self.render_session.prefix.copy_bytes(),
        )
    }

    /// Late acquisition and queue submission remain native responsibilities.
    /// A deferred or failed encoder returns its snapshot to the shared owner and
    /// never issues a presentation capability or acknowledges terminal damage.
    #[allow(clippy::too_many_arguments)]
    pub fn encode_frame<C>(
        &mut self,
        mut plan: FramePlan,
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
        self.render_session.prefix.begin(plan.prefix_key);
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
        plan.encoded_prefix = self.render_session.prefix.take_encoded();
        match result {
            Ok(true) => Ok(Some(SubmittedFrame(plan))),
            other => {
                let actual = plan.receipt.target();
                self.render_session.finish_snapshot(plan, actual, false);
                other.map(|_| None)
            }
        }
    }

    pub fn encode_capture(
        &mut self,
        mut plan: FramePlan,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        target: &wgpu::TextureView,
    ) -> anyhow::Result<SubmittedFrame> {
        anyhow::ensure!(
            self.render_session.revisions.matches(&plan.receipt),
            "capture plan belongs to a retired or foreign rendering attempt"
        );
        self.render_session.prefix.begin(plan.prefix_key);
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
        plan.encoded_prefix = self.render_session.prefix.take_encoded();
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
