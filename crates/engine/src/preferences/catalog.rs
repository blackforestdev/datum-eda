use std::collections::BTreeSet;

#[path = "catalog_part_one.rs"]
mod catalog_part_one;
#[path = "catalog_part_two.rs"]
mod catalog_part_two;

use serde_json::{Value, json};

use super::resolver::AuthorityReleaseLevel;
use super::{
    AccessiblePresentation, ApplyBehavior, DescriptorDefault, DescriptorRegistry, DirectiveKind,
    DirectivePolicy, ExportClass, MergeCategory, PreferenceDescriptor, PreferenceKey,
    ResolutionSource, SemanticSchema, SettingClass, ValueSchema,
};

#[derive(Clone, Copy)]
struct SourceDeclaration {
    session: bool,
    context: bool,
    organization: bool,
}

const fn us(organization: bool) -> SourceDeclaration {
    SourceDeclaration {
        session: true,
        context: false,
        organization,
    }
}

const fn usc(organization: bool) -> SourceDeclaration {
    SourceDeclaration {
        session: true,
        context: true,
        organization,
    }
}

const fn seed() -> SourceDeclaration {
    SourceDeclaration {
        session: false,
        context: false,
        organization: true,
    }
}

fn sources(declaration: SourceDeclaration) -> BTreeSet<ResolutionSource> {
    let mut result = BTreeSet::from([
        ResolutionSource::DescriptorDefault,
        ResolutionSource::Installation,
        ResolutionSource::User,
    ]);
    if declaration.session {
        result.insert(ResolutionSource::Session);
    }
    if declaration.context {
        result.insert(ResolutionSource::Context);
    }
    if declaration.organization {
        result.insert(ResolutionSource::Organization);
    }
    result
}

fn rc() -> Vec<DirectivePolicy> {
    vec![
        DirectivePolicy {
            kind: DirectiveKind::Recommend,
            minimum_release: AuthorityReleaseLevel::RecommendationsOnly,
        },
        DirectivePolicy {
            kind: DirectiveKind::Constrain,
            minimum_release: AuthorityReleaseLevel::Bounded,
        },
    ]
}

fn rcpl(minimum_management: AuthorityReleaseLevel) -> Vec<DirectivePolicy> {
    let mut policies = rc();
    policies.extend([
        DirectivePolicy {
            kind: DirectiveKind::Pin,
            minimum_release: minimum_management,
        },
        DirectivePolicy {
            kind: DirectiveKind::Lock,
            minimum_release: minimum_management,
        },
    ]);
    policies
}

fn enum_schema(values: &[&str]) -> ValueSchema {
    ValueSchema::Enum(values.iter().map(|value| (*value).to_owned()).collect())
}

fn literal(value: Value) -> DescriptorDefault {
    DescriptorDefault::Literal(value)
}

fn runtime(recipe: &str) -> DescriptorDefault {
    DescriptorDefault::Runtime {
        recipe: recipe.to_owned(),
    }
}

#[allow(clippy::too_many_arguments)]
fn descriptor(
    key: &str,
    owner: &str,
    schema_name: &str,
    value_schema: ValueSchema,
    default_value: DescriptorDefault,
    class: SettingClass,
    source_declaration: SourceDeclaration,
    directive_policy: Vec<DirectivePolicy>,
    apply_behavior: ApplyBehavior,
    consumers: &[&str],
    export_class: ExportClass,
    label: &str,
    description: &str,
    aliases: &[&str],
) -> PreferenceDescriptor {
    let allowed_sources = sources(source_declaration);
    let allowed_directives = directive_policy.iter().map(|policy| policy.kind).collect();
    PreferenceDescriptor {
        key: PreferenceKey::parse(key).expect("catalog keys are stable Datum identities"),
        owner: owner.to_owned(),
        schema_version: 1,
        value_schema_name: schema_name.to_owned(),
        value_schema,
        default_value,
        class,
        allowed_sources,
        allowed_directives,
        directive_policy,
        merge_category: MergeCategory::Replace,
        apply_behavior,
        consumers: consumers
            .iter()
            .map(|consumer| (*consumer).to_owned())
            .collect(),
        export_class,
        presentation: AccessiblePresentation {
            label: label.to_owned(),
            description: description.to_owned(),
        },
        retired_aliases: aliases.iter().map(|alias| (*alias).to_owned()).collect(),
    }
}

pub fn active_v1_registry() -> DescriptorRegistry {
    let mut registry = DescriptorRegistry::default();
    for descriptor in active_descriptors() {
        registry
            .register(descriptor)
            .expect("the ratified V1 descriptor inventory must register exactly once");
    }
    registry
}

