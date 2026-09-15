use std::collections::BTreeMap;

use serde_json::Value;

use super::{
    ANGLE_PRECISION_KEY, BOARD_PRECISION_KEY, DRILL_PRECISION_KEY, SCHEMATIC_PRECISION_KEY,
};

const RETIRED_LENGTH_PRECISION_KEY: &str = "datum.units.length_precision";
const RETIRED_ANGLE_FORMAT_KEY: &str = "datum.units.angle_format";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LegacyAngleFormat {
    pub notation: String,
    pub precision: Value,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LegacyMigrationEvidence {
    PreservedInvalid { key: String, value: Value },
    PreservedUnsupported { key: String, value: Value },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LegacyUnitsMigration {
    /// Contributions only. The repository applies this map as one transaction.
    pub contributions: BTreeMap<String, Value>,
    pub evidence: Vec<LegacyMigrationEvidence>,
}

/// Plans the deterministic legacy fan-out without mutating its input. Existing
/// new-key values win independently; an invalid shared value contributes none
/// of the three precision keys, preventing a partial guessed profile.
pub fn migrate_legacy_units(values: &BTreeMap<String, Value>) -> LegacyUnitsMigration {
    let mut contributions = BTreeMap::new();
    let mut evidence = Vec::new();
    if let Some(value) = values.get(RETIRED_LENGTH_PRECISION_KEY) {
        match value.as_str().and_then(legacy_length_precision) {
            Some(token) => {
                for key in [
                    BOARD_PRECISION_KEY,
                    DRILL_PRECISION_KEY,
                    SCHEMATIC_PRECISION_KEY,
                ] {
                    if !values.contains_key(key) {
                        contributions.insert(key.to_owned(), Value::String(token.to_owned()));
                    }
                }
            }
            None => evidence.push(LegacyMigrationEvidence::PreservedInvalid {
                key: RETIRED_LENGTH_PRECISION_KEY.to_owned(),
                value: value.clone(),
            }),
        }
    }
    if let Some(value) = values.get(RETIRED_ANGLE_FORMAT_KEY) {
        match migrate_angle_format(value) {
            Ok(token) if !values.contains_key(ANGLE_PRECISION_KEY) => {
                contributions.insert(ANGLE_PRECISION_KEY.to_owned(), Value::String(token));
            }
            Ok(_) => {}
            Err(unsupported) => evidence.push(unsupported),
        }
    }
    LegacyUnitsMigration {
        contributions,
        evidence,
    }
}

fn legacy_length_precision(token: &str) -> Option<&'static str> {
    match token {
        "0.1" => Some("decimal_1"),
        "0.01" => Some("decimal_2"),
        "0.001" => Some("decimal_3"),
        "0.0001" => Some("decimal_4"),
        "exact_nm" => Some("exact_nm"),
        _ => None,
    }
}

fn migrate_angle_format(value: &Value) -> Result<String, LegacyMigrationEvidence> {
    let Some(object) = value.as_object() else {
        return Err(LegacyMigrationEvidence::PreservedInvalid {
            key: RETIRED_ANGLE_FORMAT_KEY.to_owned(),
            value: value.clone(),
        });
    };
    let Some(notation) = object.get("notation").and_then(Value::as_str) else {
        return Err(LegacyMigrationEvidence::PreservedInvalid {
            key: RETIRED_ANGLE_FORMAT_KEY.to_owned(),
            value: value.clone(),
        });
    };
    if notation != "decimal_degrees" {
        return Err(LegacyMigrationEvidence::PreservedUnsupported {
            key: RETIRED_ANGLE_FORMAT_KEY.to_owned(),
            value: value.clone(),
        });
    }
    let precision = object.get("precision");
    let token = match precision {
        Some(Value::Number(number)) if number.as_f64() == Some(1.0) => "decimal_0",
        Some(Value::Number(number)) if number.as_f64() == Some(0.1) => "decimal_1",
        Some(Value::Number(number)) if number.as_f64() == Some(0.01) => "decimal_2",
        Some(Value::Number(number)) if number.as_f64() == Some(0.001) => "decimal_3",
        _ => {
            return Err(LegacyMigrationEvidence::PreservedInvalid {
                key: RETIRED_ANGLE_FORMAT_KEY.to_owned(),
                value: value.clone(),
            });
        }
    };
    Ok(token.to_owned())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn shared_precision_fans_out_atomically_and_new_values_win() {
        let values = BTreeMap::from([
            (RETIRED_LENGTH_PRECISION_KEY.to_owned(), json!("0.001")),
            (BOARD_PRECISION_KEY.to_owned(), json!("exact_nm")),
        ]);
        let plan = migrate_legacy_units(&values);
        assert_eq!(plan.contributions.len(), 2);
        assert_eq!(plan.contributions[DRILL_PRECISION_KEY], json!("decimal_3"));
        assert_eq!(
            plan.contributions[SCHEMATIC_PRECISION_KEY],
            json!("decimal_3")
        );
        assert!(!plan.contributions.contains_key(BOARD_PRECISION_KEY));
        assert!(plan.evidence.is_empty());
    }

    #[test]
    fn invalid_shared_precision_never_partially_seeds() {
        let values = BTreeMap::from([(RETIRED_LENGTH_PRECISION_KEY.to_owned(), json!("bad"))]);
        let plan = migrate_legacy_units(&values);
        assert!(plan.contributions.is_empty());
        assert_eq!(plan.evidence.len(), 1);
    }

    #[test]
    fn only_exact_decimal_degree_drafts_migrate() {
        let valid = BTreeMap::from([(
            RETIRED_ANGLE_FORMAT_KEY.to_owned(),
            json!({"notation":"decimal_degrees","precision":0.01}),
        )]);
        assert_eq!(
            migrate_legacy_units(&valid).contributions[ANGLE_PRECISION_KEY],
            json!("decimal_2")
        );
        for notation in ["dms", "radians"] {
            let values = BTreeMap::from([(
                RETIRED_ANGLE_FORMAT_KEY.to_owned(),
                json!({"notation":notation,"precision":0.001}),
            )]);
            let plan = migrate_legacy_units(&values);
            assert!(plan.contributions.is_empty());
            assert!(matches!(
                plan.evidence[0],
                LegacyMigrationEvidence::PreservedUnsupported { .. }
            ));
        }
    }
}
