//! The sole application-facing Global Preferences product service.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Serialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use super::product_acceptance::AcceptedMutationAudit;
use super::product_actor::{actor_kind, validate_actor};
use super::product_idempotency::{canonical_mutation_request_digest, mutation_request_id};
use super::repository::{GenerationRef, MutationAuditMetadata, MutationMetadata};
use super::{
    ApplyBehavior, GlobalPreferenceRow, GlobalPreferencesService, HeadExpectationV1,
    PreferenceActorKindV1, PreferenceActorV1, PreferenceConsideredFactV1, PreferenceContextV1,
    PreferenceControlPresentation, PreferenceControlViewV1, PreferenceEnumChoiceV1,
    PreferenceErrorCodeV1, PreferenceErrorV1, PreferenceExplanationV1, PreferenceKey,
    PreferenceMutationRequestV1, PreferenceMutationResultV1, PreferenceRepositoryStatusV1,
    PreferenceServiceRefusal, PreferenceServiceRefusalKind, PreferenceServiceStatus,
    PreferenceValueViewV1, ProjectUnitsSeedPreviewV1, ProjectUnitsSourceV1, active_v1_registry,
    factory_units_seed, reserved_v1_registry,
};
use crate::ir::units::project_profile_to_value;

const SCOPE: &str = "global_this_device";
const FACTORY_PROFILE: &str = "datum.units.factory.v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreferenceLocations {
    pub configuration_base: PathBuf,
    pub repository_root: PathBuf,
    pub legacy_console_path: PathBuf,
}

pub trait PreferenceLocationProvider {
    fn locations(&self) -> Result<PreferenceLocations, std::io::Error>;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct InstalledPreferenceLocationProvider;

impl PreferenceLocationProvider for InstalledPreferenceLocationProvider {
    fn locations(&self) -> Result<PreferenceLocations, std::io::Error> {
        let base = match std::env::var_os("XDG_CONFIG_HOME") {
            Some(value) if Path::new(&value).is_absolute() => PathBuf::from(value),
            Some(_) => return Err(std::io::Error::other("XDG_CONFIG_HOME must be absolute")),
            None => {
                let home = std::env::var_os("HOME")
                    .filter(|value| Path::new(value).is_absolute())
                    .ok_or_else(|| std::io::Error::other("HOME is unavailable or not absolute"))?;
                PathBuf::from(home).join(".config")
            }
        };
        let datum = base.join("datum");
        Ok(PreferenceLocations {
            configuration_base: base,
            repository_root: datum.join("preferences"),
            legacy_console_path: datum.join("gui-preferences.json"),
        })
    }
}

#[derive(Debug, Clone)]
pub struct FixedPreferenceLocationProvider(pub PreferenceLocations);

impl PreferenceLocationProvider for FixedPreferenceLocationProvider {
    fn locations(&self) -> Result<PreferenceLocations, std::io::Error> {
        Ok(self.0.clone())
    }
}

#[derive(Debug)]
pub struct GlobalPreferencesProductService {
    pub(super) service: GlobalPreferencesService,
    pub(super) active_catalog_digest: String,
    pub(super) factory_only: bool,
}

impl GlobalPreferencesProductService {
    pub fn open(
        provider: &dyn PreferenceLocationProvider,
        writer_instance: impl Into<String>,
    ) -> Result<Self, PreferenceErrorV1> {
        let locations = provider
            .locations()
            .map_err(|error| bootstrap_error(error.to_string()))?;
        validate_locations(&locations).map_err(bootstrap_error)?;
        let service = GlobalPreferencesService::open(
            locations.repository_root,
            &locations.legacy_console_path,
            writer_instance,
            "Global · this device",
        )
        .map_err(|error| bootstrap_error(error.to_string()))?;
        let active_catalog_digest = catalog_digest(&service)?;
        Ok(Self {
            service,
            active_catalog_digest,
            factory_only: false,
        })
    }

