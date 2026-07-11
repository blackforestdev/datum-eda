//! Design Book token mirror.
//!
//! The canonical values live in `docs/gui/VISUAL_LANGUAGE.md` section 2.
//! This module mirrors those tracked tables for renderer consumption; do not
//! introduce ad-hoc chrome colors or type/spacing values outside this seam.

#![allow(dead_code)]

pub(crate) type Rgb = [f32; 3];

const fn srgb(r: u8, g: u8, b: u8) -> Rgb {
    [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0]
}

pub(crate) mod chrome {
    use super::{Rgb, srgb};

    pub(crate) const CANVAS: Rgb = srgb(0x0B, 0x0C, 0x0E);
    pub(crate) const BG_BASE: Rgb = srgb(0x12, 0x13, 0x18);
    pub(crate) const SURFACE_01: Rgb = srgb(0x18, 0x1B, 0x21);
    pub(crate) const SURFACE_02: Rgb = srgb(0x1F, 0x23, 0x2A);
    pub(crate) const SURFACE_03: Rgb = srgb(0x27, 0x2C, 0x35);
    pub(crate) const BORDER_SUBTLE: Rgb = srgb(0x2E, 0x34, 0x3E);
    pub(crate) const BORDER_STRONG: Rgb = srgb(0x3A, 0x41, 0x4D);

    pub(crate) const TEXT_PRIMARY: Rgb = srgb(0xE4, 0xE7, 0xEB);
    pub(crate) const TEXT_SECONDARY: Rgb = srgb(0xB2, 0xB8, 0xC3);
    pub(crate) const TEXT_MUTED: Rgb = srgb(0x71, 0x78, 0x85);
    pub(crate) const TEXT_ON_ACCENT: Rgb = srgb(0x14, 0x16, 0x19);

    pub(crate) const ACCENT: Rgb = srgb(0xCE, 0x5A, 0x92);
    pub(crate) const ACCENT_HOVER: Rgb = srgb(0xD8, 0x6E, 0xA0);
    pub(crate) const ACCENT_PRESSED: Rgb = srgb(0xB8, 0x4A, 0x80);
    pub(crate) const ACCENT_TINT: Rgb = srgb(0x2A, 0x1D, 0x25);

    pub(crate) const STATUS_ERROR: Rgb = srgb(0xE5, 0x53, 0x4B);
    pub(crate) const STATUS_WARN: Rgb = srgb(0xE0, 0xA2, 0x3A);
    pub(crate) const STATUS_SUCCESS: Rgb = srgb(0x4F, 0xA7, 0x5A);
    pub(crate) const STATUS_INFO: Rgb = srgb(0x5B, 0x8B, 0xD0);
}

pub(crate) mod content {
    use super::{Rgb, chrome, srgb};

    pub(crate) const COPPER_FRONT: Rgb = srgb(0xC8, 0x3A, 0x34);
    pub(crate) const COPPER_BACK: Rgb = srgb(0x4D, 0x7F, 0xC4);
    pub(crate) const COPPER_IN1: Rgb = srgb(0x4F, 0xA7, 0x5A);
    pub(crate) const COPPER_IN2: Rgb = srgb(0xC2, 0xA1, 0x3A);
    pub(crate) const SILK_TOP: Rgb = srgb(0xE8, 0xE6, 0xDC);
    pub(crate) const SILK_BOTTOM: Rgb = srgb(0x96, 0x9B, 0xA1);
    pub(crate) const MASK: Rgb = srgb(0x2F, 0xA3, 0x8C);
    pub(crate) const PASTE: Rgb = srgb(0x8C, 0x92, 0x99);
    pub(crate) const EDGE: Rgb = srgb(0xCB, 0xB2, 0x4A);
    pub(crate) const PAD: Rgb = srgb(0xC9, 0x97, 0x4A);
    pub(crate) const VIA: Rgb = srgb(0xC7, 0x7B, 0x3C);
    pub(crate) const RATSNEST: Rgb = srgb(0xAE, 0xB4, 0xBB);
    // Board substrate sits one step below chrome::CANVAS and SURFACE_01 — the
    // dark field the copper/silk/edge sit on. The grid is a whisper near
    // #171A20, far below chrome BORDER_STRONG/SUBTLE: major slightly stronger,
    // minor barely visible.
    pub(crate) const BOARD_SUBSTRATE: Rgb = srgb(0x0E, 0x10, 0x13);
    pub(crate) const BOARD_GRID_MAJOR: Rgb = srgb(0x1A, 0x1D, 0x24);
    pub(crate) const BOARD_GRID_MINOR: Rgb = srgb(0x14, 0x16, 0x1B);
    pub(crate) const DRC_ERROR: Rgb = chrome::STATUS_ERROR;
    pub(crate) const DRC_WARN: Rgb = chrome::STATUS_WARN;
    pub(crate) const EXCLUSION: Rgb = srgb(0x6B, 0x72, 0x80);
    pub(crate) const SELECTION: Rgb = chrome::ACCENT;
}

