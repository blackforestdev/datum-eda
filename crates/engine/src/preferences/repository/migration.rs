use std::collections::BTreeMap;

use serde_json::Value;

use crate::preferences::{PreferenceKey, RegistrationRefusal};

use super::io::{WriterLease, copy_unknown_bytes, next_generation_number};
use super::model::{
    GenerationRef, HeadExpectation, MutationMetadata, RepositoryError, RepositoryState,
    StoredPreferenceValue,
};
use super::{PreferenceRepository, receipt};

pub type MigrationTransform = fn(&Value) -> MigrationTransformResult;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MigrationTransformResult {
    Value(Value),
    ChoiceRequired { reason: String },
    Refused { reason: String },
}

#[derive(Clone)]
pub struct PreferenceMigration {
    pub identity: String,
    pub key: PreferenceKey,
    pub from_schema_version: u32,
    pub to_schema_version: u32,
    pub reversible: bool,
    pub transform: MigrationTransform,
}

impl std::fmt::Debug for PreferenceMigration {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PreferenceMigration")
            .field("identity", &self.identity)
            .field("key", &self.key)
            .field("from_schema_version", &self.from_schema_version)
            .field("to_schema_version", &self.to_schema_version)
            .field("reversible", &self.reversible)
            .finish_non_exhaustive()
    }
}

#[derive(Debug, Clone, Default)]
pub struct MigrationRegistry {
    migrations: BTreeMap<(PreferenceKey, u32), PreferenceMigration>,
}

impl MigrationRegistry {
    pub fn register(&mut self, migration: PreferenceMigration) -> Result<(), RepositoryError> {
        if migration.from_schema_version >= migration.to_schema_version {
            return Err(RepositoryError::Invariant(format!(
                "migration {} does not advance its schema version",
                migration.identity
            )));
        }
        let identity = (migration.key.clone(), migration.from_schema_version);
        if self.migrations.insert(identity, migration).is_some() {
            return Err(RepositoryError::Invariant(
                "duplicate preference migration source".to_owned(),
            ));
        }
        Ok(())
    }

