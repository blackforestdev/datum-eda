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
                | MigrationChange::ValueMigrated { key: live_key, .. } => Some(live_key.clone()),
                MigrationChange::Unchanged { .. } => None,
            })
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
        }) {
            changes.push(MigrationChange::Unchanged {
                key: live_key.as_str().to_owned(),
                schema_version: value.schema_version,
            });
        }
    }
    Ok(())
}
