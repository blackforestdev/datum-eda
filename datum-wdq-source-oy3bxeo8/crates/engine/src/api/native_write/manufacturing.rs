//! Manufacturing-plan and panel-projection write builders.
//!
//! Migrated from `crates/cli/src/command_project_manufacturing_plans.rs` and
//! the builder halves of
//! `crates/cli/src/command_project_manufacturing_plan_proposals.rs`. Builders
//! are build-only ([`PreparedWrite`] out, never committed here) so the CLI's
//! direct-commit and draft-proposal paths share the exact same authoring.

use uuid::Uuid;

use crate::error::EngineError;
use crate::substrate::{DesignModel, ManufacturingPlan, Operation, PanelProjection};

use super::context::{BatchComposer, PreparedWrite, WriteProvenance};
use super::ids::derive_object_id;

/// Deterministic manufacturing-plan id for `prefix`, namespaced by the
/// project id (v5 seed `datum-eda:manufacturing-plan:<prefix>`).
pub fn derive_manufacturing_plan_id(project_id: &Uuid, prefix: &str) -> Uuid {
    derive_object_id(project_id, "manufacturing-plan", &[prefix.to_string()])
}

/// Deterministic panel-projection id for `key`, namespaced by the project id
/// (v5 seed `datum-eda:panel-projection:<key>`).
pub fn derive_panel_projection_id(project_id: &Uuid, key: &str) -> Uuid {
    derive_object_id(project_id, "panel-projection", &[key.to_string()])
}

/// Build a `CreateManufacturingPlan` write for a fully formed plan.
pub fn build_create_manufacturing_plan(
    model: &DesignModel,
    provenance: WriteProvenance,
    manufacturing_plan: &ManufacturingPlan,
) -> Result<PreparedWrite, EngineError> {
    BatchComposer::compose(model, provenance)
        .push_op(Operation::CreateManufacturingPlan {
            manufacturing_plan_id: manufacturing_plan.id,
            manufacturing_plan: serde_json::to_value(manufacturing_plan)?,
        })
        .primary_object(manufacturing_plan.id)
        .finish()
}

/// Build a `SetManufacturingPlan` write replacing `previous_manufacturing_plan`
/// with `manufacturing_plan` (revision guard stamped automatically).
pub fn build_set_manufacturing_plan(
    model: &DesignModel,
    provenance: WriteProvenance,
    previous_manufacturing_plan: &ManufacturingPlan,
    manufacturing_plan: &ManufacturingPlan,
) -> Result<PreparedWrite, EngineError> {
    BatchComposer::compose(model, provenance)
        .push_op(Operation::SetManufacturingPlan {
            manufacturing_plan_id: manufacturing_plan.id,
            previous_manufacturing_plan: serde_json::to_value(previous_manufacturing_plan)?,
            manufacturing_plan: serde_json::to_value(manufacturing_plan)?,
        })
        .primary_object(manufacturing_plan.id)
        .finish()
}

/// Build a `DeleteManufacturingPlan` write for the plan currently in the model
/// (revision guard stamped automatically).
pub fn build_delete_manufacturing_plan(
    model: &DesignModel,
    provenance: WriteProvenance,
    manufacturing_plan: &ManufacturingPlan,
) -> Result<PreparedWrite, EngineError> {
    BatchComposer::compose(model, provenance)
        .push_op(Operation::DeleteManufacturingPlan {
            manufacturing_plan_id: manufacturing_plan.id,
            manufacturing_plan: serde_json::to_value(manufacturing_plan)?,
        })
        .primary_object(manufacturing_plan.id)
        .finish()
}

/// Build a `CreatePanelProjection` write for a fully formed projection.
pub fn build_create_panel_projection(
    model: &DesignModel,
    provenance: WriteProvenance,
    panel_projection: &PanelProjection,
) -> Result<PreparedWrite, EngineError> {
    BatchComposer::compose(model, provenance)
        .push_op(Operation::CreatePanelProjection {
            panel_projection_id: panel_projection.id,
            panel_projection: serde_json::to_value(panel_projection)?,
        })
        .primary_object(panel_projection.id)
        .finish()
}

