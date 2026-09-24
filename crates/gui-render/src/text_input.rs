//! Borrowed rich paragraphs and admitted temporary text attributes.
use crate::text_gpu::budget::{Budget, Permit, staging_process};
use glyphon::{Attrs, AttrsList, Style, Weight};
use std::sync::Arc;

/// Preserve the installed paragraph adapter's ASCII newline profile and Unicode
/// paragraph separators without constructing a document-wide bidi/table buffer.
/// Bidi resolution for a visible paragraph remains in the shared font owner.
pub(super) struct Paragraphs<'a> {
    parts: std::str::SplitTerminator<'a, fn(char) -> bool>,
    ascii: bool,
}
impl<'a> Paragraphs<'a> {
    pub fn new(text: &'a str) -> Self {
        let ascii = text.is_ascii()
            && text
                .bytes()
                .all(|b| !b.is_ascii_control() || matches!(b, b'\n' | b'\r' | b'\t'));
        let separator: fn(char) -> bool = if ascii {
            |c| c == '\n'
        } else {
            |c| matches!(c, '\n' | '\r' | '\u{1c}'..='\u{1e}' | '\u{85}' | '\u{2029}')
        };
        Self {
            parts: text.split_terminator(separator),
            ascii,
        }
    }
}
impl<'a> Iterator for Paragraphs<'a> {
    type Item = &'a str;
    fn next(&mut self) -> Option<Self::Item> {
        self.parts.next().map(|part| {
            if self.ascii {
                part.strip_suffix('\r').unwrap_or(part)
            } else {
                part
            }
        })
    }
}

/// Field order releases constructed storage before returning reserved headroom.
/// The bound covers default/span families, feature clones, tree nodes and the
/// transient copies made while inserting disjoint paragraph attribute ranges.
pub(super) struct Attributes {
    list: AttrsList,
    _permits: [Permit; 2],
}
impl Attributes {
    pub fn rich(
        run: &crate::TextRun,
        start: usize,
        end: usize,
        defaults: &Attrs<'_>,
        host: &Arc<Budget>,
    ) -> anyhow::Result<Self> {
        let mut offset = 0;
        let spans = run
            .rich_spans
            .iter()
            .filter(|span| {
                let left = offset;
                offset += span.text.len();
                left.max(start) < offset.min(end)
            })
            .count();
        let mut attributes = Self::new(defaults, spans, host)?;
        let mut offset = 0;
        for (index, span) in run.rich_spans.iter().enumerate() {
            let span_end = offset + span.text.len();
            let left = offset.max(start);
            let right = span_end.min(end);
            if left < right {
                let mut style = defaults.clone().metadata(index + 1);
                if span.bold {
                    style = style.weight(Weight::BOLD);
                }
                if span.italic {
                    style = style.style(Style::Italic);
                }
                attributes.add_span(left - start..right - start, &style);
            }
            offset = span_end;
            if offset >= end {
                break;
            }
        }
        Ok(attributes)
    }

    pub fn new(defaults: &Attrs<'_>, spans: usize, host: &Arc<Budget>) -> anyhow::Result<Self> {
        let bytes = construction_bytes(defaults, spans)?;
        let permits = [host.reserve(bytes)?, staging_process().reserve(bytes)?];
        Ok(Self {
            list: AttrsList::new(defaults),
            _permits: permits,
        })
    }
}
impl std::ops::Deref for Attributes {
    type Target = AttrsList;
    fn deref(&self) -> &AttrsList {
        &self.list
    }
}
impl std::ops::DerefMut for Attributes {
    fn deref_mut(&mut self) -> &mut AttrsList {
        &mut self.list
    }
}

fn construction_bytes(defaults: &Attrs<'_>, spans: usize) -> anyhow::Result<u64> {
    use glyphon::cosmic_text::{AttrsOwned, Feature};
    use std::mem::size_of;
    let overflow = || anyhow::anyhow!("text attribute construction capacity overflow");
    let family = match defaults.family {
        glyphon::Family::Name(name) => name.len(),
        _ => 0,
    };
    let features = defaults
        .font_features
        .features
        .len()
        .checked_mul(size_of::<Feature>())
        .ok_or_else(overflow)?;
    // Pinned BTreeMap nodes contain eleven key/value slots and twelve edges.
    // Sixteen generously aligned slots plus256 bytes cover node metadata,
    // alignment and Datum's allocation header without reading private layouts.
    // At most two nodes per insertion plus one root covers splits. Adjacent
    // disjoint spans cannot create additional surviving ranges through splitting.
    let node = 16
        * (size_of::<AttrsOwned>() + 2 * size_of::<std::ops::Range<usize>>() + size_of::<usize>())
        + 256;
    let trees = if spans == 0 {
        0
    } else {
        spans
            .checked_mul(2)
            .and_then(|n| n.checked_add(1))
            .and_then(|n| n.checked_mul(node))
            .ok_or_else(overflow)?
    };
    // Stored values plus temporary defaults/styles and range-map clones. Family
    // strings use shared ownership; overcharge each value independently anyway.
    let values = spans
        .checked_add(4)
        .and_then(|n| n.checked_mul(4))
        .ok_or_else(overflow)?;
    let nested = family
        .checked_add(features)
        .and_then(|n| n.checked_add(128))
        .and_then(|n| n.checked_mul(values))
        .ok_or_else(overflow)?;
    Ok(trees.checked_add(nested).ok_or_else(overflow)? as u64)
}

#[cfg(test)]
#[path = "text_input_tests.rs"]
mod tests;
