use eda_engine::api::native_write::registry::{NativeWriteContext, find_native_write_verb};
use eda_engine::api::native_write::{WriteProvenance, commit_prepared};
use eda_engine::substrate::{CommitSource, Operation, ProjectResolver};

use super::check_run_view::{
    daemon_drc_check_run_view, daemon_erc_check_run_view, explain_drc_finding_by_fingerprint,
    explain_erc_finding_by_fingerprint,
};
use super::*;

pub(super) fn dispatch_request(engine: &mut Engine, request: JsonRpcRequest) -> JsonRpcResponse {
    if request.jsonrpc != "2.0" {
        return error_response(request.id, -32600, "invalid jsonrpc version");
    }

    match request.method.as_str() {
        "open_project" => match serde_json::from_value::<OpenProjectParams>(request.params) {
            Ok(params) => match open_project(engine, &params.path) {
                Ok(report) => serialized_success_response(request.id, report),
                Err(err) => error_response(request.id, -32000, &err.to_string()),
            },
            Err(err) => error_response(request.id, -32602, &format!("invalid params: {err}")),
        },
        "close_project" => {
            engine.close_project();
            success_response(request.id, json!({"closed": true}))
        }
        // FENCE: `save` is imported-session KiCad write-back only — part of the
        // one-time converter (decision 011). It persists the in-memory imported
        // board back to KiCad text plus sidecars and is never a public mutation
        // surface for native projects; native writes commit through
        // `native.write` (single journaled commit path).
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
        "undo" => match engine.undo() {
            Ok(result) => serialized_success_response(request.id, result),
            Err(err) => error_response(request.id, -32029, &err.to_string()),
        },
        "redo" => match engine.redo() {
            Ok(result) => serialized_success_response(request.id, result),
            Err(err) => error_response(request.id, -32030, &err.to_string()),
        },
        "search_pool" => match serde_json::from_value::<SearchPoolParams>(request.params) {
            Ok(params) => match engine.search_pool(&params.query) {
                Ok(parts) => serialized_success_response(request.id, parts),
                Err(err) => error_response(request.id, -32019, &err.to_string()),
            },
            Err(err) => error_response(request.id, -32602, &format!("invalid params: {err}")),
        },
        "get_part" => match serde_json::from_value::<UuidParams>(request.params) {
            Ok(params) => match engine.get_part(&params.uuid) {
                Ok(part) => serialized_success_response(request.id, part),
                Err(err) => error_response(request.id, -32024, &err.to_string()),
            },
            Err(err) => error_response(request.id, -32602, &format!("invalid params: {err}")),
        },
        "get_package" => match serde_json::from_value::<UuidParams>(request.params) {
            Ok(params) => match engine.get_package(&params.uuid) {
                Ok(package) => serialized_success_response(request.id, package),
                Err(err) => error_response(request.id, -32025, &err.to_string()),
            },
            Err(err) => error_response(request.id, -32602, &format!("invalid params: {err}")),
        },
        "get_package_change_candidates" => {
            match serde_json::from_value::<UuidParams>(request.params) {
                Ok(params) => match engine.get_package_change_candidates(&params.uuid) {
                    Ok(report) => serialized_success_response(request.id, report),
                    Err(err) => error_response(request.id, -32031, &err.to_string()),
                },
                Err(err) => error_response(request.id, -32602, &format!("invalid params: {err}")),
            }
        }
        "get_part_change_candidates" => {
            match serde_json::from_value::<UuidParams>(request.params) {
                Ok(params) => match engine.get_part_change_candidates(&params.uuid) {
                    Ok(report) => serialized_success_response(request.id, report),
                    Err(err) => error_response(request.id, -32042, &err.to_string()),
                },
                Err(err) => error_response(request.id, -32602, &format!("invalid params: {err}")),
            }
        }
        "get_component_replacement_plan" => {
            match serde_json::from_value::<UuidParams>(request.params) {
                Ok(params) => match engine.get_component_replacement_plan(&params.uuid) {
                    Ok(report) => serialized_success_response(request.id, report),
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
                    Ok(report) => serialized_success_response(request.id, report),
                    Err(err) => error_response(request.id, -32049, &err.to_string()),
                },
                Err(err) => error_response(request.id, -32602, &format!("invalid params: {err}")),
            }
        }
        "edit_scoped_component_replacement_plan" => {
            match serde_json::from_value::<EditScopedComponentReplacementPlanParams>(request.params)
            {
                Ok(params) => match engine.edit_scoped_component_replacement_plan(
                    params.plan,
                    ScopedComponentReplacementPlanEdit {
                        exclude_component_uuids: params.exclude_component_uuids,
                        overrides: params
                            .overrides
                            .into_iter()
                            .map(|item| ScopedComponentReplacementOverride {
                                component_uuid: item.component_uuid,
                                target_package_uuid: item.target_package_uuid,
                                target_part_uuid: item.target_part_uuid,
                            })
                            .collect(),
                    },
                ) {
                    Ok(report) => serialized_success_response(request.id, report),
                    Err(err) => error_response(request.id, -32051, &err.to_string()),
                },
                Err(err) => error_response(request.id, -32602, &format!("invalid params: {err}")),
            }
        }
        // FENCE — terminally frozen imported-session compatibility arms.
        //
        // The four arms below (`apply_component_replacement_policy`,
        // `apply_scoped_component_replacement_policy`,
        // `apply_scoped_component_replacement_plan`, `set_net_class`) mutate the
        // legacy in-memory imported-board `api::Engine` without the substrate
        // commit()/journal path. They are kept only for imported-session
        // compatibility inside the one-time converter (decision 011); no
        // journaled equivalent will be built for them and they die with the
        // converter session. They are fenced hidden on the MCP side by
        // `NON_JOURNALED_DAEMON_WRITE_METHODS` (tools_catalog_data.py), locked
        // two-directionally by scripts/check_daemon_write_parity.py.
        //
        // The eleven retired sibling arms (move_component, rotate_component,
        // flip_component, set_value, set_reference, assign_part, set_package,
        // set_package_with_part, replace_component, replace_components,
        // apply_component_replacement_plan) were removed once their canonical
        // journaled replacements shipped (datum.pcb.* / datum.proposal.*).
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
                    Ok(result) => serialized_success_response(request.id, result),
                    Err(err) => error_response(request.id, -32045, &err.to_string()),
                },
                Err(err) => error_response(request.id, -32602, &format!("invalid params: {err}")),
            }
        }
        "apply_scoped_component_replacement_policy" => {
            match serde_json::from_value::<ApplyScopedComponentReplacementPolicyParams>(
                request.params,
            ) {
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
                    Ok(result) => serialized_success_response(request.id, result),
                    Err(err) => error_response(request.id, -32046, &err.to_string()),
                },
                Err(err) => error_response(request.id, -32602, &format!("invalid params: {err}")),
            }
        }
        "apply_scoped_component_replacement_plan" => {
            match serde_json::from_value::<ApplyScopedComponentReplacementPlanParams>(
                request.params,
            ) {
                Ok(params) => match engine.apply_scoped_component_replacement_plan(params.plan) {
                    Ok(result) => serialized_success_response(request.id, result),
                    Err(err) => error_response(request.id, -32047, &err.to_string()),
                },
                Err(err) => error_response(request.id, -32602, &format!("invalid params: {err}")),
            }
        }
        "set_net_class" => match serde_json::from_value::<SetNetClassInput>(request.params) {
            Ok(params) => match engine.set_net_class(params) {
                Ok(result) => serialized_success_response(request.id, result),
                Err(err) => error_response(request.id, -32048, &err.to_string()),
            },
            Err(err) => error_response(request.id, -32602, &format!("invalid params: {err}")),
        },
        "get_board_summary" => match engine.get_board_summary() {
            Ok(summary) => serialized_success_response(request.id, summary),
            Err(err) => error_response(request.id, -32004, &err.to_string()),
        },
        "get_components" => match engine.get_components() {
            Ok(components) => serialized_success_response(request.id, components),
            Err(err) => error_response(request.id, -32008, &err.to_string()),
        },
        "get_netlist" => match engine.get_netlist() {
            Ok(nets) => serialized_success_response(request.id, nets),
            Err(err) => error_response(request.id, -32021, &err.to_string()),
        },
        "get_schematic_summary" => match engine.get_schematic_summary() {
            Ok(summary) => serialized_success_response(request.id, summary),
            Err(err) => error_response(request.id, -32005, &err.to_string()),
        },
        "get_sheets" => match engine.get_sheets() {
            Ok(sheets) => serialized_success_response(request.id, sheets),
            Err(err) => error_response(request.id, -32018, &err.to_string()),
        },
        "get_labels" => match engine.get_labels(None) {
            Ok(labels) => serialized_success_response(request.id, labels),
            Err(err) => error_response(request.id, -32009, &err.to_string()),
        },
        "get_symbols" => match engine.get_symbols(None) {
            Ok(symbols) => serialized_success_response(request.id, symbols),
            Err(err) => error_response(request.id, -32014, &err.to_string()),
        },
        "get_symbol_fields" => match serde_json::from_value::<SymbolFieldsParams>(request.params) {
            Ok(params) => match engine.get_symbol_fields(&params.symbol_uuid) {
                Ok(fields) => serialized_success_response(request.id, fields),
                Err(err) => error_response(request.id, -32022, &err.to_string()),
            },
            Err(err) => error_response(request.id, -32602, &format!("invalid params: {err}")),
        },
        "get_ports" => match engine.get_ports(None) {
            Ok(ports) => serialized_success_response(request.id, ports),
            Err(err) => error_response(request.id, -32010, &err.to_string()),
        },
        "get_buses" => match engine.get_buses(None) {
            Ok(buses) => serialized_success_response(request.id, buses),
            Err(err) => error_response(request.id, -32012, &err.to_string()),
        },
        "get_bus_entries" => match engine.get_bus_entries(None) {
            Ok(entries) => serialized_success_response(request.id, entries),
            Err(err) => error_response(request.id, -32023, &err.to_string()),
        },
        "get_noconnects" => match engine.get_noconnects(None) {
            Ok(noconnects) => serialized_success_response(request.id, noconnects),
            Err(err) => error_response(request.id, -32015, &err.to_string()),
        },
        "get_hierarchy" => match engine.get_hierarchy() {
            Ok(hierarchy) => serialized_success_response(request.id, hierarchy),
            Err(err) => error_response(request.id, -32013, &err.to_string()),
        },
        "get_net_info" => match engine.get_net_info() {
            Ok(nets) => serialized_success_response(request.id, nets),
            Err(err) => error_response(request.id, -32006, &err.to_string()),
        },
        "get_unrouted" => match engine.get_unrouted() {
            Ok(airwires) => serialized_success_response(request.id, airwires),
            Err(err) => error_response(request.id, -32016, &err.to_string()),
        },
        "get_schematic_net_info" => match engine.get_schematic_net_info() {
            Ok(nets) => serialized_success_response(request.id, nets),
            Err(err) => error_response(request.id, -32011, &err.to_string()),
        },
        "get_check_report" => match engine.get_check_report() {
            Ok(report) => serialized_success_response(request.id, report),
            Err(err) => error_response(request.id, -32001, &err.to_string()),
        },
        "get_connectivity_diagnostics" => match engine.get_connectivity_diagnostics() {
            Ok(diagnostics) => serialized_success_response(request.id, diagnostics),
            Err(err) => error_response(request.id, -32003, &err.to_string()),
        },
        "get_design_rules" => match engine.get_design_rules() {
            Ok(rules) => serialized_success_response(request.id, rules),
            Err(err) => error_response(request.id, -32020, &err.to_string()),
        },
        "run_erc" => match engine.run_erc_prechecks() {
            Ok(findings) => success_response(request.id, daemon_erc_check_run_view(&findings)),
            Err(err) => error_response(request.id, -32002, &err.to_string()),
        },
        "run_drc" => match serde_json::from_value::<RunDrcParams>(request.params) {
            Ok(params) => {
                match engine.run_drc(params.rules.as_deref().unwrap_or(default_drc_rules())) {
                    Ok(report) => success_response(request.id, daemon_drc_check_run_view(&report)),
                    Err(err) => error_response(request.id, -32017, &err.to_string()),
                }
            }
            Err(err) => error_response(request.id, -32602, &format!("invalid params: {err}")),
        },
        "explain_violation" => {
            match serde_json::from_value::<ExplainViolationParams>(request.params) {
                Ok(params) => explain_violation_response(engine, request.id, params),
                Err(err) => error_response(request.id, -32602, &format!("invalid params: {err}")),
            }
        }
        "native.describe" => match serde_json::from_value::<NativeDescribeParams>(request.params) {
            Ok(params) => native_describe_response(request.id, &params.project_root),
            Err(err) => error_response(request.id, -32602, &format!("invalid params: {err}")),
        },
        "native.write" => match serde_json::from_value::<NativeWriteParams>(request.params) {
            Ok(params) => native_write_response(request.id, params),
            Err(err) => error_response(request.id, -32602, &format!("invalid params: {err}")),
        },
        _ => error_response(request.id, -32601, "method not found"),
    }
}

