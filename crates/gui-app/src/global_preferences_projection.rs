//! Typed GUI projection and total live-consumer adapters for GP-F05.

use datum_gui_protocol::{
    ConsoleFeedbackDuration, GlobalPreferenceControlUi, GlobalPreferencesNoticeUi, TerminalTheme,
    WorkspaceUiState,
};
use eda_engine::ir::units::{
    BOARD_LENGTH_KEY, BOARD_PRECISION_KEY, DRILL_HOLE_KEY, DRILL_PRECISION_KEY,
    ResolvedUnitsProfile, SCHEMATIC_GEOMETRY_KEY, SCHEMATIC_PRECISION_KEY,
};
use eda_engine::preferences::{
    GlobalPreferenceRow, LegacyConsoleMigrationState, PreferenceControlPresentation,
    PreferenceLiveConsumer, PreferenceServiceStatus, PreferenceSurfaceEntry,
};
use serde_json::Value;

use crate::global_preferences_runtime::SCOPE;

pub(super) fn provenance_label(
    row: &GlobalPreferenceRow,
    status: &PreferenceServiceStatus,
) -> String {
    if !status.writable() {
        return format!("Preserved · {SCOPE}");
    }
    if row.user_value.is_some() {
        if row.key.as_str().starts_with("datum.units.") {
            format!("Set by you · {SCOPE} · Default for new Projects · Open Projects unaffected")
        } else {
            format!("Set by you · {SCOPE}")
        }
    } else if row.key.as_str().starts_with("datum.units.") {
        format!("Factory default · {SCOPE} · Default for new Projects · Open Projects unaffected")
    } else {
        format!("Factory default · {SCOPE}")
    }
}