    pub fn context(&self) -> PreferenceContextV1 {
        PreferenceContextV1 {
            scope: SCOPE.to_owned(),
            repository_status: match self.service.status() {
                PreferenceServiceStatus::DefaultsOnly => PreferenceRepositoryStatusV1::DefaultsOnly,
                PreferenceServiceStatus::Ready { .. } => PreferenceRepositoryStatusV1::Ready,
                PreferenceServiceStatus::PreservedUnreadable { .. } => {
                    PreferenceRepositoryStatusV1::PreservedUnreadable
                }
                PreferenceServiceStatus::MigrationRequired { .. } => {
                    PreferenceRepositoryStatusV1::MigrationRequired
                }
            },
            generation: self.service.status().generation().cloned(),
            active_catalog_digest: self.active_catalog_digest.clone(),
            active_descriptor_count: self.service.surface().entries().len() as u32,
            reserved_descriptor_count: reserved_v1_registry().len() as u32
                - self.service.surface().entries().len() as u32,
        }
    }

    pub fn mutate(
        &mut self,
        request: PreferenceMutationRequestV1,
        actor: &PreferenceActorV1,
    ) -> Result<PreferenceMutationResultV1, PreferenceErrorV1> {
        validate_actor(actor, &self.context())?;
        if !matches!(
            actor.kind,
            PreferenceActorKindV1::HumanGui | PreferenceActorKindV1::HumanCli
        ) {
            return Err(self.error(
                PreferenceErrorCodeV1::ProposalRequired,
                "Nonhuman preference mutations require a proposal",
                BTreeMap::from([("actor_kind".to_owned(), json!(actor_kind(actor)))]),
                None,
            ));
        }
        self.commit_mutation(request, actor, None)
    }

    pub(super) fn commit_mutation(
        &mut self,
        request: PreferenceMutationRequestV1,
        actor: &PreferenceActorV1,
        accepted: Option<&AcceptedMutationAudit>,
    ) -> Result<PreferenceMutationResultV1, PreferenceErrorV1> {
        let (key_text, expected, reason, draft, reset) = match &request {
            PreferenceMutationRequestV1::SetUser {
                key,
                value,
                expected,
                reason,
                ..
            } => (
                key.as_str(),
                expected,
                reason.as_str(),
                Some(value.clone()),
                false,
            ),
            PreferenceMutationRequestV1::ResetUser {
                key,
                expected,
                reason,
                ..
            } => (key.as_str(), expected, reason.as_str(), None, true),
        };
        if reason.trim().is_empty() {
            return Err(self.error(
                PreferenceErrorCodeV1::InvalidRequest,
                "Mutation reason must not be empty",
                field_details("reason", "empty"),
                draft,
            ));
        }
        let key = self.active_key(key_text, draft.clone())?;
        if let Some(value) = draft.as_ref() {
            let descriptor = self.service.registry().get(&key).expect("active key");
            if !descriptor.validates(value) {
                return Err(self.error(
                    PreferenceErrorCodeV1::InvalidPreferenceValue,
                    "Preference value is invalid",
                    BTreeMap::from([
                        ("key".to_owned(), json!(key.as_str())),
                        ("reason".to_owned(), json!("descriptor_validation")),
                    ]),
                    draft,
                ));
            }
        }
        let request_id = mutation_request_id(&request);
        let request_digest = canonical_mutation_request_digest(&request, actor, accepted)
            .map_err(|message| bootstrap_error(message))?;
        if let Some(receipt) = self.service.receipt_for_request(request_id).cloned() {
            if receipt.canonical_request_digest.as_deref() != Some(&request_digest) {
                return Err(self.error(
                    PreferenceErrorCodeV1::IdempotencyConflict,
                    "Request id was already committed with different canonical content",
                    BTreeMap::from([
                        ("request_id".to_owned(), json!(request_id)),
                        (
                            "original_request_digest".to_owned(),
                            json!(receipt.canonical_request_digest),
                        ),
                        ("submitted_request_digest".to_owned(), json!(request_digest)),
                    ]),
                    draft,
                ));
            }
            return self.replay_mutation(&key, receipt);
        }
        let before = self.service.status().generation().cloned();
        let expected_ref = self.validate_expectation(expected, draft.clone())?;
        let metadata = MutationMetadata {
            actor: format!(
                "{}:{}:{}:{}",
                actor_kind(actor),
                actor.local_actor_id,
                actor.session_id,
                actor.invocation_id
            ),
            reason: reason.to_owned(),
            writer_instance: actor.session_id.clone(),
            audit: Some(MutationAuditMetadata {
                request_id,
                canonical_request_digest: request_digest,
                actor_kind: actor_kind(actor).to_owned(),
                local_actor_id: actor.local_actor_id.clone(),
                actor_session_id: actor.session_id.clone(),
                invocation_id: actor.invocation_id,
                expected_generation_ref: expected_ref.clone(),
                proposal_id: accepted.map(|audit| audit.proposal_id),
                proposal_digest: accepted.map(|audit| audit.proposal_digest.clone()),
                acceptance_id: accepted.map(|audit| audit.acceptance_id),
                requesting_actor: accepted.map(|audit| audit.requesting_actor.clone()),
                accepting_actor: accepted.map(|audit| audit.accepting_actor.clone()),
                originating_mcp_session: accepted
                    .map(|audit| audit.originating_mcp_session.clone()),
            }),
        };
        let rows = if reset {
            self.service
                .reset_user_with_metadata(key.clone(), expected_ref.as_ref(), metadata)
        } else {
            let value = draft.clone().expect("SetUser carries a draft");
            self.service
                .set_user_with_metadata(key.clone(), value, expected_ref.as_ref(), metadata)
        }
        .map_err(|refusal| self.map_refusal(refusal))?;
        let after = self.service.status().generation().cloned();
        let changed = before != after;
        let row = rows
            .iter()
            .find(|row| row.key == key)
            .expect("active mutation result contains its row");
        let value = self.value_view(row)?;
        let explanation = self.explanation(row)?;
        let receipt = changed
            .then(|| self.service.latest_receipt().cloned())
            .flatten();
        Ok(PreferenceMutationResultV1 {
            changed,
            generation: after,
            value,
            explanation,
            receipt,
        })
    }

