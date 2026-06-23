use std::path::Path;

use anyhow::{Context, Result};
use eda_engine::substrate::{
    CommitProvenance, CommitSource, Operation, OperationBatch, ProjectResolver, Proposal,
    ProposalCreateRequest, ProposalSource, create_draft_proposal_from_batch,
};
use serde::Serialize;
use uuid::Uuid;

use super::command_project_schematic_symbol_library_materialization::materialize_pool_symbol_pins;
use super::*;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectWireProposalView {
    pub(crate) contract: &'static str,
    pub(crate) action: &'static str,
    pub(crate) project_id: String,
    pub(crate) model_revision: String,
    pub(crate) sheet_uuid: String,
    pub(crate) sheet_path: String,
    pub(crate) wire_uuid: String,
    pub(crate) from_x_nm: i64,
    pub(crate) from_y_nm: i64,
    pub(crate) to_x_nm: i64,
    pub(crate) to_y_nm: i64,
    pub(crate) proposal_id: Uuid,
    pub(crate) proposal: Proposal,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectLabelProposalView {
    pub(crate) contract: &'static str,
    pub(crate) action: &'static str,
    pub(crate) project_id: String,
    pub(crate) model_revision: String,
    pub(crate) sheet_uuid: String,
    pub(crate) sheet_path: String,
    pub(crate) label_uuid: String,
    pub(crate) name: String,
    pub(crate) kind: String,
    pub(crate) x_nm: i64,
    pub(crate) y_nm: i64,
    pub(crate) proposal_id: Uuid,
    pub(crate) proposal: Proposal,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectSymbolProposalView {
    pub(crate) contract: &'static str,
    pub(crate) action: &'static str,
    pub(crate) project_id: String,
    pub(crate) model_revision: String,
    pub(crate) sheet_uuid: String,
    pub(crate) sheet_path: String,
    pub(crate) symbol_uuid: String,
    pub(crate) reference: String,
    pub(crate) value: String,
    pub(crate) lib_id: Option<String>,
    pub(crate) x_nm: i64,
    pub(crate) y_nm: i64,
    pub(crate) rotation_deg: i32,
    pub(crate) mirrored: bool,
    pub(crate) proposal_id: Uuid,
    pub(crate) proposal: Proposal,
}

pub(crate) fn propose_place_native_project_symbol(
    root: &Path,
    sheet_uuid: Uuid,
    reference: String,
    value: String,
    lib_id: Option<String>,
    position: Point,
    rotation_deg: i32,
    mirrored: bool,
    proposal_id: Option<Uuid>,
    rationale: Option<&str>,
) -> Result<NativeProjectSymbolProposalView> {
    let project = load_native_project(root)?;
    let sheet_key = sheet_uuid.to_string();
    let relative_path =
        project.schematic.sheets.get(&sheet_key).ok_or_else(|| {
            anyhow::anyhow!("sheet not found in native schematic root: {sheet_uuid}")
        })?;
    let pins = materialize_pool_symbol_pins(root, lib_id.as_deref())?;
    let symbol_uuid = Uuid::new_v4();
    let symbol = PlacedSymbol {
        uuid: symbol_uuid,
        part: None,
        entity: None,
        gate: None,
        lib_id: lib_id.clone(),
        reference: reference.clone(),
        value: value.clone(),
        fields: Vec::<SymbolField>::new(),
        pins,
        position,
        rotation: rotation_deg,
        mirrored,
        unit_selection: None,
        display_mode: SymbolDisplayMode::LibraryDefault,
        pin_overrides: Vec::<PinDisplayOverride>::new(),
        hidden_power_behavior: HiddenPowerBehavior::SourceDefinedImplicit,
    };
    let mut model = ProjectResolver::new(root)
        .resolve()
        .with_context(|| format!("failed to resolve native project {}", root.display()))?;
    let batch = OperationBatch {
        batch_id: Uuid::new_v4(),
        expected_model_revision: Some(model.model_revision.clone()),
        provenance: CommitProvenance {
            actor: "datum-eda-cli".to_string(),
            source: CommitSource::Cli,
            reason: "propose place schematic symbol".to_string(),
        },
        operations: vec![Operation::CreateSchematicSymbol {
            sheet_id: sheet_uuid,
            symbol_id: symbol_uuid,
            symbol: serde_json::to_value(&symbol).context("failed to serialize symbol operation")?,
        }],
    };
    let proposal = create_draft_proposal_from_batch(
        &mut model,
        root,
        ProposalCreateRequest {
            proposal_id,
            batch,
            rationale: rationale
                .map(str::to_string)
                .unwrap_or_else(|| format!("Review schematic symbol {symbol_uuid} creation")),
            source: ProposalSource::Cli,
            checks_run: Vec::new(),
            finding_fingerprints: Vec::new(),
        },
    )?;

    Ok(NativeProjectSymbolProposalView {
        contract: "proposal_create_v1",
        action: "propose_place_symbol",
        project_id: model.project.project_id.to_string(),
        model_revision: model.model_revision.0.clone(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: project
            .root
            .join("schematic")
            .join(relative_path)
            .display()
            .to_string(),
        symbol_uuid: symbol_uuid.to_string(),
        reference,
        value,
        lib_id,
        x_nm: position.x,
        y_nm: position.y,
        rotation_deg,
        mirrored,
        proposal_id: proposal.proposal_id,
        proposal,
    })
}

pub(crate) fn propose_place_native_project_label(
    root: &Path,
    sheet_uuid: Uuid,
    name: String,
    kind: LabelKind,
    position: Point,
    proposal_id: Option<Uuid>,
    rationale: Option<&str>,
) -> Result<NativeProjectLabelProposalView> {
    let project = load_native_project(root)?;
    let sheet_key = sheet_uuid.to_string();
    let relative_path =
        project.schematic.sheets.get(&sheet_key).ok_or_else(|| {
            anyhow::anyhow!("sheet not found in native schematic root: {sheet_uuid}")
        })?;
    let label_uuid = Uuid::new_v4();
    let label = NetLabel {
        uuid: label_uuid,
        kind: kind.clone(),
        name: name.clone(),
        position,
    };
    let mut model = ProjectResolver::new(root)
        .resolve()
        .with_context(|| format!("failed to resolve native project {}", root.display()))?;
    let batch = OperationBatch {
        batch_id: Uuid::new_v4(),
        expected_model_revision: Some(model.model_revision.clone()),
        provenance: CommitProvenance {
            actor: "datum-eda-cli".to_string(),
            source: CommitSource::Cli,
            reason: "propose place schematic label".to_string(),
        },
        operations: vec![Operation::CreateSchematicLabel {
            sheet_id: sheet_uuid,
            label_id: label_uuid,
            label: serde_json::to_value(&label).context("failed to serialize label operation")?,
        }],
    };
    let proposal = create_draft_proposal_from_batch(
        &mut model,
        root,
        ProposalCreateRequest {
            proposal_id,
            batch,
            rationale: rationale
                .map(str::to_string)
                .unwrap_or_else(|| format!("Review schematic label {label_uuid} creation")),
            source: ProposalSource::Cli,
            checks_run: Vec::new(),
            finding_fingerprints: Vec::new(),
        },
    )?;

    Ok(NativeProjectLabelProposalView {
        contract: "proposal_create_v1",
        action: "propose_place_label",
        project_id: model.project.project_id.to_string(),
        model_revision: model.model_revision.0.clone(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: project
            .root
            .join("schematic")
            .join(relative_path)
            .display()
            .to_string(),
        label_uuid: label_uuid.to_string(),
        name,
        kind: render_label_kind(&kind).to_string(),
        x_nm: position.x,
        y_nm: position.y,
        proposal_id: proposal.proposal_id,
        proposal,
    })
}

pub(crate) fn propose_draw_native_project_wire(
    root: &Path,
    sheet_uuid: Uuid,
    from: Point,
    to: Point,
    proposal_id: Option<Uuid>,
    rationale: Option<&str>,
) -> Result<NativeProjectWireProposalView> {
    let project = load_native_project(root)?;
    let sheet_key = sheet_uuid.to_string();
    let relative_path =
        project.schematic.sheets.get(&sheet_key).ok_or_else(|| {
            anyhow::anyhow!("sheet not found in native schematic root: {sheet_uuid}")
        })?;
    let wire_uuid = Uuid::new_v4();
    let wire = SchematicWire {
        uuid: wire_uuid,
        from,
        to,
    };
    let mut model = ProjectResolver::new(root)
        .resolve()
        .with_context(|| format!("failed to resolve native project {}", root.display()))?;
    let batch = OperationBatch {
        batch_id: Uuid::new_v4(),
        expected_model_revision: Some(model.model_revision.clone()),
        provenance: CommitProvenance {
            actor: "datum-eda-cli".to_string(),
            source: CommitSource::Cli,
            reason: "propose draw schematic wire".to_string(),
        },
        operations: vec![Operation::CreateSchematicWire {
            sheet_id: sheet_uuid,
            wire_id: wire_uuid,
            wire: serde_json::to_value(&wire).context("failed to serialize wire operation")?,
        }],
    };
    let proposal = create_draft_proposal_from_batch(
        &mut model,
        root,
        ProposalCreateRequest {
            proposal_id,
            batch,
            rationale: rationale
                .map(str::to_string)
                .unwrap_or_else(|| format!("Review schematic wire {wire_uuid} creation")),
            source: ProposalSource::Cli,
            checks_run: Vec::new(),
            finding_fingerprints: Vec::new(),
        },
    )?;

    Ok(NativeProjectWireProposalView {
        contract: "proposal_create_v1",
        action: "propose_draw_wire",
        project_id: model.project.project_id.to_string(),
        model_revision: model.model_revision.0.clone(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: project
            .root
            .join("schematic")
            .join(relative_path)
            .display()
            .to_string(),
        wire_uuid: wire_uuid.to_string(),
        from_x_nm: from.x,
        from_y_nm: from.y,
        to_x_nm: to.x,
        to_y_nm: to.y,
        proposal_id: proposal.proposal_id,
        proposal,
    })
}
