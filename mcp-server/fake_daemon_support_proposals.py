#!/usr/bin/env python3
"""Fake daemon client proposal responses for MCP tests."""

from __future__ import annotations

from server_runtime import JsonRpcResponse


def _proposal_record(proposal: str = "proposal-test", status: str = "draft") -> dict:
    return {
        "proposal_id": proposal,
        "id": proposal,
        "project_id": "project-test",
        "model_revision": "model-rev-test",
        "status": status,
        "kind": "standards_repair",
        "operation_count": 1,
        "operations": [
            {
                "operation_id": "op-test",
                "op": "set_netclass_clearance",
                "target": {"kind": "netclass", "id": "default"},
                "status": "pending",
            }
        ],
        "review": {
            "status": status,
            "reviewer": "fixture-reviewer",
            "rationale": "fixture proposal awaiting review",
        },
    }


def _proposal_review_result(proposal: str, status: str) -> dict:
    return {
        "action": "review_proposal",
        "proposal_id": proposal,
        "id": proposal,
        "project_id": "project-test",
        "model_revision": "model-rev-test",
        "status": status,
        "review_status": status,
        "review": {
            "status": status,
            "reviewer": "fixture-reviewer",
            "rationale": "fixture review transition",
        },
    }


def _proposal_validation_result(
    proposal: str, status: str = "draft", can_apply: bool = False
) -> dict:
    blockers = []
    if not can_apply:
        blockers.append(
            {
                "code": "missing_acceptance",
                "message": "proposal status is draft; expected accepted before apply",
            }
        )
    return {
        "contract": "proposal_validation_v1",
        "policy": "accepted_revision_guarded_source_policy_v1",
        "approval_path": "draft_review_accept_then_apply",
        "project_id": "project-test",
        "model_revision": "model-rev-test",
        "proposal_id": proposal,
        "status": status,
        "prepared_against": "model-rev-test",
        "prepared_against_current_model": True,
        "batch_revision_guard_matches": True,
        "acceptance_required": True,
        "current_revision_required": True,
        "revision_guard_required": True,
        "check_source_evidence_required": True,
        "can_apply": can_apply,
        "blocker_count": len(blockers),
        "blocker_codes": [blocker["code"] for blocker in blockers],
        "blockers": blockers,
    }