    pub(super) fn active_key(
        &self,
        key: &str,
        draft: Option<Value>,
    ) -> Result<PreferenceKey, PreferenceErrorV1> {
        let parsed = PreferenceKey::parse(key.to_owned())
            .map_err(|_| self.unknown_key(key, draft.clone()))?;
        if self.service.registry().get(&parsed).is_some() {
            return Ok(parsed);
        }
        let code = if reserved_v1_registry().get(&parsed).is_some() {
            PreferenceErrorCodeV1::ReservedPreferenceKey
        } else {
            PreferenceErrorCodeV1::UnknownPreferenceKey
        };
        Err(self.error(
            code,
            "Preference key is not active",
            BTreeMap::from([("key".to_owned(), json!(key))]),
            draft,
        ))
    }

    pub(super) fn unknown_key(&self, key: &str, draft: Option<Value>) -> PreferenceErrorV1 {
        self.error(
            PreferenceErrorCodeV1::UnknownPreferenceKey,
            "Preference key is unknown",
            BTreeMap::from([("key".to_owned(), json!(key))]),
            draft,
        )
    }

    pub(super) fn validate_expectation(
        &self,
        expected: &HeadExpectationV1,
        draft: Option<Value>,
    ) -> Result<Option<GenerationRef>, PreferenceErrorV1> {
        match (self.service.status(), expected) {
            (PreferenceServiceStatus::DefaultsOnly, HeadExpectationV1::Missing) => Ok(None),
            (
                PreferenceServiceStatus::Ready { generation },
                HeadExpectationV1::Generation(value),
            ) if generation == value => Ok(Some(value.clone())),
            (_, HeadExpectationV1::Missing | HeadExpectationV1::Generation(_)) => Err(self.error(
                PreferenceErrorCodeV1::StaleGeneration,
                "Preference generation changed",
                BTreeMap::from([
                    (
                        "expected".to_owned(),
                        serde_json::to_value(expected).unwrap_or(Value::Null),
                    ),
                    (
                        "current".to_owned(),
                        serde_json::to_value(self.service.status().generation())
                            .unwrap_or(Value::Null),
                    ),
                ]),
                draft,
            )),
        }
    }

