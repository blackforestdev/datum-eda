use std::collections::BTreeMap;

use serde_json::Value;

use super::{
    DecimalDegreePrecision, LengthPrecisionChoice, LengthUnit, LengthUnitChoice, MeasurementSystem,
    QuantityUnits, UnitsProfile,
};

pub const SYSTEM_KEY: &str = "datum.units.system";
pub const BOARD_LENGTH_KEY: &str = "datum.units.board_length";
pub const BOARD_PRECISION_KEY: &str = "datum.units.board_length_precision";
pub const DRILL_HOLE_KEY: &str = "datum.units.drill_hole";
pub const DRILL_PRECISION_KEY: &str = "datum.units.drill_hole_precision";
pub const SCHEMATIC_GEOMETRY_KEY: &str = "datum.units.schematic_geometry";
pub const SCHEMATIC_PRECISION_KEY: &str = "datum.units.schematic_geometry_precision";
pub const ANGLE_PRECISION_KEY: &str = "datum.units.angle_precision";

pub const ACTIVE_UNITS_KEYS: [&str; 8] = [
    SYSTEM_KEY,
    BOARD_LENGTH_KEY,
    BOARD_PRECISION_KEY,
    DRILL_HOLE_KEY,
    DRILL_PRECISION_KEY,
    SCHEMATIC_GEOMETRY_KEY,
    SCHEMATIC_PRECISION_KEY,
    ANGLE_PRECISION_KEY,
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnitsProfileTokenRefusal {
    pub key: String,
    pub value: Option<Value>,
}

pub fn profile_from_descriptor_values(
    values: &BTreeMap<String, Value>,
) -> Result<UnitsProfile, UnitsProfileTokenRefusal> {
    if values.len() != ACTIVE_UNITS_KEYS.len()
        || values
            .keys()
            .any(|key| !ACTIVE_UNITS_KEYS.contains(&key.as_str()))
    {
        let key = values
            .keys()
            .find(|key| !ACTIVE_UNITS_KEYS.contains(&key.as_str()))
            .cloned()
            .unwrap_or_else(|| "datum.units.<missing>".to_owned());
        return Err(UnitsProfileTokenRefusal {
            value: values.get(&key).cloned(),
            key,
        });
    }
    let token = |key: &str| {
        values
            .get(key)
            .and_then(Value::as_str)
            .ok_or_else(|| UnitsProfileTokenRefusal {
                key: key.to_owned(),
                value: values.get(key).cloned(),
            })
    };
    let system = match token(SYSTEM_KEY)? {
        "metric" => MeasurementSystem::Metric,
        "imperial" => MeasurementSystem::Imperial,
        _ => return Err(refusal(values, SYSTEM_KEY)),
    };
    Ok(UnitsProfile {
        system,
        board: QuantityUnits {
            unit: parse_unit(token(BOARD_LENGTH_KEY)?)
                .ok_or_else(|| refusal(values, BOARD_LENGTH_KEY))?,
            precision: parse_length_precision(token(BOARD_PRECISION_KEY)?)
                .ok_or_else(|| refusal(values, BOARD_PRECISION_KEY))?,
        },
        drill: QuantityUnits {
            unit: parse_unit(token(DRILL_HOLE_KEY)?)
                .ok_or_else(|| refusal(values, DRILL_HOLE_KEY))?,
            precision: parse_length_precision(token(DRILL_PRECISION_KEY)?)
                .ok_or_else(|| refusal(values, DRILL_PRECISION_KEY))?,
        },
        schematic: QuantityUnits {
            unit: parse_unit(token(SCHEMATIC_GEOMETRY_KEY)?)
                .ok_or_else(|| refusal(values, SCHEMATIC_GEOMETRY_KEY))?,
            precision: parse_length_precision(token(SCHEMATIC_PRECISION_KEY)?)
                .ok_or_else(|| refusal(values, SCHEMATIC_PRECISION_KEY))?,
        },
        angle_precision: parse_angle_precision(token(ANGLE_PRECISION_KEY)?)
            .ok_or_else(|| refusal(values, ANGLE_PRECISION_KEY))?,
    })
}

pub fn profile_to_descriptor_values(profile: UnitsProfile) -> BTreeMap<String, Value> {
    BTreeMap::from([
        (
            SYSTEM_KEY.to_owned(),
            Value::String(system_token(profile.system).to_owned()),
        ),
        (
            BOARD_LENGTH_KEY.to_owned(),
            Value::String(unit_token(profile.board.unit).to_owned()),
        ),
        (
            BOARD_PRECISION_KEY.to_owned(),
            Value::String(precision_token(profile.board.precision)),
        ),
        (
            DRILL_HOLE_KEY.to_owned(),
            Value::String(unit_token(profile.drill.unit).to_owned()),
        ),
        (
            DRILL_PRECISION_KEY.to_owned(),
            Value::String(precision_token(profile.drill.precision)),
        ),
        (
            SCHEMATIC_GEOMETRY_KEY.to_owned(),
            Value::String(unit_token(profile.schematic.unit).to_owned()),
        ),
        (
            SCHEMATIC_PRECISION_KEY.to_owned(),
            Value::String(precision_token(profile.schematic.precision)),
        ),
        (
            ANGLE_PRECISION_KEY.to_owned(),
            Value::String(angle_precision_token(profile.angle_precision).to_owned()),
        ),
    ])
}

fn refusal(values: &BTreeMap<String, Value>, key: &str) -> UnitsProfileTokenRefusal {
    UnitsProfileTokenRefusal {
        key: key.to_owned(),
        value: values.get(key).cloned(),
    }
}

pub(super) fn parse_unit(token: &str) -> Option<LengthUnitChoice> {
    Some(match token {
        "follow_system" => LengthUnitChoice::FollowSystem,
        "mm" => LengthUnitChoice::Explicit(LengthUnit::Millimeter),
        "um" => LengthUnitChoice::Explicit(LengthUnit::Micrometer),
        "mil" => LengthUnitChoice::Explicit(LengthUnit::Mil),
        "inch" => LengthUnitChoice::Explicit(LengthUnit::Inch),
        _ => return None,
    })
}

pub(super) fn parse_length_precision(token: &str) -> Option<LengthPrecisionChoice> {
    Some(match token {
        "automatic" => LengthPrecisionChoice::Automatic,
        "exact_nm" => LengthPrecisionChoice::ExactNanometer,
        token if token.starts_with("decimal_") => {
            let places = token.strip_prefix("decimal_")?.parse().ok()?;
            if places > 6 {
                return None;
            }
            LengthPrecisionChoice::DecimalPlaces(places)
        }
        _ => return None,
    })
}

pub(super) fn parse_angle_precision(token: &str) -> Option<DecimalDegreePrecision> {
    Some(match token {
        "decimal_0" => DecimalDegreePrecision::Decimal0,
        "decimal_1" => DecimalDegreePrecision::Decimal1,
        "decimal_2" => DecimalDegreePrecision::Decimal2,
        "decimal_3" => DecimalDegreePrecision::Decimal3,
        _ => return None,
    })
}

fn system_token(system: MeasurementSystem) -> &'static str {
    match system {
        MeasurementSystem::Metric => "metric",
        MeasurementSystem::Imperial => "imperial",
    }
}

