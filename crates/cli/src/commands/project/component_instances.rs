use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use eda_engine::api::native_write::component_instances::{
    ComponentInstanceSpec, ComponentRoleAssignment, build_bind_component_instance,
    build_delete_component_instance, build_set_component_instance,
};
use eda_engine::api::native_write::{WriteProvenance, commit_prepared};
use eda_engine::substrate::{
    ComponentInstance, ComponentInstanceAuthority, ProjectResolver, Proposal,
    ProposalCreateRequest, ProposalSource, create_draft_proposal_from_batch,
};
use serde::Serialize;
use uuid::Uuid;

use crate::commands::project::proposals::{
    NativeProjectProposalValidationView, validate_proposal_in_model,
};
use crate::{
    OutputFormat, ProposalBindComponentInstanceArgs, ProposalDeleteComponentInstanceArgs,
    ProposalSetComponentInstanceArgs, cli_commit_source, render_output,
};

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

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectComponentInstanceProposalView {
    pub(crate) contract: &'static str,
    pub(crate) action: &'static str,
    pub(crate) project_id: String,
    pub(crate) model_revision: String,
    pub(crate) component_instance_id: Uuid,
    pub(crate) component_instance_path: String,
    pub(crate) proposal_id: Uuid,
    pub(crate) proposal: Proposal,
    pub(crate) validation: NativeProjectProposalValidationView,
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
    let spec = component_instance_spec(
        symbol_ids,
        package_id,
        part_id,
        &symbol_roles,
        &package_roles,
    )?;
    let (prepared, component_instance_id) = build_bind_component_instance(
        &model,
        cli_provenance("bind component instance")?,
        component_instance_id,
        &spec,
    )?;
    let report = commit_prepared(&mut model, root, prepared)?;
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
    let spec = component_instance_spec(
        symbol_ids,
        package_id,
        part_id,
        &symbol_roles,
        &package_roles,
    )?;
    let prepared = build_set_component_instance(
        &model,
        cli_provenance("set component instance")?,
        component_instance_id,
        &spec,
    )?;
    let report = commit_prepared(&mut model, root, prepared)?;
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
    let (prepared, previous) = build_delete_component_instance(
        &model,
        cli_provenance("delete component instance")?,
        component_instance_id,
    )?;
    let report = commit_prepared(&mut model, root, prepared)?;
    Ok(component_instance_mutation(
        "delete_component_instance",
        model.project.project_id,
        previous,
        component_instance_path(root, component_instance_id),
        report.transaction.transaction_id,
    ))
}

pub(crate) fn propose_bind_native_project_component_instance(
    root: &Path,
    component_instance_id: Option<Uuid>,
    symbol_ids: Vec<Uuid>,
    package_id: Uuid,
    part_id: Option<Uuid>,
    symbol_roles: Vec<String>,
    package_roles: Vec<String>,
    proposal_id: Option<Uuid>,
    rationale: Option<&str>,
) -> Result<NativeProjectComponentInstanceProposalView> {
    let mut model = ProjectResolver::new(root).resolve()?;
    let spec = component_instance_spec(
        symbol_ids,
        package_id,
        part_id,
        &symbol_roles,
        &package_roles,
    )?;
    let (prepared, component_instance_id) = build_bind_component_instance(
        &model,
        cli_provenance("propose bind component instance")?,
        component_instance_id,
        &spec,
    )?;
    create_component_instance_proposal(
        root,
        &mut model,
        "propose_bind_component_instance",
        component_instance_id,
        prepared.batch,
        proposal_id,
        rationale.map(str::to_string).unwrap_or_else(|| {
            format!("Review component instance {component_instance_id} binding")
        }),
    )
}

pub(crate) fn propose_set_native_project_component_instance(
    root: &Path,
    component_instance_id: Uuid,
    symbol_ids: Vec<Uuid>,
    package_id: Uuid,
    part_id: Option<Uuid>,
    symbol_roles: Vec<String>,
    package_roles: Vec<String>,
    proposal_id: Option<Uuid>,
    rationale: Option<&str>,
) -> Result<NativeProjectComponentInstanceProposalView> {
    let mut model = ProjectResolver::new(root).resolve()?;
    let spec = component_instance_spec(
        symbol_ids,
        package_id,
        part_id,
        &symbol_roles,
        &package_roles,
    )?;
    let prepared = build_set_component_instance(
        &model,
        cli_provenance("propose set component instance")?,
        component_instance_id,
        &spec,
    )?;
    create_component_instance_proposal(
        root,
        &mut model,
        "propose_set_component_instance",
        component_instance_id,
        prepared.batch,
        proposal_id,
        rationale
            .map(str::to_string)
            .unwrap_or_else(|| format!("Review component instance {component_instance_id} update")),
    )
}

