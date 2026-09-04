//! One application-facing lifecycle over registry, resolver, and repository.

use std::path::{Path, PathBuf};

use serde_json::Value;

use super::repository::{
    GenerationRef, MutationMetadata, PreferenceMutation, PreferencePartition, PreferenceRepository,
    RepositoryError, RepositorySnapshot, RepositoryStatus, UnknownEnvelope,
};
use super::{
    Contribution, DescriptorRegistry, FactProvenance, PreferenceExplanation, PreferenceKey,
    PreferenceSurfaceCatalog, ResolutionRequest, ResolutionSource, ValueDisclosure, ValueFact,
    active_v1_registry, gp_f05_surface_catalog, resolve_preference,
    service_runtime_defaults::runtime_default_contribution,
};

const LEGACY_CONSOLE_IDENTITY: &str = "legacy-console-preferences-v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LegacyConsoleMigrationState {
    Absent,
    NotNeeded,
    Migrated { value: String, backup_id: String },
    PreservedInvalid { reason: String },
    PreservedUnreadable { reason: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreferenceServiceStatus {
    DefaultsOnly,
    Ready {
        generation: GenerationRef,
    },
    PreservedUnreadable {
        identity: String,
        reason: String,
        recovery_candidates: Vec<GenerationRef>,
    },
    MigrationRequired {
        issues: Vec<String>,
    },
}

impl PreferenceServiceStatus {
    pub fn writable(&self) -> bool {
        matches!(self, Self::DefaultsOnly | Self::Ready { .. })
    }

    pub fn generation(&self) -> Option<&GenerationRef> {
        match self {
            Self::Ready { generation } => Some(generation),
            Self::DefaultsOnly
            | Self::PreservedUnreadable { .. }
            | Self::MigrationRequired { .. } => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlobalPreferenceRow {
    pub key: PreferenceKey,
    pub effective_value: Option<Value>,
    pub user_value: Option<Value>,
    pub explanation: PreferenceExplanation,
    pub writable: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreferenceServiceRefusalKind {
    StaleGeneration,
    WriterConflict,
    InvalidValue,
    IneligibleSource,
    UnreadableRepository,
    MigrationRequired,
    Repository,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreferenceServiceRefusal {
    pub kind: PreferenceServiceRefusalKind,
    pub message: String,
    pub preserved_draft: Option<Value>,
    pub current_generation: Option<Box<GenerationRef>>,
}

#[derive(Debug, Clone)]
pub struct GlobalPreferencesService {
    registry: DescriptorRegistry,
    surface: PreferenceSurfaceCatalog,
    repository: PreferenceRepository,
    writer_instance: String,
    machine_scope: String,
    status: PreferenceServiceStatus,
    snapshot: Option<RepositorySnapshot>,
    legacy_migration: LegacyConsoleMigrationState,
}

impl GlobalPreferencesService {
    pub fn open(
        repository_root: impl Into<PathBuf>,
        legacy_console_path: &Path,
        writer_instance: impl Into<String>,
        machine_scope: impl Into<String>,
    ) -> Result<Self, RepositoryError> {
        let registry = active_v1_registry();
        let surface = gp_f05_surface_catalog(&registry).map_err(|refusal| {
            RepositoryError::Invariant(format!("GP-F05 surface catalog refused: {refusal:?}"))
        })?;
        let repository = PreferenceRepository::new(repository_root, registry.clone());
        let mut service = Self {
            registry,
            surface,
            repository,
            writer_instance: writer_instance.into(),
            machine_scope: machine_scope.into(),
            status: PreferenceServiceStatus::DefaultsOnly,
            snapshot: None,
            legacy_migration: LegacyConsoleMigrationState::NotNeeded,
        };
        service.refresh();
        if matches!(service.status, PreferenceServiceStatus::DefaultsOnly) {
            service.migrate_legacy_console(legacy_console_path)?;
            service.refresh();
        }
        Ok(service)
    }

    pub fn registry(&self) -> &DescriptorRegistry {
        &self.registry
    }

    pub fn surface(&self) -> &PreferenceSurfaceCatalog {
        &self.surface
    }

    pub fn status(&self) -> &PreferenceServiceStatus {
        &self.status
    }

    pub fn legacy_migration(&self) -> &LegacyConsoleMigrationState {
        &self.legacy_migration
    }

    pub fn rows(&self) -> Vec<GlobalPreferenceRow> {
        self.surface
            .entries()
            .iter()
            .map(|entry| self.resolve_row(&entry.key))
            .collect()
    }

    pub fn set_user(
        &mut self,
        key: PreferenceKey,
        value: Value,
        expected: Option<&GenerationRef>,
    ) -> Result<Vec<GlobalPreferenceRow>, PreferenceServiceRefusal> {
        self.commit_user_mutation(
            PreferenceMutation::Set {
                partition: PreferencePartition::User,
                key,
                value: value.clone(),
            },
            expected,
            Some(value),
            "SetGlobalPreference",
        )
    }

    pub fn reset_user(
        &mut self,
        key: PreferenceKey,
        expected: Option<&GenerationRef>,
    ) -> Result<Vec<GlobalPreferenceRow>, PreferenceServiceRefusal> {
        self.commit_user_mutation(
            PreferenceMutation::Remove {
                partition: PreferencePartition::User,
                key,
            },
            expected,
            None,
            "ResetGlobalPreference",
        )
    }

    fn commit_user_mutation(
        &mut self,
        mutation: PreferenceMutation,
        expected: Option<&GenerationRef>,
        draft: Option<Value>,
        reason: &str,
    ) -> Result<Vec<GlobalPreferenceRow>, PreferenceServiceRefusal> {
        if !self.status.writable() {
            return Err(self.refusal_for_status(draft));
        }
        let metadata = self.metadata(reason);
        let actual_expected = match (&self.status, expected) {
            (PreferenceServiceStatus::DefaultsOnly, None) => {
                let generation = self
                    .repository
                    .initialize(&metadata)
                    .map_err(|error| self.map_error(error, draft.clone()))?;
                self.refresh();
                generation
            }
            (PreferenceServiceStatus::Ready { generation }, Some(expected))
                if generation == expected =>
            {
                expected.clone()
            }
            (PreferenceServiceStatus::Ready { .. }, _) => {
                self.refresh();
                return Err(PreferenceServiceRefusal {
                    kind: PreferenceServiceRefusalKind::StaleGeneration,
                    message:
                        "Preferences changed since this row was displayed; the draft was preserved."
                            .to_owned(),
                    preserved_draft: draft,
                    current_generation: self.status.generation().cloned().map(Box::new),
                });
            }
            _ => return Err(self.refusal_for_status(draft)),
        };
        match self
            .repository
            .commit_mutations(&actual_expected, &[mutation], &metadata)
        {
            Ok(_) => {
                self.refresh();
                Ok(self.rows())
            }
            Err(error) => {
                self.refresh();
                Err(self.map_error(error, draft))
            }
        }
    }

    fn refresh(&mut self) {
        match self.repository.inspect() {
            RepositoryStatus::Missing => {
                self.snapshot = None;
                self.status = PreferenceServiceStatus::DefaultsOnly;
            }
            RepositoryStatus::Ready(snapshot) => {
                let generation = snapshot.generation.clone();
                self.snapshot = Some(snapshot);
                self.status = PreferenceServiceStatus::Ready { generation };
            }
            RepositoryStatus::MigrationRequired { snapshot, issues } => {
                self.snapshot = Some(snapshot);
                self.status = PreferenceServiceStatus::MigrationRequired {
                    issues: issues.into_iter().map(|issue| issue.reason).collect(),
                };
            }
            RepositoryStatus::Unreadable(evidence) => {
                self.snapshot = None;
                self.status = PreferenceServiceStatus::PreservedUnreadable {
                    identity: evidence.identity,
                    reason: evidence.reason,
                    recovery_candidates: evidence.recovery_candidates,
                };
            }
        }
    }

    fn migrate_legacy_console(&mut self, legacy_path: &Path) -> Result<(), RepositoryError> {
        let bytes = match std::fs::read(legacy_path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                self.legacy_migration = LegacyConsoleMigrationState::Absent;
                return Ok(());
            }
            Err(error) => {
                self.legacy_migration = LegacyConsoleMigrationState::PreservedUnreadable {
                    reason: error.to_string(),
                };
                return Ok(());
            }
        };
        let parsed = serde_json::from_slice::<Value>(&bytes)
            .ok()
            .and_then(|document| {
                document
                    .get("console_duration")
                    .and_then(Value::as_str)
                    .map(str::to_owned)
            });
        let duration =
            parsed.filter(|value| ["4s", "6s", "10s", "never"].contains(&value.as_str()));
        let metadata = self.metadata("MigrateLegacyConsolePreference");
        let initial = self.repository.initialize(&metadata)?;
        let mut mutations = vec![PreferenceMutation::PutUnknown(UnknownEnvelope {
            identity: LEGACY_CONSOLE_IDENTITY.to_owned(),
            provider: Some("datum-gui-v1".to_owned()),
            scope: "Global · this device".to_owned(),
            source_version: "datum_gui_preferences_v1".to_owned(),
            required_extension: None,
            ordering: None,
            exact_bytes: bytes,
        })];
        if let Some(duration) = &duration {
            let key = self
                .registry
                .resolve_alias("console_duration")
                .expect("registered legacy alias")
                .clone();
            mutations.insert(
                0,
                PreferenceMutation::Set {
                    partition: PreferencePartition::User,
                    key,
                    value: Value::String(duration.clone()),
                },
            );
        }
        let generation = self
            .repository
            .commit_mutations(&initial, &mutations, &metadata)?;
        let backup = self
            .repository
            .create_exact_backup(&generation, &metadata)?;
        self.legacy_migration = if let Some(value) = duration {
            LegacyConsoleMigrationState::Migrated {
                value,
                backup_id: backup.backup_id,
            }
        } else {
            LegacyConsoleMigrationState::PreservedInvalid {
                reason: "legacy Console value is missing or invalid".to_owned(),
            }
        };
        Ok(())
    }

    fn resolve_row(&self, key: &PreferenceKey) -> GlobalPreferenceRow {
        let mut contributions = Vec::new();
        if let Some(runtime_default) = runtime_default_contribution(&self.registry, key) {
            contributions.push(runtime_default);
        }
        if let Some(snapshot) = &self.snapshot {
            for (partition, source, id) in [
                (
                    PreferencePartition::Installation,
                    ResolutionSource::Installation,
                    "installation",
                ),
                (PreferencePartition::User, ResolutionSource::User, "user"),
            ] {
                if let Some(stored) = snapshot.value(partition, key) {
                    contributions.push(Contribution::Value(ValueFact {
                        id: format!("{id}:{}", key.as_str()),
                        key: key.clone(),
                        source,
                        value: stored.value.clone(),
                        provenance: FactProvenance {
                            origin: "Global Preferences repository".to_owned(),
                            provider: None,
                            package: None,
                            generation: Some(snapshot.generation.generation.to_string()),
                            actor: "local user".to_owned(),
                            role: None,
                            observed_at: "repository snapshot".to_owned(),
                            effective_from: None,
                            effective_until: None,
                            offline_valid_until: None,
                            last_successful_contact: None,
                            reason: format!("{id} contribution stored for this device"),
                        },
                        disclosure: ValueDisclosure::Disclosed,
                        provider_state: None,
                        context: None,
                    }));
                }
            }
        }
        let explanation = resolve_preference(ResolutionRequest {
            registry: &self.registry,
            key: key.clone(),
            contributions: &contributions,
            authority_releases: &[],
            machine_scope: &self.machine_scope,
        });
        let effective_value = match &explanation.outcome {
            super::ResolutionOutcome::Effective { value, .. } => Some(value.clone()),
            _ => None,
        };
        let user_value = self
            .snapshot
            .as_ref()
            .and_then(|snapshot| snapshot.value(PreferencePartition::User, key))
            .map(|stored| stored.value.clone());
        GlobalPreferenceRow {
            key: key.clone(),
            effective_value,
            user_value,
            explanation,
            writable: self.status.writable(),
        }
    }

    fn metadata(&self, reason: &str) -> MutationMetadata {
        MutationMetadata {
            actor: "local-user".to_owned(),
            reason: reason.to_owned(),
            writer_instance: self.writer_instance.clone(),
        }
    }

    fn refusal_for_status(&self, draft: Option<Value>) -> PreferenceServiceRefusal {
        let (kind, message) = match &self.status {
            PreferenceServiceStatus::PreservedUnreadable { .. } => (
                PreferenceServiceRefusalKind::UnreadableRepository,
                "Preferences are preserved but unreadable; controls are unavailable.".to_owned(),
            ),
            PreferenceServiceStatus::MigrationRequired { .. } => (
                PreferenceServiceRefusalKind::MigrationRequired,
                "Preferences require a registered migration before editing.".to_owned(),
            ),
            _ => (
                PreferenceServiceRefusalKind::Repository,
                "Preferences are unavailable.".to_owned(),
            ),
        };
        PreferenceServiceRefusal {
            kind,
            message,
            preserved_draft: draft,
            current_generation: self.status.generation().cloned().map(Box::new),
        }
    }

    fn map_error(&self, error: RepositoryError, draft: Option<Value>) -> PreferenceServiceRefusal {
        let kind = match error {
            RepositoryError::ExpectedGenerationMismatch => {
                PreferenceServiceRefusalKind::StaleGeneration
            }
            RepositoryError::WriterLeaseUnavailable => PreferenceServiceRefusalKind::WriterConflict,
            RepositoryError::InvalidPreferenceValue(_) => {
                PreferenceServiceRefusalKind::InvalidValue
            }
            RepositoryError::IneligiblePreferenceSource(_) => {
                PreferenceServiceRefusalKind::IneligibleSource
            }
            RepositoryError::UnreadableStorePreserved { .. } => {
                PreferenceServiceRefusalKind::UnreadableRepository
            }
            _ => PreferenceServiceRefusalKind::Repository,
        };
        PreferenceServiceRefusal {
            kind,
            message: error.to_string(),
            preserved_draft: draft,
            current_generation: self.status.generation().cloned().map(Box::new),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT: AtomicU64 = AtomicU64::new(1);

    struct TestDirectory(PathBuf);

    impl TestDirectory {
        fn new(name: &str) -> Self {
            let path = std::env::temp_dir().join(format!(
                "datum-global-preference-service-{}-{name}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            let _ = std::fs::remove_dir_all(&path);
            std::fs::create_dir_all(&path).unwrap();
            Self(path)
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn open(directory: &TestDirectory) -> GlobalPreferencesService {
        GlobalPreferencesService::open(
            directory.0.join("repository"),
            &directory.0.join("legacy.json"),
            "test-writer",
            "Global · this device",
        )
        .unwrap()
    }

    #[test]
    fn clean_open_uses_defaults_without_creating_repository() {
        let directory = TestDirectory::new("clean");
        let service = open(&directory);
        assert_eq!(service.status(), &PreferenceServiceStatus::DefaultsOnly);
        assert!(!directory.0.join("repository").exists());
        assert_eq!(
            service
                .rows()
                .iter()
                .map(|row| row.key.as_str())
                .collect::<Vec<_>>(),
            vec![
                "datum.console.feedback_duration",
                "datum.accessibility.reduced_motion",
                "datum.accessibility.high_contrast_noncolor",
                "datum.units.system",
                "datum.units.board_length",
                "datum.units.board_length_precision",
                "datum.units.drill_hole",
                "datum.units.drill_hole_precision",
                "datum.units.schematic_geometry",
                "datum.units.schematic_geometry_precision",
                "datum.units.angle_precision",
            ]
        );
        assert_eq!(
            service.rows()[0].effective_value,
            Some(Value::String("6s".to_owned()))
        );
    }

    #[test]
    fn write_reset_and_restart_share_one_durable_truth() {
        let directory = TestDirectory::new("write-reset");
        let mut service = open(&directory);
        let key = PreferenceKey::parse("datum.accessibility.reduced_motion").unwrap();
        let rows = service
            .set_user(key.clone(), Value::Bool(true), None)
            .unwrap();
        assert_eq!(rows[1].effective_value, Some(Value::Bool(true)));
        let expected = service.status().generation().cloned().unwrap();
        drop(service);
        let mut restarted = open(&directory);
        assert_eq!(restarted.rows()[1].effective_value, Some(Value::Bool(true)));
        let rows = restarted.reset_user(key, Some(&expected)).unwrap();
        assert_eq!(rows[1].effective_value, Some(Value::Bool(false)));
        assert_eq!(rows[1].user_value, None);
    }

    #[test]
    fn valid_legacy_value_migrates_by_alias_and_keeps_exact_evidence() {
        let directory = TestDirectory::new("legacy");
        let bytes = br#"{"schema":"datum_gui_preferences_v1","console_duration":"10s","future":7}"#;
        std::fs::write(directory.0.join("legacy.json"), bytes).unwrap();
        let service = open(&directory);
        assert_eq!(
            service.rows()[0].effective_value,
            Some(Value::String("10s".to_owned()))
        );
        assert!(
            matches!(service.legacy_migration(), LegacyConsoleMigrationState::Migrated { value, .. } if value == "10s")
        );
        assert_eq!(
            std::fs::read(directory.0.join("legacy.json")).unwrap(),
            bytes
        );
        let snapshot = match service.repository.inspect() {
            RepositoryStatus::Ready(snapshot) => snapshot,
            other => panic!("expected ready repository, got {other:?}"),
        };
        assert_eq!(
            service
                .repository
                .read_unknown_exact(&snapshot, LEGACY_CONSOLE_IDENTITY)
                .unwrap(),
            bytes
        );
    }

    #[test]
    fn invalid_legacy_bytes_are_preserved_without_guessing_a_value() {
        let directory = TestDirectory::new("legacy-invalid");
        let bytes = b"not-json";
        std::fs::write(directory.0.join("legacy.json"), bytes).unwrap();
        let service = open(&directory);
        assert_eq!(
            service.rows()[0].effective_value,
            Some(Value::String("6s".to_owned()))
        );
        assert!(matches!(
            service.legacy_migration(),
            LegacyConsoleMigrationState::PreservedInvalid { .. }
        ));
        assert_eq!(
            std::fs::read(directory.0.join("legacy.json")).unwrap(),
            bytes
        );
    }

    #[test]
    fn stale_generation_preserves_draft_and_refreshes_truth() {
        let directory = TestDirectory::new("stale");
        let mut first = open(&directory);
        let key = PreferenceKey::parse("datum.accessibility.reduced_motion").unwrap();
        first
            .set_user(key.clone(), Value::Bool(true), None)
            .unwrap();
        let stale = first.status().generation().cloned().unwrap();
        let mut second = open(&directory);
        second
            .set_user(
                PreferenceKey::parse("datum.accessibility.high_contrast_noncolor").unwrap(),
                Value::Bool(true),
                Some(&stale),
            )
            .unwrap();
        let refusal = first
            .set_user(key, Value::Bool(false), Some(&stale))
            .unwrap_err();
        assert_eq!(refusal.kind, PreferenceServiceRefusalKind::StaleGeneration);
        assert_eq!(refusal.preserved_draft, Some(Value::Bool(false)));
    }

    #[test]
    fn writer_conflict_preserves_draft_and_last_valid_truth() {
        let directory = TestDirectory::new("writer-conflict");
        let mut service = open(&directory);
        let key = PreferenceKey::parse("datum.accessibility.reduced_motion").unwrap();
        service
            .set_user(key.clone(), Value::Bool(true), None)
            .unwrap();
        let expected = service.status().generation().cloned().unwrap();
        std::fs::write(directory.0.join("repository/writer.lock"), b"other-writer").unwrap();
        let refusal = service
            .set_user(key, Value::Bool(false), Some(&expected))
            .unwrap_err();
        assert_eq!(refusal.kind, PreferenceServiceRefusalKind::WriterConflict);
        assert_eq!(refusal.preserved_draft, Some(Value::Bool(false)));
        assert_eq!(service.rows()[1].effective_value, Some(Value::Bool(true)));
    }

    #[test]
    fn invalid_value_is_refused_without_becoming_effective() {
        let directory = TestDirectory::new("invalid-draft");
        let mut service = open(&directory);
        let key = PreferenceKey::parse("datum.accessibility.reduced_motion").unwrap();
        let draft = Value::String("yes".to_owned());
        let refusal = service.set_user(key, draft.clone(), None).unwrap_err();
        assert_eq!(refusal.kind, PreferenceServiceRefusalKind::InvalidValue);
        assert_eq!(refusal.preserved_draft, Some(draft));
        assert_eq!(service.rows()[1].effective_value, Some(Value::Bool(false)));
    }

    #[test]
    fn corrupt_head_stays_exact_and_disables_all_rows() {
        let directory = TestDirectory::new("corrupt-head");
        let mut service = open(&directory);
        let key = PreferenceKey::parse("datum.accessibility.reduced_motion").unwrap();
        service.set_user(key, Value::Bool(true), None).unwrap();
        let head = directory.0.join("repository/head.json");
        let corrupt = b"{truncated-head";
        std::fs::write(&head, corrupt).unwrap();

        let mut reopened = open(&directory);
        assert!(matches!(
            reopened.status(),
            PreferenceServiceStatus::PreservedUnreadable { .. }
        ));
        assert!(reopened.rows().iter().all(|row| !row.writable));
        let draft = Value::Bool(false);
        let refusal = reopened
            .set_user(
                PreferenceKey::parse("datum.accessibility.reduced_motion").unwrap(),
                draft.clone(),
                None,
            )
            .unwrap_err();
        assert_eq!(
            refusal.kind,
            PreferenceServiceRefusalKind::UnreadableRepository
        );
        assert_eq!(refusal.preserved_draft, Some(draft));
        assert_eq!(std::fs::read(head).unwrap(), corrupt);
    }

    #[test]
    fn unreadable_legacy_source_is_reported_without_creating_repository() {
        let directory = TestDirectory::new("legacy-unreadable");
        std::fs::create_dir(directory.0.join("legacy.json")).unwrap();
        let service = open(&directory);
        assert_eq!(service.status(), &PreferenceServiceStatus::DefaultsOnly);
        assert!(matches!(
            service.legacy_migration(),
            LegacyConsoleMigrationState::PreservedUnreadable { .. }
        ));
        assert!(!directory.0.join("repository").exists());
    }
}
