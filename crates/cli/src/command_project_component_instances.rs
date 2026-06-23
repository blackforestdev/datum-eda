use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use eda_engine::substrate::{
    CommitProvenance, CommitSource, ComponentInstance, ObjectId, Operation, OperationBatch,
    ProjectResolver,
};
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectComponentInstancesView {
    pub(crate) contract: &'static str,
    pub(crate) project_id: String,
    pub(crate) model_revision: String,
    pub(crate) component_instance_count: usize,
    pub(crate) component_instances: BTreeMap<Uuid, ComponentInstance>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectComponentInstanceMutationView {
    pub(crate) contract: &'static str,
    pub(crate) action: &'static str,
    pub(crate) project_id: String,
    pub(crate) component_instance: ComponentInstance,
    pub(crate) component_instance_path: String,
    pub(crate) transaction_id: String,
}

pub(crate) fn query_native_project_component_instances(
    root: &Path,
) -> Result<NativeProjectComponentInstancesView> {
    let model = ProjectResolver::new(root).resolve()?;
    Ok(NativeProjectComponentInstancesView {
        contract: "component_instances_query_v1",
        project_id: model.project.project_id.to_string(),
        model_revision: model.model_revision.0,
        component_instance_count: model.component_instances.len(),
        component_instances: model.component_instances,
    })
}

pub(crate) fn bind_native_project_component_instance(
    root: &Path,
    component_instance_id: Option<Uuid>,
    symbol_id: Uuid,
    package_id: Uuid,
) -> Result<NativeProjectComponentInstanceMutationView> {
    let mut model = ProjectResolver::new(root).resolve()?;
    let component_instance_id = component_instance_id.unwrap_or_else(|| {
        Uuid::new_v5(
            &model.project.project_id,
            format!("datum-eda:component-instance:{symbol_id}:{package_id}").as_bytes(),
        )
    });
    let payload =
        component_instance_payload(&model, component_instance_id, 0, symbol_id, package_id)?;
    let expected_model_revision = model.model_revision.clone();
    let report = model.commit_journaled(
        root,
        OperationBatch {
            batch_id: Uuid::new_v4(),
            expected_model_revision: Some(expected_model_revision),
            provenance: CommitProvenance {
                actor: "datum-eda-cli".to_string(),
                source: CommitSource::Cli,
                reason: "bind component instance".to_string(),
            },
            operations: vec![Operation::CreateComponentInstance {
                component_instance_id,
                component_instance: payload,
            }],
        },
    )?;
    let instance = model
        .component_instances
        .get(&component_instance_id)
        .cloned()
        .with_context(|| format!("component instance {component_instance_id} was not committed"))?;
    Ok(component_instance_mutation(
        "bind_component_instance",
        model.project.project_id,
        instance,
        component_instance_path(root, component_instance_id),
        report.transaction.transaction_id,
    ))
}

pub(crate) fn set_native_project_component_instance(
    root: &Path,
    component_instance_id: Uuid,
    symbol_id: Uuid,
    package_id: Uuid,
) -> Result<NativeProjectComponentInstanceMutationView> {
    let mut model = ProjectResolver::new(root).resolve()?;
    let previous = model
        .component_instances
        .get(&component_instance_id)
        .cloned()
        .with_context(|| format!("component instance {component_instance_id} was not found"))?;
    let previous_payload = component_instance_payload_from_instance(&model, &previous)?;
    let next_revision = previous.object_revision.0 + 1;
    let payload = component_instance_payload(
        &model,
        component_instance_id,
        next_revision,
        symbol_id,
        package_id,
    )?;
    let expected_model_revision = model.model_revision.clone();
    let report = model.commit_journaled(
        root,
        OperationBatch {
            batch_id: Uuid::new_v4(),
            expected_model_revision: Some(expected_model_revision),
            provenance: CommitProvenance {
                actor: "datum-eda-cli".to_string(),
                source: CommitSource::Cli,
                reason: "set component instance".to_string(),
            },
            operations: vec![Operation::SetComponentInstance {
                component_instance_id,
                previous_component_instance: previous_payload,
                component_instance: payload,
            }],
        },
    )?;
    let instance = model
        .component_instances
        .get(&component_instance_id)
        .cloned()
        .with_context(|| format!("component instance {component_instance_id} was not committed"))?;
    Ok(component_instance_mutation(
        "set_component_instance",
        model.project.project_id,
        instance,
        component_instance_path(root, component_instance_id),
        report.transaction.transaction_id,
    ))
}

pub(crate) fn delete_native_project_component_instance(
    root: &Path,
    component_instance_id: Uuid,
) -> Result<NativeProjectComponentInstanceMutationView> {
    let mut model = ProjectResolver::new(root).resolve()?;
    let previous = model
        .component_instances
        .get(&component_instance_id)
        .cloned()
        .with_context(|| format!("component instance {component_instance_id} was not found"))?;
    let previous_payload = component_instance_payload_from_instance(&model, &previous)?;
    let expected_model_revision = model.model_revision.clone();
    let report = model.commit_journaled(
        root,
        OperationBatch {
            batch_id: Uuid::new_v4(),
            expected_model_revision: Some(expected_model_revision),
            provenance: CommitProvenance {
                actor: "datum-eda-cli".to_string(),
                source: CommitSource::Cli,
                reason: "delete component instance".to_string(),
            },
            operations: vec![Operation::DeleteComponentInstance {
                component_instance_id,
                component_instance: previous_payload,
            }],
        },
    )?;
    Ok(component_instance_mutation(
        "delete_component_instance",
        model.project.project_id,
        previous,
        component_instance_path(root, component_instance_id),
        report.transaction.transaction_id,
    ))
}

fn component_instance_payload_from_instance(
    model: &eda_engine::substrate::DesignModel,
    instance: &ComponentInstance,
) -> Result<serde_json::Value> {
    let symbol_id = instance
        .placed_symbol_refs
        .first()
        .copied()
        .context("component instance has no placed symbol refs")?;
    let package_id = instance
        .placed_package_refs
        .first()
        .copied()
        .context("component instance has no placed package refs")?;
    component_instance_payload(
        model,
        instance.id,
        instance.object_revision.0,
        symbol_id,
        package_id,
    )
}

fn component_instance_payload(
    model: &eda_engine::substrate::DesignModel,
    component_instance_id: Uuid,
    object_revision: u64,
    symbol_id: Uuid,
    package_id: Uuid,
) -> Result<serde_json::Value> {
    Ok(serde_json::json!({
        "uuid": component_instance_id,
        "object_revision": object_revision,
        "placed_symbol_refs": [revisioned_ref(model, symbol_id)?],
        "placed_package_refs": [revisioned_ref(model, package_id)?]
    }))
}

fn revisioned_ref(
    model: &eda_engine::substrate::DesignModel,
    object_id: ObjectId,
) -> Result<serde_json::Value> {
    let object = model
        .objects
        .get(&object_id)
        .with_context(|| format!("component instance target object {object_id} was not found"))?;
    Ok(serde_json::json!({
        "object_id": object_id,
        "object_revision": object.object_revision.0
    }))
}

fn component_instance_path(root: &Path, component_instance_id: Uuid) -> PathBuf {
    root.join(".datum/component_instances")
        .join(format!("{component_instance_id}.json"))
}

fn component_instance_mutation(
    action: &'static str,
    project_id: Uuid,
    component_instance: ComponentInstance,
    component_instance_path: PathBuf,
    transaction_id: Uuid,
) -> NativeProjectComponentInstanceMutationView {
    NativeProjectComponentInstanceMutationView {
        contract: "component_instance_mutation_v1",
        action,
        project_id: project_id.to_string(),
        component_instance,
        component_instance_path: component_instance_path.display().to_string(),
        transaction_id: transaction_id.to_string(),
    }
}
