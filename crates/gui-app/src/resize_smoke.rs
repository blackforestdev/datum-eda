//! Explicit native-window resize proof. This state never schedules normal frames.

use super::*;

type Extent = (u32, u32);
const TARGETS: [Extent; 6] = [
    (1280, 900),
    (1440, 900),
    (1280, 800),
    (1440, 800),
    (1280, 900),
    (1280, 800),
];

#[derive(Default)]
pub(super) struct ResizeSmoke {
    start_after: Option<std::time::Instant>,
    requested: usize,
    waiting: Option<Extent>,
    finished: bool,
}

#[derive(Debug, PartialEq)]
enum Advance {
    Wait,
    Request(Extent),
    Complete,
    Finished,
}

impl ResizeSmoke {
    pub(super) fn note_native_event(&mut self) {
        if self.requested == 0 {
            self.start_after =
                Some(std::time::Instant::now() + std::time::Duration::from_millis(200));
        }
    }

    fn advance(&mut self, observed: Extent, configured: Extent, presented: bool) -> Advance {
        if self.finished {
            return Advance::Finished;
        }
        if !presented || observed != configured {
            return Advance::Wait;
        }
        if let Some(target) = self.waiting {
            if observed != target {
                return Advance::Wait;
            }
            self.waiting = None;
        }
        if self.requested == TARGETS.len() {
            self.finished = true;
            return Advance::Complete;
        }
        let next = TARGETS[self.requested];
        self.requested += 1;
        self.waiting = Some(next);
        Advance::Request(next)
    }
}

impl App {
    pub(super) fn poll_resize_smoke_start(&mut self, event_loop: &ActiveEventLoop) {
        if !self.args.resize_torture_smoke || self.resize_smoke.requested != 0 {
            return;
        }
        let Some(deadline) = self.resize_smoke.start_after else {
            return;
        };
        if std::time::Instant::now() >= deadline {
            if let Some(window) = self.window {
                window.request_redraw();
            }
        } else {
            let next = match event_loop.control_flow() {
                ControlFlow::WaitUntil(existing) => existing.min(deadline),
                ControlFlow::Poll => return,
                ControlFlow::Wait => deadline,
            };
            event_loop.set_control_flow(ControlFlow::WaitUntil(next));
        }
    }

    /// True while native size events/presentations are still required. Never
    /// overwrite configured dimensions to pretend a native resize took place.
    pub(super) fn advance_native_resize_smoke(&mut self, presented: bool) -> bool {
        if !self.args.resize_torture_smoke {
            return false;
        }
        // Initial compositor configuration may arrive after the first frame.
        if self.resize_smoke.requested == 0
            && self
                .resize_smoke
                .start_after
                .is_none_or(|deadline| std::time::Instant::now() < deadline)
        {
            return true;
        }
        let Some(runtime) = &mut self.runtime else {
            return true;
        };
        let native = runtime.window.inner_size();
        let observed = (native.width, native.height);
        let configured = (runtime.config.width, runtime.config.height);
        match self.resize_smoke.advance(observed, configured, presented) {
            Advance::Wait => true,
            Advance::Request((width, height)) => {
                if self.resize_smoke.requested == 1 {
                    append_gui_diagnostic_line("native resize smoke begin");
                } else {
                    append_gui_diagnostic_line(format!(
                        "native resize smoke observed {}x{} configured {}x{} presented=true",
                        observed.0, observed.1, configured.0, configured.1
                    ));
                }
                append_gui_diagnostic_line(format!("native resize smoke request {width}x{height}"));
                if let Some(size) = runtime
                    .window
                    .request_inner_size(winit::dpi::PhysicalSize::new(width, height))
                {
                    // Fractional scale conversion or native constraints can round
                    // the accepted size. Verify that actual result, not an impossible
                    // exact physical request. Preserve both values in the evidence.
                    let tolerance = runtime.window.scale_factor().ceil() as u32;
                    if size.width.abs_diff(width) <= tolerance
                        && size.height.abs_diff(height) <= tolerance
                    {
                        self.resize_smoke.waiting = Some((size.width, size.height));
                    } else {
                        append_gui_diagnostic_line(
                            "native resize smoke request constrained; exact target remains pending",
                        );
                    }
                    append_gui_diagnostic_line(format!(
                        "native resize smoke locally returned {}x{}",
                        size.width, size.height
                    ));
                    runtime.resize(size.width, size.height);
                    runtime.window.request_redraw();
                }
                // A no-op request need not produce a native size event.
                if observed == (width, height) {
                    runtime.window.request_redraw();
                }
                // Otherwise None awaits a native Resized event.
                true
            }
            Advance::Complete => {
                append_gui_diagnostic_line(format!(
                    "native resize smoke observed {}x{} configured {}x{} presented=true",
                    observed.0, observed.1, configured.0, configured.1
                ));
                append_gui_diagnostic_line("native resize smoke end");
                false
            }
            Advance::Finished => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_extent_and_successful_presentation_are_required() {
        let mut smoke = ResizeSmoke::default();
        assert_eq!(
            smoke.advance((1280, 800), (1280, 800), false),
            Advance::Wait
        );
        assert_eq!(smoke.advance((1280, 800), (1440, 900), true), Advance::Wait);
        assert_eq!(
            smoke.advance((1280, 800), (1280, 800), true),
            Advance::Request(TARGETS[0])
        );
        assert_eq!(smoke.advance((1280, 800), (1280, 800), true), Advance::Wait);
        assert_eq!(smoke.advance(TARGETS[0], TARGETS[0], false), Advance::Wait);
        assert_eq!(
            smoke.advance(TARGETS[0], TARGETS[0], true),
            Advance::Request(TARGETS[1])
        );
    }

    #[test]
    fn initial_extent_equal_to_first_target_can_advance() {
        let mut smoke = ResizeSmoke::default();
        assert_eq!(
            smoke.advance(TARGETS[0], TARGETS[0], true),
            Advance::Request(TARGETS[0])
        );
        assert_eq!(
            smoke.advance(TARGETS[0], TARGETS[0], true),
            Advance::Request(TARGETS[1])
        );
    }

    #[test]
    fn native_resize_sequence_finishes_once_and_never_reenters() {
        let mut smoke = ResizeSmoke::default();
        assert_eq!(
            smoke.advance((1280, 800), (1280, 800), true),
            Advance::Request(TARGETS[0])
        );
        for (index, target) in TARGETS.iter().copied().enumerate() {
            let expected = TARGETS
                .get(index + 1)
                .copied()
                .map(Advance::Request)
                .unwrap_or(Advance::Complete);
            assert_eq!(smoke.advance(target, target, true), expected);
        }
        for extent in [(1, 1), (1280, 800), (1440, 900)] {
            assert_eq!(smoke.advance(extent, extent, true), Advance::Finished);
        }
    }
}