fn active_descriptors() -> Vec<PreferenceDescriptor> {
    let mut descriptors = catalog_part_one::descriptors();
    descriptors.extend(catalog_part_two::descriptors());
    descriptors
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn literal_inventory_is_complete_unique_and_default_valid() {
        let descriptors = active_descriptors();
        assert_eq!(descriptors.len(), 56);
        let keys: BTreeSet<_> = descriptors.iter().map(|item| item.key.as_str()).collect();
        assert_eq!(keys.len(), 56);
        assert!(keys.iter().all(|key| !key.starts_with("datum.revision.")));
        for descriptor in descriptors {
            if let Some(value) = descriptor.default_value.literal() {
                assert!(
                    descriptor.validates(value),
                    "invalid default for {}: {value}",
                    descriptor.key.as_str()
                );
            }
            assert_eq!(
                descriptor.allowed_directives,
                descriptor
                    .directive_policy
                    .iter()
                    .map(|policy| policy.kind)
                    .collect()
            );
        }
    }

    #[test]
    fn project_policy_seed_inventory_is_the_twelve_ratified_rows() {
        let registry = active_v1_registry();
        let seeds: BTreeSet<_> = registry
            .keys()
            .filter(|key| registry.get(key).unwrap().class == SettingClass::ProjectPolicySeed)
            .map(PreferenceKey::as_str)
            .collect();
        assert_eq!(seeds.len(), 14);
        assert_eq!(
            seeds,
            BTreeSet::from([
                "datum.projects.seed_profile",
                "datum.projects.template_set",
                "datum.projects.unit_policy_seed",
                "datum.publish.scale_fraction_style_seed",
                "datum.publish.sheet_format_seed",
                "datum.publish.title_block_template_seed",
                "datum.units.angle_precision",
                "datum.units.board_length",
                "datum.units.board_length_precision",
                "datum.units.drill_hole",
                "datum.units.drill_hole_precision",
                "datum.units.schematic_geometry",
                "datum.units.schematic_geometry_precision",
                "datum.units.system",
            ])
        );
    }

    #[test]
    fn catalog_semantic_golden_covers_every_descriptor_field() {
        let snapshot = format!("{:#?}", active_descriptors());
        let digest = snapshot
            .bytes()
            .fold(0xcbf29ce484222325_u64, |digest, byte| {
                (digest ^ u64::from(byte)).wrapping_mul(0x100000001b3)
            });
        assert_eq!(
            digest, 18_401_852_061_362_712_385,
            "update only after catalog-to-authority review"
        );
    }

    #[test]
    fn every_descriptor_has_the_exact_class_source_and_directive_boundary() {
        let context_keys = BTreeSet::from([
            "datum.checks.profile_prefill",
            "datum.files.autosave",
            "datum.input.editor_keymap",
            "datum.output.destination_prefill",
            "datum.output.job_prefill",
            "datum.pcb.airwires_hidden_layers",
            "datum.pcb.curved_airwires",
            "datum.pcb.ghost_via_through_pad",
            "datum.pcb.global_airwire_color",
            "datum.pcb.inactive_layer_dim_percent",
            "datum.pcb.layer_color_scheme",
            "datum.pcb.net_color_application",
            "datum.pcb.object_opacity",
            "datum.pcb.pad_outline_mode",
            "datum.pcb.rounded_track_corners",
            "datum.pcb.route_profile",
            "datum.pcb.selected_ratsnest_only",
            "datum.pcb.viewport_airwire_culling",
            "datum.publish.publish_set_naming",
            "datum.publish.viewport_creation_prefill",
            "datum.viewport.fine_grid_divisor",
            "datum.viewport.grid_mark_style",
            "datum.viewport.object_snap_types",
            "datum.viewport.snap_capture_px",
            "datum.viewport.snap_enabled",
        ]);
        for descriptor in active_descriptors() {
            let key = descriptor.key.as_str();
            let personal_accessibility = key.starts_with("datum.accessibility.");
            let mut expected_sources = BTreeSet::from([
                ResolutionSource::DescriptorDefault,
                ResolutionSource::Installation,
                ResolutionSource::User,
            ]);
            if descriptor.class != SettingClass::ProjectPolicySeed {
                expected_sources.insert(ResolutionSource::Session);
            }
            if !personal_accessibility {
                expected_sources.insert(ResolutionSource::Organization);
            }
            if context_keys.contains(key) {
                expected_sources.insert(ResolutionSource::Context);
            }
            assert_eq!(descriptor.allowed_sources, expected_sources, "{key}");

            let expected_directives = if personal_accessibility {
                BTreeSet::new()
            } else if descriptor.class == SettingClass::Presentation {
                BTreeSet::from([DirectiveKind::Recommend, DirectiveKind::Constrain])
            } else {
                BTreeSet::from([
                    DirectiveKind::Recommend,
                    DirectiveKind::Constrain,
                    DirectiveKind::Pin,
                    DirectiveKind::Lock,
                ])
            };
            assert_eq!(descriptor.allowed_directives, expected_directives, "{key}");
            assert!(
                !descriptor
                    .allowed_sources
                    .contains(&ResolutionSource::ProjectPolicy)
            );
        }
    }
}