pub(super) fn explanation_lines(row: &GlobalPreferenceRow) -> Vec<String> {
    let explanation = &row.explanation;
    let effective = row
        .effective_value
        .as_ref()
        .map(Value::to_string)
        .unwrap_or_else(|| "unavailable".to_owned());
    vec![
        format!("Descriptor  {}", row.key.as_str()),
        format!("Scope       {}", explanation.machine_scope),
        format!(
            "Effective   {} ({})",
            effective,
            if row.user_value.is_some() {
                "set by you"
            } else {
                "factory default"
            }
        ),
        format!(
            "Eligible absences  {}",
            explanation
                .absent_sources
                .iter()
                .map(|source| format!("{source:?}"))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        explanation
            .evaluation_stages
            .last()
            .cloned()
            .unwrap_or_else(|| "Resolved through the typed preference resolver.".to_owned()),
    ]
}

pub(super) fn repository_notice(
    status: &PreferenceServiceStatus,
    legacy: &LegacyConsoleMigrationState,
) -> Option<GlobalPreferencesNoticeUi> {
    match status {
        PreferenceServiceStatus::PreservedUnreadable { reason, .. } => {
            Some(GlobalPreferencesNoticeUi::PreservedUnreadable(format!(
                "Preferences could not be read. Factory defaults are active for this session; damaged data remains preserved and controls are unavailable. {reason}"
            )))
        }
        PreferenceServiceStatus::MigrationRequired { .. } => {
            Some(GlobalPreferencesNoticeUi::Assertive(
                "Preferences require a registered migration; controls are unavailable.".to_owned(),
            ))
        }
        PreferenceServiceStatus::DefaultsOnly | PreferenceServiceStatus::Ready { .. } => {
            match legacy {
                LegacyConsoleMigrationState::Migrated { value, .. } => {
                    Some(GlobalPreferencesNoticeUi::Polite(format!(
                        "Legacy Console feedback duration migrated as {value}; the original file remains preserved."
                    )))
                }
                LegacyConsoleMigrationState::PreservedInvalid { reason } => {
                    Some(GlobalPreferencesNoticeUi::Assertive(format!(
                        "Legacy Console preference was not migrated; its exact bytes remain preserved and the factory default is active. {reason}"
                    )))
                }
                LegacyConsoleMigrationState::PreservedUnreadable { reason } => {
                    Some(GlobalPreferencesNoticeUi::Assertive(format!(
                        "Legacy Console preference could not be read and remains unmodified; the factory default is active. {reason}"
                    )))
                }
                LegacyConsoleMigrationState::Absent | LegacyConsoleMigrationState::NotNeeded => {
                    None
                }
            }
        }
    }
}

pub(super) fn control_projection(
    control: &PreferenceControlPresentation,
    row: &GlobalPreferenceRow,
    units: Option<&ResolvedUnitsProfile>,
) -> GlobalPreferenceControlUi {
    control_value_projection(
        control,
        row.effective_value.as_ref(),
        row.key.as_str(),
        units,
    )
}

pub(super) fn control_value_projection(
    control: &PreferenceControlPresentation,
    effective_value: Option<&Value>,
    key: &str,
    units: Option<&ResolvedUnitsProfile>,
) -> GlobalPreferenceControlUi {
    match control {
        PreferenceControlPresentation::BooleanSwitch {
            off_label,
            on_label,
        } => GlobalPreferenceControlUi::Boolean {
            value: effective_value.and_then(Value::as_bool).unwrap_or(false),
            off_label: off_label.clone(),
            on_label: on_label.clone(),
        },
        PreferenceControlPresentation::EnumeratedSingleChoice { choices } => {
            GlobalPreferenceControlUi::SingleChoice {
                value: effective_value
                    .and_then(Value::as_str)
                    .unwrap_or("unavailable")
                    .to_owned(),
                choices: choices
                    .iter()
                    .map(|choice| {
                        (
                            choice.value.clone(),
                            resolved_units_choice_label(key, &choice.value, &choice.label, units),
                        )
                    })
                    .collect(),
            }
        }
    }
}

fn resolved_units_choice_label(
    key: &str,
    value: &str,
    fallback: &str,
    units: Option<&ResolvedUnitsProfile>,
) -> String {
    let Some(units) = units else {
        return fallback.to_owned();
    };
    let resolved = match key {
        BOARD_LENGTH_KEY | BOARD_PRECISION_KEY => units.board,
        DRILL_HOLE_KEY | DRILL_PRECISION_KEY => units.drill,
        SCHEMATIC_GEOMETRY_KEY | SCHEMATIC_PRECISION_KEY => units.schematic,
        _ => return fallback.to_owned(),
    };
    if value == "follow_system" {
        return format!("Follow system ({})", resolved.unit.suffix());
    }
    if value == "automatic" {
        let decimals = match resolved.precision {
            eda_engine::ir::units::DisplayPrecision::DecimalPlaces(places) => places,
            eda_engine::ir::units::DisplayPrecision::ExactNanometer => return fallback.to_owned(),
        };
        let resolution = if decimals == 0 {
            "1".to_owned()
        } else {
            format!("0.{}1", "0".repeat(decimals.saturating_sub(1) as usize))
        };
        return format!("Automatic ({resolution} {})", resolved.unit.suffix());
    }
    fallback.to_owned()
}

pub(super) fn bool_consumer_value(
    surface: &[PreferenceSurfaceEntry],
    rows: &[GlobalPreferenceRow],
    consumer: PreferenceLiveConsumer,
) -> bool {
    surface
        .iter()
        .zip(rows)
        .find(|(entry, _)| entry.live_consumer == consumer)
        .map(|(_, row)| row)
        .and_then(|row| row.effective_value.as_ref())
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

pub(super) fn apply_live_consumers(
    ui: &mut WorkspaceUiState,
    surface: &[PreferenceSurfaceEntry],
    rows: &[GlobalPreferenceRow],
    terminal_theme_before_high_contrast: &mut Option<TerminalTheme>,
) {
    let duration = surface
        .iter()
        .zip(rows)
        .find(|(entry, _)| entry.live_consumer == PreferenceLiveConsumer::ConsoleFeedbackTimer)
        .map(|(_, row)| row)
        .and_then(|row| row.effective_value.as_ref())
        .and_then(Value::as_str)
        .and_then(|value| match value {
            "4s" => Some(ConsoleFeedbackDuration::FourSeconds),
            "6s" => Some(ConsoleFeedbackDuration::SixSeconds),
            "10s" => Some(ConsoleFeedbackDuration::TenSeconds),
            "never" => Some(ConsoleFeedbackDuration::Never),
            _ => None,
        })
        .expect("validated Console duration has a total consumer mapping");
    ui.console.set_duration_preference(duration);
    let high_contrast =
        bool_consumer_value(surface, rows, PreferenceLiveConsumer::HighContrastNonColor);
    if high_contrast {
        if terminal_theme_before_high_contrast.is_none() {
            *terminal_theme_before_high_contrast = Some(ui.terminal.theme);
        }
        ui.terminal.theme = TerminalTheme::HighContrast;
    } else if let Some(previous) = terminal_theme_before_high_contrast.take() {
        ui.terminal.theme = previous;
    }
}
