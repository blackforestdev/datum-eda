//! Trusted GUI transport facts for the engine-owned Preferences product service.

use std::path::PathBuf;

use eda_engine::preferences::{
    HeadExpectationV1, PreferenceActorKindV1, PreferenceActorV1, PreferenceServiceStatus,
    new_product_id,
};

pub(super) fn human_gui_actor() -> PreferenceActorV1 {
    PreferenceActorV1 {
        kind: PreferenceActorKindV1::HumanGui,
        session_id: format!("datum-gui-{}", std::process::id()),
        local_actor_id: std::env::var("USER").unwrap_or_else(|_| "unavailable".to_owned()),
        invocation_id: new_product_id(),
    }
}

pub(super) fn head_expectation(status: &PreferenceServiceStatus) -> HeadExpectationV1 {
    status
        .generation()
        .cloned()
        .map_or(HeadExpectationV1::Missing, HeadExpectationV1::Generation)
}

pub(super) fn platform_config_root() -> Option<PathBuf> {
    std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
        .map(|base| base.join("datum"))
}
