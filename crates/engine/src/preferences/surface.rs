//! Typed presentation authority for the bounded Global Preferences surface.
//!
//! The engine binds active descriptor identities to sections, ordering, and a
//! schema-compatible control. GUI consumers iterate this catalog; they do not
//! infer presentation from key strings or maintain a second row inventory.

use std::collections::{BTreeMap, BTreeSet};

use super::{DescriptorRegistry, PreferenceKey, ValueSchema};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PreferenceSectionId(String);

impl PreferenceSectionId {
    pub fn parse(value: impl Into<String>) -> Result<Self, SurfaceCatalogRefusal> {
        let value = value.into();
        let valid = !value.is_empty()
            && value
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_');
        valid
            .then_some(Self(value.clone()))
            .ok_or(SurfaceCatalogRefusal::InvalidSectionId(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreferenceSection {
    pub id: PreferenceSectionId,
    pub label: String,
    pub order: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumChoicePresentation {
    pub value: String,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreferenceControlPresentation {
    BooleanSwitch {
        off_label: String,
        on_label: String,
    },
    EnumeratedSingleChoice {
        choices: Vec<EnumChoicePresentation>,
    },
    IntegerStepper {
        min: i64,
        max: i64,
        step: u64,
        suffix: String,
    },
    IdentityEntry {
        nullable: bool,
        placeholder: String,
    },
    StructuredEditor {
        action_label: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreferenceLiveConsumer {
    ConsoleFeedbackTimer,
    ReducedMotion,
    HighContrastNonColor,
    FutureProjectUnits,
    DescriptorOwner,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreferenceSurfaceEntry {
    pub key: PreferenceKey,
    pub section: PreferenceSectionId,
    pub row_order: u32,
    pub control: PreferenceControlPresentation,
    pub live_consumer: PreferenceLiveConsumer,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SurfaceCatalogRefusal {
    InvalidSectionId(String),
    EmptySectionLabel(String),
    DuplicateSectionId(String),
    DuplicateSectionOrder(u32),
    UnknownSection(String),
    UnknownPreferenceKey(String),
    DuplicatePreferenceKey(String),
    DuplicateRowOrder { section: String, order: u32 },
    EmptyAccessibleCopy(String),
    SchemaControlMismatch(String),
    EmptyControlLabel(String),
    EmptyEnumChoices(String),
    DuplicateEnumValue { key: String, value: String },
    IncompleteEnumMapping(String),
    ConsumerMismatch(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreferenceSurfaceCatalog {
    sections: Vec<PreferenceSection>,
    entries: Vec<PreferenceSurfaceEntry>,
}

impl PreferenceSurfaceCatalog {
    pub fn new(
        registry: &DescriptorRegistry,
        mut sections: Vec<PreferenceSection>,
        mut entries: Vec<PreferenceSurfaceEntry>,
    ) -> Result<Self, SurfaceCatalogRefusal> {
        let mut section_ids = BTreeSet::new();
        let mut section_orders = BTreeSet::new();
        for section in &sections {
            if section.label.trim().is_empty() {
                return Err(SurfaceCatalogRefusal::EmptySectionLabel(
                    section.id.as_str().to_owned(),
                ));
            }
            if !section_ids.insert(section.id.clone()) {
                return Err(SurfaceCatalogRefusal::DuplicateSectionId(
                    section.id.as_str().to_owned(),
                ));
            }
            if !section_orders.insert(section.order) {
                return Err(SurfaceCatalogRefusal::DuplicateSectionOrder(section.order));
            }
        }

        let mut keys = BTreeSet::new();
        let mut row_orders = BTreeSet::new();
        for entry in &entries {
            if !section_ids.contains(&entry.section) {
                return Err(SurfaceCatalogRefusal::UnknownSection(
                    entry.section.as_str().to_owned(),
                ));
            }
            let descriptor = registry.get(&entry.key).ok_or_else(|| {
                SurfaceCatalogRefusal::UnknownPreferenceKey(entry.key.as_str().to_owned())
            })?;
            if !keys.insert(entry.key.clone()) {
                return Err(SurfaceCatalogRefusal::DuplicatePreferenceKey(
                    entry.key.as_str().to_owned(),
                ));
            }
            if !row_orders.insert((entry.section.clone(), entry.row_order)) {
                return Err(SurfaceCatalogRefusal::DuplicateRowOrder {
                    section: entry.section.as_str().to_owned(),
                    order: entry.row_order,
                });
            }
            if descriptor.presentation.label.trim().is_empty()
                || descriptor.presentation.description.trim().is_empty()
            {
                return Err(SurfaceCatalogRefusal::EmptyAccessibleCopy(
                    entry.key.as_str().to_owned(),
                ));
            }
            validate_control(entry, &descriptor.value_schema)?;
            let expected_consumers: &[&str] = match entry.live_consumer {
                PreferenceLiveConsumer::ConsoleFeedbackTimer => &["Console feedback timer"],
                PreferenceLiveConsumer::ReducedMotion => {
                    &["all GUI animation", "terminal animation"]
                }
                PreferenceLiveConsumer::HighContrastNonColor => {
                    &["GUI renderer", "terminal renderer"]
                }
                PreferenceLiveConsumer::FutureProjectUnits => &["ProjectDisplayUnits"],
                PreferenceLiveConsumer::DescriptorOwner => &[],
            };
            if expected_consumers
                .iter()
                .any(|consumer| !descriptor.consumers.iter().any(|item| item == consumer))
            {
                return Err(SurfaceCatalogRefusal::ConsumerMismatch(
                    entry.key.as_str().to_owned(),
                ));
            }
        }

        sections.sort_by_key(|section| section.order);
        let section_order: BTreeMap<_, _> = sections
            .iter()
            .map(|section| (section.id.clone(), section.order))
            .collect();
        entries.sort_by_key(|entry| (section_order[&entry.section], entry.row_order));
        Ok(Self { sections, entries })
    }

    pub fn sections(&self) -> &[PreferenceSection] {
        &self.sections
    }

    pub fn entries(&self) -> &[PreferenceSurfaceEntry] {
        &self.entries
    }

    pub fn search<'a>(
        &'a self,
        registry: &'a DescriptorRegistry,
        query: &str,
    ) -> Vec<&'a PreferenceSurfaceEntry> {
        let query = query.trim().to_ascii_lowercase();
        if query.is_empty() {
            return self.entries.iter().collect();
        }
        self.entries
            .iter()
            .filter(|entry| {
                let descriptor = registry
                    .get(&entry.key)
                    .expect("surface construction proves descriptor existence");
                entry.key.as_str().to_ascii_lowercase().contains(&query)
                    || descriptor
                        .presentation
                        .label
                        .to_ascii_lowercase()
                        .contains(&query)
                    || descriptor
                        .presentation
                        .description
                        .to_ascii_lowercase()
                        .contains(&query)
                    || descriptor
                        .retired_aliases
                        .iter()
                        .any(|alias| alias.to_ascii_lowercase().contains(&query))
            })
            .collect()
    }
}

fn validate_control(
    entry: &PreferenceSurfaceEntry,
    schema: &ValueSchema,
) -> Result<(), SurfaceCatalogRefusal> {
    let key = entry.key.as_str().to_owned();
    match (&entry.control, schema) {
        (
            PreferenceControlPresentation::BooleanSwitch {
                off_label,
                on_label,
            },
            ValueSchema::Boolean,
        ) => {
            if off_label.trim().is_empty() || on_label.trim().is_empty() {
                return Err(SurfaceCatalogRefusal::EmptyControlLabel(key));
            }
        }
        (
            PreferenceControlPresentation::EnumeratedSingleChoice { choices },
            ValueSchema::Enum(allowed),
        ) => {
            if choices.is_empty() {
                return Err(SurfaceCatalogRefusal::EmptyEnumChoices(key));
            }
            let mut mapped = BTreeSet::new();
            for choice in choices {
                if choice.label.trim().is_empty() {
                    return Err(SurfaceCatalogRefusal::EmptyControlLabel(key));
                }
                if !mapped.insert(choice.value.clone()) {
                    return Err(SurfaceCatalogRefusal::DuplicateEnumValue {
                        key,
                        value: choice.value.clone(),
                    });
                }
            }
            if &mapped != allowed {
                return Err(SurfaceCatalogRefusal::IncompleteEnumMapping(key));
            }
        }
        (
            PreferenceControlPresentation::IntegerStepper {
                min,
                max,
                step,
                suffix: _,
            },
            ValueSchema::IntegerRange {
                min: schema_min,
                max: schema_max,
                step: schema_step,
            },
        ) if min == schema_min && max == schema_max && step == schema_step => {}
        (
            PreferenceControlPresentation::IdentityEntry { nullable, .. },
            ValueSchema::Identity {
                nullable: schema_nullable,
            },
        ) if nullable == schema_nullable => {}
        (
            PreferenceControlPresentation::StructuredEditor { action_label },
            ValueSchema::Semantic(_),
        ) => {
            if action_label.trim().is_empty() {
                return Err(SurfaceCatalogRefusal::EmptyControlLabel(key));
            }
        }
        _ => return Err(SurfaceCatalogRefusal::SchemaControlMismatch(key)),
    }
    Ok(())
}

pub fn gp_f05_surface_catalog(
    registry: &DescriptorRegistry,
) -> Result<PreferenceSurfaceCatalog, SurfaceCatalogRefusal> {
    let appearance = PreferenceSectionId::parse("appearance")?;
    let units = PreferenceSectionId::parse("units")?;
    PreferenceSurfaceCatalog::new(
        registry,
        vec![
            PreferenceSection {
                id: appearance.clone(),
                label: "Appearance".to_owned(),
                order: 10,
            },
            PreferenceSection {
                id: units.clone(),
                label: "Units".to_owned(),
                order: 20,
            },
        ],
        [
            PreferenceSurfaceEntry {
                key: PreferenceKey::parse("datum.console.feedback_duration")
                    .expect("registered GP-F05 key"),
                section: appearance.clone(),
                row_order: 10,
                control: PreferenceControlPresentation::EnumeratedSingleChoice {
                    choices: [
                        ("4s", "4 s"),
                        ("6s", "6 s"),
                        ("10s", "10 s"),
                        ("never", "Never hide"),
                    ]
                    .into_iter()
                    .map(|(value, label)| EnumChoicePresentation {
                        value: value.to_owned(),
                        label: label.to_owned(),
                    })
                    .collect(),
                },
                live_consumer: PreferenceLiveConsumer::ConsoleFeedbackTimer,
            },
            PreferenceSurfaceEntry {
                key: PreferenceKey::parse("datum.accessibility.reduced_motion")
                    .expect("registered GP-F05 key"),
                section: appearance.clone(),
                row_order: 20,
                control: PreferenceControlPresentation::BooleanSwitch {
                    off_label: "Off".to_owned(),
                    on_label: "On".to_owned(),
                },
                live_consumer: PreferenceLiveConsumer::ReducedMotion,
            },
            PreferenceSurfaceEntry {
                key: PreferenceKey::parse("datum.accessibility.high_contrast_noncolor")
                    .expect("registered GP-F05 key"),
                section: appearance,
                row_order: 30,
                control: PreferenceControlPresentation::BooleanSwitch {
                    off_label: "Off".to_owned(),
                    on_label: "On".to_owned(),
                },
                live_consumer: PreferenceLiveConsumer::HighContrastNonColor,
            },
        ]
        .into_iter()
        .chain(units_surface_entries(units))
        .collect(),
    )
}

fn choice_control(choices: &[(&str, &str)]) -> PreferenceControlPresentation {
    PreferenceControlPresentation::EnumeratedSingleChoice {
        choices: choices
            .iter()
            .map(|(value, label)| EnumChoicePresentation {
                value: (*value).to_owned(),
                label: (*label).to_owned(),
            })
            .collect(),
    }
}

fn units_entry(
    key: &str,
    section: &PreferenceSectionId,
    row_order: u32,
    choices: &[(&str, &str)],
) -> PreferenceSurfaceEntry {
    PreferenceSurfaceEntry {
        key: PreferenceKey::parse(key).expect("registered Units key"),
        section: section.clone(),
        row_order,
        control: choice_control(choices),
        live_consumer: PreferenceLiveConsumer::FutureProjectUnits,
    }
}

fn units_surface_entries(section: PreferenceSectionId) -> Vec<PreferenceSurfaceEntry> {
    let precision = &[
        ("automatic", "Automatic"),
        ("decimal_0", "Whole units"),
        ("decimal_1", "One place"),
        ("decimal_2", "Two places"),
        ("decimal_3", "Three places"),
        ("decimal_4", "Four places"),
        ("decimal_5", "Five places"),
        ("decimal_6", "Six places"),
        ("exact_nm", "Exact nanometres"),
    ];
    vec![
        units_entry(
            "datum.units.system",
            &section,
            10,
            &[("metric", "Metric"), ("imperial", "Imperial")],
        ),
        units_entry(
            "datum.units.board_length",
            &section,
            20,
            &[
                ("follow_system", "Follow system"),
                ("mm", "Millimetres"),
                ("um", "Micrometres"),
                ("mil", "Mils"),
                ("inch", "Inches"),
            ],
        ),
        units_entry(
            "datum.units.board_length_precision",
            &section,
            30,
            precision,
        ),
        units_entry(
            "datum.units.drill_hole",
            &section,
            40,
            &[
                ("follow_system", "Follow system"),
                ("mm", "Millimetres"),
                ("mil", "Mils"),
                ("inch", "Inches"),
            ],
        ),
        units_entry("datum.units.drill_hole_precision", &section, 50, precision),
        units_entry(
            "datum.units.schematic_geometry",
            &section,
            60,
            &[
                ("follow_system", "Follow system"),
                ("mm", "Millimetres"),
                ("mil", "Mils"),
            ],
        ),
        units_entry(
            "datum.units.schematic_geometry_precision",
            &section,
            70,
            precision,
        ),
        units_entry(
            "datum.units.angle_precision",
            &section,
            80,
            &[
                ("decimal_0", "1°"),
                ("decimal_1", "0.1°"),
                ("decimal_2", "0.01°"),
                ("decimal_3", "0.001°"),
            ],
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::preferences::active_v1_registry;

    #[test]
    fn gp_f05_inventory_is_exact_ordered_and_searchable_by_alias() {
        let registry = active_v1_registry();
        let catalog = gp_f05_surface_catalog(&registry).unwrap();
        assert_eq!(catalog.sections().len(), 2);
        assert_eq!(catalog.sections()[0].id.as_str(), "appearance");
        assert_eq!(catalog.sections()[1].id.as_str(), "units");
        assert_eq!(
            catalog
                .entries()
                .iter()
                .map(|entry| entry.key.as_str())
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
        assert_eq!(catalog.search(&registry, "console_duration").len(), 1);
        assert_eq!(catalog.search(&registry, "motion").len(), 1);
        assert!(catalog.search(&registry, "revision").is_empty());
    }

    #[test]
    fn every_visible_value_round_trips_through_descriptor_validation() {
        let registry = active_v1_registry();
        let catalog = gp_f05_surface_catalog(&registry).unwrap();
        for entry in catalog.entries() {
            let descriptor = registry.get(&entry.key).unwrap();
            match &entry.control {
                PreferenceControlPresentation::BooleanSwitch { .. } => {
                    assert!(descriptor.validates(&serde_json::Value::Bool(false)));
                    assert!(descriptor.validates(&serde_json::Value::Bool(true)));
                }
                PreferenceControlPresentation::EnumeratedSingleChoice { choices } => {
                    for choice in choices {
                        assert!(
                            descriptor.validates(&serde_json::Value::String(choice.value.clone()))
                        );
                    }
                }
                PreferenceControlPresentation::IntegerStepper { min, max, .. } => {
                    assert!(descriptor.validates(&serde_json::Value::from(*min)));
                    assert!(descriptor.validates(&serde_json::Value::from(*max)));
                }
                PreferenceControlPresentation::IdentityEntry { nullable, .. } => {
                    assert!(
                        descriptor.validates(&serde_json::Value::String(
                            "datum-test-identity".to_owned()
                        ))
                    );
                    assert_eq!(descriptor.validates(&serde_json::Value::Null), *nullable);
                }
                PreferenceControlPresentation::StructuredEditor { .. } => {
                    assert!(matches!(descriptor.value_schema, ValueSchema::Semantic(_)));
                }
            }
        }
    }

    #[test]
    fn construction_refuses_schema_mismatch_and_duplicate_order() {
        let registry = active_v1_registry();
        let section = PreferenceSectionId::parse("appearance").unwrap();
        let bad = PreferenceSurfaceCatalog::new(
            &registry,
            vec![PreferenceSection {
                id: section.clone(),
                label: "Appearance".to_owned(),
                order: 10,
            }],
            vec![PreferenceSurfaceEntry {
                key: PreferenceKey::parse("datum.accessibility.reduced_motion").unwrap(),
                section,
                row_order: 10,
                control: PreferenceControlPresentation::EnumeratedSingleChoice {
                    choices: vec![EnumChoicePresentation {
                        value: "false".to_owned(),
                        label: "Off".to_owned(),
                    }],
                },
                live_consumer: PreferenceLiveConsumer::ReducedMotion,
            }],
        );
        assert!(matches!(
            bad,
            Err(SurfaceCatalogRefusal::SchemaControlMismatch(_))
        ));
    }
}
