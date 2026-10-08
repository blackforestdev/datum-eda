//! Bounded display-only serialization over borrowed complete inspection values.
//! The canonical field and all exact consumers remain unchanged.
use super::InspectionValue;
use serde_json::Value;
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
fn human(value: &Value, columns: usize) -> String {
    match value {
        Value::Null => "None".into(),
        Value::Bool(value) => if *value { "Yes" } else { "No" }.into(),
        Value::Number(value) => value.to_string(),
        Value::String(value) => elide(value, columns),
        Value::Array(values) => format!("{} items", values.len()),
        Value::Object(values) => {
            let mut text = String::new();
            for (key, value) in values {
                if !text.is_empty() {
                    text.push_str(" · ");
                }
                // Structured coverage remains complete in the canonical field;
                // the display identifies its scope and count, never JSON IDs.
                let label = if key.parse::<uuid::Uuid>().is_ok() {
                    "Net".into()
                } else {
                    match key.as_str() {
                        "footprint" => "Parts".into(),
                        "board" => "Board".into(),
                        "schematic" => "Schematic".into(),
                        "track" => "Tracks".into(),
                        "symbol" => "Symbols".into(),
                        _ => key.rsplit(':').next().unwrap_or(key).replace('_', " "),
                    }
                };
                let value = match value {
                    Value::Array(items) => items.len().to_string(),
                    Value::Object(items) => format!("{} properties", items.len()),
                    _ => human(value, columns),
                };
                text.push_str(&format!("{label} {value}"));
                if text.chars().count() > columns {
                    break;
                }
            }
            if text.is_empty() { "None".into() } else { text }
        }
    }
}
pub(super) fn value(value: &InspectionValue, columns: usize) -> String {
    let text = match value {
        InspectionValue::Mixed => "Mixed".into(),
        InspectionValue::Unavailable(reason) => {
            let parsed = reason.find("\"reason\":").and_then(|start| {
                let text = &reason[start + "\"reason\":".len()..];
                // Decode only a bounded first cause, never the complete member
                // evidence array. Exact evidence remains in the canonical field.
                let prefix: String = text
                    .chars()
                    .take(columns.saturating_mul(8).max(64))
                    .collect();
                serde_json::Deserializer::from_str(&prefix)
                    .into_iter::<String>()
                    .next()?
                    .ok()
            });
            let reason = parsed.as_deref().unwrap_or(reason);
            let reason = if reason.trim_start().starts_with(['[', '{']) {
                "Source relationships unresolved"
            } else {
                reason.split(['[', '{']).next().unwrap_or(reason).trim()
            };
            let reason = match reason {
                "UnavailableAssignment" => "Electrical assignment unavailable",
                "StaleSource" | "StaleRevision" => "Source has changed",
                "AmbiguousAssignment" => "Electrical assignment is ambiguous",
                _ => reason,
            };
            format!("Unavailable — {}", elide(reason, columns))
        }
        InspectionValue::Common(value) => human(value, columns),
    };
    elide(&text, columns)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn large_values_and_unicode_are_bounded_without_altering_exact_fields() {
        let source = InspectionValue::Common(serde_json::json!((0..100_000).collect::<Vec<_>>()));
        let original = source.clone();
        let preview = value(&source, 32);
        assert_eq!(preview, "100000 items");
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
    #[test]
    fn coverage_and_unavailable_evidence_have_human_previews_without_json() {
        let source = InspectionValue::Common(serde_json::json!({"footprint":2,"track":3}));
        assert_eq!(value(&source, 80), "Parts 2 · Tracks 3");
        let reason = InspectionValue::Unavailable(
            "[{\"member\":\"id\",\"reason\":\"missing source\"}]".into(),
        );
        assert_eq!(value(&reason, 80), "Unavailable — missing source");
        assert_eq!(
            value(&InspectionValue::Common(serde_json::json!(true)), 8),
            "Yes"
        );
        assert_eq!(
            source,
            InspectionValue::Common(serde_json::json!({"footprint":2,"track":3}))
        );
    }
}