    pub(super) fn value_view(
        &self,
        row: &GlobalPreferenceRow,
    ) -> Result<PreferenceValueViewV1, PreferenceErrorV1> {
        let entry = self.entry(&row.key);
        let descriptor = self.service.registry().get(&row.key).expect("catalog key");
        let section = self
            .service
            .surface()
            .sections()
            .iter()
            .find(|item| item.id == entry.section)
            .expect("catalog section");
        Ok(PreferenceValueViewV1 {
            key: row.key.as_str().to_owned(),
            label: descriptor.presentation.label.clone(),
            description: descriptor.presentation.description.clone(),
            section_id: section.id.as_str().to_owned(),
            section_label: section.label.clone(),
            row_order: entry.row_order,
            control: control_view(&entry.control),
            effective_value: row.effective_value.clone(),
            user_value: row.user_value.clone(),
            effect_timing: effect_timing(descriptor.apply_behavior).to_owned(),
            writable: row.writable,
            provenance_summary: provenance_summary(row),
            generation: self.service.status().generation().cloned(),
        })
    }

    pub(super) fn explanation(
        &self,
        row: &GlobalPreferenceRow,
    ) -> Result<PreferenceExplanationV1, PreferenceErrorV1> {
        let descriptor = self.service.registry().get(&row.key).expect("catalog key");
        let considered = row
            .explanation
            .considered
            .iter()
            .map(|fact| PreferenceConsideredFactV1 {
                id: fact.id.clone(),
                source: debug_token(fact.source),
                value: matches!(fact.disclosure, super::ValueDisclosure::Disclosed)
                    .then(|| fact.value.clone())
                    .flatten(),
                directive: fact.directive.map(debug_token),
                disposition: debug_token(fact.disposition),
                reason: fact.reason.clone(),
                origin: fact.provenance.origin.clone(),
                actor: fact.provenance.actor.clone(),
                disclosure: debug_token(fact.disclosure),
            })
            .collect();
        let (effective_value, outcome_reason) = match &row.explanation.outcome {
            super::ResolutionOutcome::Effective {
                value,
                winning_fact_ids,
            } => (
                Some(value.clone()),
                format!("effective from {}", winning_fact_ids.join(",")),
            ),
            super::ResolutionOutcome::NoValue => (None, "no effective value".to_owned()),
            super::ResolutionOutcome::UnresolvedConflict { reason, .. } => (None, reason.clone()),
            super::ResolutionOutcome::UnknownKey => (None, "unknown key".to_owned()),
        };
        Ok(PreferenceExplanationV1 {
            key: row.key.as_str().to_owned(),
            effective_value,
            outcome_reason,
            considered,
            absent_sources: row
                .explanation
                .absent_sources
                .iter()
                .map(|value| debug_token(*value))
                .collect(),
            evaluation_stages: row.explanation.evaluation_stages.clone(),
            available_actions: row
                .explanation
                .available_actions
                .iter()
                .map(|value| debug_token(*value))
                .collect(),
            writable: row.writable,
            effect_timing: effect_timing(descriptor.apply_behavior).to_owned(),
            generation: self.service.status().generation().cloned(),
        })
    }

