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
    pub(crate) const DRC_ERROR: Rgb = srgb(0xFF, 0x4D, 0x4D);
    pub(crate) const DRC_WARN: Rgb = srgb(0xFF, 0xB0, 0x2E);
    pub(crate) const EXCLUSION: Rgb = srgb(0x6B, 0x72, 0x80);
    pub(crate) const SELECTION: Rgb = chrome::ACCENT;
}

pub(crate) mod typography {
    pub(crate) const DISPLAY_SIZE: f32 = 16.0;
    pub(crate) const DISPLAY_LINE: f32 = 22.0;
    pub(crate) const HEADER_SIZE: f32 = 12.0;
    pub(crate) const HEADER_LINE: f32 = 16.0;
    pub(crate) const BODY_SIZE: f32 = 13.0;
    pub(crate) const BODY_LINE: f32 = 18.0;
    pub(crate) const STRONG_SIZE: f32 = 13.0;
    pub(crate) const STRONG_LINE: f32 = 18.0;
    pub(crate) const DATA_SIZE: f32 = 12.0;
    pub(crate) const DATA_LINE: f32 = 16.0;
    pub(crate) const CAPTION_SIZE: f32 = 11.0;
    pub(crate) const CAPTION_LINE: f32 = 14.0;
    pub(crate) const MICRO_SIZE: f32 = 10.0;
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
