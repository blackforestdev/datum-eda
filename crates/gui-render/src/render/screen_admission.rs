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
    /// First draw only, for compatibility with single-draw evidence.
    pub scissor: [u32; 4],
    pub draws: u32,
    scissors: [[u32; 4]; 32],
}
impl EncodedScreenGeometry {
    pub fn scissors(&self) -> &[[u32; 4]] {
        &self.scissors[..self.draws as usize]
    }
}
impl Renderer {
    pub(crate) fn observe_screen_draw_clipped(
        &self,
        group: ScreenGroup,
        vertices: u32,
        scissor: Option<[u32; 4]>,
    ) {
        if let Some(scissor) = scissor {
            self.observe_screen_draw(group, vertices, scissor);
        }
    }
    pub(crate) fn observe_screen_draw(&self, group: ScreenGroup, vertices: u32, scissor: [u32; 4]) {
        if self.frame_observer.is_none() {
            return;
        }
        let mut state = self.screen_admission.borrow_mut();
        let Some(draws) = state.as_mut() else {
            return;
        };
        if let Some(previous) = &mut draws[group as usize] {
            let rect = |s: [u32; 4]| crate::gpu_frame::interaction_damage::Region {
                x: s[0],
                y: s[1],
                width: s[2],
                height: s[3],
            };
            // Repeated stream draws are valid only over disjoint damage regions.
            // An accidental duplicate/overlap still invalidates the observation.
            if previous.vertices != vertices
                || previous.draws == 32
                || previous
                    .scissors()
                    .iter()
                    .any(|old| rect(*old).intersect(rect(scissor)).is_some())
            {
                *state = None;
                return;
            }
            previous.scissors[previous.draws as usize] = scissor;
            previous.draws += 1;
        } else {
            let mut scissors = [[0; 4]; 32];
            scissors[0] = scissor;
            draws[group as usize] = Some(EncodedScreenGeometry {
                group: group.name(),
                vertices,
                scissor,
                draws: 1,
                scissors,
            });
        }
    }
    /// Valid only during the frame callback. None means unavailable/invalid.
    pub fn encoded_screen_geometry(&self) -> Option<[Option<EncodedScreenGeometry>; 8]> {
        *self.screen_admission.borrow()
    }
}
