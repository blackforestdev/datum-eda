//! Complete active Global Preferences presentation catalog for GP-CM01.
//!
//! Placement and controls are explicit typed metadata. Consumers never infer a
//! section or widget from a key prefix, and excluded/deferred identities never
//! enter this catalog.

use super::{
    DescriptorRegistry, EnumChoicePresentation, PreferenceControlPresentation,
    PreferenceLiveConsumer, PreferenceSection, PreferenceSectionId, PreferenceSurfaceCatalog,
    PreferenceSurfaceEntry, SurfaceCatalogRefusal, ValueSchema,
};

struct SectionSpec {
    id: &'static str,
    label: &'static str,
    keys: &'static [&'static str],
}

const SECTIONS: &[SectionSpec] = &[
    SectionSpec {
        id: "appearance",
        label: "Appearance",
        keys: &[
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
        ],
    },
    SectionSpec {
        id: "workspace",
        label: "Workspace",
        keys: &[
            "datum.viewport.snap_enabled",
            "datum.viewport.snap_capture_px",
            "datum.viewport.fine_grid_divisor",
            "datum.viewport.object_snap_types",
            "datum.viewport.grid_mark_style",
            "datum.input.editor_keymap",
        ],
    },
    SectionSpec {
        id: "units",
        label: "Units",
        keys: &[
            "datum.units.system",
            "datum.units.board_length",
            "datum.units.board_length_precision",
            "datum.units.drill_hole",
            "datum.units.drill_hole_precision",
            "datum.units.schematic_geometry",
            "datum.units.schematic_geometry_precision",
            "datum.units.angle_precision",
        ],
    },
    SectionSpec {
        id: "pcb_board",
        label: "PCB / Board",
        keys: &[
            "datum.pcb.route_profile",
            "datum.pcb.net_color_application",
            "datum.pcb.global_airwire_color",
            "datum.pcb.curved_airwires",
            "datum.pcb.selected_ratsnest_only",
            "datum.pcb.airwires_hidden_layers",
            "datum.pcb.viewport_airwire_culling",
        ],
    },
    SectionSpec {
        id: "publish_space",
        label: "Publish Space",
        keys: &[
            "datum.publish.title_block_template_seed",
            "datum.publish.sheet_format_seed",
            "datum.publish.scale_fraction_style_seed",
            "datum.publish.viewport_creation_prefill",
            "datum.publish.publish_set_naming",
        ],
    },
    SectionSpec {
        id: "rules_checks",
        label: "Rules & Checks",
        keys: &["datum.checks.profile_prefill"],
    },
    SectionSpec {
        id: "terminal",
        label: "Terminal",
        keys: &[
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
        ],
    },
    SectionSpec {
        id: "files_projects",
        label: "Files & Projects",
        keys: &[
            "datum.projects.startup_view",
            "datum.files.default_locations",
            "datum.files.autosave",
            "datum.projects.seed_profile",
            "datum.projects.template_set",
            "datum.projects.unit_policy_seed",
        ],
    },
    SectionSpec {
        id: "output",
        label: "Output",
        keys: &[
            "datum.output.job_prefill",
            "datum.output.destination_prefill",
        ],
    },
];

pub fn global_preferences_surface_catalog(
    registry: &DescriptorRegistry,
) -> Result<PreferenceSurfaceCatalog, SurfaceCatalogRefusal> {
    let sections = SECTIONS
        .iter()
        .enumerate()
        .map(|(index, spec)| {
            Ok(PreferenceSection {
                id: PreferenceSectionId::parse(spec.id)?,
                label: spec.label.to_owned(),
                order: (index as u32 + 1) * 10,
            })
        })
        .collect::<Result<Vec<_>, SurfaceCatalogRefusal>>()?;
    let mut entries = Vec::new();
    for (section_index, spec) in SECTIONS.iter().enumerate() {
        let section = sections[section_index].id.clone();
        for (row_index, key) in spec.keys.iter().enumerate() {
            let key = super::PreferenceKey::parse(*key).expect("catalog key is registered syntax");
            let descriptor = registry.get(&key).ok_or_else(|| {
                SurfaceCatalogRefusal::UnknownPreferenceKey(key.as_str().to_owned())
            })?;
            entries.push(PreferenceSurfaceEntry {
                key,
                section: section.clone(),
                row_order: (row_index as u32 + 1) * 10,
                control: control_for(descriptor.value_schema.clone(), descriptor.key.as_str()),
                live_consumer: live_consumer(descriptor.key.as_str()),
            });
        }
    }
    PreferenceSurfaceCatalog::new(registry, sections, entries)
}

