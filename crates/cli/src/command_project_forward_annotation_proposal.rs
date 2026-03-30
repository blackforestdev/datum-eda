use super::*;

use std::collections::BTreeMap;

pub(crate) fn query_native_project_forward_annotation_audit(
    root: &Path,
) -> Result<NativeProjectForwardAnnotationAuditView> {
    let symbols = query_native_project_symbols(root)?;
    let components = query_native_project_board_components(root)?;

    let mut symbols_by_reference = BTreeMap::new();
    for symbol in symbols {
        symbols_by_reference.insert(symbol.reference.clone(), symbol);
    }
    let mut components_by_reference = BTreeMap::new();
    for component in components {
        components_by_reference.insert(component.reference.clone(), component);
    }

    let mut missing_on_board = Vec::new();
    let mut orphaned_on_board = Vec::new();
    let mut value_mismatches = Vec::new();
    let mut part_mismatches = Vec::new();
    let mut unresolved_symbol_count = 0usize;
    let mut matched_count = 0usize;

    for (reference, symbol) in &symbols_by_reference {
        if symbol.part_uuid.is_none() {
            unresolved_symbol_count += 1;
        }

        if let Some(component) = components_by_reference.get(reference) {
            matched_count += 1;
            if symbol.value != component.value {
                value_mismatches.push(NativeProjectForwardAnnotationValueMismatchView {
                    reference: reference.clone(),
                    symbol_uuid: symbol.uuid.to_string(),
                    component_uuid: component.uuid.to_string(),
                    schematic_value: symbol.value.clone(),
                    board_value: component.value.clone(),
                });
            }
            if let Some(part_uuid) = symbol.part_uuid
                && part_uuid != component.part
            {
                part_mismatches.push(NativeProjectForwardAnnotationPartMismatchView {
                    reference: reference.clone(),
                    symbol_uuid: symbol.uuid.to_string(),
                    component_uuid: component.uuid.to_string(),
                    schematic_part_uuid: part_uuid.to_string(),
                    board_part_uuid: component.part.to_string(),
                });
            }
        } else {
            missing_on_board.push(NativeProjectForwardAnnotationMissingView {
                symbol_uuid: symbol.uuid.to_string(),
                sheet_uuid: symbol.sheet.to_string(),
                reference: reference.clone(),
                value: symbol.value.clone(),
                part_uuid: symbol.part_uuid.map(|uuid| uuid.to_string()),
            });
        }
    }

    for (reference, component) in &components_by_reference {
        if !symbols_by_reference.contains_key(reference) {
            orphaned_on_board.push(NativeProjectForwardAnnotationOrphanView {
                component_uuid: component.uuid.to_string(),
                reference: reference.clone(),
                value: component.value.clone(),
                part_uuid: component.part.to_string(),
            });
        }
    }

    Ok(NativeProjectForwardAnnotationAuditView {
        domain: "native_project",
        schematic_symbol_count: symbols_by_reference.len(),
        board_component_count: components_by_reference.len(),
        matched_count,
        unresolved_symbol_count,
        missing_on_board,
        orphaned_on_board,
        value_mismatches,
        part_mismatches,
    })
}