/// Build a `SetPanelProjection` write replacing `previous_panel_projection`
/// with `panel_projection` (revision guard stamped automatically).
pub fn build_set_panel_projection(
    model: &DesignModel,
    provenance: WriteProvenance,
    previous_panel_projection: &PanelProjection,
    panel_projection: &PanelProjection,
) -> Result<PreparedWrite, EngineError> {
    BatchComposer::compose(model, provenance)
        .push_op(Operation::SetPanelProjection {
            panel_projection_id: panel_projection.id,
            previous_panel_projection: serde_json::to_value(previous_panel_projection)?,
            panel_projection: serde_json::to_value(panel_projection)?,
        })
        .primary_object(panel_projection.id)
        .finish()
}

/// Build a `DeletePanelProjection` write for the projection currently in the
/// model (revision guard stamped automatically).
pub fn build_delete_panel_projection(
    model: &DesignModel,
    provenance: WriteProvenance,
    panel_projection: &PanelProjection,
) -> Result<PreparedWrite, EngineError> {
    BatchComposer::compose(model, provenance)
        .push_op(Operation::DeletePanelProjection {
            panel_projection_id: panel_projection.id,
            panel_projection: serde_json::to_value(panel_projection)?,
        })
        .primary_object(panel_projection.id)
        .finish()
}

#[cfg(test)]
mod tests {
    use super::super::context::commit_prepared;
    use super::super::test_support::resolved_model_with_board_package;
    use super::*;
    use crate::substrate::{
        CommitSource, ObjectRevision, PRODUCTION_RECORD_SCHEMA_VERSION, ProjectResolver,
    };

    fn test_provenance(reason: &str) -> WriteProvenance {
        WriteProvenance::new("unit-test", CommitSource::Test, reason)
    }

    fn test_plan(project_id: &Uuid, board_id: Uuid) -> ManufacturingPlan {
        let plan_id = derive_manufacturing_plan_id(project_id, "rev-a");
        ManufacturingPlan {
            schema_version: PRODUCTION_RECORD_SCHEMA_VERSION,
            id: plan_id,
            name: "Manufacturing plan rev-a".to_string(),
            board_or_panel: board_id,
            variant: None,
            prefix: "rev-a".to_string(),
            object_revision: ObjectRevision(0),
        }
    }

    #[test]
    fn derive_ids_match_cli_seed_layout() {
        let project_id = Uuid::new_v4();
        assert_eq!(
            derive_manufacturing_plan_id(&project_id, "rev-a"),
            Uuid::new_v5(&project_id, b"datum-eda:manufacturing-plan:rev-a"),
        );
        assert_eq!(
            derive_panel_projection_id(&project_id, "panel-2x2"),
            Uuid::new_v5(&project_id, b"datum-eda:panel-projection:panel-2x2"),
        );
    }

    #[test]
    fn create_plan_builds_unguarded_single_op_batch() {
        let (_root, model, board_id, _package_id) =
            resolved_model_with_board_package("mfg_create_build");
        let plan = test_plan(&model.project.project_id, board_id);

        let prepared = build_create_manufacturing_plan(
            &model,
            test_provenance("create manufacturing plan"),
            &plan,
        )
        .expect("create plan should build");

        assert_eq!(prepared.primary_object_id, Some(plan.id));
        assert_eq!(
            prepared.batch.expected_model_revision,
            Some(model.model_revision.clone())
        );
        assert_eq!(
            prepared.batch.operations,
            vec![Operation::CreateManufacturingPlan {
                manufacturing_plan_id: plan.id,
                manufacturing_plan: serde_json::to_value(&plan).unwrap(),
            }]
        );
    }