fn live_consumer(key: &str) -> PreferenceLiveConsumer {
    match key {
        "datum.console.feedback_duration" => PreferenceLiveConsumer::ConsoleFeedbackTimer,
        "datum.accessibility.reduced_motion" => PreferenceLiveConsumer::ReducedMotion,
        "datum.accessibility.high_contrast_noncolor" => {
            PreferenceLiveConsumer::HighContrastNonColor
        }
        "datum.units.system"
        | "datum.units.board_length"
        | "datum.units.board_length_precision"
        | "datum.units.drill_hole"
        | "datum.units.drill_hole_precision"
        | "datum.units.schematic_geometry"
        | "datum.units.schematic_geometry_precision"
        | "datum.units.angle_precision" => PreferenceLiveConsumer::FutureProjectUnits,
        _ => PreferenceLiveConsumer::DescriptorOwner,
    }
}

fn control_for(schema: ValueSchema, key: &str) -> PreferenceControlPresentation {
    match schema {
        ValueSchema::Boolean => PreferenceControlPresentation::BooleanSwitch {
            off_label: "Off".to_owned(),
            on_label: "On".to_owned(),
        },
        ValueSchema::IntegerRange { min, max, step } => {
            PreferenceControlPresentation::IntegerStepper {
                min,
                max,
                step,
                suffix: integer_suffix(key).to_owned(),
            }
        }
        ValueSchema::Enum(values) => {
            let ordered = enum_order(key);
            debug_assert_eq!(
                ordered
                    .iter()
                    .copied()
                    .collect::<std::collections::BTreeSet<_>>(),
                values.iter().map(String::as_str).collect()
            );
            PreferenceControlPresentation::EnumeratedSingleChoice {
                choices: ordered
                    .iter()
                    .map(|value| EnumChoicePresentation {
                        label: enum_label(key, value),
                        value: (*value).to_owned(),
                    })
                    .collect(),
            }
        }
        ValueSchema::Identity { nullable } => PreferenceControlPresentation::IdentityEntry {
            nullable,
            placeholder: if nullable {
                "None or registered identity"
            } else {
                "Registered identity"
            }
            .to_owned(),
        },
        ValueSchema::Semantic(_) => PreferenceControlPresentation::StructuredEditor {
            action_label: "Configure…".to_owned(),
        },
    }
}

fn integer_suffix(key: &str) -> &'static str {
    match key {
        "datum.viewport.snap_capture_px" => " px",
        "datum.pcb.inactive_layer_dim_percent" => "%",
        _ => "",
    }
}

fn enum_order(key: &str) -> &'static [&'static str] {
    match key {
        "datum.console.feedback_duration" => &["4s", "6s", "10s", "never"],
        "datum.pcb.layer_color_scheme" => &["datum", "high_contrast_mono", "photonics"],
        "datum.schematic.drawing_theme" => &["dark", "light"],
        "datum.viewport.fine_grid_divisor" => &["2", "5", "10"],
        "datum.units.system" => &["metric", "imperial"],
        "datum.units.board_length" => &["follow_system", "mm", "um", "mil", "inch"],
        "datum.units.board_length_precision"
        | "datum.units.drill_hole_precision"
        | "datum.units.schematic_geometry_precision" => &[
            "automatic",
            "decimal_0",
            "decimal_1",
            "decimal_2",
            "decimal_3",
            "decimal_4",
            "decimal_5",
            "decimal_6",
            "exact_nm",
        ],
        "datum.units.drill_hole" => &["follow_system", "mm", "mil", "inch"],
        "datum.units.schematic_geometry" => &["follow_system", "mm", "mil"],
        "datum.units.angle_precision" => &["decimal_0", "decimal_1", "decimal_2", "decimal_3"],
        "datum.pcb.route_profile" => &["conservative", "balanced", "high_density"],
        "datum.pcb.net_color_application" => &["none", "ratsnest", "copper_ratsnest", "all"],
        "datum.pcb.airwires_hidden_layers" => &["visible_layers", "all_layers"],
        "datum.checks.profile_prefill" => &["full", "fast", "last_used"],
        "datum.publish.scale_fraction_style_seed" => &["one_to_n", "n_over_one", "custom_pattern"],
        "datum.publish.viewport_creation_prefill" => &["on_demand", "on_demand_remember_style"],
        "datum.terminal.theme" => &["datum_dark", "high_contrast"],
        "datum.terminal.notifications" => &["off", "unfocused", "always"],
        "datum.terminal.multiline_paste" | "datum.terminal.osc52_write" => {
            &["ask", "allow", "block"]
        }
        "datum.terminal.open_target_policy" => &["ask", "trusted_only", "never"],
        "datum.projects.startup_view" => &["start_page", "last_session", "empty"],
        "datum.output.job_prefill" => &["ask", "last_used"],
        "datum.output.destination_prefill" => &["ask", "project_outputs"],
        _ => panic!("missing typed enum presentation order for {key}"),
    }
}