pub(crate) fn query_native_project_forward_annotation_proposal(
    root: &Path,
) -> Result<NativeProjectForwardAnnotationProposalView> {
    let audit = query_native_project_forward_annotation_audit(root)?;
    let mut actions = Vec::new();

    for entry in &audit.missing_on_board {
        actions.push(NativeProjectForwardAnnotationProposalActionView {
            action_id: forward_annotation_action_id(
                "add_component",
                &entry.reference,
                Some(&entry.symbol_uuid),
                None,
                if entry.part_uuid.is_some() {
                    "symbol_missing_on_board"
                } else {
                    "symbol_missing_on_board_unresolved_part"
                },
            ),
            action: "add_component".to_string(),
            reference: entry.reference.clone(),
            symbol_uuid: Some(entry.symbol_uuid.clone()),
            component_uuid: None,
            reason: if entry.part_uuid.is_some() {
                "symbol_missing_on_board".to_string()
            } else {
                "symbol_missing_on_board_unresolved_part".to_string()
            },
            schematic_value: Some(entry.value.clone()),
            board_value: None,
            schematic_part_uuid: entry.part_uuid.clone(),
            board_part_uuid: None,
        });
    }

    for entry in &audit.orphaned_on_board {
        actions.push(NativeProjectForwardAnnotationProposalActionView {
            action_id: forward_annotation_action_id(
                "remove_component",
                &entry.reference,
                None,
                Some(&entry.component_uuid),
                "board_component_missing_in_schematic",
            ),
            action: "remove_component".to_string(),
            reference: entry.reference.clone(),
            symbol_uuid: None,
            component_uuid: Some(entry.component_uuid.clone()),
            reason: "board_component_missing_in_schematic".to_string(),
            schematic_value: None,
            board_value: Some(entry.value.clone()),
            schematic_part_uuid: None,
            board_part_uuid: Some(entry.part_uuid.clone()),
        });
    }

    for entry in &audit.value_mismatches {
        actions.push(NativeProjectForwardAnnotationProposalActionView {
            action_id: forward_annotation_action_id(
                "update_component",
                &entry.reference,
                Some(&entry.symbol_uuid),
                Some(&entry.component_uuid),
                "value_mismatch",
            ),
            action: "update_component".to_string(),
            reference: entry.reference.clone(),
            symbol_uuid: Some(entry.symbol_uuid.clone()),
            component_uuid: Some(entry.component_uuid.clone()),
            reason: "value_mismatch".to_string(),
            schematic_value: Some(entry.schematic_value.clone()),
            board_value: Some(entry.board_value.clone()),
            schematic_part_uuid: None,
            board_part_uuid: None,
        });
    }

    for entry in &audit.part_mismatches {
        actions.push(NativeProjectForwardAnnotationProposalActionView {
            action_id: forward_annotation_action_id(
                "update_component",
                &entry.reference,
                Some(&entry.symbol_uuid),
                Some(&entry.component_uuid),
                "part_mismatch",
            ),
            action: "update_component".to_string(),
            reference: entry.reference.clone(),
            symbol_uuid: Some(entry.symbol_uuid.clone()),
            component_uuid: Some(entry.component_uuid.clone()),
            reason: "part_mismatch".to_string(),
            schematic_value: None,
            board_value: None,
            schematic_part_uuid: Some(entry.schematic_part_uuid.clone()),
            board_part_uuid: Some(entry.board_part_uuid.clone()),
        });
    }

    actions.sort_by(|a, b| {
        a.reference
            .cmp(&b.reference)
            .then_with(|| a.action.cmp(&b.action))
            .then_with(|| a.reason.cmp(&b.reason))
    });

    let add_component_actions = actions
        .iter()
        .filter(|action| action.action == "add_component")
        .count();
    let remove_component_actions = actions
        .iter()
        .filter(|action| action.action == "remove_component")
        .count();
    let update_component_actions = actions
        .iter()
        .filter(|action| action.action == "update_component")
        .count();
    let add_component_group = actions
        .iter()
        .filter(|action| action.action == "add_component")
        .cloned()
        .collect::<Vec<_>>();
    let remove_component_group = actions
        .iter()
        .filter(|action| action.action == "remove_component")
        .cloned()
        .collect::<Vec<_>>();
    let update_component_group = actions
        .iter()
        .filter(|action| action.action == "update_component")
        .cloned()
        .collect::<Vec<_>>();

    Ok(NativeProjectForwardAnnotationProposalView {
        domain: "native_project",
        total_actions: actions.len(),
        add_component_actions,
        remove_component_actions,
        update_component_actions,
        add_component_group,
        remove_component_group,
        update_component_group,
        actions,
    })
}

fn forward_annotation_action_id(
    action: &str,
    reference: &str,
    symbol_uuid: Option<&str>,
    component_uuid: Option<&str>,
    reason: &str,
) -> String {
    let stable_key = format!(
        "{action}|{reference}|{}|{}|{reason}",
        symbol_uuid.unwrap_or(""),
        component_uuid.unwrap_or("")
    );
    compute_source_hash_bytes(stable_key.as_bytes())
}
