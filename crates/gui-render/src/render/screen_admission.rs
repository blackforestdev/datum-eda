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
    SchematicUnderlay,
    SchematicOverlay,
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
            Self::SchematicUnderlay => "schematic_underlay",
            Self::SchematicOverlay => "schematic_overlay",
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct EncodedScreenGeometry {
    pub group: &'static str,
    pub vertices: u32,
    pub scissor: [u32; 4],
}
impl Renderer {
    pub(crate) fn observe_screen_draw(&self, group: ScreenGroup, vertices: u32, scissor: [u32; 4]) {
        if self.frame_observer.is_none() {
            return;
        }
        let Some(mut draws) = self.screen_admission.get() else {
            return;
        };
        // Every existing stream has one draw site per frame. A future duplicate
        // must invalidate observation rather than silently overwrite a record.
        if draws[group as usize].is_some() {
            self.screen_admission.set(None);
            return;
        }
        draws[group as usize] = Some(EncodedScreenGeometry {
            group: group.name(),
            vertices,
            scissor,
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
