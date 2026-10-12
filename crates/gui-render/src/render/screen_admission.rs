//! Fixed-size observations at the actual immediate draw sites.
use super::*;

#[derive(Clone, Copy, Debug)]
#[repr(usize)]
pub(crate) enum ScreenGroup {
    Panel,
    Menu,
    Underlay,
    Overlay,
    BoardInteraction,
    Console,
    SchematicOverlay,
    Selection,
}
impl ScreenGroup {
    fn name(self) -> &'static str {
        match self {
            Self::Panel => "panel",
            Self::Menu => "menu_overlay",
            Self::Underlay => "viewport_underlay",
            Self::Overlay => "viewport_overlay",
            Self::BoardInteraction => "board_interaction",
            Self::Console => "console_overlay",
            Self::SchematicOverlay => "schematic_overlay",
            Self::Selection => "selection_strokes",
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct EncodedScreenGeometry {
    pub group: &'static str,
    pub vertices: u64,
    pub commands: u64,
    /// Original virtual source range only for exactly one draw command.
    pub range: Option<[u32; 2]>,
    /// Exact only when all recorded commands share this scissor. None is not zero coverage.
    pub scissor: Option<[u32; 4]>,
}
impl Renderer {
    pub(crate) fn observe_screen_draw(&self, group: ScreenGroup, vertices: u32, scissor: [u32; 4]) {
        self.observe_screen_range(group, 0..vertices, scissor);
    }
    pub(crate) fn observe_screen_range(
        &self,
        group: ScreenGroup,
        range: std::ops::Range<u32>,
        scissor: [u32; 4],
    ) {
        if self.frame_observer.is_none() {
            return;
        }
        let Some(mut draws) = self.screen_admission.get() else {
            return;
        };
        let Some(vertices) = range.end.checked_sub(range.start) else {
            self.screen_admission.set(None);
            return;
        };
        let previous = draws[group as usize];
        let Some(commands) = previous.map_or(Some(1), |d| d.commands.checked_add(1)) else {
            self.screen_admission.set(None);
            return;
        };
        let Some(total) = previous.map_or(Some(u64::from(vertices)), |d| {
            d.vertices.checked_add(u64::from(vertices))
        }) else {
            self.screen_admission.set(None);
            return;
        };
        // Bands and damaged tiles legitimately revisit a stream. Keep actual
        // command totals without presenting a bounding rectangle as coverage.
        let common_scissor = previous.map_or(Some(scissor), |d| {
            d.scissor.filter(|previous| *previous == scissor)
        });
        draws[group as usize] = Some(EncodedScreenGeometry {
            group: group.name(),
            vertices: total,
            commands,
            range: previous.is_none().then_some([range.start, range.end]),
            scissor: common_scissor,
        });
        self.screen_admission.set(Some(draws));
    }
    /// Only valid during the frame callback. `None` means unavailable/invalid,
    /// never zero draws; callers separately establish successful submission.
    pub fn encoded_screen_geometry(&self) -> Option<[Option<EncodedScreenGeometry>; 8]> {
        self.screen_admission.get()
    }
}

pub(crate) fn scissor(rect: RectPx) -> [u32; 4] {
    [
        rect.x.max(0.0).floor() as u32,
        rect.y.max(0.0).floor() as u32,
        rect.width.max(1.0).ceil() as u32,
        rect.height.max(1.0).ceil() as u32,
    ]
}