fn unit_token(unit: LengthUnitChoice) -> &'static str {
    match unit {
        LengthUnitChoice::FollowSystem => "follow_system",
        LengthUnitChoice::Explicit(LengthUnit::Nanometer) => "nm",
        LengthUnitChoice::Explicit(LengthUnit::Micrometer) => "um",
        LengthUnitChoice::Explicit(LengthUnit::Millimeter) => "mm",
        LengthUnitChoice::Explicit(LengthUnit::Mil) => "mil",
        LengthUnitChoice::Explicit(LengthUnit::Inch) => "inch",
    }
}

fn precision_token(precision: LengthPrecisionChoice) -> String {
    match precision {
        LengthPrecisionChoice::Automatic => "automatic".to_owned(),
        LengthPrecisionChoice::DecimalPlaces(places) => format!("decimal_{places}"),
        LengthPrecisionChoice::ExactNanometer => "exact_nm".to_owned(),
    }
}

fn angle_precision_token(precision: DecimalDegreePrecision) -> &'static str {
    match precision {
        DecimalDegreePrecision::Decimal0 => "decimal_0",
        DecimalDegreePrecision::Decimal1 => "decimal_1",
        DecimalDegreePrecision::Decimal2 => "decimal_2",
        DecimalDegreePrecision::Decimal3 => "decimal_3",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_eight_descriptor_profile_round_trips() {
        let values = profile_to_descriptor_values(UnitsProfile::default());
        assert_eq!(values.len(), 8);
        assert_eq!(
            profile_from_descriptor_values(&values).unwrap(),
            UnitsProfile::default()
        );
    }

    #[test]
    fn missing_extra_or_invalid_tokens_do_not_form_a_profile() {
        let mut values = profile_to_descriptor_values(UnitsProfile::default());
        values.remove(DRILL_PRECISION_KEY);
        assert!(profile_from_descriptor_values(&values).is_err());
        let mut values = profile_to_descriptor_values(UnitsProfile::default());
        values.insert("datum.units.future".to_owned(), Value::Bool(true));
        assert_eq!(
            profile_from_descriptor_values(&values).unwrap_err().key,
            "datum.units.future"
        );
        let mut values = profile_to_descriptor_values(UnitsProfile::default());
        values.insert(
            SCHEMATIC_GEOMETRY_KEY.to_owned(),
            Value::String("inch".to_owned()),
        );
        assert!(
            profile_from_descriptor_values(&values)
                .unwrap()
                .resolve()
                .is_err()
        );
    }
}