fn explain_violation_response(
    engine: &mut Engine,
    request_id: Value,
    params: ExplainViolationParams,
) -> JsonRpcResponse {
    if let Some(fingerprint) = params.fingerprint.as_deref() {
        return match params.domain {
            ViolationDomain::Erc => match engine.run_erc_prechecks() {
                Ok(findings) => match explain_erc_finding_by_fingerprint(&findings, fingerprint) {
                    Some(explanation) => success_response(request_id, explanation),
                    None => error_response(
                        request_id,
                        -32026,
                        &format!("erc finding fingerprint `{fingerprint}` was not found"),
                    ),
                },
                Err(err) => error_response(request_id, -32026, &err.to_string()),
            },
            ViolationDomain::Drc => match engine.run_drc(default_drc_rules()) {
                Ok(report) => match explain_drc_finding_by_fingerprint(&report, fingerprint) {
                    Some(explanation) => success_response(request_id, explanation),
                    None => error_response(
                        request_id,
                        -32026,
                        &format!("drc finding fingerprint `{fingerprint}` was not found"),
                    ),
                },
                Err(err) => error_response(request_id, -32026, &err.to_string()),
            },
        };
    }
    let Some(index) = params.index else {
        return error_response(
            request_id,
            -32602,
            "invalid params: explain_violation requires either fingerprint or index",
        );
    };
    match engine.explain_violation(params.domain, index) {
        Ok(explanation) => serialized_success_response(request_id, explanation),
        Err(err) => error_response(request_id, -32026, &err.to_string()),
    }
}