fn enum_label(key: &str, value: &str) -> String {
    match (key, value) {
        ("datum.console.feedback_duration", "4s") => "4 s".to_owned(),
        ("datum.console.feedback_duration", "6s") => "6 s".to_owned(),
        ("datum.console.feedback_duration", "10s") => "10 s".to_owned(),
        ("datum.console.feedback_duration", "never") => "Never hide".to_owned(),
        ("datum.units.board_length", "follow_system")
        | ("datum.units.drill_hole", "follow_system")
        | ("datum.units.schematic_geometry", "follow_system") => "Follow system".to_owned(),
        (key, value) if key.ends_with("_precision") && value == "automatic" => {
            "Automatic".to_owned()
        }
        (key, "exact_nm") if key.ends_with("_precision") => "Exact nanometres".to_owned(),
        ("datum.units.angle_precision", "decimal_0") => "1°".to_owned(),
        ("datum.units.angle_precision", "decimal_1") => "0.1°".to_owned(),
        ("datum.units.angle_precision", "decimal_2") => "0.01°".to_owned(),
        ("datum.units.angle_precision", "decimal_3") => "0.001°".to_owned(),
        ("datum.pcb.layer_color_scheme", "datum") => "Datum default".to_owned(),
        ("datum.pcb.layer_color_scheme", "high_contrast_mono") => "High-contrast mono".to_owned(),
        ("datum.pcb.layer_color_scheme", "photonics") => "Photonics-friendly".to_owned(),
        ("datum.pcb.net_color_application", "copper_ratsnest") => "Copper + ratsnest".to_owned(),
        ("datum.publish.scale_fraction_style_seed", "one_to_n") => "1 : n".to_owned(),
        ("datum.publish.scale_fraction_style_seed", "n_over_one") => "n / 1".to_owned(),
        ("datum.publish.scale_fraction_style_seed", "custom_pattern") => {
            "Custom pattern".to_owned()
        }
        ("datum.terminal.theme", "datum_dark") => "Datum Dark".to_owned(),
        ("datum.terminal.notifications", "unfocused") => "When unfocused".to_owned(),
        ("datum.output.destination_prefill", "project_outputs") => {
            "Project outputs folder".to_owned()
        }
        _ => sentence_case(value),
    }
}

fn sentence_case(value: &str) -> String {
    let words = value.split('_').collect::<Vec<_>>();
    let mut label = words.join(" ");
    if let Some(first) = label.get_mut(0..1) {
        first.make_ascii_uppercase();
    }
    label
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::preferences::active_v1_registry;
    use std::collections::BTreeSet;

    #[test]
    fn complete_surface_is_a_bijection_over_the_active_non_revision_registry() {
        let registry = active_v1_registry();
        let catalog = global_preferences_surface_catalog(&registry).unwrap();
        assert_eq!(catalog.entries().len(), registry.len());
        assert_eq!(catalog.sections().len(), 9);
        assert_eq!(
            catalog
                .entries()
                .iter()
                .map(|entry| entry.key.as_str())
                .collect::<BTreeSet<_>>(),
            registry
                .keys()
                .map(|key| key.as_str())
                .collect::<BTreeSet<_>>()
        );
        assert!(catalog.entries().iter().all(|entry| {
            !entry.key.as_str().starts_with("datum.revision.")
                && !entry.section.as_str().contains("revision")
        }));
    }

    #[test]
    fn generated_enum_labels_never_expose_storage_snake_case() {
        let registry = active_v1_registry();
        let catalog = global_preferences_surface_catalog(&registry).unwrap();
        for entry in catalog.entries() {
            if let PreferenceControlPresentation::EnumeratedSingleChoice { choices } =
                &entry.control
            {
                assert!(choices.iter().all(|choice| !choice.label.contains('_')));
            }
        }
    }
}