    #[test]
    fn plan_create_set_delete_round_trip_through_commit() {
        let (root, mut model, board_id, _package_id) =
            resolved_model_with_board_package("mfg_round_trip");
        let plan = test_plan(&model.project.project_id, board_id);

        let prepared = build_create_manufacturing_plan(
            &model,
            test_provenance("create manufacturing plan"),
            &plan,
        )
        .expect("create plan should build");
        commit_prepared(&mut model, &root, prepared).expect("create plan should commit");
        assert!(model.manufacturing_plans.contains_key(&plan.id));

        // Update: guard must precede the Set op with the live object revision.
        let mut updated = plan.clone();
        updated.name = "Renamed plan".to_string();
        updated.object_revision = ObjectRevision(plan.object_revision.0 + 1);
        let prepared = build_set_manufacturing_plan(
            &model,
            test_provenance("update manufacturing plan"),
            &plan,
            &updated,
        )
        .expect("set plan should build");
        assert_eq!(prepared.batch.operations.len(), 2);
        assert_eq!(
            prepared.batch.operations[0],
            Operation::GuardObjectRevision {
                object_id: plan.id,
                expected_object_revision: model.objects[&plan.id].object_revision,
            }
        );
        commit_prepared(&mut model, &root, prepared).expect("set plan should commit");
        assert_eq!(model.manufacturing_plans[&plan.id].name, "Renamed plan");

        // The committed state must survive a fresh resolve (real shards).
        let reloaded = ProjectResolver::new(&root)
            .resolve()
            .expect("project should re-resolve");
        assert_eq!(reloaded.manufacturing_plans[&plan.id].name, "Renamed plan");

        let live = model.manufacturing_plans[&plan.id].clone();
        let prepared = build_delete_manufacturing_plan(
            &model,
            test_provenance("delete manufacturing plan"),
            &live,
        )
        .expect("delete plan should build");
        assert!(matches!(
            prepared.batch.operations[0],
            Operation::GuardObjectRevision { object_id, .. } if object_id == plan.id
        ));
        commit_prepared(&mut model, &root, prepared).expect("delete plan should commit");
        assert!(!model.manufacturing_plans.contains_key(&plan.id));
    }

    #[test]
    fn panel_projection_create_and_set_build_expected_operations() {
        let (root, mut model, board_id, _package_id) =
            resolved_model_with_board_package("panel_build");
        let panel_id = derive_panel_projection_id(&model.project.project_id, "panel-2x2");
        let panel = PanelProjection {
            schema_version: PRODUCTION_RECORD_SCHEMA_VERSION,
            id: panel_id,
            name: "Panel projection panel-2x2".to_string(),
            board_instances: vec![crate::substrate::PanelBoardInstance {
                board: board_id,
                x_nm: 0,
                y_nm: 0,
                rotation_deg: 0,
            }],
            object_revision: ObjectRevision(0),
        };

        let prepared = build_create_panel_projection(
            &model,
            test_provenance("create panel projection"),
            &panel,
        )
        .expect("create panel should build");
        assert_eq!(
            prepared.batch.operations,
            vec![Operation::CreatePanelProjection {
                panel_projection_id: panel_id,
                panel_projection: serde_json::to_value(&panel).unwrap(),
            }]
        );
        commit_prepared(&mut model, &root, prepared).expect("create panel should commit");

        let mut updated = panel.clone();
        updated.name = "Renamed panel".to_string();
        updated.object_revision = ObjectRevision(panel.object_revision.0 + 1);
        let prepared = build_set_panel_projection(
            &model,
            test_provenance("update panel projection"),
            &panel,
            &updated,
        )
        .expect("set panel should build");
        assert!(matches!(
            prepared.batch.operations[0],
            Operation::GuardObjectRevision { object_id, .. } if object_id == panel_id
        ));
        assert!(matches!(
            &prepared.batch.operations[1],
            Operation::SetPanelProjection { panel_projection_id, .. } if *panel_projection_id == panel_id
        ));

        let live = model.panel_projections[&panel_id].clone();
        let prepared = build_delete_panel_projection(
            &model,
            test_provenance("delete panel projection"),
            &live,
        )
        .expect("delete panel should build");
        commit_prepared(&mut model, &root, prepared).expect("delete panel should commit");
        assert!(!model.panel_projections.contains_key(&panel_id));
    }
}