pub(crate) fn propose_delete_native_project_component_instance(
    root: &Path,
    component_instance_id: Uuid,
    proposal_id: Option<Uuid>,
    rationale: Option<&str>,
) -> Result<NativeProjectComponentInstanceProposalView> {
    let mut model = ProjectResolver::new(root).resolve()?;
    let (prepared, _) = build_delete_component_instance(
        &model,
        cli_provenance("propose delete component instance")?,
        component_instance_id,
    )?;
    create_component_instance_proposal(
        root,
        &mut model,
        "propose_delete_component_instance",
        component_instance_id,
        prepared.batch,
        proposal_id,
        rationale.map(str::to_string).unwrap_or_else(|| {
            format!("Review component instance {component_instance_id} deletion")
        }),
    )
}

fn create_component_instance_proposal(
    root: &Path,
    model: &mut eda_engine::substrate::DesignModel,
    action: &'static str,
    component_instance_id: Uuid,
    batch: eda_engine::substrate::OperationBatch,
    proposal_id: Option<Uuid>,
    rationale: String,
) -> Result<NativeProjectComponentInstanceProposalView> {
    let proposal = create_draft_proposal_from_batch(
        model,
        root,
        ProposalCreateRequest {
            proposal_id,
            batch,
            rationale,
            source: ProposalSource::Cli,
            checks_run: Vec::new(),
            finding_fingerprints: Vec::new(),
        },
    )?;
    let validation = validate_proposal_in_model(model, proposal.proposal_id, &proposal);
    Ok(NativeProjectComponentInstanceProposalView {
        contract: "proposal_create_v1",
        action,
        project_id: model.project.project_id.to_string(),
        model_revision: model.model_revision.0.clone(),
        component_instance_id,
        component_instance_path: component_instance_path(root, component_instance_id)
            .display()
            .to_string(),
        proposal_id: proposal.proposal_id,
        proposal,
        validation,
    })
}

fn cli_provenance(reason: &str) -> Result<WriteProvenance> {
    Ok(WriteProvenance::new(
        "datum-eda-cli",
        cli_commit_source()?,
        reason,
    ))
}

fn component_instance_spec(
    symbol_ids: Vec<Uuid>,
    package_id: Uuid,
    part_id: Option<Uuid>,
    symbol_role_specs: &[String],
    package_role_specs: &[String],
) -> Result<ComponentInstanceSpec> {
    Ok(ComponentInstanceSpec {
        part_id,
        symbol_ids,
        package_id,
        symbol_roles: parse_component_role_specs(symbol_role_specs)?,
        package_roles: parse_component_role_specs(package_role_specs)?,
    })
}

fn parse_component_role_specs(specs: &[String]) -> Result<Vec<ComponentRoleAssignment>> {
    specs
        .iter()
        .map(|spec| parse_component_role_spec(spec))
        .collect()
}

fn parse_component_role_spec(spec: &str) -> Result<ComponentRoleAssignment> {
    let (object_id, role) = spec
        .split_once('=')
        .with_context(|| format!("component role spec must be <uuid>=<role>[:label]: {spec}"))?;
    let object_id = Uuid::parse_str(object_id)
        .with_context(|| format!("component role spec has invalid uuid: {object_id}"))?;
    let (role, label) = match role.split_once(':') {
        Some((role, label)) => (role.to_string(), Some(label.to_string())),
        None => (role.to_string(), None),
    };
    Ok(ComponentRoleAssignment {
        object_id,
        role,
        label,
    })
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

impl ProposalBindComponentInstanceArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            component_instance,
            symbols,
            package,
            part,
            symbol_roles,
            package_roles,
            proposal,
            rationale,
        } = self;
        Ok((
            render_output(
                format,
                &propose_bind_native_project_component_instance(
                    &path,
                    component_instance,
                    symbols,
                    package,
                    part,
                    symbol_roles,
                    package_roles,
                    proposal,
                    rationale.as_deref(),
                )?,
            ),
            0,
        ))
    }
}

impl ProposalSetComponentInstanceArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            component_instance,
            symbols,
            package,
            part,
            symbol_roles,
            package_roles,
            proposal,
            rationale,
        } = self;
        Ok((
            render_output(
                format,
                &propose_set_native_project_component_instance(
                    &path,
                    component_instance,
                    symbols,
                    package,
                    part,
                    symbol_roles,
                    package_roles,
                    proposal,
                    rationale.as_deref(),
                )?,
            ),
            0,
        ))
    }
}

impl ProposalDeleteComponentInstanceArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            component_instance,
            proposal,
            rationale,
        } = self;
        Ok((
            render_output(
                format,
                &propose_delete_native_project_component_instance(
                    &path,
                    component_instance,
                    proposal,
                    rationale.as_deref(),
                )?,
            ),
            0,
        ))
    }
}