    pub(super) fn preview_units_seed(
        &self,
        source: ProjectUnitsSourceV1,
    ) -> Result<ProjectUnitsSeedPreviewV1, PreferenceErrorV1> {
        let (seed, pinned_generation) = match &source {
            ProjectUnitsSourceV1::Global {
                expected_generation,
            } => {
                if self.factory_only {
                    return Err(self.error(
                        PreferenceErrorCodeV1::SeedSourceUnavailable,
                        "Factory-only genesis cannot resolve Global Preferences",
                        BTreeMap::from([("units_source".to_owned(), json!("global"))]),
                        None,
                    ));
                }
                if let Some(expected) = expected_generation
                    && self.service.status().generation() != Some(expected)
                {
                    return Err(self.error(
                        PreferenceErrorCodeV1::StaleGeneration,
                        "Preference generation changed",
                        BTreeMap::new(),
                        None,
                    ));
                }
                let seed = self.service.resolved_units_seed().map_err(|error| {
                    self.error(
                        PreferenceErrorCodeV1::SeedSourceUnavailable,
                        &error.message,
                        BTreeMap::from([("units_source".to_owned(), json!("global"))]),
                        None,
                    )
                })?;
                (seed, self.service.status().generation().cloned())
            }
            ProjectUnitsSourceV1::Factory { profile_id } if profile_id == FACTORY_PROFILE => {
                (factory_units_seed(), None)
            }
            ProjectUnitsSourceV1::Factory { .. } => {
                return Err(self.error(
                    PreferenceErrorCodeV1::SeedSourceUnavailable,
                    "Factory Units profile is not supported",
                    BTreeMap::from([("units_source".to_owned(), json!("factory"))]),
                    None,
                ));
            }
        };
        let profile = project_profile_to_value(seed.profile);
        let receipt_preview = serde_json::to_value(&seed.receipt)
            .map_err(|error| bootstrap_error(error.to_string()))?;
        let units_seed_catalog_digest =
            super::units_seed::units_seed_catalog_digest(&self.service).map_err(bootstrap_error)?;
        let seed_application_digest =
            digest(&(source.clone(), &profile, &units_seed_catalog_digest))
                .map_err(bootstrap_error)?;
        Ok(ProjectUnitsSeedPreviewV1 {
            source,
            pinned_generation,
            profile,
            receipt_preview,
            units_seed_catalog_digest,
            seed_application_digest,
        })
    }

    pub(super) fn entry(&self, key: &PreferenceKey) -> &super::PreferenceSurfaceEntry {
        self.service
            .surface()
            .entries()
            .iter()
            .find(|entry| &entry.key == key)
            .expect("active row has catalog entry")
    }

    pub(super) fn error(
        &self,
        code: PreferenceErrorCodeV1,
        message: &str,
        details: BTreeMap<String, Value>,
        preserved_draft: Option<Value>,
    ) -> PreferenceErrorV1 {
        PreferenceErrorV1 {
            code,
            message: message.to_owned(),
            details,
            current_context: self.context(),
            preserved_draft,
            preserved_proposal: None,
        }
    }

    fn map_refusal(&self, refusal: PreferenceServiceRefusal) -> PreferenceErrorV1 {
        let code = match refusal.kind {
            PreferenceServiceRefusalKind::StaleGeneration => PreferenceErrorCodeV1::StaleGeneration,
            PreferenceServiceRefusalKind::WriterConflict => PreferenceErrorCodeV1::WriterConflict,
            PreferenceServiceRefusalKind::InvalidValue => {
                PreferenceErrorCodeV1::InvalidPreferenceValue
            }
            PreferenceServiceRefusalKind::IneligibleSource => {
                PreferenceErrorCodeV1::IneligibleSource
            }
            PreferenceServiceRefusalKind::UnreadableRepository => {
                PreferenceErrorCodeV1::RepositoryUnreadable
            }
            PreferenceServiceRefusalKind::MigrationRequired => {
                PreferenceErrorCodeV1::MigrationRequired
            }
            PreferenceServiceRefusalKind::Repository => PreferenceErrorCodeV1::RepositoryIo,
        };
        self.error(
            code,
            &refusal.message,
            BTreeMap::new(),
            refusal.preserved_draft,
        )
    }
}

fn validate_locations(locations: &PreferenceLocations) -> Result<(), String> {
    if !locations.configuration_base.is_absolute()
        || !locations.repository_root.is_absolute()
        || !locations
            .repository_root
            .starts_with(&locations.configuration_base)
    {
        return Err(
            "preference repository must be under an absolute configuration base".to_owned(),
        );
    }
    if locations.repository_root.exists() {
        let base = locations
            .configuration_base
            .canonicalize()
            .map_err(|error| error.to_string())?;
        let root = locations
            .repository_root
            .canonicalize()
            .map_err(|error| error.to_string())?;
        if !root.starts_with(base) {
            return Err("preference repository resolves outside its configuration base".to_owned());
        }
    }
    Ok(())
}

pub(super) fn control_view(control: &PreferenceControlPresentation) -> PreferenceControlViewV1 {
    match control {
        PreferenceControlPresentation::BooleanSwitch {
            off_label,
            on_label,
        } => PreferenceControlViewV1::BooleanSwitch {
            off_label: off_label.clone(),
            on_label: on_label.clone(),
        },
        PreferenceControlPresentation::EnumeratedSingleChoice { choices } => {
            PreferenceControlViewV1::EnumeratedSingleChoice {
                choices: choices
                    .iter()
                    .map(|choice| PreferenceEnumChoiceV1 {
                        value: choice.value.clone(),
                        label: choice.label.clone(),
                    })
                    .collect(),
            }
        }
        PreferenceControlPresentation::IntegerStepper {
            min,
            max,
            step,
            suffix,
        } => PreferenceControlViewV1::IntegerStepper {
            min: *min,
            max: *max,
            step: *step,
            suffix: suffix.clone(),
        },
        PreferenceControlPresentation::IdentityEntry {
            nullable,
            placeholder,
        } => PreferenceControlViewV1::IdentityEntry {
            nullable: *nullable,
            placeholder: placeholder.clone(),
        },
        PreferenceControlPresentation::StructuredEditor { action_label } => {
            PreferenceControlViewV1::StructuredEditor {
                action_label: action_label.clone(),
            }
        }
    }
}

pub(super) fn effect_timing(value: ApplyBehavior) -> &'static str {
    match value {
        ApplyBehavior::Live => "live",
        ApplyBehavior::LiveAfterWholeValueValidation => "live_after_whole_value_validation",
        ApplyBehavior::LiveWithBoundedTrim => "live_with_bounded_trim",
        ApplyBehavior::LiveAndNewProjectSeed => "live_and_new_project_seed",
        ApplyBehavior::PrefillOnly => "prefill_only",
        ApplyBehavior::NextApplicationLaunch => "next_application_launch",
        ApplyBehavior::NextTerminalLaunch => "next_terminal_launch",
        ApplyBehavior::NewProjectOnly => "new_project_only",
    }
}

