//! Schematic symbol place/set/delete builders for the native write facade.
//!
//! Family C of the native-write migration: all symbol operation authoring
//! from `crates/cli/src/command_project_schematic_symbol_mutations.rs` and
//! the symbol half of `crates/cli/src/command_project_schematic_proposals.rs`
//! lives here. Placing a pool-bound symbol atomically authors the
//! `CreateSchematicSymbol` operation plus the component-instance binding op
//! contributed by
//! [`super::component_instances::build_placed_symbol_component_instance_op`],
//! so direct commits and draft proposals share one composition.
//!
//! Builders are build-only; id derivation for the placed-symbol component
//! instance is the byte-exact historical CLI convention (see
//! [`placed_symbol_component_instance_id`]).

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::EngineError;
use crate::schematic::PlacedSymbol;
use crate::substrate::{DesignModel, ObjectId, Operation};

use super::component_instances::build_placed_symbol_component_instance_op;
use super::context::{BatchComposer, PreparedWrite, WriteProvenance};
use super::ids;

/// Pool binding for a freshly placed symbol: the pool symbol it was
/// materialized from and the resolved pool part to bind a component instance
/// to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlacedSymbolPartBinding {
    /// Pool symbol the placed symbol was materialized from.
    pub pool_symbol_id: Uuid,
    /// Resolved pool part the component instance binds.
    pub part_id: Uuid,
}

/// Deterministic component-instance id for a placed pool symbol.
///
/// Seed layout (persistence-visible, must never drift):
/// `datum-eda:component-instance:schematic:<pool_symbol_id>:<placed_symbol_id>`
/// namespaced by the project id — byte-identical to the id authored by
/// [`build_placed_symbol_component_instance_op`], so read-only reporting can
/// name the instance a placement (or placement proposal) will create.
pub fn placed_symbol_component_instance_id(
    project_id: &Uuid,
    pool_symbol_id: Uuid,
    placed_symbol_id: Uuid,
) -> Uuid {
    ids::derive_object_id(
        project_id,
        "component-instance",
        &[
            "schematic".to_string(),
            pool_symbol_id.to_string(),
            placed_symbol_id.to_string(),
        ],
    )
}

/// Build the batch that places a schematic symbol on `sheet_id`.
///
/// When `binding` is present the batch also creates the component instance
/// binding the placed symbol to its resolved pool part (one atomic batch,
/// matching the CLI's historical composition).
pub fn build_place_schematic_symbol(
    model: &DesignModel,
    provenance: WriteProvenance,
    sheet_id: ObjectId,
    symbol: &PlacedSymbol,
    binding: Option<&PlacedSymbolPartBinding>,
) -> Result<PreparedWrite, EngineError> {
    let mut operations = vec![Operation::CreateSchematicSymbol {
        sheet_id,
        symbol_id: symbol.uuid,
        symbol: serde_json::to_value(symbol)?,
    }];
    if let Some(binding) = binding {
        operations.push(build_placed_symbol_component_instance_op(
            model,
            symbol.uuid,
            binding.pool_symbol_id,
            binding.part_id,
        )?);
    }
    BatchComposer::compose(model, provenance)
        .push_ops(operations)
        .primary_object(symbol.uuid)
        .finish()
}

/// Build the batch that rewrites an existing schematic symbol (revision
/// guard stamped automatically).
pub fn build_set_schematic_symbol(
    model: &DesignModel,
    provenance: WriteProvenance,
    sheet_id: ObjectId,
    symbol: &PlacedSymbol,
) -> Result<PreparedWrite, EngineError> {
    BatchComposer::compose(model, provenance)
        .push_op(Operation::SetSchematicSymbol {
            sheet_id,
            symbol_id: symbol.uuid,
            symbol: serde_json::to_value(symbol)?,
        })
        .primary_object(symbol.uuid)
        .finish()
}

/// Build the batch that deletes an existing schematic symbol (revision guard
/// stamped automatically).
pub fn build_delete_schematic_symbol(
    model: &DesignModel,
    provenance: WriteProvenance,
    sheet_id: ObjectId,
    symbol: &PlacedSymbol,
) -> Result<PreparedWrite, EngineError> {
    BatchComposer::compose(model, provenance)
        .push_op(Operation::DeleteSchematicSymbol {
            sheet_id,
            symbol_id: symbol.uuid,
            symbol: serde_json::to_value(symbol)?,
        })
        .primary_object(symbol.uuid)
        .finish()
}

#[cfg(test)]
mod tests {
    use super::super::context::commit_prepared;
    use super::super::test_support::resolved_model_with_board_package;
    use super::*;
    use crate::ir::geometry::Point;
    use crate::schematic::{HiddenPowerBehavior, SymbolDisplayMode};
    use crate::substrate::{CommitSource, ObjectRevision, ProjectResolver};

    fn test_provenance(reason: &str) -> WriteProvenance {
        WriteProvenance::new("unit-test", CommitSource::Test, reason)
    }

    fn fixture_sheet_id(model: &DesignModel) -> Uuid {
        Uuid::new_v5(&model.project.project_id, b"sheet")
    }

    fn fixture_symbol() -> PlacedSymbol {
        PlacedSymbol {
            uuid: Uuid::new_v4(),
            part: None,
            entity: None,
            gate: None,
            lib_id: Some("test:R".to_string()),
            reference: "R1".to_string(),
            value: "10k".to_string(),
            fields: Vec::new(),
            pins: Vec::new(),
            position: Point { x: 100, y: 200 },
            rotation: 0,
            mirrored: false,
            unit_selection: None,
            display_mode: SymbolDisplayMode::LibraryDefault,
            pin_overrides: Vec::new(),
            hidden_power_behavior: HiddenPowerBehavior::SourceDefinedImplicit,
        }
    }

