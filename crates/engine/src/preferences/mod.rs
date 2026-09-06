//! Pure Global Preferences descriptor and resolution authority.
//!
//! This module owns no persistence, GUI, Project mutation, provider transport,
//! or Revision behavior. It validates an immutable descriptor registry and
//! delegates one side-effect-free resolution/explanation algorithm to
//! [`resolver`].

use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;

mod catalog;
mod product_acceptance;
mod product_actor;
mod product_dispatch;
mod product_factory;
mod product_idempotency;
mod product_model;
mod product_proposal;
mod product_query;
mod product_service;
#[cfg(test)]
mod product_tests;
mod project_genesis;
#[cfg(test)]
mod project_genesis_lifecycle_tests;
pub mod repository;
pub mod resolver;
mod schema;
mod service;
mod service_mutation;
mod service_runtime_defaults;
mod surface;
mod surface_full;
mod units_seed;

pub use catalog::{active_v1_registry, reserved_v1_registry};
pub use product_acceptance::{
    PreferenceAcceptanceBroker, PreferenceAcceptanceHandleV1, PreferenceAcceptanceRefusal,
};
pub use product_model::*;
pub use product_service::{
    FixedPreferenceLocationProvider, GlobalPreferencesProductService,
    InstalledPreferenceLocationProvider, PreferenceLocationProvider, PreferenceLocations,
};
pub use resolver::{
    AuthorityRelease, AuthorityReleaseLevel, AuthorityReleaseState, AvailableAction,
    ConsideredFact, ContextApplicability, Contribution, ContributionDisposition, FactProvenance,
    OrganizationDirective, OrganizationFact, PreferenceExplanation, ProviderGenerationState,
    ResolutionOutcome, ResolutionRequest, RuntimeDefaultFact, ValueConstraint, ValueDisclosure,
    ValueFact, resolve_preference,
};
pub use service::{
    GlobalPreferenceRow, GlobalPreferencesService, LegacyConsoleMigrationState,
    PreferenceServiceRefusal, PreferenceServiceRefusalKind, PreferenceServiceStatus,
};
pub use surface::{
    EnumChoicePresentation, PreferenceControlPresentation, PreferenceLiveConsumer,
    PreferenceSection, PreferenceSectionId, PreferenceSurfaceCatalog, PreferenceSurfaceEntry,
    SurfaceCatalogRefusal, gp_f05_surface_catalog,
};
pub use surface_full::candidate_global_preferences_surface_catalog;
pub use units_seed::{ResolvedUnitsSeed, UnitsSeedRefusal, factory_units_seed};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PreferenceKey(String);

