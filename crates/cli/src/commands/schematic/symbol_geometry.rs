//! Compatibility CLI arguments delegate native symbol geometry to the GUI writer.
use super::symbol_reports::{
    binding_evidence_for_pool_symbol, component_instance_uuid_for_pool_symbol,
    symbol_mutation_report, symbol_mutation_report_with_binding,
};
use super::*;
use eda_engine::{
    api::native_write::{
        WriteProvenance, commit_prepared,
        registry::{NativeWriteContext, find_native_write_verb},
        schematic_library,
    },
    substrate::{DesignModel, ElectricalSelectionSnapshot, ProjectResolver, SelectionSourceDomain},
};

fn commit_verb(
    model: &mut DesignModel,
    root: &Path,
    verb: &str,
    reason: &str,
    params: serde_json::Value,
) -> Result<Uuid> {
    let builder = find_native_write_verb(verb).context("Native schematic writer is unavailable")?;
    let prepared = (builder.build)(
        &NativeWriteContext {
            model,
            project_root: root,
        },
        WriteProvenance::new("datum-eda-cli", cli_commit_source()?, reason),
        params,
    )?;
    let id = prepared
        .primary_object_id
        .context("Native schematic writer returned no primary symbol")?;
    commit_prepared(model, root, prepared)?;
    Ok(id)
}

// Existing CLI handler forwards its parsed argument set unchanged.
#[allow(clippy::too_many_arguments)]
pub(super) fn place_pool_symbol(
    root: &Path,
    sheet: Uuid,
    library_symbol: Uuid,
    reference: String,
    value: String,
    position: Point,
    rotation: i32,
    mirrored: bool,
) -> Result<NativeProjectSymbolMutationReportView> {
    let mut model = ProjectResolver::new(root).resolve()?;
    let library_id = library_symbol.to_string();
    let resolution =
        schematic_library::resolve_pool_symbol_component_binding(&model, Some(&library_id))?;
    let id = commit_verb(
        &mut model,
        root,
        "datum.schematic.place_symbol",
        "place schematic symbol",
        serde_json::json!({"sheet":sheet,"library_symbol":library_symbol,"reference":reference,"value":value,"x_nm":position.x,"y_nm":position.y,"rotation_deg":rotation,"mirrored":mirrored}),
    )?;
    let project = load_native_project_with_resolved_board(root)?;
    let (sheet, path, _, symbol) = load_native_symbol_mutation_target(&project, id)?;
    let component = resolution.binding.as_ref().and_then(|binding| {
        binding
            .part
            .as_ref()
            .map(|_| component_instance_uuid_for_pool_symbol(&project, id, binding))
    });
    let evidence = resolution
        .binding
        .as_ref()
        .map(|binding| binding_evidence_for_pool_symbol(binding, id, component));
    Ok(symbol_mutation_report_with_binding(
        "place_symbol",
        &project,
        sheet,
        &path,
        &symbol,
        resolution.status.into(),
        resolution.diagnostics,
        evidence,
        component,
    ))
}

pub(super) enum Transform {
    Move(Point),
    Rotate(i32),
    Mirror,
}

pub(super) fn transform_symbol(
    root: &Path,
    id: Uuid,
    transform: Transform,
) -> Result<NativeProjectSymbolMutationReportView> {
    let mut model = ProjectResolver::new(root).resolve()?;
    let snapshot = ElectricalSelectionSnapshot::capture(&model)
        .map_err(|error| anyhow::anyhow!("Symbol source unavailable: {error:?}"))?;
    let authored = snapshot
        .authored_sources()
        .map_err(|error| anyhow::anyhow!("Symbol occurrences unavailable: {error:?}"))?;
    let occurrences: Vec<_> = authored
        .iter()
        .filter(|source| {
            source.domain == SelectionSourceDomain::Schematic
                && source.occurrence.class == "symbols"
                && source.occurrence.source_id == id
        })
        .collect();
    anyhow::ensure!(
        occurrences.len() == 1,
        "Symbol transform requires exactly one complete matching occurrence"
    );
    let source = snapshot
        .schematic_source()
        .context("Native schematic source unavailable")?;
    let symbols: Vec<_> = source
        .sheets
        .values()
        .filter_map(|sheet| sheet.symbols.get(&id))
        .collect();
    anyhow::ensure!(symbols.len() == 1, "Symbol source ownership is ambiguous");
    let symbol = symbols[0];
    let (action, reason, position, rotation, mirrored) = match transform {
        Transform::Move(position) => ("move_symbol", "move schematic symbol", position, None, None),
        Transform::Rotate(rotation) => (
            "rotate_symbol",
            "rotate schematic symbol",
            symbol.position,
            Some(rotation),
            None,
        ),
        Transform::Mirror => (
            "mirror_symbol",
            "mirror schematic symbol",
            symbol.position,
            None,
            Some(!symbol.mirrored),
        ),
    };
    let params = serde_json::json!({"symbol":id,"instance_path":occurrences[0].occurrence.instance_path,
        "x_nm":position.x,"y_nm":position.y,"rotation_deg":rotation,"mirrored":mirrored});
    commit_verb(
        &mut model,
        root,
        "datum.schematic.move_symbol",
        reason,
        params,
    )?;
    let project = load_native_project_with_resolved_board(root)?;
    let (sheet, path, _, symbol) = load_native_symbol_mutation_target(&project, id)?;
    Ok(symbol_mutation_report(
        action, &project, sheet, &path, &symbol,
    ))
}
