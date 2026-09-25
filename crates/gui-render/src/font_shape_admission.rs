//! Pre-call admission of public Basic-shaping outputs, without dependency changes.
use super::{Shape, Source};
use crate::text_buffer_cache::budget::Owner;
use glyphon::{AttrsList, ShapeGlyph, ShapeSpan, ShapeWord};
use std::sync::Arc;

/// The installed Basic shaper emits at most one glyph per Unicode scalar and
/// partitions input into spans/words. Each count is therefore bounded by UTF-8
/// bytes. Its fresh Vec growth is at most twice the requested count, with a
/// four-element minimum. ShapeLine also reserves input bytes plus one span.
/// Four times (bytes + 1) bounds every summed capacity, including one-element
/// words. Charging a tracking header per slot overestimates nested headers.
///
/// This bounds returned output and its growth, not private font/bidi scratch.
/// Keep the pinned implementation review and independent capacity tests with
/// dependency upgrades; no private layout is read or third-party source copied.
fn bound(text: &str) -> anyhow::Result<usize> {
    let overflow = || anyhow::anyhow!("shaped text construction capacity overflow");
    let units = text.len().checked_add(1).ok_or_else(overflow)?;
    let slots = units.checked_mul(4).ok_or_else(overflow)?;
    let (spans, words, glyphs) = if text
        .bytes()
        .all(|b| b.is_ascii_graphic() || b == b' ' || b == b'\t')
    {
        // Printable ASCII has one bidi span. A contiguous alphanumeric run has
        // no internal Unicode line-break opportunity; charge every other scalar
        // as a separate word, conservatively including punctuation and whitespace.
        let mut words = 0usize;
        let mut alphanumeric = false;
        for byte in text.bytes() {
            let next = byte.is_ascii_alphanumeric();
            words += usize::from(!next || !alphanumeric);
            alphanumeric = next;
        }
        (
            units.max(4),
            words.saturating_mul(2).max(4),
            text.len()
                .saturating_mul(2)
                .saturating_add(words.saturating_mul(4)),
        )
    } else {
        (slots, slots, slots)
    };
    // Charge a header for every slot, an upper bound on separate nested vectors.
    [
        (
            spans,
            crate::cpu_alloc::heap::capacity_bytes::<ShapeSpan>(1),
        ),
        (
            words,
            crate::cpu_alloc::heap::capacity_bytes::<ShapeWord>(1),
        ),
        (
            glyphs,
            crate::cpu_alloc::heap::capacity_bytes::<ShapeGlyph>(1),
        ),
    ]
    .into_iter()
    .try_fold(super::shape_container_bytes(), |bytes, (count, size)| {
        count
            .checked_mul(size)
            .and_then(|n| bytes.checked_add(n))
            .ok_or_else(overflow)
    })
}

pub(super) fn shape(
    fonts: &mut (impl Source + ?Sized),
    text: &str,
    attrs: &AttrsList,
    owner: Option<&Owner>,
) -> anyhow::Result<Arc<Shape>> {
    let maximum = bound(text)?;
    let mut reservation = owner.map(|owner| owner.reserve(maximum)).transpose()?;
    let mut result = fonts.shape(text, attrs)?;
    let actual = result
        .payload_bytes()
        .checked_add(super::shape_container_bytes())
        .ok_or_else(|| anyhow::anyhow!("shaped text capacity overflow"))?;
    anyhow::ensure!(
        actual <= maximum,
        "shaper exceeded reviewed output capacity bound"
    );
    // Include the Arc before allocating it; the lease survives through publication
    // or until a rejected/abandoned shape releases all its nested allocations.
    if let Some(reservation) = &mut reservation {
        reservation.shrink_to(actual);
    }
    result.construction = std::sync::Mutex::new(reservation);
    Ok(Arc::new(result))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu_alloc::Scope;

    #[test]
    fn public_output_bound_covers_basic_unicode_and_rich_shape_capacities() {
        let mut fonts = crate::load_datum_fonts();
        let mut attrs = AttrsList::new(&crate::text_attrs(crate::TextFace::Ui));
        attrs.add_span(2..8, &crate::text_attrs(crate::TextFace::Ui).metadata(1));
        for source in [
            "",
            "a",
            " ",
            "\t",
            "a b c",
            "مرحبا world",
            "a\u{301}😀中",
            "\u{2067}abc\u{2069}",
        ] {
            for repeats in [1, 3, 17, 257] {
                let text = source.repeat(repeats);
                let scope = Scope::new("shape-bound-proof");
                let output = scope
                    .with(|| fonts.shape_admitted(&text, &attrs, None))
                    .unwrap();
                assert!(
                    output.payload_bytes() + super::super::shape_container_bytes()
                        <= bound(&text).unwrap()
                );
                drop(output);
                // FontSystem's private retained caches are a separate owner;
                // this test intentionally does not assert a private-scratch bound.
            }
        }
    }

    #[test]
    fn construction_reservation_tracks_exact_shared_output_until_publication() {
        let owner = Owner::new(0);
        let mut fonts = crate::load_datum_fonts();
        let attrs = AttrsList::new(&crate::text_attrs(crate::TextFace::Ui));
        let usage = || {
            crate::Renderer::text_cache_process_usage()
                .into_iter()
                .find(|entry| entry.owner_id == owner.id())
                .unwrap()
        };
        let shape = fonts
            .shape_admitted("Shared output", &attrs, Some(&owner))
            .unwrap();
        let exact = shape.payload_bytes() + super::super::shape_container_bytes();
        assert_eq!(usage().constructing_bytes, exact);
        let shared = shape.clone();
        drop(shape);
        assert_eq!(usage().constructing_bytes, exact);
        owner.publish(exact);
        shared.published();
        assert_eq!(usage().constructing_bytes, 0);
        assert_eq!(usage().bytes, exact);
        drop(shared);
        owner.publish(0);
        let shape = fonts
            .shape_admitted("Abandoned output", &attrs, Some(&owner))
            .unwrap();
        assert!(usage().constructing_bytes > 0);
        drop(shape);
        assert_eq!(usage().constructing_bytes, 0);
    }

    #[test]
    fn oversized_shape_is_refused_before_the_font_owner_allocates() {
        let owner = Owner::new(0);
        let host = crate::text_gpu::budget::Budget::new(16 * 1024 * 1024);
        let mut fonts = super::super::Fonts::new(host).unwrap();
        let before = fonts.usage();
        let text = "x".repeat(128 * 1024);
        let attrs = AttrsList::new(&crate::text_attrs(crate::TextFace::Ui));
        assert!(fonts.shape_admitted(&text, &attrs, Some(&owner)).is_err());
        let after = fonts.usage();
        assert_eq!(
            before.allocation.peak_payload_bytes,
            after.allocation.peak_payload_bytes
        );
        assert_eq!(
            before.allocation.payload_bytes,
            after.allocation.payload_bytes
        );
        assert_eq!(after.returned_shape_bytes, 0);
        assert_eq!(fonts.reserved_bytes(), 0);
    }
}