/// `native.describe`: resolve the native project at `project_root` and report
/// the stale-guard anchor a `native.write` client needs (current
/// `model_revision`), plus the manifest name and journal length.
fn native_describe_response(request_id: Value, project_root: &Path) -> JsonRpcResponse {
    match ProjectResolver::new(project_root).resolve() {
        Ok(model) => success_response(
            request_id,
            json!({
                "project_root": project_root.display().to_string(),
                "project_id": model.project.project_id,
                "project_name": model.project.name,
                "model_revision": model.model_revision.0,
                "journal_len": model.journal.len(),
            }),
        ),
        Err(err) => error_response(request_id, -32060, &err.to_string()),
    }
}

/// `native.write`: verb-addressed native mutation through the engine's
/// native-write verb registry. Resolves the project per request, enforces the
/// optional `expected_model_revision` stale guard, builds via the registered
/// verb, then either previews (`dry_run`) or commits through the one
/// journaled commit path.
///
/// Error codes: -32602 invalid params (bad source, empty reason),
/// -32060 engine resolve/build/commit failure, -32061 stale
/// `expected_model_revision`, -32062 unknown verb.
fn native_write_response(request_id: Value, params: NativeWriteParams) -> JsonRpcResponse {
    let reason = params.reason.trim().to_string();
    if reason.is_empty() {
        return error_response(
            request_id,
            -32602,
            "invalid params: native.write requires a non-empty reason",
        );
    }
    let source = match params.source.as_deref() {
        None | Some("tool") => CommitSource::Tool,
        Some("assistant") => CommitSource::Assistant,
        Some(other) => {
            return error_response(
                request_id,
                -32602,
                &format!(
                    "invalid params: native.write source must be \"tool\" or \"assistant\", got \"{other}\""
                ),
            );
        }
    };
    let actor = params
        .actor
        .unwrap_or_else(|| "datum-eda-daemon".to_string());

    let mut model = match ProjectResolver::new(&params.project_root).resolve() {
        Ok(model) => model,
        Err(err) => return error_response(request_id, -32060, &err.to_string()),
    };
    if let Some(expected) = params.expected_model_revision.as_deref()
        && expected != model.model_revision.0 {
            return error_response(
                request_id,
                -32061,
                &format!(
                    "stale expected_model_revision: expected {expected}, current {}",
                    model.model_revision.0
                ),
            );
        }
    let Some(verb) = find_native_write_verb(&params.verb) else {
        return error_response(
            request_id,
            -32062,
            &format!("unknown native write verb: {}", params.verb),
        );
    };

    let provenance = WriteProvenance::new(actor, source, reason);
    let prepared = {
        let context = NativeWriteContext {
            model: &model,
            project_root: &params.project_root,
        };
        match (verb.build)(&context, provenance, params.params) {
            Ok(prepared) => prepared,
            Err(err) => return error_response(request_id, -32060, &err.to_string()),
        }
    };

    if params.dry_run {
        let operation_kinds: Vec<String> = prepared
            .batch
            .operations
            .iter()
            .map(operation_kind)
            .collect();
        return success_response(
            request_id,
            json!({
                "verb": params.verb,
                "status": "dry_run",
                "operation_kinds": operation_kinds,
                "operation_count": prepared.batch.operations.len(),
                "primary_object_id": prepared.primary_object_id,
                "expected_model_revision": model.model_revision.0,
            }),
        );
    }

    let primary_object_id = prepared.primary_object_id;
    let operation_count = prepared.batch.operations.len();
    match commit_prepared(&mut model, &params.project_root, prepared) {
        Ok(report) => success_response(
            request_id,
            json!({
                "verb": params.verb,
                "status": "applied",
                "transaction_id": report.transaction.transaction_id,
                "before_model_revision": report.transaction.before_model_revision.0,
                "after_model_revision": report.transaction.after_model_revision.0,
                "operation_count": operation_count,
                "primary_object_id": primary_object_id,
                "journal_len": report.journal_len,
            }),
        ),
        Err(err) => error_response(request_id, -32060, &err.to_string()),
    }
}

/// The serialized `kind` tag of one operation, for dry-run previews.
fn operation_kind(operation: &Operation) -> String {
    serde_json::to_value(operation)
        .ok()
        .and_then(|value| {
            value
                .get("kind")
                .and_then(Value::as_str)
                .map(str::to_string)
        })
        .unwrap_or_else(|| "unknown".to_string())
}

fn default_drc_rules() -> &'static [RuleType] {
    &[
        RuleType::Connectivity,
        RuleType::ClearanceCopper,
        RuleType::TrackWidth,
        RuleType::ViaHole,
        RuleType::ViaAnnularRing,
        RuleType::SilkClearance,
        RuleType::ProcessAperture,
    ]
}