fn provenance_summary(row: &GlobalPreferenceRow) -> String {
    if row.user_value.is_some() {
        "set_by_user · global · this_device".to_owned()
    } else {
        "factory_default · global · this_device".to_owned()
    }
}

fn debug_token(value: impl std::fmt::Debug) -> String {
    format!("{value:?}").to_ascii_lowercase()
}

pub(super) fn catalog_digest(
    service: &GlobalPreferencesService,
) -> Result<String, PreferenceErrorV1> {
    let registry = active_v1_registry();
    let material: Vec<_> = service
        .surface()
        .entries()
        .iter()
        .map(|entry| {
            let descriptor = registry.get(&entry.key).expect("active catalog key");
            (
                entry.key.as_str(),
                descriptor.schema_version,
                entry.section.as_str(),
                entry.row_order,
                format!("{:?}", entry.control),
                descriptor.presentation.label.as_str(),
                descriptor.presentation.description.as_str(),
                effect_timing(descriptor.apply_behavior),
            )
        })
        .collect();
    digest(&material).map_err(bootstrap_error)
}

fn digest(value: &impl Serialize) -> Result<String, String> {
    let bytes =
        crate::ir::serialization::to_json_bytes(value).map_err(|error| error.to_string())?;
    let hash = Sha256::digest(bytes);
    Ok(format!("sha256:{hash:x}"))
}

pub(super) fn bootstrap_error(message: String) -> PreferenceErrorV1 {
    PreferenceErrorV1 {
        code: PreferenceErrorCodeV1::RepositoryIo,
        message,
        details: BTreeMap::new(),
        current_context: PreferenceContextV1 {
            scope: SCOPE.to_owned(),
            repository_status: PreferenceRepositoryStatusV1::DefaultsOnly,
            generation: None,
            active_catalog_digest: "sha256:unavailable".to_owned(),
            active_descriptor_count: 11,
            reserved_descriptor_count: 45,
        },
        preserved_draft: None,
        preserved_proposal: None,
    }
}

pub(super) fn field_details(field: &str, reason: &str) -> BTreeMap<String, Value> {
    BTreeMap::from([
        ("field".to_owned(), json!(field)),
        ("reason".to_owned(), json!(reason)),
    ])
}
