//! Bounded display-only serialization over borrowed complete inspection values.
//! The canonical field and all exact consumers remain unchanged.
use super::InspectionValue;
use std::io::{self, Write};
pub(super) fn elide(text: &str, columns: usize) -> String {
    if columns == 0 {
        return String::new();
    }
    let mut chars = text.chars().flat_map(|c| {
        match c {
            '\n' => [Some('\\'), Some('n')],
            '\r' => [Some('\\'), Some('r')],
            '\t' => [Some('\\'), Some('t')],
            _ => [Some(c), None],
        }
        .into_iter()
        .flatten()
    });
    let head: String = chars.by_ref().take(columns).collect();
    if chars.next().is_none() {
        head
    } else {
        head.chars()
            .take(columns.saturating_sub(1))
            .chain(std::iter::once('…'))
            .collect()
    }
}
struct Prefix {
    bytes: Vec<u8>,
    limit: usize,
    full: bool,
}
impl Write for Prefix {
    fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        let count = data.len().min(self.limit.saturating_sub(self.bytes.len()));
        self.bytes.extend_from_slice(&data[..count]);
        if count < data.len() {
            self.full = true;
            Err(io::Error::other("display preview complete"))
        } else {
            Ok(count)
        }
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
pub(super) fn value(value: &InspectionValue, columns: usize) -> String {
    match value {
        InspectionValue::Mixed => elide("Mixed", columns),
        InspectionValue::Unavailable(reason) => elide(
            &format!("Unavailable — {}", elide(reason, columns)),
            columns,
        ),
        InspectionValue::Common(value) => {
            if let Some(text) = value.as_str() {
                return elide(text, columns);
            }
            let mut prefix = Prefix {
                bytes: Vec::new(),
                limit: columns.saturating_mul(4),
                full: false,
            };
            // A width-derived stop bounds compound traversal and preview storage.
            // The intentional IO error affects this preview only.
            let result = serde_json::to_writer(&mut prefix, value);
            debug_assert!(result.is_ok() || prefix.full);
            if let Err(error) = std::str::from_utf8(&prefix.bytes) {
                prefix.bytes.truncate(error.valid_up_to());
            }
            let mut text = String::from_utf8(prefix.bytes).expect("serializer emits UTF-8");
            if prefix.full {
                text.push('…');
            }
            elide(&text, columns)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn large_values_and_unicode_are_bounded_without_altering_exact_fields() {
        let source = InspectionValue::Common(serde_json::json!((0..100_000).collect::<Vec<_>>()));
        let original = source.clone();
        let preview = value(&source, 32);
        assert!(preview.ends_with('…'));
        assert!(preview.chars().count() <= 32);
        assert_eq!(source, original);
        for columns in [0, 1, 2, 3, 8, 32] {
            let common = InspectionValue::Common(serde_json::json!("あ🙂é".repeat(100)));
            assert!(value(&common, columns).chars().count() <= columns);
        }
        assert_eq!(
            value(&InspectionValue::Common(serde_json::json!("a\nb\tc")), 32),
            "a\\nb\\tc"
        );
        assert_eq!(value(&InspectionValue::Mixed, 32), "Mixed");
        assert_eq!(
            value(&InspectionValue::Common(serde_json::json!(42)), 32),
            "42"
        );
        assert!(
            value(&InspectionValue::Unavailable("reason ".repeat(100_000)), 32)
                .starts_with("Unavailable")
        );
    }
}