class FakeDaemonClientProposalsMixin:
    def create_proposal(
        self,
        path: str,
        batch: str,
        rationale: str,
        proposal: str | None = None,
        source: str | None = None,
        checks_run: list[str] | None = None,
        finding_fingerprints: list[str] | None = None,
    ) -> JsonRpcResponse:
        proposal = proposal or "proposal-test"
        self.calls.append(
            (
                "create_proposal",
                path,
                batch,
                rationale,
                proposal,
                source,
                checks_run or [],
                finding_fingerprints or [],
            )
        )
        return JsonRpcResponse(
            "2.0",
            142,
            {
                "contract": "proposal_create_v1",
                "action": "create_proposal",
                "project_root": path,
                "proposal_id": proposal,
                "proposal": _proposal_record(proposal),
                "validation": _proposal_validation_result(proposal),
            },
            None,
        )

    def create_draw_wire_proposal(
        self,
        path: str,
        sheet: str,
        from_x_nm: int,
        from_y_nm: int,
        to_x_nm: int,
        to_y_nm: int,
        proposal: str | None = None,
        rationale: str | None = None,
    ) -> JsonRpcResponse:
        proposal = proposal or "proposal-wire-test"
        self.calls.append(("create_draw_wire_proposal", path, sheet, from_x_nm, from_y_nm, to_x_nm, to_y_nm, proposal, rationale))
        return JsonRpcResponse("2.0", 151, {"contract": "proposal_create_v1", "action": "propose_draw_wire", "project_root": path, "proposal_id": proposal, "sheet_uuid": sheet}, None)

    def create_place_label_proposal(
        self,
        path: str,
        sheet: str,
        name: str,
        x_nm: int,
        y_nm: int,
        kind: str | None = None,
        proposal: str | None = None,
        rationale: str | None = None,
    ) -> JsonRpcResponse:
        proposal = proposal or "proposal-label-test"
        self.calls.append(("create_place_label_proposal", path, sheet, name, x_nm, y_nm, kind, proposal, rationale))
        return JsonRpcResponse("2.0", 152, {"contract": "proposal_create_v1", "action": "propose_place_label", "project_root": path, "proposal_id": proposal, "sheet_uuid": sheet, "name": name, "kind": kind or "local"}, None)

    def create_place_symbol_proposal(
        self,
        path: str,
        sheet: str,
        reference: str,
        value: str,
        x_nm: int,
        y_nm: int,
        lib_id: str | None = None,
        rotation_deg: int | None = None,
        mirrored: bool | None = None,
        proposal: str | None = None,
        rationale: str | None = None,
    ) -> JsonRpcResponse:
        proposal = proposal or "proposal-symbol-test"
        self.calls.append(("create_place_symbol_proposal", path, sheet, reference, value, x_nm, y_nm, lib_id, rotation_deg, mirrored, proposal, rationale))
        return JsonRpcResponse("2.0", 153, {"contract": "proposal_create_v1", "action": "propose_place_symbol", "project_root": path, "proposal_id": proposal, "sheet_uuid": sheet, "reference": reference, "value": value}, None)

    def create_board_component_replacement_proposal(
        self,
        path: str,
        component: str,
        package: str | None = None,
        part: str | None = None,
        value: str | None = None,
        proposal: str | None = None,
        rationale: str | None = None,
    ) -> JsonRpcResponse:
        proposal = proposal or "proposal-component-replacement-test"
        self.calls.append(("create_board_component_replacement_proposal", path, component, package, part, value, proposal, rationale))
        return JsonRpcResponse("2.0", 154, {"contract": "proposal_create_v1", "action": "propose_board_component_replacement", "project_root": path, "proposal_id": proposal, "component_uuid": component, "package_uuid": package, "part_uuid": part, "value": value}, None)

    def create_board_component_replacements_proposal(
        self,
        path: str,
        replacements: list[dict],
        proposal: str | None = None,
        rationale: str | None = None,
    ) -> JsonRpcResponse:
        proposal = proposal or "proposal-component-replacements-test"
        self.calls.append(("create_board_component_replacements_proposal", path, replacements, proposal, rationale))
        return JsonRpcResponse("2.0", 155, {"contract": "proposal_create_v1", "action": "propose_board_component_replacement", "project_root": path, "proposal_id": proposal, "replacement_count": len(replacements)}, None)

    def create_board_component_replacement_plan_proposal(
        self,
        path: str,
        selections: list[dict],
        proposal: str | None = None,
        rationale: str | None = None,
    ) -> JsonRpcResponse:
        proposal = proposal or "proposal-component-replacement-plan-test"
        self.calls.append(("create_board_component_replacement_plan_proposal", path, selections, proposal, rationale))
        return JsonRpcResponse("2.0", 156, {"contract": "proposal_create_v1", "action": "propose_board_component_replacement", "project_root": path, "proposal_id": proposal, "selection_count": len(selections)}, None)

    def create_pool_library_object_proposal(
        self,
        path: str,
        kind: str,
        object: str,
        from_json: str,
        pool: str | None = None,
        proposal: str | None = None,
        rationale: str | None = None,
    ) -> JsonRpcResponse:
        proposal = proposal or "proposal-pool-library-object-test"
        self.calls.append(
            (
                "create_pool_library_object_proposal",
                path,
                kind,
                object,
                from_json,
                pool,
                proposal,
                rationale,
            )
        )
        return JsonRpcResponse(
            "2.0",
            157,
            {
                "contract": "proposal_create_v1",
                "action": "create_pool_library_object_proposal",
                "project_root": path,
                "proposal_id": proposal,
                "object_kind": kind,
                "object_uuid": object,
                "pool_path": pool or "pool",
            },
            None,
        )

    def create_pool_unit_proposal(
        self,
        path: str,
        unit: str,
        name: str,
        manufacturer: str | None = None,
        pool: str | None = None,
        proposal: str | None = None,
        rationale: str | None = None,
    ) -> JsonRpcResponse:
        proposal = proposal or "proposal-pool-unit-test"
        self.calls.append(
            (
                "create_pool_unit_proposal",
                path,
                unit,
                name,
                manufacturer,
                pool,
                proposal,
                rationale,
            )
        )
        return JsonRpcResponse(
            "2.0",
            158,
            {
                "contract": "proposal_create_v1",
                "action": "create_pool_unit_proposal",
                "project_root": path,
                "proposal_id": proposal,
                "unit_uuid": unit,
                "name": name,
                "manufacturer": manufacturer or "",
                "pool_path": pool or "pool",
            },
            None,
        )

    def create_pool_symbol_proposal(
        self,
        path: str,
        symbol: str,
        unit: str,
        name: str,
        pool: str | None = None,
        proposal: str | None = None,
        rationale: str | None = None,
    ) -> JsonRpcResponse:
        proposal = proposal or "proposal-pool-symbol-test"
        self.calls.append(
            (
                "create_pool_symbol_proposal",
                path,
                symbol,
                unit,
                name,
                pool,
                proposal,
                rationale,
            )
        )
        return JsonRpcResponse(
            "2.0",
            159,
            {
                "contract": "proposal_create_v1",
                "action": "create_pool_symbol_proposal",
                "project_root": path,
                "proposal_id": proposal,
                "symbol_uuid": symbol,
                "unit_uuid": unit,
                "name": name,
                "pool_path": pool or "pool",
            },
            None,
        )

    def create_pool_entity_proposal(
        self,
        path: str,
        entity: str,
        gate: str,
        unit: str,
        symbol: str,
        name: str,
        prefix: str,
        manufacturer: str | None = None,
        gate_name: str | None = None,
        pool: str | None = None,
        proposal: str | None = None,
        rationale: str | None = None,
    ) -> JsonRpcResponse:
        proposal = proposal or "proposal-pool-entity-test"
        self.calls.append(
            (
                "create_pool_entity_proposal",
                path,
                entity,
                gate,
                unit,
                symbol,
                name,
                prefix,
                manufacturer,
                gate_name,
                pool,
                proposal,
                rationale,
            )
        )
        return JsonRpcResponse(
            "2.0",
            160,
            {
                "contract": "proposal_create_v1",
                "action": "create_pool_entity_proposal",
                "project_root": path,
                "proposal_id": proposal,
                "entity_uuid": entity,
                "gate_uuid": gate,
                "unit_uuid": unit,
                "symbol_uuid": symbol,
                "name": name,
                "prefix": prefix,
                "manufacturer": manufacturer or "",
                "gate_name": gate_name or "A",
                "pool_path": pool or "pool",
            },
            None,
        )

    def create_pool_padstack_proposal(
        self,
        path: str,
        padstack: str,
        name: str,
        aperture: str | None = None,
        diameter_nm: int | None = None,
        width_nm: int | None = None,
        height_nm: int | None = None,
        drill_nm: int | None = None,
        pool: str | None = None,
        proposal: str | None = None,
        rationale: str | None = None,
    ) -> JsonRpcResponse:
        proposal = proposal or "proposal-pool-padstack-test"
        self.calls.append(
            (
                "create_pool_padstack_proposal",
                path,
                padstack,
                name,
                aperture,
                diameter_nm,
                width_nm,
                height_nm,
                drill_nm,
                pool,
                proposal,
                rationale,
            )
        )
        return JsonRpcResponse(
            "2.0",
            161,
            {
                "contract": "proposal_create_v1",
                "action": "create_pool_padstack_proposal",
                "project_root": path,
                "proposal_id": proposal,
                "padstack_uuid": padstack,
                "name": name,
                "aperture": aperture,
                "diameter_nm": diameter_nm,
                "width_nm": width_nm,
                "height_nm": height_nm,
                "drill_nm": drill_nm,
                "pool_path": pool or "pool",
            },
            None,
        )

    def create_pool_package_proposal(
        self,
        path: str,
        package: str,
        name: str,
        pad: str,
        padstack: str,
        pad_name: str | None = None,
        x_nm: int | None = None,
        y_nm: int | None = None,
        layer: int | None = None,
        pool: str | None = None,
        proposal: str | None = None,
        rationale: str | None = None,
    ) -> JsonRpcResponse:
        proposal = proposal or "proposal-pool-package-test"
        self.calls.append(
            (
                "create_pool_package_proposal",
                path,
                package,
                name,
                pad,
                padstack,
                pad_name,
                x_nm,
                y_nm,
                layer,
                pool,
                proposal,
                rationale,
            )
        )
        return JsonRpcResponse(
            "2.0",
            162,
            {
                "contract": "proposal_create_v1",
                "action": "create_pool_package_proposal",
                "project_root": path,
                "proposal_id": proposal,
                "package_uuid": package,
                "name": name,
                "pad_uuid": pad,
                "padstack_uuid": padstack,
                "pad_name": pad_name or "1",
                "x_nm": x_nm or 0,
                "y_nm": y_nm or 0,
                "layer": layer or 1,
                "pool_path": pool or "pool",
            },
            None,
        )

    def set_pool_package_pad_proposal(
        self,
        path: str,
        package: str,
        pad: str,
        padstack: str,
        pad_name: str | None = None,
        x_nm: int | None = None,
        y_nm: int | None = None,
        layer: int | None = None,
        pool: str | None = None,
        proposal: str | None = None,
        rationale: str | None = None,
    ) -> JsonRpcResponse:
        proposal = proposal or "proposal-pool-package-pad-test"
        self.calls.append(
            (
                "set_pool_package_pad_proposal",
                path,
                package,
                pad,
                padstack,
                pad_name,
                x_nm,
                y_nm,
                layer,
                pool,
                proposal,
                rationale,
            )
        )
        return JsonRpcResponse(
            "2.0",
            163,
            {
                "contract": "proposal_create_v1",
                "action": "set_pool_package_pad_proposal",
                "project_root": path,
                "proposal_id": proposal,
                "package_uuid": package,
                "pad_uuid": pad,
                "padstack_uuid": padstack,
                "pad_name": pad_name or "1",
                "x_nm": x_nm or 0,
                "y_nm": y_nm or 0,
                "layer": layer or 1,
                "pool_path": pool or "pool",
            },
            None,
        )

    def set_pool_package_courtyard_rect_proposal(
        self,
        path: str,
        package: str,
        min_x_nm: int,
        min_y_nm: int,
        max_x_nm: int,
        max_y_nm: int,
        pool: str | None = None,
        proposal: str | None = None,
        rationale: str | None = None,
    ) -> JsonRpcResponse:
        proposal = proposal or "proposal-pool-package-courtyard-rect-test"
        self.calls.append(
            (
                "set_pool_package_courtyard_rect_proposal",
                path,
                package,
                min_x_nm,
                min_y_nm,
                max_x_nm,
                max_y_nm,
                pool,
                proposal,
                rationale,
            )
        )
        return JsonRpcResponse(
            "2.0",
            164,
            {
                "contract": "proposal_create_v1",
                "action": "set_pool_package_courtyard_rect_proposal",
                "project_root": path,
                "proposal_id": proposal,
                "package_uuid": package,
                "min_x_nm": min_x_nm,
                "min_y_nm": min_y_nm,
                "max_x_nm": max_x_nm,
                "max_y_nm": max_y_nm,
                "pool_path": pool or "pool",
            },
            None,
        )

    def set_pool_package_courtyard_polygon_proposal(
        self,
        path: str,
        package: str,
        vertices: str,
        pool: str | None = None,
        proposal: str | None = None,
        rationale: str | None = None,
    ) -> JsonRpcResponse:
        proposal = proposal or "proposal-pool-package-courtyard-polygon-test"
        self.calls.append(
            (
                "set_pool_package_courtyard_polygon_proposal",
                path,
                package,
                vertices,
                pool,
                proposal,
                rationale,
            )
        )
        return JsonRpcResponse(
            "2.0",
            165,
            {
                "contract": "proposal_create_v1",
                "action": "set_pool_package_courtyard_polygon_proposal",
                "project_root": path,
                "proposal_id": proposal,
                "package_uuid": package,
                "vertices": vertices,
                "pool_path": pool or "pool",
            },
            None,
        )

    def get_proposals(self, path: str) -> JsonRpcResponse:
        self.calls.append(("get_proposals", path))
        proposal_record = _proposal_record()
        return JsonRpcResponse(
            "2.0",
            143,
            {
                "contract": "proposals_query_v1",
                "project_root": path,
                "project_id": "project-test",
                "model_revision": "model-rev-test",
                "proposal_count": 1,
                "proposals": {"proposal-test": proposal_record},
                "proposal_records": [proposal_record],
            },
            None,
        )

    def review_proposal(self, path: str, proposal: str, status: str) -> JsonRpcResponse:
        self.calls.append(("review_proposal", path, proposal, status))
        return JsonRpcResponse("2.0", 144, _proposal_review_result(proposal, status), None)

    def show_proposal(self, path: str, proposal: str) -> JsonRpcResponse:
        self.calls.append(("show_proposal", path, proposal))
        return JsonRpcResponse(
            "2.0",
            146,
            {
                "contract": "proposal_show_v1",
                "project_root": path,
                "project_id": "project-test",
                "model_revision": "model-rev-test",
                "proposal_id": proposal,
                "proposal": _proposal_record(proposal),
                "validation": _proposal_validation_result(proposal),
            },
            None,
        )

    def preview_proposal(self, path: str, proposal: str) -> JsonRpcResponse:
        self.calls.append(("preview_proposal", path, proposal))
        return JsonRpcResponse(
            "2.0",
            156,
            {
                "contract": "proposal_preview_v1",
                "project_root": path,
                "project_id": "project-test",
                "model_revision": "model-rev-test",
                "proposal_id": proposal,
                "prepared_against": "model-rev-test",
                "preview_after_model_revision": "model-rev-preview",
                "affected_objects": ["object-test"],
                "diff": {"created": ["object-test"], "modified": [], "deleted": []},
                "validation": _proposal_validation_result(proposal),
            },
            None,
        )

    def validate_proposal(self, path: str, proposal: str) -> JsonRpcResponse:
        self.calls.append(("validate_proposal", path, proposal))
        return JsonRpcResponse(
            "2.0",
            147,
            {"project_root": path, **_proposal_validation_result(proposal)},
            None,
        )

    def defer_proposal(self, path: str, proposal: str) -> JsonRpcResponse:
        self.calls.append(("defer_proposal", path, proposal))
        return JsonRpcResponse("2.0", 148, _proposal_review_result(proposal, "deferred"), None)

    def reject_proposal(self, path: str, proposal: str) -> JsonRpcResponse:
        self.calls.append(("reject_proposal", path, proposal))
        return JsonRpcResponse("2.0", 149, _proposal_review_result(proposal, "rejected"), None)

    def accept_apply_proposal(self, path: str, proposal: str) -> JsonRpcResponse:
        self.calls.append(("accept_apply_proposal", path, proposal))
        return JsonRpcResponse(
            "2.0",
            150,
            {
                "action": "apply_proposal",
                "proposal_id": proposal,
                "id": proposal,
                "project_id": "project-test",
                "model_revision": "model-rev-test",
                "status": "applied",
                "policy": "accepted_revision_guarded_source_policy_v1",
                "approval_path": "draft_review_accept_then_apply",
                "review_status": "accepted",
                "applied_operation_count": 1,
                "transaction_id": "txn-proposal-test",
                "validation": _proposal_validation_result(proposal, "accepted", True),
            },
            None,
        )

    def apply_proposal(self, path: str, proposal: str) -> JsonRpcResponse:
        self.calls.append(("apply_proposal", path, proposal))
        return JsonRpcResponse(
            "2.0",
            145,
            {
                "action": "apply_proposal",
                "proposal_id": proposal,
                "id": proposal,
                "project_id": "project-test",
                "model_revision": "model-rev-test",
                "status": "applied",
                "policy": "accepted_revision_guarded_source_policy_v1",
                "approval_path": "draft_review_accept_then_apply",
                "applied_operation_count": 1,
                "transaction_id": "txn-proposal-test",
                "validation": _proposal_validation_result(proposal, "accepted", True),
            },
            None,
        )