    #[test]
    fn place_without_binding_authors_single_create() {
        let (_root, model, _board_id, _package_id) =
            resolved_model_with_board_package("schematic_symbols_place");
        let sheet_id = fixture_sheet_id(&model);
        let symbol = fixture_symbol();

        let prepared = build_place_schematic_symbol(
            &model,
            test_provenance("place schematic symbol"),
            sheet_id,
            &symbol,
            None,
        )
        .expect("place should build");

        assert_eq!(prepared.primary_object_id, Some(symbol.uuid));
        assert_eq!(
            prepared.batch.expected_model_revision,
            Some(model.model_revision.clone())
        );
        assert_eq!(
            prepared.batch.operations,
            vec![Operation::CreateSchematicSymbol {
                sheet_id,
                symbol_id: symbol.uuid,
                symbol: serde_json::to_value(&symbol).expect("symbol should serialize"),
            }]
        );
    }

    #[test]
    fn place_with_binding_composes_component_instance_op() {
        let (_root, model, _board_id, package_id) =
            resolved_model_with_board_package("schematic_symbols_place_binding");
        let sheet_id = fixture_sheet_id(&model);
        let symbol = fixture_symbol();
        // The fixture board package doubles as the resolvable pool-part
        // target object.
        let binding = PlacedSymbolPartBinding {
            pool_symbol_id: Uuid::new_v4(),
            part_id: package_id,
        };

        let prepared = build_place_schematic_symbol(
            &model,
            test_provenance("place schematic symbol"),
            sheet_id,
            &symbol,
            Some(&binding),
        )
        .expect("place should build");

        assert_eq!(prepared.batch.operations.len(), 2);
        assert!(matches!(
            &prepared.batch.operations[0],
            Operation::CreateSchematicSymbol { symbol_id, .. } if *symbol_id == symbol.uuid
        ));
        let Operation::CreateComponentInstance {
            component_instance_id,
            ..
        } = &prepared.batch.operations[1]
        else {
            panic!("expected CreateComponentInstance as second op");
        };
        assert_eq!(
            *component_instance_id,
            placed_symbol_component_instance_id(
                &model.project.project_id,
                binding.pool_symbol_id,
                symbol.uuid,
            )
        );
    }

    #[test]
    fn placed_symbol_component_instance_id_matches_historical_derivation() {
        let project_id = Uuid::new_v4();
        let pool_symbol_id = Uuid::new_v4();
        let placed_symbol_id = Uuid::new_v4();
        // Byte-exact historical CLI derivation
        // (command_project_schematic_symbol_reports.rs).
        let expected = Uuid::new_v5(
            &project_id,
            format!("datum-eda:component-instance:schematic:{pool_symbol_id}:{placed_symbol_id}")
                .as_bytes(),
        );
        assert_eq!(
            placed_symbol_component_instance_id(&project_id, pool_symbol_id, placed_symbol_id),
            expected,
        );
    }

    #[test]
    fn placed_symbol_component_instance_id_matches_authored_op_id() {
        let (_root, model, _board_id, package_id) =
            resolved_model_with_board_package("schematic_symbols_id_parity");
        let pool_symbol_id = Uuid::new_v4();
        let placed_symbol_id = Uuid::new_v4();

        let operation = build_placed_symbol_component_instance_op(
            &model,
            placed_symbol_id,
            pool_symbol_id,
            package_id,
        )
        .expect("component-instance op should build");
        let Operation::CreateComponentInstance {
            component_instance_id,
            ..
        } = &operation
        else {
            panic!("expected CreateComponentInstance");
        };
        assert_eq!(
            *component_instance_id,
            placed_symbol_component_instance_id(
                &model.project.project_id,
                pool_symbol_id,
                placed_symbol_id,
            )
        );
    }

    #[test]
    fn set_and_delete_guard_the_symbol_object() {
        let (root, mut model, _board_id, _package_id) =
            resolved_model_with_board_package("schematic_symbols_set");
        let sheet_id = fixture_sheet_id(&model);
        let mut symbol = fixture_symbol();
        let prepared = build_place_schematic_symbol(
            &model,
            test_provenance("place schematic symbol"),
            sheet_id,
            &symbol,
            None,
        )
        .expect("place should build");
        commit_prepared(&mut model, &root, prepared).expect("place should commit");
        let model = ProjectResolver::new(&root)
            .resolve()
            .expect("project should re-resolve");

        symbol.value = "22k".to_string();
        let prepared = build_set_schematic_symbol(
            &model,
            test_provenance("set schematic symbol value"),
            sheet_id,
            &symbol,
        )
        .expect("set should build");
        assert_eq!(
            prepared.batch.operations[0],
            Operation::GuardObjectRevision {
                object_id: symbol.uuid,
                expected_object_revision: ObjectRevision(0),
            }
        );
        assert!(matches!(
            &prepared.batch.operations[1],
            Operation::SetSchematicSymbol { symbol_id, .. } if *symbol_id == symbol.uuid
        ));

        let prepared = build_delete_schematic_symbol(
            &model,
            test_provenance("delete schematic symbol"),
            sheet_id,
            &symbol,
        )
        .expect("delete should build");
        assert!(matches!(
            prepared.batch.operations[0],
            Operation::GuardObjectRevision { object_id, .. } if object_id == symbol.uuid
        ));
        assert!(matches!(
            &prepared.batch.operations[1],
            Operation::DeleteSchematicSymbol { symbol_id, .. } if *symbol_id == symbol.uuid
        ));
    }
}
