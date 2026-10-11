//! Existing replacement edit producer shared by post-adoption rebuilds.
use super::*;
use crate::{
    NativeBoardRoot, board_package_materialization_payload_for_component,
    current_board_component_materialization_payload,
};
use eda_engine::board::PlacedPackage;
pub(super) fn append_board_component_replacement_edits(
    root: &Path,
    board: &NativeBoardRoot,
    replacement: BoardComponentReplacementSpec,
    edits: &mut Vec<(Uuid, BoardPackageEdit)>,
    component_ids: &mut Vec<Uuid>,
    source_guards: &mut Vec<eda_engine::substrate::Operation>,
) -> Result<()> {
    if replacement.package.is_none()
        && replacement.part.is_none()
        && replacement.value.is_none()
        && replacement.process_update.is_none()
    {
        bail!(
            "board component replacement proposal for {} requires package, part, or value",
            replacement.component
        );
    }
    let component_uuid = replacement.component;
    let key = component_uuid.to_string();
    let entry = board.packages.get(&key).cloned().ok_or_else(|| {
        anyhow::anyhow!("board component not found in native project: {component_uuid}")
    })?;
    let mut component: PlacedPackage = serde_json::from_value(entry)
        .with_context(|| format!("failed to parse board component {component_uuid}"))?;
    let initial_len = edits.len();
    if let Some(request) = &replacement.process_update {
        if request.materialization.placement != component
            || replacement.package.is_some()
            || replacement.part.is_some()
            || replacement.value.is_some()
        {
            bail!(
                "process refresh must use the exact current placement preimage and separate authored replacement intent"
            );
        }
        let model = ProjectResolver::new(root).resolve()?;
        let prepared =
            eda_engine::api::native_write::board_components::process_update::build_process_update(
                &model,
                WriteProvenance::new(
                    "datum-eda-cli",
                    cli_commit_source()?,
                    "review explicit component process refresh",
                ),
                request,
            )?;
        for operation in prepared.batch.operations {
            if matches!(
                operation,
                eda_engine::substrate::Operation::GuardObjectRevision { .. }
            ) {
                source_guards.push(operation);
                continue;
            }
            if let eda_engine::substrate::Operation::SetBoardPackagePackage {
                package_ref_id,
                previous_materialized,
                materialized,
                ..
            } = operation
            {
                edits.push((
                    component_uuid,
                    BoardPackageEdit::Package {
                        package_ref_id,
                        previous_materialized,
                        materialized,
                    },
                ));
            }
        }
    }

    if let Some(part_id) = replacement.part
        && part_id != component.part
    {
        edits.push((component_uuid, BoardPackageEdit::Part { part_id }));
        component.part = part_id;
    }

    if let Some(package_ref_id) = replacement.package
        && package_ref_id != component.package
    {
        let previous_materialized =
            current_board_component_materialization_payload(root, component_uuid)?;
        component.package = package_ref_id;
        let materialized = board_package_materialization_payload_for_component(root, &component)?;
        edits.push((
            component_uuid,
            BoardPackageEdit::Package {
                package_ref_id,
                previous_materialized,
                materialized,
            },
        ));
    }

    if let Some(value) = replacement.value
        && value != component.value
    {
        edits.push((component_uuid, BoardPackageEdit::Value { value }));
    }

    if edits.len() == initial_len {
        bail!("board component replacement proposal is a no-op for component {component_uuid}");
    }
    component_ids.push(component_uuid);
    Ok(())
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct BoardComponentReplacementSpec {
    #[serde(default)]
    pub(crate) process_update: Option<
        eda_engine::api::native_write::board_components::process_update::ProcessUpdateRequest,
    >,
    #[serde(alias = "component_uuid")]
    pub(crate) component: Uuid,
    #[serde(default, alias = "package_uuid")]
    pub(crate) package: Option<Uuid>,
    #[serde(default, alias = "part_uuid")]
    pub(crate) part: Option<Uuid>,
    #[serde(default)]
    pub(crate) value: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct BoardComponentReplacementPlanSelectionSpec {
    #[serde(alias = "component", alias = "component_uuid")]
    pub(crate) uuid: Uuid,
    #[serde(default, alias = "package")]
    pub(crate) package_uuid: Option<Uuid>,
    #[serde(default, alias = "part")]
    pub(crate) part_uuid: Option<Uuid>,
    #[serde(default)]
    pub(crate) value: Option<String>,
}

impl From<BoardComponentReplacementPlanSelectionSpec> for BoardComponentReplacementSpec {
    fn from(selection: BoardComponentReplacementPlanSelectionSpec) -> Self {
        Self {
            component: selection.uuid,
            package: selection.package_uuid,
            part: selection.part_uuid,
            value: selection.value,
            process_update: None,
        }
    }
}

pub(crate) fn propose_native_project_board_component_replacement(
    root: &Path,
    component_uuid: Uuid,
    package_uuid: Option<Uuid>,
    part_uuid: Option<Uuid>,
    value: Option<String>,
    proposal_id: Option<Uuid>,
    rationale: Option<&str>,
) -> Result<NativeProjectProposalCreateView> {
    propose_native_project_board_component_replacements(
        root,
        vec![BoardComponentReplacementSpec {
            component: component_uuid,
            package: package_uuid,
            part: part_uuid,
            value,
            process_update: None,
        }],
        proposal_id,
        rationale,
    )
}

pub(crate) fn propose_native_project_board_component_replacements(
    root: &Path,
    replacements: Vec<BoardComponentReplacementSpec>,
    proposal_id: Option<Uuid>,
    rationale: Option<&str>,
) -> Result<NativeProjectProposalCreateView> {
    if replacements.is_empty() {
        bail!("board component replacement proposal requires at least one replacement");
    }
    let (project, mut model) = load_native_project_with_resolved_board_and_model(root)?;
    if let Some(id) = proposal_id
        && model.proposals.contains_key(&id)
    {
        bail!("proposal {id} already exists");
    }
    let mut edits = Vec::new();
    let mut seen_components = BTreeSet::new();
    let mut component_ids = Vec::new();
    let mut source_guards = Vec::new();
    for replacement in &replacements {
        if !seen_components.insert(replacement.component) {
            bail!(
                "board component replacement proposal repeats component {}",
                replacement.component
            );
        }
        append_board_component_replacement_edits(
            root,
            &project.board,
            replacement.clone(),
            &mut edits,
            &mut component_ids,
            &mut source_guards,
        )?;
    }

    if edits.is_empty() {
        bail!("board component replacement proposal is a no-op for all components");
    }
    let mut batch = build_board_component_replacement_proposal_batch(
        &model,
        WriteProvenance::new(
            "datum-eda-cli",
            cli_commit_source()?,
            "propose board component replacement",
        ),
        edits,
    )?;
    let source_profile_adoption_transaction =
        adopt_component_pad_profile_for_write(root, &mut model, &batch)?;
    if source_profile_adoption_transaction.is_some() {
        edits = Vec::new();
        component_ids = Vec::new();
        source_guards = Vec::new();
        for replacement in replacements {
            append_board_component_replacement_edits(
                root,
                &project.board,
                replacement,
                &mut edits,
                &mut component_ids,
                &mut source_guards,
            )?;
        }
        batch = build_board_component_replacement_proposal_batch(
            &model,
            WriteProvenance::new(
                "datum-eda-cli",
                cli_commit_source()?,
                "propose board component replacement",
            ),
            edits,
        )?;
    }
    batch.operations.extend(source_guards);
    let proposal = create_draft_proposal_from_batch(
        &mut model,
        root,
        ProposalCreateRequest {
            proposal_id,
            batch,
            rationale: rationale.map(str::to_string).unwrap_or_else(|| {
                if component_ids.len() == 1 {
                    format!("Review board component {} replacement", component_ids[0])
                } else {
                    format!(
                        "Review {} board component replacements",
                        component_ids.len()
                    )
                }
            }),
            source: ProposalSource::Cli,
            checks_run: Vec::new(),
            finding_fingerprints: Vec::new(),
        },
    )?;
    let validation = validate_proposal_in_model(&model, proposal.proposal_id, &proposal);
    Ok(NativeProjectProposalCreateView {
        contract: "proposal_create_v1",
        action: "propose_board_component_replacement",
        source_profile_adoption_transaction,
        project_id: model.project.project_id.to_string(),
        model_revision: model.model_revision.0,
        proposal_id: proposal.proposal_id,
        proposal,
        validation,
    })
}

pub(crate) fn propose_native_project_board_component_replacement_plan(
    root: &Path,
    selections: Vec<BoardComponentReplacementPlanSelectionSpec>,
    proposal_id: Option<Uuid>,
    rationale: Option<&str>,
) -> Result<NativeProjectProposalCreateView> {
    let replacements = selections
        .into_iter()
        .map(BoardComponentReplacementSpec::from)
        .collect();
    propose_native_project_board_component_replacements(root, replacements, proposal_id, rationale)
}
