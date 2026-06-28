use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use eda_engine::substrate::{
    CommitProvenance, CommitSource, ComponentInstance, ComponentInstanceAuthority, ObjectId,
    Operation, OperationBatch, ProjectResolver,
};
use serde::Serialize;
use uuid::Uuid;

use super::command_project_operation_guards::guarded_existing_object_operation;

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
    let component_instances = authored_component_instances(&model.component_instances);
    Ok(NativeProjectComponentInstancesView {
        contract: "component_instances_query_v1",
        project_id: model.project.project_id.to_string(),
        model_revision: model.model_revision.0,
        component_instance_count: component_instances.len(),
        component_instances,
    })
}

pub(crate) fn bind_native_project_component_instance(
    root: &Path,
    component_instance_id: Option<Uuid>,
    symbol_ids: Vec<Uuid>,
    package_id: Uuid,
    part_id: Option<Uuid>,
    symbol_roles: Vec<String>,
    package_roles: Vec<String>,
) -> Result<NativeProjectComponentInstanceMutationView> {
    let mut model = ProjectResolver::new(root).resolve()?;
    let component_instance_id = component_instance_id.unwrap_or_else(|| {
        Uuid::new_v5(
            &model.project.project_id,
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
    });
    let package_ids = vec![package_id];
    let payload = component_instance_payload(
        &model,
        component_instance_id,
        0,
        part_id,
        &symbol_ids,
        &package_ids,
        &symbol_roles,
        &package_roles,
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
    symbol_ids: Vec<Uuid>,
    package_id: Uuid,
    part_id: Option<Uuid>,
    symbol_roles: Vec<String>,
    package_roles: Vec<String>,
) -> Result<NativeProjectComponentInstanceMutationView> {
    let mut model = ProjectResolver::new(root).resolve()?;
    let previous = authored_component_instance(&model, component_instance_id)?;
    let previous_payload = component_instance_payload_from_instance(&model, &previous)?;
    let next_revision = previous.object_revision.0 + 1;
    let package_ids = vec![package_id];
    let payload = component_instance_payload(
        &model,
        component_instance_id,
        next_revision,
        part_id,
        &symbol_ids,
        &package_ids,
        &symbol_roles,
        &package_roles,
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
            operations: guarded_existing_object_operation(
                &model,
                Operation::SetComponentInstance {
                    component_instance_id,
                    previous_component_instance: previous_payload,
                    component_instance: payload,
                },
            )?,
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
    let previous = authored_component_instance(&model, component_instance_id)?;
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
            operations: guarded_existing_object_operation(
                &model,
                Operation::DeleteComponentInstance {
                    component_instance_id,
                    component_instance: previous_payload,
                },
            )?,
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
    let symbol_refs = instance
        .placed_symbol_refs
        .iter()
        .map(|symbol_id| revisioned_ref(model, *symbol_id))
        .collect::<Result<Vec<_>>>()?;
    let package_refs = instance
        .placed_package_refs
        .iter()
        .map(|package_id| revisioned_ref(model, *package_id))
        .collect::<Result<Vec<_>>>()?;
    let part_ref = instance
        .part_ref
        .map(|part_id| revisioned_ref(model, part_id))
        .transpose()?;
    Ok(serde_json::json!({
        "uuid": instance.id,
        "object_revision": instance.object_revision.0,
        "part_ref": part_ref,
        "placed_symbol_refs": symbol_refs,
        "placed_package_refs": package_refs,
        "placed_symbol_roles": instance.placed_symbol_roles,
        "placed_package_roles": instance.placed_package_roles
    }))
}

fn authored_component_instances(
    component_instances: &BTreeMap<Uuid, ComponentInstance>,
) -> BTreeMap<Uuid, ComponentInstance> {
    component_instances
        .iter()
        .filter(|(_, instance)| instance.authority == ComponentInstanceAuthority::Authored)
        .map(|(id, instance)| (*id, instance.clone()))
        .collect()
}

fn authored_component_instance(
    model: &eda_engine::substrate::DesignModel,
    component_instance_id: Uuid,
) -> Result<ComponentInstance> {
    let instance = model
        .component_instances
        .get(&component_instance_id)
        .with_context(|| format!("component instance {component_instance_id} was not found"))?;
    if instance.authority != ComponentInstanceAuthority::Authored {
        anyhow::bail!(
            "component instance {component_instance_id} is compatibility-derived; author an explicit ComponentInstance before mutation"
        );
    }
    Ok(instance.clone())
}

fn component_instance_payload(
    model: &eda_engine::substrate::DesignModel,
    component_instance_id: Uuid,
    object_revision: u64,
    part_id: Option<Uuid>,
    symbol_ids: &[Uuid],
    package_ids: &[Uuid],
    symbol_role_specs: &[String],
    package_role_specs: &[String],
) -> Result<serde_json::Value> {
    if symbol_ids.is_empty() {
        anyhow::bail!("component instance payload requires at least one symbol ref");
    }
    if package_ids.is_empty() {
        anyhow::bail!("component instance payload requires at least one package ref");
    }
    let symbol_refs = symbol_ids
        .iter()
        .map(|symbol_id| revisioned_ref(model, *symbol_id))
        .collect::<Result<Vec<_>>>()?;
    let package_refs = package_ids
        .iter()
        .map(|package_id| revisioned_ref(model, *package_id))
        .collect::<Result<Vec<_>>>()?;
    let part_ref = part_id
        .map(|part_id| revisioned_ref(model, part_id))
        .transpose()?;
    let placed_symbol_roles = component_role_map(symbol_ids, symbol_role_specs, "primary", "unit")?;
    let placed_package_roles =
        component_role_map(package_ids, package_role_specs, "primary", "alternate")?;
    Ok(serde_json::json!({
        "uuid": component_instance_id,
        "object_revision": object_revision,
        "part_ref": part_ref,
        "placed_symbol_refs": symbol_refs,
        "placed_package_refs": package_refs,
        "placed_symbol_roles": placed_symbol_roles,
        "placed_package_roles": placed_package_roles
    }))
}

fn component_role_map(
    object_ids: &[Uuid],
    specs: &[String],
    first_role: &str,
    later_role: &str,
) -> Result<BTreeMap<Uuid, serde_json::Value>> {
    let mut roles = object_ids
        .iter()
        .enumerate()
        .map(|(index, object_id)| {
            (
                *object_id,
                serde_json::json!({
                    "role": if index == 0 { first_role } else { later_role },
                }),
            )
        })
        .collect::<BTreeMap<_, _>>();
    for spec in specs {
        let (object_id, role, label) = parse_component_role_spec(spec)?;
        if !roles.contains_key(&object_id) {
            anyhow::bail!("component role spec {object_id} does not match a selected ref");
        }
        roles.insert(
            object_id,
            serde_json::json!({
                "role": role,
                "label": label,
            }),
        );
    }
    Ok(roles)
}

fn parse_component_role_spec(spec: &str) -> Result<(Uuid, String, Option<String>)> {
    let (object_id, role) = spec
        .split_once('=')
        .with_context(|| format!("component role spec must be <uuid>=<role>[:label]: {spec}"))?;
    let object_id = Uuid::parse_str(object_id)
        .with_context(|| format!("component role spec has invalid uuid: {object_id}"))?;
    let (role, label) = match role.split_once(':') {
        Some((role, label)) => (role.to_string(), Some(label.to_string())),
        None => (role.to_string(), None),
    };
    Ok((object_id, role, label))
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
