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
) -> Result<()> {
    if replacement.package.is_none() && replacement.part.is_none() && replacement.value.is_none() {
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
