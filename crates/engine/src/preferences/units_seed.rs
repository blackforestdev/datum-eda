//! Copy-once Units snapshot produced by the Global Preferences resolver for
//! native New-Project genesis.

use std::collections::BTreeMap;

use serde_json::Value;
use sha2::{Digest, Sha256};

use super::{GlobalPreferencesService, PreferenceServiceStatus};
use crate::ir::units::{
    ACTIVE_UNITS_KEYS, FACTORY_UNITS_PROFILE_V1, ProjectUnitsSeedReceipt, ProjectUnitsSeedSource,
    UnitsProfile, profile_from_descriptor_values, profile_to_descriptor_values,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedUnitsSeed {
    pub profile: UnitsProfile,
    pub receipt: ProjectUnitsSeedReceipt,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnitsSeedRefusal {
    pub message: String,
}

/// Deterministic headless New-Project seed. It is the same versioned factory
/// contribution an empty Global repository resolves, without consulting any
/// machine state.
pub fn factory_units_seed() -> ResolvedUnitsSeed {
    let values = profile_to_descriptor_values(FACTORY_UNITS_PROFILE_V1);
    ResolvedUnitsSeed {
        profile: FACTORY_UNITS_PROFILE_V1,
        receipt: ProjectUnitsSeedReceipt {
            source: ProjectUnitsSeedSource::GlobalDefaults {
                repository_generation: "factory-defaults".to_owned(),
                profile_digest: digest_values(&values)
                    .expect("factory Units values must serialize"),
            },
            copied_values: values,
        },
    }
}

fn digest_values(values: &BTreeMap<String, Value>) -> Result<String, serde_json::Error> {
    let encoded = serde_json::to_vec(values)?;
    Ok(Sha256::digest(encoded)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
}

impl GlobalPreferencesService {
    /// Resolve all eight Global Units descriptors as one immutable New-Project
    /// seed. Existing Projects never call this operation.
    pub fn resolved_units_seed(&self) -> Result<ResolvedUnitsSeed, UnitsSeedRefusal> {
        if !self.status().writable() {
            return Err(UnitsSeedRefusal {
                message: "Global Units defaults are unreadable; New Project cannot copy a partial or guessed Units seed"
                    .to_owned(),
            });
        }
        let values: BTreeMap<String, Value> = self
            .rows()
            .into_iter()
            .filter(|row| ACTIVE_UNITS_KEYS.contains(&row.key.as_str()))
            .map(|row| {
                row.effective_value
                    .map(|value| (row.key.as_str().to_owned(), value))
                    .ok_or_else(|| UnitsSeedRefusal {
                        message: format!(
                            "Global Units default {} has no effective value",
                            row.key.as_str()
                        ),
                    })
            })
            .collect::<Result<_, _>>()?;
        let profile =
            profile_from_descriptor_values(&values).map_err(|reason| UnitsSeedRefusal {
                message: format!("Global Units defaults do not form one valid profile: {reason:?}"),
            })?;
        let profile_digest = digest_values(&values).map_err(|error| UnitsSeedRefusal {
            message: format!("Global Units seed could not be encoded: {error}"),
        })?;
        let repository_generation = match self.status() {
            PreferenceServiceStatus::DefaultsOnly => "factory-defaults".to_owned(),
            PreferenceServiceStatus::Ready { generation } => format!(
                "{}:g{:020}:{}",
                generation.repository_id,
                generation.generation,
                generation.canonical_manifest_digest
            ),
            PreferenceServiceStatus::PreservedUnreadable { .. }
            | PreferenceServiceStatus::MigrationRequired { .. } => {
                unreachable!("non-writable status refused above")
            }
        };
        Ok(ResolvedUnitsSeed {
            profile,
            receipt: ProjectUnitsSeedReceipt {
                source: ProjectUnitsSeedSource::GlobalDefaults {
                    repository_generation,
                    profile_digest,
                },
                copied_values: values,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::units::{BOARD_LENGTH_KEY, LengthUnit, LengthUnitChoice};
    use crate::preferences::PreferenceKey;

    #[test]
    fn resolved_seed_is_one_itemized_snapshot_of_the_current_global_generation() {
        let root =
            std::env::temp_dir().join(format!("datum-global-units-seed-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let mut service = GlobalPreferencesService::open(
            root.join("repository"),
            &root.join("legacy.json"),
            "seed-test-writer",
            "Global · this device",
        )
        .unwrap();
        service
            .set_user(
                PreferenceKey::parse(BOARD_LENGTH_KEY).unwrap(),
                Value::String("mil".to_owned()),
                None,
            )
            .unwrap();
        let seed = service.resolved_units_seed().unwrap();
        assert_eq!(
            seed.profile.board.unit,
            LengthUnitChoice::Explicit(LengthUnit::Mil)
        );
        assert_eq!(
            seed.receipt.copied_values[BOARD_LENGTH_KEY],
            Value::String("mil".to_owned())
        );
        let ProjectUnitsSeedSource::GlobalDefaults {
            repository_generation,
            profile_digest,
        } = seed.receipt.source
        else {
            panic!("New Project must identify the Global generation")
        };
        assert!(repository_generation.contains(":g00000000000000000001:"));
        assert_eq!(profile_digest.len(), 64);
        let _ = std::fs::remove_dir_all(root);
    }
}