impl PreferenceKey {
    pub fn parse(value: impl Into<String>) -> Result<Self, RegistrationRefusal> {
        let value = value.into();
        let valid = value.starts_with("datum.")
            && value.split('.').all(|segment| {
                !segment.is_empty()
                    && segment.bytes().all(|byte| {
                        byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_'
                    })
            });
        if !valid {
            return Err(RegistrationRefusal::InvalidKey(value));
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SettingClass {
    Presentation,
    WorkflowDefault,
    Capability,
    ProjectPolicySeed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ResolutionSource {
    DescriptorDefault,
    Installation,
    Organization,
    User,
    ProjectPolicy,
    Session,
    Context,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DirectiveKind {
    Recommend,
    Constrain,
    Pin,
    Lock,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MergeCategory {
    Replace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApplyBehavior {
    Live,
    LiveAfterWholeValueValidation,
    LiveWithBoundedTrim,
    LiveAndNewProjectSeed,
    PrefillOnly,
    NextApplicationLaunch,
    NextTerminalLaunch,
    NewProjectOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportClass {
    Portable,
    Protected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SemanticSchema {
    ObjectOpacity,
    ObjectSnapTypes,
    GridMarkStyle,
    VersionedKeymap,
    RgbaColor,
    SheetFormat,
    PublishSetNaming,
    TerminalLaunchProfiles,
    TerminalTextRendering,
    TerminalCursor,
    TerminalFeedback,
    TerminalScrollback,
    DefaultLocations,
    AutosavePolicy,
    ProjectDisplayUnits,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValueSchema {
    Boolean,
    IntegerRange { min: i64, max: i64, step: u64 },
    Enum(BTreeSet<String>),
    Identity { nullable: bool },
    Semantic(SemanticSchema),
}

impl ValueSchema {
    pub fn validate(&self, value: &Value) -> bool {
        match self {
            Self::Boolean => value.is_boolean(),
            Self::IntegerRange { min, max, step } => value.as_i64().is_some_and(|candidate| {
                candidate >= *min
                    && candidate <= *max
                    && (candidate - *min).unsigned_abs().is_multiple_of(*step)
            }),
            Self::Enum(allowed) => value
                .as_str()
                .is_some_and(|candidate| allowed.contains(candidate)),
            Self::Identity { nullable } => {
                (*nullable && value.is_null())
                    || value
                        .as_str()
                        .is_some_and(|identity| !identity.trim().is_empty())
            }
            Self::Semantic(schema) => schema.validate(value),
        }
    }

    pub fn finite_domain(&self) -> Option<Vec<Value>> {
        match self {
            Self::Boolean => Some(vec![Value::Bool(false), Value::Bool(true)]),
            Self::Enum(values) => Some(values.iter().cloned().map(Value::String).collect()),
            Self::IntegerRange { min, max, step } => {
                let count = (*max - *min).unsigned_abs() / *step + 1;
                (count <= 1024).then(|| {
                    (0..count)
                        .map(|index| Value::from(*min + (index * *step) as i64))
                        .collect()
                })
            }
            Self::Identity { .. } | Self::Semantic(_) => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DescriptorDefault {
    Literal(Value),
    Runtime { recipe: String },
    Absent,
}

impl DescriptorDefault {
    pub fn literal(&self) -> Option<&Value> {
        match self {
            Self::Literal(value) => Some(value),
            Self::Runtime { .. } | Self::Absent => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectivePolicy {
    pub kind: DirectiveKind,
    pub minimum_release: resolver::AuthorityReleaseLevel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccessiblePresentation {
    pub label: String,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreferenceDescriptor {
    pub key: PreferenceKey,
    pub owner: String,
    pub schema_version: u32,
    pub value_schema_name: String,
    pub value_schema: ValueSchema,
    pub default_value: DescriptorDefault,
    pub class: SettingClass,
    pub allowed_sources: BTreeSet<ResolutionSource>,
    pub allowed_directives: BTreeSet<DirectiveKind>,
    pub directive_policy: Vec<DirectivePolicy>,
    pub merge_category: MergeCategory,
    pub apply_behavior: ApplyBehavior,
    pub consumers: Vec<String>,
    pub export_class: ExportClass,
    pub presentation: AccessiblePresentation,
    pub retired_aliases: BTreeSet<String>,
}

impl PreferenceDescriptor {
    pub fn validates(&self, value: &Value) -> bool {
        self.value_schema.validate(value)
    }

    pub fn directive_policy(&self, kind: DirectiveKind) -> Option<&DirectivePolicy> {
        self.directive_policy
            .iter()
            .find(|policy| policy.kind == kind)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegistrationRefusal {
    InvalidKey(String),
    DuplicateKey(String),
    DuplicateAlias(String),
    AliasCollidesWithLiveKey(String),
    InvalidDefault(String),
}

#[derive(Debug, Clone, Default)]
pub struct DescriptorRegistry {
    descriptors: BTreeMap<PreferenceKey, PreferenceDescriptor>,
    aliases: BTreeMap<String, PreferenceKey>,
}

impl DescriptorRegistry {
    pub fn register(
        &mut self,
        descriptor: PreferenceDescriptor,
    ) -> Result<(), RegistrationRefusal> {
        let key = descriptor.key.clone();
        if self.descriptors.contains_key(&key) {
            return Err(RegistrationRefusal::DuplicateKey(key.as_str().to_owned()));
        }
        if self.aliases.contains_key(key.as_str()) {
            return Err(RegistrationRefusal::AliasCollidesWithLiveKey(
                key.as_str().to_owned(),
            ));
        }
        if descriptor
            .default_value
            .literal()
            .is_some_and(|value| !descriptor.validates(value))
        {
            return Err(RegistrationRefusal::InvalidDefault(key.as_str().to_owned()));
        }
        for alias in &descriptor.retired_aliases {
            if self.aliases.contains_key(alias) {
                return Err(RegistrationRefusal::DuplicateAlias(alias.clone()));
            }
            if self
                .descriptors
                .keys()
                .any(|registered| registered.as_str() == alias)
            {
                return Err(RegistrationRefusal::AliasCollidesWithLiveKey(alias.clone()));
            }
        }
        for alias in &descriptor.retired_aliases {
            self.aliases.insert(alias.clone(), key.clone());
        }
        self.descriptors.insert(key, descriptor);
        Ok(())
    }

    pub fn get(&self, key: &PreferenceKey) -> Option<&PreferenceDescriptor> {
        self.descriptors.get(key)
    }

    pub fn resolve_alias(&self, alias: &str) -> Option<&PreferenceKey> {
        self.aliases.get(alias)
    }

    pub fn len(&self) -> usize {
        self.descriptors.len()
    }

    pub fn is_empty(&self) -> bool {
        self.descriptors.is_empty()
    }

    pub fn keys(&self) -> impl Iterator<Item = &PreferenceKey> {
        self.descriptors.keys()
    }
}

#[cfg(test)]
mod tests;
