use super::*;

pub(super) fn dispatch_request(engine: &mut Engine, request: JsonRpcRequest) -> JsonRpcResponse {
    if request.jsonrpc != "2.0" {
        return error_response(request.id, -32600, "invalid jsonrpc version");
    }

    match request.method.as_str() {
        "open_project" => match serde_json::from_value::<OpenProjectParams>(request.params) {
            Ok(params) => match open_project(engine, &params.path) {
                Ok(report) => success_response(request.id, serde_json::to_value(report).unwrap()),
                Err(err) => error_response(request.id, -32000, &err.to_string()),
            },
            Err(err) => error_response(request.id, -32602, &format!("invalid params: {err}")),
        },
        "close_project" => {
            engine.close_project();
            success_response(request.id, json!({"closed": true}))
        }
        "save" => match serde_json::from_value::<SaveParams>(request.params) {
            Ok(params) => {
                let saved = match params.path {
                    Some(path) => engine.save(&path).map(|_| path),
                    None => engine.save_to_original(),
                };
                match saved {
                    Ok(path) => {
                        success_response(request.id, json!({"path": path.display().to_string()}))
                    }
                    Err(err) => error_response(request.id, -32027, &err.to_string()),
                }
            }
            Err(err) => error_response(request.id, -32602, &format!("invalid params: {err}")),
        },
        "delete_track" => match serde_json::from_value::<UuidParams>(request.params) {
            Ok(params) => match engine.delete_track(&params.uuid) {
                Ok(result) => success_response(request.id, serde_json::to_value(result).unwrap()),
                Err(err) => error_response(request.id, -32028, &err.to_string()),
            },
            Err(err) => error_response(request.id, -32602, &format!("invalid params: {err}")),
        },
        "delete_via" => match serde_json::from_value::<UuidParams>(request.params) {
            Ok(params) => match engine.delete_via(&params.uuid) {
                Ok(result) => success_response(request.id, serde_json::to_value(result).unwrap()),
                Err(err) => error_response(request.id, -32031, &err.to_string()),
            },
            Err(err) => error_response(request.id, -32602, &format!("invalid params: {err}")),
        },
        "delete_component" => match serde_json::from_value::<UuidParams>(request.params) {
            Ok(params) => match engine.delete_component(&params.uuid) {
                Ok(result) => success_response(request.id, serde_json::to_value(result).unwrap()),
                Err(err) => error_response(request.id, -32036, &err.to_string()),
            },
            Err(err) => error_response(request.id, -32602, &format!("invalid params: {err}")),
        },
        "move_component" => match serde_json::from_value::<MoveComponentParams>(request.params) {
            Ok(params) => match engine.move_component(MoveComponentInput {
                uuid: params.uuid,
                position: Point::new(mm_to_nm(params.x_mm), mm_to_nm(params.y_mm)),
                rotation: params.rotation_deg.map(|deg| deg.round() as i32),
            }) {
                Ok(result) => success_response(request.id, serde_json::to_value(result).unwrap()),
                Err(err) => error_response(request.id, -32033, &err.to_string()),
            },
            Err(err) => error_response(request.id, -32602, &format!("invalid params: {err}")),
        },
        "rotate_component" => match serde_json::from_value::<MoveComponentParams>(request.params) {
            Ok(params) => {
                let rotation = match params.rotation_deg {
                    Some(deg) => deg.round() as i32,
                    None => {
                        return error_response(
                            request.id,
                            -32602,
                            "invalid params: rotate_component requires rotation_deg",
                        );
                    }
                };
                match engine.rotate_component(RotateComponentInput {
                    uuid: params.uuid,
                    rotation,
                }) {
                    Ok(result) => success_response(request.id, serde_json::to_value(result).unwrap()),
                    Err(err) => error_response(request.id, -32037, &err.to_string()),
                }
            }
            Err(err) => error_response(request.id, -32602, &format!("invalid params: {err}")),
        },
        "set_value" => match serde_json::from_value::<SetValueParams>(request.params) {
            Ok(params) => match engine.set_value(SetValueInput {
                uuid: params.uuid,
                value: params.value,
            }) {
                Ok(result) => success_response(request.id, serde_json::to_value(result).unwrap()),
                Err(err) => error_response(request.id, -32034, &err.to_string()),
            },
            Err(err) => error_response(request.id, -32602, &format!("invalid params: {err}")),
        },
        "set_reference" => match serde_json::from_value::<SetReferenceParams>(request.params) {
            Ok(params) => match engine.set_reference(SetReferenceInput {
                uuid: params.uuid,
                reference: params.reference,
            }) {
                Ok(result) => success_response(request.id, serde_json::to_value(result).unwrap()),
                Err(err) => error_response(request.id, -32035, &err.to_string()),
            },
            Err(err) => error_response(request.id, -32602, &format!("invalid params: {err}")),
        },
        "assign_part" => match serde_json::from_value::<AssignPartParams>(request.params) {
            Ok(params) => match engine.assign_part(AssignPartInput {
                uuid: params.uuid,
                part_uuid: params.part_uuid,
            }) {
                Ok(result) => success_response(request.id, serde_json::to_value(result).unwrap()),
                Err(err) => error_response(request.id, -32038, &err.to_string()),
            },
            Err(err) => error_response(request.id, -32602, &format!("invalid params: {err}")),
        },
        "set_package" => match serde_json::from_value::<SetPackageParams>(request.params) {
            Ok(params) => match engine.set_package(SetPackageInput {
                uuid: params.uuid,
                package_uuid: params.package_uuid,
            }) {
                Ok(result) => success_response(request.id, serde_json::to_value(result).unwrap()),
                Err(err) => error_response(request.id, -32040, &err.to_string()),
            },
            Err(err) => error_response(request.id, -32602, &format!("invalid params: {err}")),
        },
        "set_package_with_part" => {
            match serde_json::from_value::<SetPackageWithPartParams>(request.params) {
                Ok(params) => match engine.set_package_with_part(SetPackageWithPartInput {
                    uuid: params.uuid,
                    package_uuid: params.package_uuid,
                    part_uuid: params.part_uuid,
                }) {
                    Ok(result) => {
                        success_response(request.id, serde_json::to_value(result).unwrap())
                    }
                    Err(err) => error_response(request.id, -32041, &err.to_string()),
                },
                Err(err) => error_response(request.id, -32602, &format!("invalid params: {err}")),
            }
        }
        "replace_component" => {
            match serde_json::from_value::<ReplaceComponentParams>(request.params) {
                Ok(params) => match engine.replace_component(ReplaceComponentInput {
                    uuid: params.uuid,
                    package_uuid: params.package_uuid,
                    part_uuid: params.part_uuid,
                }) {
                    Ok(result) => {
                        success_response(request.id, serde_json::to_value(result).unwrap())
                    }
                    Err(err) => error_response(request.id, -32044, &err.to_string()),
                },
                Err(err) => error_response(request.id, -32602, &format!("invalid params: {err}")),
            }
        }
        "replace_components" => {
            match serde_json::from_value::<ReplaceComponentsParams>(request.params) {
                Ok(params) => match engine.replace_components(
                    params
                        .replacements
                        .into_iter()
                        .map(|item| ReplaceComponentInput {
                            uuid: item.uuid,
                            package_uuid: item.package_uuid,
                            part_uuid: item.part_uuid,
                        })
                        .collect(),
                ) {
                    Ok(result) => {
                        success_response(request.id, serde_json::to_value(result).unwrap())
                    }
                    Err(err) => error_response(request.id, -32045, &err.to_string()),
                },
                Err(err) => error_response(request.id, -32602, &format!("invalid params: {err}")),
            }
        }
        "apply_component_replacement_plan" => {
            match serde_json::from_value::<ApplyComponentReplacementPlanParams>(request.params) {
                Ok(params) => match engine.apply_component_replacement_plan(
                    params
                        .replacements
                        .into_iter()
                        .map(|item| PlannedComponentReplacementInput {
                            uuid: item.uuid,
                            package_uuid: item.package_uuid,
                            part_uuid: item.part_uuid,
                        })
                        .collect(),
                ) {
                    Ok(result) => {
                        success_response(request.id, serde_json::to_value(result).unwrap())
                    }
                    Err(err) => error_response(request.id, -32046, &err.to_string()),
                },
                Err(err) => error_response(request.id, -32602, &format!("invalid params: {err}")),
            }
        }
        "apply_component_replacement_policy" => {
            match serde_json::from_value::<ApplyComponentReplacementPolicyParams>(request.params) {
                Ok(params) => match engine.apply_component_replacement_policy(
                    params
                        .replacements
                        .into_iter()
                        .map(|item| PolicyDrivenComponentReplacementInput {
                            uuid: item.uuid,
                            policy: item.policy,
                        })
                        .collect(),
                ) {
                    Ok(result) => {
                        success_response(request.id, serde_json::to_value(result).unwrap())
                    }
                    Err(err) => error_response(request.id, -32047, &err.to_string()),
                },
                Err(err) => error_response(request.id, -32602, &format!("invalid params: {err}")),
            }
        }
        "apply_scoped_component_replacement_policy" => {
            match serde_json::from_value::<ApplyScopedComponentReplacementPolicyParams>(request.params) {
                Ok(params) => match engine.apply_scoped_component_replacement_policy(
                    ScopedComponentReplacementPolicyInput {
                        scope: ComponentReplacementScope {
                            reference_prefix: params.scope.reference_prefix,
                            value_equals: params.scope.value_equals,
                            current_package_uuid: params.scope.current_package_uuid,
                            current_part_uuid: params.scope.current_part_uuid,
                        },
                        policy: params.policy,
                    },
                ) {
                    Ok(result) => {
                        success_response(request.id, serde_json::to_value(result).unwrap())
                    }
                    Err(err) => error_response(request.id, -32048, &err.to_string()),
                },
                Err(err) => error_response(request.id, -32602, &format!("invalid params: {err}")),
            }
        }
        "apply_scoped_component_replacement_plan" => {
            match serde_json::from_value::<ApplyScopedComponentReplacementPlanParams>(request.params)
            {
                Ok(params) => match engine.apply_scoped_component_replacement_plan(params.plan) {
                    Ok(result) => {
                        success_response(request.id, serde_json::to_value(result).unwrap())
                    }
                    Err(err) => error_response(request.id, -32050, &err.to_string()),
                },
                Err(err) => error_response(request.id, -32602, &format!("invalid params: {err}")),
            }
        }
        "set_net_class" => match serde_json::from_value::<SetNetClassParams>(request.params) {
            Ok(params) => match engine.set_net_class(SetNetClassInput {
                net_uuid: params.net_uuid,
                class_name: params.class_name,
                clearance: params.clearance,
                track_width: params.track_width,
                via_drill: params.via_drill,
                via_diameter: params.via_diameter,
                diffpair_width: params.diffpair_width,
                diffpair_gap: params.diffpair_gap,
            }) {
                Ok(result) => success_response(request.id, serde_json::to_value(result).unwrap()),
                Err(err) => error_response(request.id, -32039, &err.to_string()),
            },
            Err(err) => error_response(request.id, -32602, &format!("invalid params: {err}")),
        },
        "set_design_rule" => match serde_json::from_value::<SetDesignRuleParams>(request.params) {
            Ok(params) => match engine.set_design_rule(SetDesignRuleInput {
                rule_type: params.rule_type,
                scope: params.scope,
                parameters: params.parameters,
                priority: params.priority,
                name: params.name,
            }) {
                Ok(result) => success_response(request.id, serde_json::to_value(result).unwrap()),
                Err(err) => error_response(request.id, -32032, &err.to_string()),
            },
            Err(err) => error_response(request.id, -32602, &format!("invalid params: {err}")),
        },
        "undo" => match engine.undo() {
            Ok(result) => success_response(request.id, serde_json::to_value(result).unwrap()),
            Err(err) => error_response(request.id, -32029, &err.to_string()),
        },
        "redo" => match engine.redo() {
            Ok(result) => success_response(request.id, serde_json::to_value(result).unwrap()),
            Err(err) => error_response(request.id, -32030, &err.to_string()),
        },
        "search_pool" => match serde_json::from_value::<SearchPoolParams>(request.params) {
            Ok(params) => match engine.search_pool(&params.query) {
                Ok(parts) => success_response(request.id, serde_json::to_value(parts).unwrap()),
                Err(err) => error_response(request.id, -32019, &err.to_string()),
            },
            Err(err) => error_response(request.id, -32602, &format!("invalid params: {err}")),
        },
        "get_part" => match serde_json::from_value::<UuidParams>(request.params) {
            Ok(params) => match engine.get_part(&params.uuid) {
                Ok(part) => success_response(request.id, serde_json::to_value(part).unwrap()),
                Err(err) => error_response(request.id, -32024, &err.to_string()),
            },
            Err(err) => error_response(request.id, -32602, &format!("invalid params: {err}")),
        },
        "get_package" => match serde_json::from_value::<UuidParams>(request.params) {
            Ok(params) => match engine.get_package(&params.uuid) {
                Ok(package) => success_response(request.id, serde_json::to_value(package).unwrap()),
                Err(err) => error_response(request.id, -32025, &err.to_string()),
            },
            Err(err) => error_response(request.id, -32602, &format!("invalid params: {err}")),
        },
        "get_package_change_candidates" => match serde_json::from_value::<UuidParams>(request.params) {
            Ok(params) => match engine.get_package_change_candidates(&params.uuid) {
                Ok(report) => success_response(request.id, serde_json::to_value(report).unwrap()),
                Err(err) => error_response(request.id, -32031, &err.to_string()),
            },
            Err(err) => error_response(request.id, -32602, &format!("invalid params: {err}")),
        },
        "get_part_change_candidates" => match serde_json::from_value::<UuidParams>(request.params) {
            Ok(params) => match engine.get_part_change_candidates(&params.uuid) {
                Ok(report) => success_response(request.id, serde_json::to_value(report).unwrap()),
                Err(err) => error_response(request.id, -32042, &err.to_string()),
            },
            Err(err) => error_response(request.id, -32602, &format!("invalid params: {err}")),
        },
        "get_component_replacement_plan" => {
            match serde_json::from_value::<UuidParams>(request.params) {
                Ok(params) => match engine.get_component_replacement_plan(&params.uuid) {
                    Ok(report) => success_response(request.id, serde_json::to_value(report).unwrap()),
                    Err(err) => error_response(request.id, -32043, &err.to_string()),
                },
                Err(err) => error_response(request.id, -32602, &format!("invalid params: {err}")),
            }
        }
        "get_scoped_component_replacement_plan" => {
            match serde_json::from_value::<GetScopedComponentReplacementPlanParams>(request.params)
            {
                Ok(params) => match engine.get_scoped_component_replacement_plan(
                    ScopedComponentReplacementPolicyInput {
                        scope: ComponentReplacementScope {
                            reference_prefix: params.scope.reference_prefix,
                            value_equals: params.scope.value_equals,
                            current_package_uuid: params.scope.current_package_uuid,
                            current_part_uuid: params.scope.current_part_uuid,
                        },
                        policy: params.policy,
                    },
                ) {
                    Ok(report) => success_response(request.id, serde_json::to_value(report).unwrap()),
                    Err(err) => error_response(request.id, -32049, &err.to_string()),
                },
                Err(err) => error_response(request.id, -32602, &format!("invalid params: {err}")),
            }
        }
        "get_board_summary" => match engine.get_board_summary() {
            Ok(summary) => success_response(request.id, serde_json::to_value(summary).unwrap()),
            Err(err) => error_response(request.id, -32004, &err.to_string()),
        },
        "get_components" => match engine.get_components() {
            Ok(components) => {
                success_response(request.id, serde_json::to_value(components).unwrap())
            }
            Err(err) => error_response(request.id, -32008, &err.to_string()),
        },
        "get_netlist" => match engine.get_netlist() {
            Ok(nets) => success_response(request.id, serde_json::to_value(nets).unwrap()),
            Err(err) => error_response(request.id, -32021, &err.to_string()),
        },
        "get_schematic_summary" => match engine.get_schematic_summary() {
            Ok(summary) => success_response(request.id, serde_json::to_value(summary).unwrap()),
            Err(err) => error_response(request.id, -32005, &err.to_string()),
        },
        "get_sheets" => match engine.get_sheets() {
            Ok(sheets) => success_response(request.id, serde_json::to_value(sheets).unwrap()),
            Err(err) => error_response(request.id, -32018, &err.to_string()),
        },
        "get_labels" => match engine.get_labels(None) {
            Ok(labels) => success_response(request.id, serde_json::to_value(labels).unwrap()),
            Err(err) => error_response(request.id, -32009, &err.to_string()),
        },
        "get_symbols" => match engine.get_symbols(None) {
            Ok(symbols) => success_response(request.id, serde_json::to_value(symbols).unwrap()),
            Err(err) => error_response(request.id, -32014, &err.to_string()),
        },
        "get_symbol_fields" => match serde_json::from_value::<SymbolFieldsParams>(request.params) {
            Ok(params) => match engine.get_symbol_fields(&params.symbol_uuid) {
                Ok(fields) => success_response(request.id, serde_json::to_value(fields).unwrap()),
                Err(err) => error_response(request.id, -32022, &err.to_string()),
            },
            Err(err) => error_response(request.id, -32602, &format!("invalid params: {err}")),
        },
        "get_ports" => match engine.get_ports(None) {
            Ok(ports) => success_response(request.id, serde_json::to_value(ports).unwrap()),
            Err(err) => error_response(request.id, -32010, &err.to_string()),
        },
        "get_buses" => match engine.get_buses(None) {
            Ok(buses) => success_response(request.id, serde_json::to_value(buses).unwrap()),
            Err(err) => error_response(request.id, -32012, &err.to_string()),
        },
        "get_bus_entries" => match engine.get_bus_entries(None) {
            Ok(entries) => success_response(request.id, serde_json::to_value(entries).unwrap()),
            Err(err) => error_response(request.id, -32023, &err.to_string()),
        },
        "get_noconnects" => match engine.get_noconnects(None) {
            Ok(noconnects) => {
                success_response(request.id, serde_json::to_value(noconnects).unwrap())
            }
            Err(err) => error_response(request.id, -32015, &err.to_string()),
        },
        "get_hierarchy" => match engine.get_hierarchy() {
            Ok(hierarchy) => success_response(request.id, serde_json::to_value(hierarchy).unwrap()),
            Err(err) => error_response(request.id, -32013, &err.to_string()),
        },
        "get_net_info" => match engine.get_net_info() {
            Ok(nets) => success_response(request.id, serde_json::to_value(nets).unwrap()),
            Err(err) => error_response(request.id, -32006, &err.to_string()),
        },
        "get_unrouted" => match engine.get_unrouted() {
            Ok(airwires) => success_response(request.id, serde_json::to_value(airwires).unwrap()),
            Err(err) => error_response(request.id, -32016, &err.to_string()),
        },
        "get_schematic_net_info" => match engine.get_schematic_net_info() {
            Ok(nets) => success_response(request.id, serde_json::to_value(nets).unwrap()),
            Err(err) => error_response(request.id, -32011, &err.to_string()),
        },
        "get_check_report" => match engine.get_check_report() {
            Ok(report) => success_response(request.id, serde_json::to_value(report).unwrap()),
            Err(err) => error_response(request.id, -32001, &err.to_string()),
        },
        "get_connectivity_diagnostics" => match engine.get_connectivity_diagnostics() {
            Ok(diagnostics) => {
                success_response(request.id, serde_json::to_value(diagnostics).unwrap())
            }
            Err(err) => error_response(request.id, -32003, &err.to_string()),
        },
        "get_design_rules" => match engine.get_design_rules() {
            Ok(rules) => success_response(request.id, serde_json::to_value(rules).unwrap()),
            Err(err) => error_response(request.id, -32020, &err.to_string()),
        },
        "run_erc" => match engine.run_erc_prechecks() {
            Ok(findings) => success_response(request.id, serde_json::to_value(findings).unwrap()),
            Err(err) => error_response(request.id, -32002, &err.to_string()),
        },
        "run_drc" => match engine.run_drc(&[
            RuleType::Connectivity,
            RuleType::ClearanceCopper,
            RuleType::TrackWidth,
            RuleType::ViaHole,
            RuleType::ViaAnnularRing,
            RuleType::SilkClearance,
        ]) {
            Ok(report) => success_response(request.id, serde_json::to_value(report).unwrap()),
            Err(err) => error_response(request.id, -32017, &err.to_string()),
        },
        "explain_violation" => {
            match serde_json::from_value::<ExplainViolationParams>(request.params) {
                Ok(params) => match engine.explain_violation(params.domain, params.index) {
                    Ok(explanation) => {
                        success_response(request.id, serde_json::to_value(explanation).unwrap())
                    }
                    Err(err) => error_response(request.id, -32026, &err.to_string()),
                },
                Err(err) => error_response(request.id, -32602, &format!("invalid params: {err}")),
            }
        }
        _ => error_response(request.id, -32601, "method not found"),
    }
}
