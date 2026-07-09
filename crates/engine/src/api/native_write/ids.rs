//! Deterministic object-id derivation conventions for native writes.
//!
//! Native authoring derives stable object ids with `Uuid::new_v5`, namespaced
//! by the project id, over a seed string of the form
//! `datum-eda:<namespace-tag>:<seed-part>:<seed-part>...`. The seed format is
//! a persistence-visible contract: ids derived here land in journal records
//! and shards, so the byte-exact seed layout must never drift.
//!
//! The component-instance convention is extracted from
//! `crates/cli/src/command_project_component_instances.rs` (the CLI callsite
//! stays in place until family A migrates onto this module; the tests below
//! prove byte-for-byte parity with the CLI's derivation).

use uuid::Uuid;

use crate::substrate::{ComponentInstanceId, ObjectId};

/// Seed prefix shared by every native-write id derivation.
const SEED_PREFIX: &str = "datum-eda";

/// Derive a deterministic object id namespaced by `project_id`.
///
/// The v5 seed is `datum-eda:<namespace_tag>:<seed_parts joined by ':'>`,
/// e.g. `derive_object_id(&pid, "sheet", &["Main".into()])` seeds
/// `datum-eda:sheet:Main`.
pub fn derive_object_id(project_id: &Uuid, namespace_tag: &str, seed_parts: &[String]) -> ObjectId {
    Uuid::new_v5(
        project_id,
        format!("{SEED_PREFIX}:{namespace_tag}:{}", seed_parts.join(":")).as_bytes(),
    )
}

/// Derive the deterministic id for a component instance binding the given
/// placed symbols to a placed package.
///
/// Matches the CLI derivation in
/// `crates/cli/src/command_project_component_instances.rs` exactly: the v5
/// seed is `datum-eda:component-instance:<symbol ids joined by '+'>:<package id>`
/// namespaced by the project id.
pub fn derive_component_instance_id(
    project_id: &Uuid,
    symbol_ids: &[Uuid],
    package_id: Uuid,
) -> ComponentInstanceId {
    derive_object_id(
        project_id,
        "component-instance",
        &[
            symbol_ids
                .iter()
                .map(Uuid::to_string)
                .collect::<Vec<_>>()
                .join("+"),
            package_id.to_string(),
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The exact derivation the CLI performs today
    /// (`crates/cli/src/command_project_component_instances.rs:57-70`),
    /// reproduced verbatim as the parity oracle.
    fn cli_component_instance_derivation(
        project_id: &Uuid,
        symbol_ids: &[Uuid],
        package_id: Uuid,
    ) -> Uuid {
        Uuid::new_v5(
            project_id,
            format!(
                "datum-eda:component-instance:{}:{package_id}",
                symbol_ids
                    .iter()
                    .map(Uuid::to_string)
                    .collect::<Vec<_>>()
                    .join("+")
            )
            .as_bytes(),
        )
    }

    #[test]
    fn component_instance_id_matches_cli_derivation_single_symbol() {
        let project_id = Uuid::new_v4();
        let symbol_ids = vec![Uuid::new_v4()];
        let package_id = Uuid::new_v4();
        assert_eq!(
            derive_component_instance_id(&project_id, &symbol_ids, package_id),
            cli_component_instance_derivation(&project_id, &symbol_ids, package_id),
        );
    }

    #[test]
    fn component_instance_id_matches_cli_derivation_multi_symbol() {
        let project_id = Uuid::new_v4();
        let symbol_ids = vec![Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()];
        let package_id = Uuid::new_v4();
        assert_eq!(
            derive_component_instance_id(&project_id, &symbol_ids, package_id),
            cli_component_instance_derivation(&project_id, &symbol_ids, package_id),
        );
    }

    #[test]
    fn component_instance_id_matches_cli_derivation_no_symbols() {
        let project_id = Uuid::new_v4();
        let package_id = Uuid::new_v4();
        assert_eq!(
            derive_component_instance_id(&project_id, &[], package_id),
            cli_component_instance_derivation(&project_id, &[], package_id),
        );
    }

    #[test]
    fn derive_object_id_uses_documented_seed_layout() {
        let project_id = Uuid::new_v4();
        let derived = derive_object_id(
            &project_id,
            "example",
            &["alpha".to_string(), "beta".to_string()],
        );
        assert_eq!(
            derived,
            Uuid::new_v5(&project_id, b"datum-eda:example:alpha:beta"),
        );
    }

    #[test]
    fn derive_object_id_is_project_namespaced() {
        let parts = vec!["alpha".to_string()];
        let a = derive_object_id(&Uuid::new_v4(), "example", &parts);
        let b = derive_object_id(&Uuid::new_v4(), "example", &parts);
        assert_ne!(a, b);
    }
}
