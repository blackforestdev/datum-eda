//! Pure Global Preferences descriptor and resolution authority.
//!
//! This module owns no persistence, GUI, Project mutation, provider transport,
//! or Revision behavior. It validates an immutable descriptor registry and
//! delegates one side-effect-free resolution/explanation algorithm to
//! [`resolver`].

use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;

pub mod resolver;

pub use resolver::{
    AuthorityRelease, AuthorityReleaseLevel, ConsideredFact, Contribution, ContributionDisposition,
    OrganizationDirective, OrganizationFact, PreferenceExplanation, ResolutionOutcome,
    ResolutionRequest, ValueConstraint, ValueFact, resolve_preference,
};

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
    NextLaunch,
    NewProjectOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportClass {
    Portable,
    Protected,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValueSchema {
    Boolean,
    IntegerRange { min: i64, max: i64 },
    Enum(BTreeSet<String>),
    AllowedValues(Vec<Value>),
    String,
    Object,
    Array,
    OptionalObject,
    CanonicalJson,
}

impl ValueSchema {
    pub fn validate(&self, value: &Value) -> bool {
        match self {
            Self::Boolean => value.is_boolean(),
            Self::IntegerRange { min, max } => value
                .as_i64()
                .is_some_and(|candidate| candidate >= *min && candidate <= *max),
            Self::Enum(allowed) => value
                .as_str()
                .is_some_and(|candidate| allowed.contains(candidate)),
            Self::AllowedValues(allowed) => allowed.contains(value),
            Self::String => value.is_string(),
            Self::Object => value.is_object(),
            Self::Array => value.is_array(),
            Self::OptionalObject => value.is_null() || value.is_object(),
            Self::CanonicalJson => !value.is_null(),
        }
    }
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
    pub default_value: Option<Value>,
    pub class: SettingClass,
    pub allowed_sources: BTreeSet<ResolutionSource>,
    pub allowed_directives: BTreeSet<DirectiveKind>,
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
            .as_ref()
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

const ACTIVE_V1_KEYS: [&str; 54] = [
    "datum.console.feedback_duration",
    "datum.accessibility.reduced_motion",
    "datum.accessibility.high_contrast_noncolor",
    "datum.pcb.layer_color_scheme",
    "datum.schematic.drawing_theme",
    "datum.pcb.object_opacity",
    "datum.pcb.inactive_layer_dim_percent",
    "datum.pcb.pad_outline_mode",
    "datum.pcb.ghost_via_through_pad",
    "datum.pcb.rounded_track_corners",
    "datum.viewport.snap_enabled",
    "datum.viewport.snap_capture_px",
    "datum.viewport.fine_grid_divisor",
    "datum.viewport.object_snap_types",
    "datum.viewport.grid_mark_style",
    "datum.input.editor_keymap",
    "datum.units.system",
    "datum.units.board_length",
    "datum.units.drill_hole",
    "datum.units.schematic_geometry",
    "datum.units.length_precision",
    "datum.units.angle_format",
    "datum.pcb.route_profile",
    "datum.pcb.net_color_application",
    "datum.pcb.global_airwire_color",
    "datum.pcb.curved_airwires",
    "datum.pcb.selected_ratsnest_only",
    "datum.pcb.airwires_hidden_layers",
    "datum.pcb.viewport_airwire_culling",
    "datum.checks.profile_prefill",
    "datum.publish.title_block_template_seed",
    "datum.publish.sheet_format_seed",
    "datum.publish.scale_fraction_style_seed",
    "datum.publish.viewport_creation_prefill",
    "datum.publish.publish_set_naming",
    "datum.terminal.launch_profiles",
    "datum.terminal.theme",
    "datum.terminal.text_rendering",
    "datum.terminal.cursor",
    "datum.terminal.feedback",
    "datum.terminal.scrollback",
    "datum.terminal.notifications",
    "datum.terminal.keymap",
    "datum.terminal.multiline_paste",
    "datum.terminal.osc52_write",
    "datum.terminal.open_target_policy",
    "datum.projects.startup_view",
    "datum.files.default_locations",
    "datum.files.autosave",
    "datum.projects.seed_profile",
    "datum.projects.template_set",
    "datum.projects.unit_policy_seed",
    "datum.output.job_prefill",
    "datum.output.destination_prefill",
];

pub fn active_v1_registry() -> DescriptorRegistry {
    let mut registry = DescriptorRegistry::default();
    for key in ACTIVE_V1_KEYS {
        registry
            .register(active_descriptor(key))
            .expect("the ratified V1 descriptor inventory must register exactly once");
    }
    registry
}

fn active_descriptor(key: &str) -> PreferenceDescriptor {
    let class = descriptor_class(key);
    let mut allowed_sources = BTreeSet::from([
        ResolutionSource::DescriptorDefault,
        ResolutionSource::Installation,
        ResolutionSource::User,
    ]);
    let personal_accessibility = key.starts_with("datum.accessibility.");
    if !personal_accessibility {
        allowed_sources.insert(ResolutionSource::Organization);
    }
    if class != SettingClass::ProjectPolicySeed {
        allowed_sources.insert(ResolutionSource::Session);
    }
    if context_eligible(key) {
        allowed_sources.insert(ResolutionSource::Context);
    }

    let allowed_directives = if personal_accessibility {
        BTreeSet::new()
    } else if class == SettingClass::Presentation {
        BTreeSet::from([DirectiveKind::Recommend, DirectiveKind::Constrain])
    } else {
        BTreeSet::from([
            DirectiveKind::Recommend,
            DirectiveKind::Constrain,
            DirectiveKind::Pin,
            DirectiveKind::Lock,
        ])
    };
    let (value_schema_name, value_schema, default_value) = schema_and_default(key);
    let owner = key.split('.').nth(1).unwrap_or("engine").to_owned();
    let apply_behavior = if class == SettingClass::ProjectPolicySeed {
        ApplyBehavior::NewProjectOnly
    } else if key == "datum.projects.startup_view" {
        ApplyBehavior::NextLaunch
    } else {
        ApplyBehavior::Live
    };
    let export_class =
        if key.starts_with("datum.terminal.") && matches!(class, SettingClass::Capability) {
            ExportClass::Protected
        } else {
            ExportClass::Portable
        };
    PreferenceDescriptor {
        key: PreferenceKey::parse(key).expect("catalog keys are stable Datum identities"),
        owner,
        schema_version: 1,
        value_schema_name: value_schema_name.to_owned(),
        value_schema,
        default_value,
        class,
        allowed_sources,
        allowed_directives,
        merge_category: MergeCategory::Replace,
        apply_behavior,
        consumers: vec![key.split('.').nth(1).unwrap_or("engine").to_owned()],
        export_class,
        presentation: AccessiblePresentation {
            label: key.rsplit('.').next().unwrap_or(key).replace('_', " "),
            description: format!(
                "Controls {} behavior.",
                key.rsplit('.').next().unwrap_or(key)
            ),
        },
        retired_aliases: retired_aliases(key),
    }
}

fn descriptor_class(key: &str) -> SettingClass {
    if key.starts_with("datum.units.")
        || matches!(
            key,
            "datum.publish.title_block_template_seed"
                | "datum.publish.sheet_format_seed"
                | "datum.publish.scale_fraction_style_seed"
                | "datum.projects.seed_profile"
                | "datum.projects.template_set"
                | "datum.projects.unit_policy_seed"
        )
    {
        SettingClass::ProjectPolicySeed
    } else if matches!(
        key,
        "datum.terminal.launch_profiles"
            | "datum.terminal.scrollback"
            | "datum.terminal.notifications"
            | "datum.terminal.multiline_paste"
            | "datum.terminal.osc52_write"
            | "datum.terminal.open_target_policy"
    ) {
        SettingClass::Capability
    } else if key.starts_with("datum.viewport.")
        || key == "datum.input.editor_keymap"
        || key == "datum.pcb.route_profile"
        || key == "datum.checks.profile_prefill"
        || key == "datum.publish.viewport_creation_prefill"
        || key == "datum.publish.publish_set_naming"
        || key == "datum.terminal.keymap"
        || key == "datum.projects.startup_view"
        || key.starts_with("datum.files.")
        || key.starts_with("datum.output.")
    {
        SettingClass::WorkflowDefault
    } else {
        SettingClass::Presentation
    }
}

fn context_eligible(key: &str) -> bool {
    key.starts_with("datum.viewport.")
        || key == "datum.input.editor_keymap"
        || key == "datum.pcb.route_profile"
        || key == "datum.checks.profile_prefill"
        || key == "datum.publish.viewport_creation_prefill"
        || key == "datum.publish.publish_set_naming"
        || key == "datum.files.autosave"
        || key.starts_with("datum.output.")
        || matches!(
            key,
            "datum.pcb.layer_color_scheme"
                | "datum.pcb.object_opacity"
                | "datum.pcb.inactive_layer_dim_percent"
                | "datum.pcb.pad_outline_mode"
                | "datum.pcb.ghost_via_through_pad"
                | "datum.pcb.rounded_track_corners"
                | "datum.pcb.net_color_application"
                | "datum.pcb.global_airwire_color"
                | "datum.pcb.curved_airwires"
                | "datum.pcb.selected_ratsnest_only"
                | "datum.pcb.airwires_hidden_layers"
                | "datum.pcb.viewport_airwire_culling"
        )
}

fn schema_and_default(key: &str) -> (&'static str, ValueSchema, Option<Value>) {
    if let Some((allowed, default)) = enum_values_and_default(key) {
        return (
            "closed catalog enum",
            ValueSchema::AllowedValues(allowed),
            Some(default),
        );
    }
    let boolean_keys = [
        "datum.accessibility.reduced_motion",
        "datum.accessibility.high_contrast_noncolor",
        "datum.pcb.pad_outline_mode",
        "datum.pcb.ghost_via_through_pad",
        "datum.pcb.rounded_track_corners",
        "datum.viewport.snap_enabled",
        "datum.pcb.curved_airwires",
        "datum.pcb.selected_ratsnest_only",
        "datum.pcb.viewport_airwire_culling",
    ];
    if boolean_keys.contains(&key) {
        let default = matches!(
            key,
            "datum.viewport.snap_enabled" | "datum.pcb.selected_ratsnest_only"
        );
        return ("bool", ValueSchema::Boolean, Some(Value::Bool(default)));
    }
    match key {
        "datum.pcb.inactive_layer_dim_percent" => (
            "integer 0..100",
            ValueSchema::IntegerRange { min: 0, max: 100 },
            Some(Value::from(50)),
        ),
        "datum.viewport.snap_capture_px" => (
            "integer 1..64",
            ValueSchema::IntegerRange { min: 1, max: 64 },
            Some(Value::from(10)),
        ),
        "datum.projects.unit_policy_seed" => (
            "optional ProjectDisplayUnits aggregate",
            ValueSchema::OptionalObject,
            None,
        ),
        "datum.publish.title_block_template_seed"
        | "datum.projects.seed_profile"
        | "datum.projects.template_set" => (
            "resolvable identity",
            ValueSchema::String,
            Some(Value::String("datum_factory".to_owned())),
        ),
        key if key.contains("keymap") || key.contains("locations") => (
            "versioned map",
            ValueSchema::Object,
            Some(Value::Object(Default::default())),
        ),
        key if key.contains("profiles") => (
            "versioned named profile list",
            ValueSchema::Array,
            Some(Value::Array(Vec::new())),
        ),
        key if key.contains("opacity")
            || key.contains("style")
            || key.contains("format")
            || key.contains("rendering")
            || key.contains("cursor")
            || key.contains("feedback")
            || key.contains("scrollback")
            || key.contains("autosave")
            || key.contains("naming")
            || key.contains("seed") =>
        {
            (
                "typed object",
                ValueSchema::Object,
                Some(Value::Object(Default::default())),
            )
        }
        _ => (
            "catalog enum or canonical scalar",
            ValueSchema::CanonicalJson,
            Some(Value::String(default_scalar(key).to_owned())),
        ),
    }
}

fn enum_values_and_default(key: &str) -> Option<(Vec<Value>, Value)> {
    let (values, default): (&[&str], &str) = match key {
        "datum.console.feedback_duration" => (&["4s", "6s", "10s", "never"], "6s"),
        "datum.pcb.layer_color_scheme" => (&["datum", "high_contrast_mono", "photonics"], "datum"),
        "datum.schematic.drawing_theme" => (&["dark", "light"], "dark"),
        "datum.units.system" => (&["metric", "imperial"], "metric"),
        "datum.units.board_length" => (
            &["follow_system", "mm", "um", "mil", "inch"],
            "follow_system",
        ),
        "datum.units.drill_hole" => (&["follow_system", "mm", "mil", "inch"], "follow_system"),
        "datum.units.schematic_geometry" => (&["follow_system", "mm", "mil"], "follow_system"),
        "datum.units.length_precision" => (&["0.1", "0.01", "0.001", "0.0001", "exact_nm"], "0.01"),
        "datum.pcb.route_profile" => (
            &["conservative", "balanced", "high_density"],
            "conservative",
        ),
        "datum.pcb.net_color_application" => {
            (&["none", "ratsnest", "copper_ratsnest", "all"], "ratsnest")
        }
        "datum.pcb.airwires_hidden_layers" => (&["visible_layers", "all_layers"], "visible_layers"),
        "datum.checks.profile_prefill" => (&["full", "fast", "last_used"], "last_used"),
        "datum.publish.scale_fraction_style_seed" => {
            (&["one_to_n", "n_over_one", "custom_pattern"], "one_to_n")
        }
        "datum.publish.viewport_creation_prefill" => {
            (&["on_demand", "on_demand_remember_style"], "on_demand")
        }
        "datum.terminal.theme" => (&["datum_dark", "high_contrast"], "datum_dark"),
        "datum.terminal.notifications" => (&["off", "unfocused", "always"], "unfocused"),
        "datum.terminal.multiline_paste" | "datum.terminal.osc52_write" => {
            (&["ask", "allow", "block"], "ask")
        }
        "datum.terminal.open_target_policy" => (&["ask", "trusted_only", "never"], "ask"),
        "datum.projects.startup_view" => (&["start_page", "last_session", "empty"], "start_page"),
        "datum.output.job_prefill" => (&["ask", "last_used"], "ask"),
        "datum.output.destination_prefill" => (&["ask", "project_outputs"], "ask"),
        _ => return None,
    };
    Some((
        values
            .iter()
            .map(|value| Value::String((*value).to_owned()))
            .collect(),
        Value::String(default.to_owned()),
    ))
}

fn default_scalar(key: &str) -> &'static str {
    match key {
        "datum.console.feedback_duration" => "6s",
        "datum.schematic.drawing_theme" => "dark",
        "datum.units.system" => "metric",
        "datum.pcb.route_profile" => "conservative",
        "datum.checks.profile_prefill" => "last_used",
        "datum.projects.startup_view" => "start_page",
        "datum.output.job_prefill" | "datum.output.destination_prefill" => "ask",
        _ => "datum_default",
    }
}

fn retired_aliases(key: &str) -> BTreeSet<String> {
    let aliases: &[&str] = match key {
        "datum.console.feedback_duration" => &["console_duration"],
        "datum.schematic.drawing_theme" => &["datum.schematic.theme"],
        "datum.publish.publish_set_naming" => &["datum.publish.set_name_prefill"],
        "datum.projects.startup_view" => &["datum.files.startup_mode"],
        "datum.projects.template_set" => &["datum.projects.template_seed"],
        _ => &[],
    };
    aliases.iter().map(|alias| (*alias).to_owned()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn active_registry_matches_ratified_identity_inventory() {
        let registry = active_v1_registry();
        assert_eq!(registry.len(), 54);
        assert!(
            registry
                .keys()
                .all(|key| !key.as_str().starts_with("datum.revision."))
        );
        assert_eq!(
            registry
                .resolve_alias("datum.schematic.theme")
                .map(PreferenceKey::as_str),
            Some("datum.schematic.drawing_theme")
        );
    }

    #[test]
    fn duplicate_live_identity_and_alias_collision_are_refused() {
        let descriptor = active_descriptor("datum.viewport.snap_enabled");
        let mut registry = DescriptorRegistry::default();
        registry.register(descriptor.clone()).unwrap();
        assert!(matches!(
            registry.register(descriptor),
            Err(RegistrationRefusal::DuplicateKey(_))
        ));
    }

    #[test]
    fn source_and_control_boundaries_follow_catalog_classes() {
        let registry = active_v1_registry();
        let accessibility = registry
            .get(&PreferenceKey::parse("datum.accessibility.reduced_motion").unwrap())
            .unwrap();
        assert!(
            !accessibility
                .allowed_sources
                .contains(&ResolutionSource::Organization)
        );
        assert!(accessibility.allowed_directives.is_empty());

        let seed = registry
            .get(&PreferenceKey::parse("datum.units.system").unwrap())
            .unwrap();
        assert_eq!(seed.class, SettingClass::ProjectPolicySeed);
        assert!(!seed.allowed_sources.contains(&ResolutionSource::Session));
        assert!(!seed.allowed_sources.contains(&ResolutionSource::Context));
    }
}
