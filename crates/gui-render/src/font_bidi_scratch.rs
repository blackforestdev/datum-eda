//! Pre-call scratch admission for the pinned UTF-8 bidi analysis used by shaping.
use crate::text_gpu::budget::{Budget, Permit, staging_process};
use std::sync::Arc;

/// The installed unicode-bidi path stores byte-indexed classes/levels, paragraph
/// records, level runs, isolating sequences and their stacks, and bracket pairs.
/// Sixteen vector families, each at most four input units of capacity per byte
/// and at most32 bytes per element, give2048 bytes/input byte. Payloads moved
/// between run sequences are not duplicated. Fixed headroom includes bounded
/// explicit/bracket stacks, minimum capacities and allocation headers.
/// ASCII follows the pure-LTR path: only classes, levels, paragraph flags/info
/// and ShapeLine's adjusted-level clone are allocated;128 bytes/byte bounds it.
///
/// This is NOT a bound on font loading/caches, glyph outputs or rasterization.
/// Those have separate owners; dependency/toolchain changes require re-review.
fn bytes(text: &str) -> anyhow::Result<u64> {
    let per_byte = if text.is_ascii() { 128u64 } else { 2048u64 };
    (text.len() as u64)
        .checked_mul(per_byte)
        .and_then(|n| n.checked_add(32 * 1024))
        .ok_or_else(|| anyhow::anyhow!("bidi scratch construction capacity overflow"))
}

pub(super) fn required(text: &str) -> anyhow::Result<u64> {
    bytes(text)
}
pub(super) fn reserve(bytes: u64, host: &Arc<Budget>) -> anyhow::Result<[Permit; 2]> {
    Ok([host.reserve(bytes)?, staging_process().reserve(bytes)?])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::text_layout::fonts::{Fonts, Source};

    #[test]
    fn bidi_pressure_refuses_before_shaping_and_retries_without_changing_glyphs() {
        let host = Budget::new(16 * 1024 * 1024);
        let mut fonts = Fonts::new(host.clone());
        let attrs = glyphon::AttrsList::new(&crate::text_attrs(crate::TextFace::Ui));
        let text = "mixed אבג (123) \u{2067}ع\u{2069}";
        let expected = format!("{:?}", &*fonts.shape(text, &attrs).unwrap());
        fonts.release_for(host.available() + 1);
        let before = fonts.usage().allocation;
        let held = host.reserve(host.available()).unwrap();
        assert!(fonts.shape(text, &attrs).is_err());
        assert_eq!(
            fonts.usage().allocation.peak_payload_bytes,
            before.peak_payload_bytes
        );
        assert_eq!(fonts.usage().returned_shape_bytes, 0);
        drop(held);
        let actual = fonts.shape(text, &attrs).unwrap();
        assert_eq!(format!("{:?}", &*actual), expected);
        assert_eq!(host.used(), fonts.reserved_bytes());
    }

    #[test]
    fn bidi_reservation_covers_both_profiles_and_rolls_back_on_refusal() {
        let ascii = "a\r\nb\t (123)";
        let complex = "\u{2067}א\u{2066}(abc)\u{2069}ع\u{2069}";
        for text in [ascii, complex] {
            let amount = bytes(text).unwrap();
            let host = Budget::new(amount);
            let held = reserve(amount, &host).unwrap();
            assert_eq!(host.used(), amount);
            assert!(reserve(1, &host).is_err());
            drop(held);
            assert_eq!(host.used(), 0);
        }
    }
}