    fn get(&self, key: &PreferenceKey, from: u32) -> Option<&PreferenceMigration> {
        self.migrations.get(&(key.clone(), from))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MigrationChange {
    AliasMoved {
        retired_key: String,
        live_key: String,
    },
    ValueMigrated {
        key: String,
        migration_identity: String,
        from_schema_version: u32,
        to_schema_version: u32,
    },
    LegacyUnitsMigrated {
        retired_keys: Vec<String>,
        live_keys: Vec<String>,
    },
    Unchanged {
        key: String,
        schema_version: u32,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrationPlan {
    pub expected_head: GenerationRef,
    pub changes: Vec<MigrationChange>,
    pub reversible: bool,
    pub pre_migration_backup_required: bool,
    pub(crate) target_state: RepositoryState,
}

impl PreferenceRepository {
    pub fn plan_migration(
        &self,
        expected: &GenerationRef,
        migrations: &MigrationRegistry,
    ) -> Result<MigrationPlan, RepositoryError> {
        self.verify_expectation(&HeadExpectation::Generation(expected.clone()))?;
        let snapshot = super::io::load_snapshot(&self.root, expected)?;
        let mut state = snapshot.state();
        let mut changes = Vec::new();
        let mut reversible = true;
        migrate_partition(
            &self.registry,
            migrations,
            &mut state.installation,
            &mut changes,
            &mut reversible,
        )?;
        migrate_partition(
            &self.registry,
            migrations,
            &mut state.user,
            &mut changes,
            &mut reversible,
        )?;
        Ok(MigrationPlan {
            expected_head: expected.clone(),
            changes,
            reversible,
            pre_migration_backup_required: true,
            target_state: state,
        })
    }

    pub fn commit_migration(
        &self,
        plan: &MigrationPlan,
        metadata: &MutationMetadata,
    ) -> Result<(GenerationRef, super::ExactBackupRef), RepositoryError> {
        let _lease = WriterLease::acquire(&self.root, &metadata.writer_instance)?;
        self.verify_expectation(&HeadExpectation::Generation(plan.expected_head.clone()))?;
        let backup = self.create_backup_locked(&plan.expected_head, metadata)?;
        let snapshot = super::io::load_snapshot(&self.root, &plan.expected_head)?;
        let before = snapshot.state();
        let mut state = plan.target_state.clone();
        let affected = plan
            .changes
            .iter()
            .filter_map(|change| match change {
                MigrationChange::AliasMoved { live_key, .. }
                | MigrationChange::ValueMigrated { key: live_key, .. } => {
                    Some(vec![live_key.clone()])
                }
                MigrationChange::LegacyUnitsMigrated {
                    retired_keys,
                    live_keys,
                } => Some(retired_keys.iter().chain(live_keys).cloned().collect()),
                MigrationChange::Unchanged { .. } => None,
            })
            .flatten()
            .collect();
        let next = next_generation_number(&self.root);
        state.receipts.push(receipt(
            "CommitMigration",
            metadata,
            Some(plan.expected_head.generation),
            next,
            affected,
            &before,
            &state,
        )?);
        let unknown_bytes = copy_unknown_bytes(&self.root, &snapshot)?;
        let generation =
            self.commit_state_locked(&plan.expected_head, next, state, unknown_bytes, metadata)?;
        Ok((generation, backup))
    }
}

fn migrate_partition(
    registry: &crate::preferences::DescriptorRegistry,
    migrations: &MigrationRegistry,
    values: &mut BTreeMap<String, StoredPreferenceValue>,
    changes: &mut Vec<MigrationChange>,
    reversible: &mut bool,
) -> Result<(), RepositoryError> {
    migrate_legacy_units_partition(values, changes, reversible)?;
    let original_keys: Vec<_> = values.keys().cloned().collect();
    for stored_key in original_keys {
        let parsed = PreferenceKey::parse(stored_key.clone()).map_err(|refusal| match refusal {
            RegistrationRefusal::InvalidKey(key) => RepositoryError::UnknownPreferenceKey(key),
            _ => RepositoryError::UnknownPreferenceKey(stored_key.clone()),
        })?;
        let live_key = if registry.get(&parsed).is_some() {
            parsed
        } else if let Some(alias) = registry.resolve_alias(&stored_key) {
            if values.contains_key(alias.as_str()) {
                return Err(RepositoryError::AliasCollision(stored_key));
            }
            let value = values
                .remove(&stored_key)
                .expect("key came from the same map");
            values.insert(alias.as_str().to_owned(), value);
            changes.push(MigrationChange::AliasMoved {
                retired_key: stored_key.clone(),
                live_key: alias.as_str().to_owned(),
            });
            alias.clone()
        } else {
            continue;
        };
        let descriptor = registry
            .get(&live_key)
            .expect("live or resolved alias has a descriptor");
        let value = values
            .get_mut(live_key.as_str())
            .expect("live value remains present");
        while value.schema_version < descriptor.schema_version {
            let migration = migrations
                .get(&live_key, value.schema_version)
                .ok_or_else(|| {
                    RepositoryError::MigrationTransformUnavailable(format!(
                        "{} schema {}",
                        live_key.as_str(),
                        value.schema_version
                    ))
                })?;
            match (migration.transform)(&value.value) {
                MigrationTransformResult::Value(transformed) => {
                    let from = value.schema_version;
                    value.value = transformed;
                    value.schema_version = migration.to_schema_version;
                    *reversible &= migration.reversible;
                    changes.push(MigrationChange::ValueMigrated {
                        key: live_key.as_str().to_owned(),
                        migration_identity: migration.identity.clone(),
                        from_schema_version: from,
                        to_schema_version: migration.to_schema_version,
                    });
                }
                MigrationTransformResult::ChoiceRequired { reason } => {
                    return Err(RepositoryError::MigrationValueChoiceRequired(format!(
                        "{}: {reason}",
                        live_key.as_str()
                    )));
                }
                MigrationTransformResult::Refused { reason } => {
                    return Err(RepositoryError::MigrationTransformUnavailable(format!(
                        "{}: {reason}",
                        live_key.as_str()
                    )));
                }
            }
        }
        if value.schema_version > descriptor.schema_version {
            return Err(RepositoryError::MigrationTransformUnavailable(format!(
                "{} schema {} is newer than registered schema {}",
                live_key.as_str(),
                value.schema_version,
                descriptor.schema_version
            )));
        }
        if !descriptor.validates(&value.value) {
            return Err(RepositoryError::MigrationValueChoiceRequired(
                live_key.as_str().to_owned(),
            ));
        }
        if !changes.iter().any(|change| match change {
            MigrationChange::AliasMoved { live_key: key, .. }
            | MigrationChange::ValueMigrated { key, .. }
            | MigrationChange::Unchanged { key, .. } => key == live_key.as_str(),
            MigrationChange::LegacyUnitsMigrated { live_keys, .. } => {
                live_keys.iter().any(|key| key == live_key.as_str())
            }
        }) {
            changes.push(MigrationChange::Unchanged {
                key: live_key.as_str().to_owned(),
                schema_version: value.schema_version,
            });
        }
    }
    Ok(())
}

fn migrate_legacy_units_partition(
    values: &mut BTreeMap<String, StoredPreferenceValue>,
    changes: &mut Vec<MigrationChange>,
    reversible: &mut bool,
) -> Result<(), RepositoryError> {
    const RETIRED: [&str; 2] = ["datum.units.length_precision", "datum.units.angle_format"];
    let retired_keys: Vec<_> = RETIRED
        .into_iter()
        .filter(|key| values.contains_key(*key))
        .map(str::to_owned)
        .collect();
    if retired_keys.is_empty() {
        return Ok(());
    }
    let raw_values = values
        .iter()
        .map(|(key, stored)| (key.clone(), stored.value.clone()))
        .collect();
    let migration = crate::ir::units::migrate_legacy_units(&raw_values);
    if !migration.evidence.is_empty() {
        return Err(RepositoryError::MigrationValueChoiceRequired(format!(
            "legacy Units values preserved without reinterpretation: {:?}",
            migration.evidence
        )));
    }
    let live_keys: Vec<_> = migration.contributions.keys().cloned().collect();
    for (key, value) in migration.contributions {
        values.insert(
            key,
            StoredPreferenceValue {
                schema_version: 1,
                value,
            },
        );
    }
    for key in &retired_keys {
        values.remove(key);
    }
    *reversible = false;
    changes.push(MigrationChange::LegacyUnitsMigrated {
        retired_keys,
        live_keys,
    });
    Ok(())
}

#[cfg(test)]
mod units_tests {
    use serde_json::json;

    use super::*;

    fn stored(value: Value) -> StoredPreferenceValue {
        StoredPreferenceValue {
            schema_version: 1,
            value,
        }
    }

    #[test]
    fn repository_units_fanout_is_atomic_and_removes_retired_key() {
        let mut values = BTreeMap::from([
            (
                "datum.units.length_precision".to_owned(),
                stored(json!("0.001")),
            ),
            (
                "datum.units.board_length_precision".to_owned(),
                stored(json!("exact_nm")),
            ),
        ]);
        let mut changes = Vec::new();
        let mut reversible = true;
        migrate_legacy_units_partition(&mut values, &mut changes, &mut reversible).unwrap();
        assert!(!values.contains_key("datum.units.length_precision"));
        assert_eq!(
            values["datum.units.board_length_precision"].value,
            json!("exact_nm")
        );
        assert_eq!(
            values["datum.units.drill_hole_precision"].value,
            json!("decimal_3")
        );
        assert_eq!(
            values["datum.units.schematic_geometry_precision"].value,
            json!("decimal_3")
        );
        assert!(!reversible);
        assert!(matches!(
            changes[0],
            MigrationChange::LegacyUnitsMigrated { .. }
        ));
    }

    #[test]
    fn unsupported_angle_draft_preserves_partition_byte_semantics() {
        let original = BTreeMap::from([(
            "datum.units.angle_format".to_owned(),
            stored(json!({"notation":"radians","precision":0.001})),
        )]);
        let mut values = original.clone();
        let mut changes = Vec::new();
        let mut reversible = true;
        assert!(
            migrate_legacy_units_partition(&mut values, &mut changes, &mut reversible).is_err()
        );
        assert_eq!(values, original);
        assert!(changes.is_empty());
        assert!(reversible);
    }
}
