use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{
    ANGLE_PRECISION_KEY, BOARD_LENGTH_KEY, BOARD_PRECISION_KEY, DRILL_HOLE_KEY,
    DRILL_PRECISION_KEY, DecimalDegreePrecision, LengthPrecisionChoice, LengthUnitChoice,
    MeasurementSystem, QuantityUnits, SCHEMATIC_GEOMETRY_KEY, SCHEMATIC_PRECISION_KEY, SYSTEM_KEY,
    UnitsProfile, UnitsProfileTokenRefusal, profile_from_descriptor_values,
    profile_to_descriptor_values,
};

pub const PROJECT_UNITS_SCHEMA_VERSION: u32 = 1;

/// Versioned factory profile used only to migrate Projects created before
/// Project Working Units existed. It deliberately cannot consult machine state.
pub const FACTORY_UNITS_PROFILE_V1: UnitsProfile = UnitsProfile {
    system: MeasurementSystem::Metric,
    board: QuantityUnits {
        unit: LengthUnitChoice::FollowSystem,
        precision: LengthPrecisionChoice::Automatic,
    },
    drill: QuantityUnits {
        unit: LengthUnitChoice::FollowSystem,
        precision: LengthPrecisionChoice::Automatic,
    },
    schematic: QuantityUnits {
        unit: LengthUnitChoice::FollowSystem,
        precision: LengthPrecisionChoice::Automatic,
    },
    angle_precision: DecimalDegreePrecision::Decimal1,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ProjectUnitsSeedSource {
    GlobalDefaults {
        repository_generation: String,
        profile_digest: String,
    },
    FactoryMigrationV1,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectUnitsSeedReceipt {
    pub source: ProjectUnitsSeedSource,
    pub copied_values: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreFeatureProjectUnitsMigration {
    Unchanged(UnitsProfile),
    Create {
        profile: UnitsProfile,
        receipt: ProjectUnitsSeedReceipt,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectUnitsMigrationRefusal {
    pub preserved_value: Value,
    pub reason: UnitsProfileTokenRefusal,
}

pub fn project_profile_to_value(profile: UnitsProfile) -> Value {
    let descriptor_values = profile_to_descriptor_values(profile);
    Value::Object(
        aggregate_key_pairs()
            .map(|(aggregate, descriptor)| {
                (aggregate.to_owned(), descriptor_values[descriptor].clone())
            })
            .collect(),
    )
}

pub fn project_profile_from_value(value: &Value) -> Result<UnitsProfile, UnitsProfileTokenRefusal> {
    let object = value.as_object().ok_or_else(|| UnitsProfileTokenRefusal {
        key: "ProjectDisplayUnits".to_owned(),
        value: Some(value.clone()),
    })?;
    if object.len() != 8 {
        return Err(UnitsProfileTokenRefusal {
            key: "ProjectDisplayUnits".to_owned(),
            value: Some(value.clone()),
        });
    }
    let mut descriptor_values = BTreeMap::new();
    for (aggregate, descriptor) in aggregate_key_pairs() {
        let Some(field_value) = object.get(aggregate) else {
            return Err(UnitsProfileTokenRefusal {
                key: aggregate.to_owned(),
                value: None,
            });
        };
        descriptor_values.insert(descriptor.to_owned(), field_value.clone());
    }
    profile_from_descriptor_values(&descriptor_values)
}

fn aggregate_key_pairs() -> impl Iterator<Item = (&'static str, &'static str)> {
    [
        ("system", SYSTEM_KEY),
        ("board_length", BOARD_LENGTH_KEY),
        ("board_length_precision", BOARD_PRECISION_KEY),
        ("drill_hole", DRILL_HOLE_KEY),
        ("drill_hole_precision", DRILL_PRECISION_KEY),
        ("schematic_geometry", SCHEMATIC_GEOMETRY_KEY),
        ("schematic_geometry_precision", SCHEMATIC_PRECISION_KEY),
        ("angle_precision", ANGLE_PRECISION_KEY),
    ]
    .into_iter()
}

/// Plans an idempotent migration. The caller owns the one canonical Project
/// mutation; this function neither writes a Project nor accepts Global values.
pub fn migrate_pre_feature_project_units(
    existing: Option<&Value>,
) -> Result<PreFeatureProjectUnitsMigration, ProjectUnitsMigrationRefusal> {
    if let Some(value) = existing {
        return project_profile_from_value(value)
            .map(PreFeatureProjectUnitsMigration::Unchanged)
            .map_err(|reason| ProjectUnitsMigrationRefusal {
                preserved_value: value.clone(),
                reason,
            });
    }
    let copied_values = profile_to_descriptor_values(FACTORY_UNITS_PROFILE_V1);
    Ok(PreFeatureProjectUnitsMigration::Create {
        profile: FACTORY_UNITS_PROFILE_V1,
        receipt: ProjectUnitsSeedReceipt {
            source: ProjectUnitsSeedSource::FactoryMigrationV1,
            copied_values,
        },
    })
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn pre_feature_migration_uses_only_versioned_factory_values() {
        let migration = migrate_pre_feature_project_units(None).unwrap();
        let PreFeatureProjectUnitsMigration::Create { profile, receipt } = migration else {
            panic!("missing Project must create one deterministic snapshot")
        };
        assert_eq!(profile, FACTORY_UNITS_PROFILE_V1);
        assert_eq!(receipt.source, ProjectUnitsSeedSource::FactoryMigrationV1);
        assert_eq!(receipt.copied_values, profile_to_descriptor_values(profile));
    }

    #[test]
    fn migration_is_idempotent_and_preserves_invalid_existing_evidence() {
        let value = project_profile_to_value(FACTORY_UNITS_PROFILE_V1);
        assert_eq!(
            migrate_pre_feature_project_units(Some(&value)).unwrap(),
            PreFeatureProjectUnitsMigration::Unchanged(FACTORY_UNITS_PROFILE_V1)
        );
        let invalid = json!({"system":"metric"});
        let refusal = migrate_pre_feature_project_units(Some(&invalid)).unwrap_err();
        assert_eq!(refusal.preserved_value, invalid);
    }
}
