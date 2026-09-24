use super::*;

#[test]
fn borrowed_paragraphs_match_installed_ascii_and_unicode_boundaries_without_allocation() {
    let delimiters = [
        "", "\n", "\r", "\r\n", "\n\r", "\u{1c}", "\u{1d}", "\u{1e}", "\u{85}", "\u{2028}",
        "\u{2029}",
    ];
    for prefix in ["", "a", "א", "ع", "a\u{2067}ב\u{2069}"] {
        for first in delimiters {
            for second in delimiters {
                let text = format!("{prefix}{first}{second}{prefix}{first}");
                let expected: Vec<_> = glyphon::cosmic_text::BidiParagraphs::new(&text).collect();
                let scope = crate::cpu_alloc::Scope::new("borrowed-paragraphs");
                scope.with(|| assert!(Paragraphs::new(&text).eq(expected.into_iter()), "{text:?}"));
                assert_eq!(scope.usage().peak_payload_bytes, 0);
            }
        }
    }
    let text = "א rich hidden paragraph\n".repeat(10000);
    let scope = crate::cpu_alloc::Scope::new("hidden-paragraph-prefix");
    scope.with(|| assert_eq!(Paragraphs::new(&text).take(2).count(), 2));
    assert_eq!(scope.usage().allocations, 0);
    assert_eq!(scope.usage().peak_payload_bytes, 0);
}

#[test]
fn attributes_reserve_before_construction_cover_peaks_and_release_with_storage() {
    let host = Budget::new(16 * 1024 * 1024);
    let defaults = crate::text_attrs(crate::TextFace::Ui);
    for count in [0, 1, 11, 12, 127, 511] {
        let scope = crate::cpu_alloc::Scope::new("attribute-construction-proof");
        let admitted = construction_bytes(&defaults, count).unwrap();
        let mut attrs = scope
            .with(|| Attributes::new(&defaults, count, &host))
            .unwrap();
        assert_eq!(host.used(), admitted);
        scope.with(|| {
            for index in 0..count {
                attrs.add_span(
                    index * 2..index * 2 + 2,
                    &defaults.clone().metadata(index + 1),
                );
            }
        });
        let usage = scope.usage();
        assert!(usage.peak_payload_bytes + usage.tracking_bytes <= admitted);
        assert_eq!(attrs.spans_iter().count(), count);
        drop(attrs);
        assert_eq!(host.used(), 0);
        assert_eq!(scope.usage().allocations, 0);
    }
    let scope = crate::cpu_alloc::Scope::new("attribute-preconstruction-refusal");
    assert!(
        scope
            .with(|| Attributes::new(&defaults, 1, &Budget::new(0)))
            .is_err()
    );
    // Error construction may allocate, but no attribute owner survives refusal.
    assert_eq!(scope.usage().allocations, 0);
    assert!(construction_bytes(&defaults, usize::MAX).is_err());
}
