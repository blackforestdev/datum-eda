//! Engine-owned evaluation of registered runtime-default recipes.

use serde_json::{Value, json};

use super::{
    Contribution, DescriptorDefault, DescriptorRegistry, FactProvenance, PreferenceKey,
    RuntimeDefaultFact, ValueDisclosure,
};

pub(super) fn runtime_default_contribution(
    registry: &DescriptorRegistry,
    key: &PreferenceKey,
) -> Option<Contribution> {
    let descriptor = registry.get(key)?;
    let DescriptorDefault::Runtime { recipe } = &descriptor.default_value else {
        return None;
    };
    let value = evaluate(recipe)?;
    Some(Contribution::RuntimeDefault(RuntimeDefaultFact {
        id: format!("runtime-default:{}", key.as_str()),
        key: key.clone(),
        recipe: recipe.clone(),
        value,
        provenance: FactProvenance {
            origin: descriptor.owner.clone(),
            provider: Some("Datum runtime-default evaluator".to_owned()),
            package: None,
            generation: Some("1".to_owned()),
            actor: "runtime-default evaluator".to_owned(),
            role: Some("subsystem owner".to_owned()),
            observed_at: "application open".to_owned(),
            effective_from: None,
            effective_until: None,
            offline_valid_until: None,
            last_successful_contact: None,
            reason: format!("evaluated registered recipe {recipe}"),
        },
        disclosure: ValueDisclosure::Disclosed,
    }))
}

fn evaluate(recipe: &str) -> Option<Value> {
    match recipe {
        "shared_viewport.grid_mark_style.v1" => {
            Some(json!({"shape":"dot","size":1,"min_spacing_px":8}))
        }
        "input.editor_keymap.datum_v1" | "terminal.keymap.datum_v1" => {
            Some(json!({"version":1,"bindings":{}}))
        }
        "datum_palette.airwire.v1" => Some(json!("datum_airwire")),
        "platform.default_shell_profile.v1" => {
            let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_owned());
            Some(json!([{
                "name":"Default shell",
                "executable":shell,
                "argv":[],
                "cwd":".",
                "environment":{}
            }]))
        }
        "platform.documents_datum_locations.v1" => {
            let root = std::env::var("HOME")
                .map(|home| format!("{home}/Documents/Datum"))
                .unwrap_or_else(|_| "Documents/Datum".to_owned());
            Some(json!({
                "projects":format!("{root}/Projects"),
                "libraries":format!("{root}/Libraries"),
                "output_jobs":format!("{root}/Output Jobs"),
                "new_project":format!("{root}/Projects")
            }))
        }
        "document_persistence.autosave_defaults.v1" => Some(json!({
            "interval":"10m",
            "retained_versions":10,
            "backup_age":"30d",
            "recovery":true,
            "reminder":true
        })),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::preferences::{ResolutionOutcome, active_v1_registry, resolve_preference};

    #[test]
    fn every_registered_runtime_recipe_produces_a_valid_effective_value() {
        let registry = active_v1_registry();
        for key in registry.keys() {
            let descriptor = registry.get(key).unwrap();
            if !matches!(descriptor.default_value, DescriptorDefault::Runtime { .. }) {
                continue;
            }
            let contribution = runtime_default_contribution(&registry, key)
                .expect("every registered runtime recipe has one evaluator");
            let explanation = resolve_preference(super::super::ResolutionRequest {
                registry: &registry,
                key: key.clone(),
                contributions: &[contribution],
                authority_releases: &[],
                machine_scope: "Global · this device",
            });
            assert!(
                matches!(explanation.outcome, ResolutionOutcome::Effective { .. }),
                "runtime default failed for {}",
                key.as_str()
            );
        }
    }
}
