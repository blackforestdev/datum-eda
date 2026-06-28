use super::*;

use std::collections::{BTreeMap, BTreeSet};

use eda_engine::board::PlacedPackage;
use eda_engine::schematic::SymbolInfo;
use eda_engine::substrate::{ComponentInstanceAuthority, ProjectResolver};

pub(crate) fn query_native_project_forward_annotation_audit(
    root: &Path,
) -> Result<NativeProjectForwardAnnotationAuditView> {
    let symbols = query_native_project_symbols(root)?;
    let components = query_native_project_board_components(root)?;
    let model = ProjectResolver::new(root).resolve()?;

    let symbols_by_uuid = symbols
        .iter()
        .map(|symbol| (symbol.uuid, symbol))
        .collect::<BTreeMap<_, _>>();
    let components_by_uuid = components
        .iter()
        .map(|component| (component.uuid, component))
        .collect::<BTreeMap<_, _>>();

    let mut missing_on_board = Vec::new();
    let mut orphaned_on_board = Vec::new();
    let mut value_mismatches = Vec::new();
    let mut part_mismatches = Vec::new();
    let mut unresolved_symbol_count = 0usize;
    let mut matched_count = 0usize;
    let mut matched_symbol_ids = BTreeSet::new();
    let mut matched_component_ids = BTreeSet::new();

    for symbol in &symbols {
        if symbol.part_uuid.is_none() {
            unresolved_symbol_count += 1;
        }
    }

    for component_instance in model.component_instances.values() {
        if component_instance.authority != ComponentInstanceAuthority::Authored {
            continue;
        }
        let Some(component) = component_instance
            .placed_package_refs
            .iter()
            .find_map(|component_id| components_by_uuid.get(component_id).copied())
        else {
            continue;
        };
        matched_component_ids.insert(component.uuid);
        for symbol_id in &component_instance.placed_symbol_refs {
            let Some(symbol) = symbols_by_uuid.get(symbol_id).copied() else {
                continue;
            };
            matched_symbol_ids.insert(symbol.uuid);
            matched_count += 1;
            collect_forward_annotation_mismatches(
                symbol,
                component,
                component_instance.part_ref.or(symbol.part_uuid),
                &mut value_mismatches,
                &mut part_mismatches,
            );
        }
    }

    let mut uncovered_symbols_by_reference = BTreeMap::new();
    for symbol in &symbols {
        if !matched_symbol_ids.contains(&symbol.uuid) {
            uncovered_symbols_by_reference.insert(symbol.reference.clone(), symbol);
        }
    }
    let mut uncovered_components_by_reference = BTreeMap::new();
    for component in &components {
        if !matched_component_ids.contains(&component.uuid) {
            uncovered_components_by_reference.insert(component.reference.clone(), component);
        }
    }

    for (reference, symbol) in &uncovered_symbols_by_reference {
        if let Some(component) = uncovered_components_by_reference.get(reference) {
            matched_symbol_ids.insert(symbol.uuid);
            matched_component_ids.insert(component.uuid);
            matched_count += 1;
            collect_forward_annotation_mismatches(
                symbol,
                component,
                symbol.part_uuid,
                &mut value_mismatches,
                &mut part_mismatches,
            );
        }
    }

    for symbol in &symbols {
        if !matched_symbol_ids.contains(&symbol.uuid) {
            missing_on_board.push(NativeProjectForwardAnnotationMissingView {
                symbol_uuid: symbol.uuid.to_string(),
                sheet_uuid: symbol.sheet.to_string(),
                reference: symbol.reference.clone(),
                value: symbol.value.clone(),
                part_uuid: symbol.part_uuid.map(|uuid| uuid.to_string()),
            });
        }
    }

    for component in &components {
        if !matched_component_ids.contains(&component.uuid) {
            orphaned_on_board.push(NativeProjectForwardAnnotationOrphanView {
                component_uuid: component.uuid.to_string(),
                reference: component.reference.clone(),
                value: component.value.clone(),
                part_uuid: component.part.to_string(),
            });
        }
    }

    Ok(NativeProjectForwardAnnotationAuditView {
        domain: "native_project",
        schematic_symbol_count: symbols.len(),
        board_component_count: components.len(),
        matched_count,
        unresolved_symbol_count,
        missing_on_board,
        orphaned_on_board,
        value_mismatches,
        part_mismatches,
    })
}

fn collect_forward_annotation_mismatches(
    symbol: &SymbolInfo,
    component: &PlacedPackage,
    expected_part_uuid: Option<Uuid>,
    value_mismatches: &mut Vec<NativeProjectForwardAnnotationValueMismatchView>,
    part_mismatches: &mut Vec<NativeProjectForwardAnnotationPartMismatchView>,
) {
    if symbol.value != component.value {
        value_mismatches.push(NativeProjectForwardAnnotationValueMismatchView {
            reference: symbol.reference.clone(),
            symbol_uuid: symbol.uuid.to_string(),
            component_uuid: component.uuid.to_string(),
            schematic_value: symbol.value.clone(),
            board_value: component.value.clone(),
        });
    }
    if let Some(part_uuid) = expected_part_uuid
        && part_uuid != component.part
    {
        part_mismatches.push(NativeProjectForwardAnnotationPartMismatchView {
            reference: symbol.reference.clone(),
            symbol_uuid: symbol.uuid.to_string(),
            component_uuid: component.uuid.to_string(),
            schematic_part_uuid: part_uuid.to_string(),
            board_part_uuid: component.part.to_string(),
        });
    }
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