/// Schematic net-role colours (P2.2c). These mirror the schematic-editor
/// prototype tokens (`docs/gui/prototypes/schematic-editor.html` :16-21); each
/// alias reuses the existing Design Book value whose hex already matches, so the
/// schematic pane and the board pane draw from one palette rather than a rival
/// copy. The schematic colour path in `draw_primitives` maps `Schematic.*`
/// layer names to these.
pub(crate) mod schematic {
    use super::{Rgb, chrome, content, srgb};

    pub(crate) const WIRE: Rgb = content::COPPER_IN1; // --wire #4FA75A
    pub(crate) const SYMBOL: Rgb = content::RATSNEST; // --sym  #AEB4BB
    pub(crate) const REFDES: Rgb = chrome::TEXT_PRIMARY; // --tx   #E4E7EB
    pub(crate) const PIN_NAME: Rgb = chrome::TEXT_SECONDARY; // --tx2 #B2B8C3
    pub(crate) const VALUE: Rgb = chrome::TEXT_MUTED; // --tx3  #717885
    // P2.2e typed-object colours (schematic-editor prototype :21). Bus is the gold
    // signal-bundle path (`--bus`, reusing the existing COPPER_IN2 whose hex matches
    // exactly); power flags/stacks are the cool grey `--pwr`; global/hierarchical
    // net-label tags take the `--info` blue (reusing STATUS_INFO, hex-exact).
    pub(crate) const BUS: Rgb = content::COPPER_IN2; // --bus  #C2A13A
    pub(crate) const POWER: Rgb = srgb(0xB7, 0xBE, 0xC9); // --pwr  #B7BEC9
    pub(crate) const GLOBAL_LABEL: Rgb = chrome::STATUS_INFO; // --info #5B8BD0
    // Schematic canvas grid (P2.2f). A subtle SQUARE line grid, one whisper above
    // the schematic substrate — the prototype's `#sgrid` (`schematic-editor.html`)
    // is a single uniform `#141821`; MINOR matches it exactly and MAJOR sits a
    // touch stronger so the coarse tier reads as a sensible pitch without ever
    // competing with the green wires.
    pub(crate) const GRID_MINOR: Rgb = srgb(0x14, 0x18, 0x21); // --sgrid #141821
    pub(crate) const GRID_MAJOR: Rgb = srgb(0x18, 0x1C, 0x27);
}

pub(crate) mod typography {
    pub(crate) const DISPLAY_SIZE: f32 = 16.0;
    pub(crate) const DISPLAY_WEIGHT: u16 = 600;
    pub(crate) const DISPLAY_LINE: f32 = 22.0;
    pub(crate) const HEADER_SIZE: f32 = 12.0;
    pub(crate) const HEADER_WEIGHT: u16 = 600;
    pub(crate) const HEADER_LINE: f32 = 16.0;
    pub(crate) const BODY_SIZE: f32 = 13.0;
    pub(crate) const BODY_WEIGHT: u16 = 400;
    pub(crate) const BODY_LINE: f32 = 18.0;
    pub(crate) const STRONG_SIZE: f32 = 13.0;
    pub(crate) const STRONG_WEIGHT: u16 = 500;
    pub(crate) const STRONG_LINE: f32 = 18.0;
    pub(crate) const DATA_SIZE: f32 = 12.0;
    pub(crate) const DATA_WEIGHT: u16 = 400;
    pub(crate) const DATA_LINE: f32 = 16.0;
    pub(crate) const CAPTION_SIZE: f32 = 11.0;
    pub(crate) const CAPTION_WEIGHT: u16 = 400;
    pub(crate) const CAPTION_LINE: f32 = 14.0;
    pub(crate) const MICRO_SIZE: f32 = 10.0;
    pub(crate) const MICRO_WEIGHT: u16 = 500;
    pub(crate) const MICRO_LINE: f32 = 12.0;
}

pub(crate) mod spacing {
    pub(crate) const SP_01: f32 = 2.0;
    pub(crate) const SP_02: f32 = 4.0;
    pub(crate) const SP_03: f32 = 8.0;
    pub(crate) const SP_04: f32 = 12.0;
    pub(crate) const SP_05: f32 = 16.0;
    pub(crate) const SP_06: f32 = 24.0;
    pub(crate) const SP_07: f32 = 32.0;
    pub(crate) const SP_08: f32 = 40.0;
    pub(crate) const SP_09: f32 = 48.0;
    pub(crate) const SP_10: f32 = 64.0;
    pub(crate) const SP_11: f32 = 80.0;
    pub(crate) const SP_12: f32 = 96.0;
    pub(crate) const SP_13: f32 = 160.0;
}

pub(crate) mod radius {
    pub(crate) const SM: f32 = 4.0;
    pub(crate) const MD: f32 = 6.0;
    pub(crate) const LG: f32 = 8.0;
}
