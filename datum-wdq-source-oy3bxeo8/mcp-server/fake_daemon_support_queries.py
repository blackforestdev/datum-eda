#!/usr/bin/env python3
"""Fake daemon client read/query responses for MCP tests."""

from __future__ import annotations

from server_runtime import JsonRpcResponse


def _context_provenance(session: str | None, path: str | None, project_root: str | None) -> dict:
    session_id = session or "session-test"
    return {
        "context_id": "context-test",
        "session_id": session_id,
        "provenance_seed": "datum-context:session-test:context-test:model-test",
        "project_id": "project-test",
        "project_root": project_root,
        "model_revision": "model-test",
        "source_revision": "model-test",
        "actor_type": "ExternalAgent",
        "context_path": path or "/tmp/context.json",
        "event_log_path": "/tmp/native-project/.datum/tool-sessions/session-test.events.jsonl",
    }


def _active_context_commands(project_root: str | None) -> dict:
    root_arg = project_root or "$DATUM_PROJECT_ROOT"
    return {
        "artifact_list": f"datum-eda artifact list {root_arg}",
        "artifact_show": f"datum-eda artifact show {root_arg} --artifact artifact-gerber",
        "artifact_files": f"datum-eda artifact files {root_arg} --artifact artifact-gerber",
        "artifact_preview": (
            f"datum-eda artifact preview {root_arg} --artifact artifact-gerber "
            "--file build/fab/doa2526.gbr"
        ),
        "artifact_compare": (
            f"datum-eda artifact compare {root_arg} --before artifact-previous --after artifact-gerber"
        ),
        "artifact_validate": f"datum-eda artifact validate {root_arg} --artifact artifact-gerber",
        "output_job_generate": f"datum-eda artifact generate {root_arg} --output-job job-gerber",
        "output_job_start_run": (
            f"datum-eda artifact start-output-job-run {root_arg} --output-job job-gerber"
        ),
        "output_job_cancel_run": (
            f"datum-eda artifact cancel-output-job-run {root_arg} --run run-gerber-2"
        ),
        "proposal_list": f"datum-eda proposal list {root_arg}",
        "proposal_show": f"datum-eda proposal show {root_arg} --proposal proposal-repair",
        "proposal_preview": f"datum-eda proposal preview {root_arg} --proposal proposal-repair",
        "proposal_validate": f"datum-eda proposal validate {root_arg} --proposal proposal-repair",
        "proposal_review_accept": (
            f"datum-eda proposal review {root_arg} --proposal proposal-repair --status accepted"
        ),
        "proposal_review_reject": (
            f"datum-eda proposal review {root_arg} --proposal proposal-repair --status rejected"
        ),
        "proposal_defer": f"datum-eda proposal defer {root_arg} --proposal proposal-repair",
        "proposal_reject": f"datum-eda proposal reject {root_arg} --proposal proposal-repair",
        "proposal_accept_apply": (
            f"datum-eda proposal accept-apply {root_arg} --proposal proposal-repair"
        ),
        "proposal_apply": f"datum-eda proposal apply {root_arg} --proposal proposal-repair",
        "journal_list": f"datum-eda journal list {root_arg}",
        "journal_show_tip": (
            f"datum-eda journal show {root_arg} --transaction transaction-tip"
        ),
        "journal_undo": f"datum-eda journal undo {root_arg}",
        "journal_redo": f"datum-eda journal redo {root_arg}",
        "source_shards": f"datum-eda project query {root_arg} resolve-debug",
        "check_run": f"datum-eda check run {root_arg}",
        "check_list": f"datum-eda check list {root_arg}",
        "check_profiles": f"datum-eda check profiles {root_arg}",
        "check_fill_zones": f"datum-eda check fill-zones {root_arg}",
        "check_show": None,
        "check_repair_standards": None,
        "check_waive_finding": (
            f"datum-eda check waive {root_arg} --fingerprint 'sha256:selected-finding' "
            "--rationale '<rationale>'"
        ),
        "check_accept_deviation": (
            f"datum-eda check accept-deviation {root_arg} "
            "--fingerprint 'sha256:selected-finding' --rationale '<rationale>'"
        ),
    }


class FakeDaemonClientQueriesMixin:
    def datum_context_get(
        self,
        session: str | None = None,
        path: str | None = None,
        project_root: str | None = None,
    ) -> JsonRpcResponse:
        self.calls.append(("datum_context_get", session, path, project_root))
        return JsonRpcResponse(
            "2.0",
            125,
            {
                "contract": "datum_terminal_context_v1",
                "session_id": session or "session-test",
                "context_id": "context-test",
                "actor_type": "ExternalAgent",
                "capabilities": ["read", "check", "artifact", "propose", "apply-approved"],
                "project_root": project_root,
                "discovery": path,
                "visible_artifact_ids": [],
                "visible_output_job_ids": [],
                "visible_artifact_file_paths": [],
                "latest_output_job_id": "job-gerber",
                "latest_output_job_run_id": "run-gerber-2",
                "latest_output_job_artifact_id": None,
                "focused_artifact_id": "artifact-gerber",
                "focused_artifact_file_path": "build/fab/doa2526.gbr",
                "visible_proposal_ids": ["proposal-repair"],
                "latest_proposal_id": "proposal-repair",
                "accepted_transaction_tip": "transaction-tip",
                "source_shard_status": {"total": 0, "clean": 0, "dirty": 0, "missing": 0, "unknown": 0, "attention": []},
                "visible_check_run_ids": [],
                "selection_context": {"kind": "check_finding", "id": "sha256:selected-finding"},
                "active_context_commands": _active_context_commands(project_root),
                "provenance_seed": "datum-context:session-test:context-test:model-test",
                "expires_at": None,
            },
            None,
        )

    def datum_context_refresh(
        self,
        session: str | None = None,
        path: str | None = None,
        project_root: str | None = None,
    ) -> JsonRpcResponse:
        self.calls.append(("datum_context_refresh", session, path, project_root))
        return JsonRpcResponse(
            "2.0",
            126,
            {
                "contract": "datum_terminal_context_v1",
                "session_id": session or "session-test",
                "context_id": "context-test",
                "actor_type": "ExternalAgent",
                "capabilities": ["read", "check", "artifact", "propose", "apply-approved"],
                "project_root": project_root,
                "discovery": path,
                "visible_artifact_ids": [],
                "visible_output_job_ids": [],
                "visible_artifact_file_paths": [],
                "latest_output_job_id": "job-gerber",
                "latest_output_job_run_id": "run-gerber-2",
                "latest_output_job_artifact_id": None,
                "focused_artifact_id": "artifact-gerber",
                "focused_artifact_file_path": "build/fab/doa2526.gbr",
                "visible_proposal_ids": ["proposal-repair"],
                "latest_proposal_id": "proposal-repair",
                "accepted_transaction_tip": "transaction-tip",
                "source_shard_status": {"total": 0, "clean": 0, "dirty": 0, "missing": 0, "unknown": 0, "attention": []},
                "visible_check_run_ids": [],
                "selection_context": {"kind": "check_finding", "id": "sha256:selected-finding"},
                "active_context_commands": _active_context_commands(project_root),
                "provenance_seed": "datum-context:session-test:context-test:model-test",
                "expires_at": None,
                "refreshed": True,
            },
            None,
        )

    def datum_context_session_events(
        self,
        session: str | None = None,
        path: str | None = None,
        project_root: str | None = None,
        event_kind: str | None = None,
        origin: str | None = None,
        command_id: str | None = None,
        execution_id: str | None = None,
        limit: int | None = None,
    ) -> JsonRpcResponse:
        self.calls.append(("datum_context_session_events", session, path, project_root, event_kind, origin, command_id, execution_id, limit))
        context_provenance = _context_provenance(session, path, project_root)
        return JsonRpcResponse(
            "2.0",
            127,
            {
                "contract": "datum_tool_session_events_v1",
                "session_id": session or "session-test",
                "context_provenance": context_provenance,
                "context_path": path or "/tmp/context.json",
                "event_log_path": "/tmp/native-project/.datum/tool-sessions/session-test.events.jsonl",
                "event_count": 1,
                "total_event_count": 1,
                "matched_event_count": 1,
                "limit": limit,
                "filters": {"event_kind": event_kind, "origin": origin, "command_id": command_id, "execution_id": execution_id},
                "events": [
                    {
                        "event": "terminal_command_handoff",
                        "schema_version": 1,
                        "session_id": session or "session-test",
                        "command_id": "datum.artifact.generate",
                        "execution_id": execution_id or "exec-test",
                        "mcp_alias": "datum.artifact.generate",
                    }
                ],
                "project_root": project_root,
            },
            None,
        )

    def datum_context_session_activity(
        self,
        session: str | None = None,
        path: str | None = None,
        project_root: str | None = None,
        event_kind: str | None = None,
        origin: str | None = None,
        command_id: str | None = None,
        execution_id: str | None = None,
        limit: int | None = None,
    ) -> JsonRpcResponse:
        self.calls.append(("datum_context_session_activity", session, path, project_root, event_kind, origin, command_id, execution_id, limit))
        context_provenance = _context_provenance(session, path, project_root)
        return JsonRpcResponse(
            "2.0",
            128,
            {
                "contract": "datum_tool_session_activity_summary_v1",
                "session_id": session or "session-test",
                "context_provenance": context_provenance,
                "context_path": path or "/tmp/context.json",
                "event_log_path": "/tmp/native-project/.datum/tool-sessions/session-test.events.jsonl",
                "total_event_count": 4,
                "matched_event_count": 4,
                "activity_event_count": 4,
                "first_occurred_unix_ms": 1,
                "last_occurred_unix_ms": 4,
                "filters": {"event_kind": event_kind, "origin": origin, "command_id": command_id, "execution_id": execution_id},
                "limit": limit,
                "event_kinds": {"terminal_command_handoff": 1},
                "origins": {"production_terminal_command": 1},
                "terminal_io": {
                    "input_event_count": 1,
                    "output_event_count": 1,
                    "input_byte_count": 7,
                    "output_byte_count": 12,
                    "last_input_preview": "ls -al\r",
                    "last_output_preview": "total 8\n",
                },
                "activity_spans": [
                    {
                        "span_id": "span-000001",
                        "span_kind": "command",
                        "session_id": session or "session-test",
                        "start_occurred_unix_ms": 1,
                        "end_occurred_unix_ms": 4,
                        "event_count": 4,
                        "event_kinds": {"terminal_command_handoff": 1, "terminal_command_lifecycle": 2, "terminal_io": 1},
                        "execution_id": execution_id or "exec-test",
                        "handoff": {
                            "origin": "production_terminal_command",
                            "command_id": "datum.artifact.generate",
                            "execution_id": execution_id or "exec-test",
                            "mcp_alias": "datum.artifact.generate",
                            "handoff_mode": "execute",
                            "command": "datum-eda artifact generate \"$DATUM_PROJECT_ROOT\" --output-job job-1",
                            "context_provenance": context_provenance,
                        },
                        "terminal_io": {
                            "input_event_count": 1,
                            "output_event_count": 1,
                            "input_byte_count": 7,
                            "output_byte_count": 12,
                            "last_input_preview": "ls -al\r",
                            "last_output_preview": "total 8\n",
                        },
                        "lifecycle": None,
                        "command_lifecycle": {
                            "origin": "production_terminal_command",
                            "command_id": "datum.artifact.generate",
                            "execution_id": execution_id or "exec-test",
                            "command": None,
                            "lifecycle": "finished",
                            "process_exit_code": 0,
                        },
                        "end_reason": "command_finished",
                    }
                ],
                "executions": [{
                    "execution_id": execution_id or "exec-test",
                    "command_id": "datum.artifact.generate",
                    "origin": "production_terminal_command",
                    "command": "datum-eda artifact generate \"$DATUM_PROJECT_ROOT\" --output-job job-1",
                    "event_count": 4,
                    "event_kinds": {"terminal_command_handoff": 1, "terminal_command_lifecycle": 2, "terminal_io": 1},
                    "start_occurred_unix_ms": 1,
                    "end_occurred_unix_ms": 4,
                    "duration_ms": 3,
                    "lifecycle": "finished",
                    "process_exit_code": 0,
                    "context_provenance": context_provenance,
                    "terminal_io": {
                        "input_event_count": 1,
                        "output_event_count": 1,
                        "input_byte_count": 7,
                        "output_byte_count": 12,
                        "last_input_preview": "ls -al\r",
                        "last_output_preview": "total 8\n",
                    },
                }],
                "commands": [{"command_id": "datum.artifact.generate", "mcp_alias": "datum.artifact.generate", "origin": "production_terminal_command", "handoff_mode": "execute", "count": 1, "last_execution_id": execution_id or "exec-test", "last_occurred_unix_ms": 1}],
                "project_root": project_root,
            },
            None,
        )

    def get_source_shards(self, path: str) -> JsonRpcResponse:
        self.calls.append(("get_source_shards", path))
        return JsonRpcResponse(
            "2.0",
            129,
            {
                "source_shards": [
                    {
                        "path": "board/board.json",
                        "kind": "BoardRoot",
                        "authority": "AuthoredDesign",
                        "dirty_state": "Clean",
                    },
                    {
                        "path": "pool/symbols/symbol-test.json",
                        "kind": "Pool",
                        "taxon": "PoolSymbol",
                        "authority": "AuthoredDesign",
                        "dirty_state": "Clean",
                    }
                ]
            },
            None,
        )

    def validate_project(self, path: str) -> JsonRpcResponse:
        self.calls.append(("validate_project", path))
        return JsonRpcResponse(
            "2.0",
            1,
            {
                "action": "validate_project",
                "project_root": path,
                "valid": True,
                "schema_compatible": True,
                "required_files_expected": 4,
                "required_files_validated": 4,
                "checked_sheet_files": 0,
                "checked_definition_files": 0,
                "issue_count": 0,
                "issues": [],
            },
            None,
        )

    def get_output_jobs(self, path: str) -> JsonRpcResponse:
        self.calls.append(("get_output_jobs", path))
        return JsonRpcResponse(
            "2.0",
            124,
            {
                "action": "output_jobs",
                "project_root": path,
                "output_job_count": 1,
                "output_jobs": [{"id": "gerber-set-default", "kind": "gerber_set"}],
            },
            None,
        )

    def get_panel_projections(self, path: str) -> JsonRpcResponse:
        self.calls.append(("get_panel_projections", path))
        return JsonRpcResponse(
            "2.0",
            126,
            {
                "action": "panel_projections",
                "project_root": path,
                "panel_projection_count": 1,
                "panel_projections": [{"key": "main-panel", "name": "Main Panel"}],
            },
            None,
        )

    def create_panel_projection(
        self,
        path: str,
        key: str,
        name: str | None,
        board: str | None,
        x_nm: int | None,
        y_nm: int | None,
        rotation_deg: int | None,
    ) -> JsonRpcResponse:
        self.calls.append(
            (
                "create_panel_projection",
                {
                    "path": path,
                    "key": key,
                    "name": name,
                    "board": board,
                    "x_nm": x_nm,
                    "y_nm": y_nm,
                    "rotation_deg": rotation_deg,
                },
            )
        )
        return JsonRpcResponse(
            "2.0",
            127,
            {
                "action": "create_panel_projection",
                "project_root": path,
                "panel_projection": {"key": key, "name": name},
            },
            None,
        )

    def create_panel_projection_proposal(
        self, path: str, key: str, name: str | None, board: str | None, x_nm: int | None, y_nm: int | None, rotation_deg: int | None, proposal: str | None = None, rationale: str | None = None
    ) -> JsonRpcResponse:
        self.calls.append(("create_panel_projection_proposal", {"path": path, "key": key, "proposal": proposal, "rationale": rationale}))
        return JsonRpcResponse("2.0", 137, {"contract": "proposal_create_v1", "action": "propose_create_panel_projection", "project_root": path, "proposal_id": proposal or "proposal-panel-create-test", "panel_projection": {"key": key}}, None)

    def delete_panel_projection(self, path: str, panel_projection: str) -> JsonRpcResponse:
        self.calls.append(
            (
                "delete_panel_projection",
                {"path": path, "panel_projection": panel_projection},
            )
        )
        return JsonRpcResponse(
            "2.0",
            134,
            {
                "action": "delete_panel_projection",
                "project_root": path,
                "panel_projection": {"id": panel_projection},
                "created": False,
            },
            None,
        )

    def delete_panel_projection_proposal(self, path: str, panel_projection: str, proposal: str | None = None, rationale: str | None = None) -> JsonRpcResponse:
        self.calls.append(("delete_panel_projection_proposal", {"path": path, "panel_projection": panel_projection, "proposal": proposal, "rationale": rationale}))
        return JsonRpcResponse("2.0", 138, {"contract": "proposal_create_v1", "action": "propose_delete_panel_projection", "project_root": path, "proposal_id": proposal or "proposal-panel-delete-test", "panel_projection": {"id": panel_projection}}, None)

    def update_panel_projection(
        self,
        path: str,
        panel_projection: str,
        name: str | None,
        board: str | None,
        x_nm: int | None,
        y_nm: int | None,
        rotation_deg: int | None,
    ) -> JsonRpcResponse:
        self.calls.append(
            (
                "update_panel_projection",
                {
                    "path": path,
                    "panel_projection": panel_projection,
                    "name": name,
                    "board": board,
                    "x_nm": x_nm,
                    "y_nm": y_nm,
                    "rotation_deg": rotation_deg,
                },
            )
        )
        return JsonRpcResponse(
            "2.0",
            135,
            {
                "action": "update_panel_projection",
                "project_root": path,
                "panel_projection": {"id": panel_projection, "name": name},
                "created": False,
            },
            None,
        )

    def update_panel_projection_proposal(
        self, path: str, panel_projection: str, name: str | None, board: str | None, x_nm: int | None, y_nm: int | None, rotation_deg: int | None, proposal: str | None = None, rationale: str | None = None
    ) -> JsonRpcResponse:
        self.calls.append(("update_panel_projection_proposal", {"path": path, "panel_projection": panel_projection, "proposal": proposal, "rationale": rationale}))
        return JsonRpcResponse("2.0", 139, {"contract": "proposal_create_v1", "action": "propose_update_panel_projection", "project_root": path, "proposal_id": proposal or "proposal-panel-update-test", "panel_projection": {"id": panel_projection}}, None)

    def get_manufacturing_plans(self, path: str) -> JsonRpcResponse:
        self.calls.append(("get_manufacturing_plans", path))
        return JsonRpcResponse(
            "2.0",
            128,
            {
                "action": "manufacturing_plans",
                "project_root": path,
                "manufacturing_plan_count": 1,
                "manufacturing_plans": [{"prefix": "fab/doa2526", "name": "Fab"}],
            },
            None,
        )

    def create_manufacturing_plan(
        self,
        path: str,
        prefix: str,
        name: str | None,
        variant: str | None,
        panel_projection: str | None,
    ) -> JsonRpcResponse:
        self.calls.append(
            (
                "create_manufacturing_plan",
                {
                    "path": path,
                    "prefix": prefix,
                    "name": name,
                    "variant": variant,
                    "panel_projection": panel_projection,
                },
            )
        )
        return JsonRpcResponse(
            "2.0",
            129,
            {
                "action": "create_manufacturing_plan",
                "project_root": path,
                "manufacturing_plan": {
                    "prefix": prefix,
                    "name": name,
                    "panel_projection": panel_projection,
                },
            },
            None,
        )

    def create_manufacturing_plan_proposal(
        self, path: str, prefix: str, name: str | None, variant: str | None, panel_projection: str | None, proposal: str | None = None, rationale: str | None = None
    ) -> JsonRpcResponse:
        self.calls.append(("create_manufacturing_plan_proposal", {"path": path, "prefix": prefix, "proposal": proposal, "rationale": rationale}))
        return JsonRpcResponse("2.0", 140, {"contract": "proposal_create_v1", "action": "propose_create_manufacturing_plan", "project_root": path, "proposal_id": proposal or "proposal-plan-create-test", "manufacturing_plan": {"prefix": prefix}}, None)

    def delete_manufacturing_plan(self, path: str, manufacturing_plan: str) -> JsonRpcResponse:
        self.calls.append(
            (
                "delete_manufacturing_plan",
                {"path": path, "manufacturing_plan": manufacturing_plan},
            )
        )
        return JsonRpcResponse(
            "2.0",
            133,
            {
                "action": "delete_manufacturing_plan",
                "project_root": path,
                "manufacturing_plan": {"id": manufacturing_plan},
                "created": False,
            },
            None,
        )

    def delete_manufacturing_plan_proposal(self, path: str, manufacturing_plan: str, proposal: str | None = None, rationale: str | None = None) -> JsonRpcResponse:
        self.calls.append(("delete_manufacturing_plan_proposal", {"path": path, "manufacturing_plan": manufacturing_plan, "proposal": proposal, "rationale": rationale}))
        return JsonRpcResponse("2.0", 141, {"contract": "proposal_create_v1", "action": "propose_delete_manufacturing_plan", "project_root": path, "proposal_id": proposal or "proposal-plan-delete-test", "manufacturing_plan": {"id": manufacturing_plan}}, None)

    def update_manufacturing_plan(
        self,
        path: str,
        manufacturing_plan: str,
        name: str | None,
        prefix: str | None,
        variant: str | None,
        clear_variant: bool | None,
        panel_projection: str | None,
        clear_panel_projection: bool | None,
    ) -> JsonRpcResponse:
        self.calls.append(
            (
                "update_manufacturing_plan",
                {
                    "path": path,
                    "manufacturing_plan": manufacturing_plan,
                    "name": name,
                    "prefix": prefix,
                    "variant": variant,
                    "clear_variant": clear_variant,
                    "panel_projection": panel_projection,
                    "clear_panel_projection": clear_panel_projection,
                },
            )
        )
        return JsonRpcResponse(
            "2.0",
            136,
            {
                "action": "update_manufacturing_plan",
                "project_root": path,
                "manufacturing_plan": {"id": manufacturing_plan, "prefix": prefix},
                "created": False,
            },
            None,
        )

    def update_manufacturing_plan_proposal(
        self, path: str, manufacturing_plan: str, name: str | None, prefix: str | None, variant: str | None, clear_variant: bool | None, panel_projection: str | None, clear_panel_projection: bool | None, proposal: str | None = None, rationale: str | None = None
    ) -> JsonRpcResponse:
        self.calls.append(("update_manufacturing_plan_proposal", {"path": path, "manufacturing_plan": manufacturing_plan, "proposal": proposal, "rationale": rationale}))
        return JsonRpcResponse("2.0", 142, {"contract": "proposal_create_v1", "action": "propose_update_manufacturing_plan", "project_root": path, "proposal_id": proposal or "proposal-plan-update-test", "manufacturing_plan": {"id": manufacturing_plan}}, None)

    def create_gerber_output_job(
        self,
        path: str,
        prefix: str,
        name: str | None,
        manufacturing_plan: str | None,
        output_dir: str | None = None,
        variant: str | None = None,
    ) -> JsonRpcResponse:
        self.calls.append(
            (
                "create_gerber_output_job",
                {
                    "path": path,
                    "prefix": prefix,
                    "name": name,
                    "manufacturing_plan": manufacturing_plan,
                    "output_dir": output_dir,
                    **({"variant": variant} if variant is not None else {}),
                },
            )
        )
        return JsonRpcResponse(
            "2.0",
            125,
            {
                "action": "create_gerber_output_job",
                "project_root": path,
                "output_job": {
                    "id": "gerber-set-default",
                    "prefix": prefix,
                    "manufacturing_plan": manufacturing_plan,
                    "output_dir": output_dir,
                    **({"variant": variant} if variant is not None else {}),
                },
            },
            None,
        )

    def create_output_job(
        self,
        path: str,
        prefix: str,
        include: str,
        name: str | None,
        manufacturing_plan: str | None,
        output_dir: str | None = None,
        variant: str | None = None,
    ) -> JsonRpcResponse:
        self.calls.append(
            (
                "create_output_job",
                {
                    "path": path,
                    "prefix": prefix,
                    "include": include,
                    "name": name,
                    "manufacturing_plan": manufacturing_plan,
                    "output_dir": output_dir,
                    **({"variant": variant} if variant is not None else {}),
                },
            )
        )
        return JsonRpcResponse(
            "2.0",
            126,
            {
                "action": "create_output_job",
                "project_root": path,
                "output_job": {
                    "id": f"{include}-default",
                    "prefix": prefix,
                    "include": [include.replace("-", "_")],
                    "manufacturing_plan": manufacturing_plan,
                    "output_dir": output_dir,
                    **({"variant": variant} if variant is not None else {}),
                },
            },
            None,
        )

    def create_output_job_proposal(
        self,
        path: str,
        prefix: str,
        include: str,
        name: str | None,
        manufacturing_plan: str | None,
        output_dir: str | None = None,
        proposal: str | None = None,
        rationale: str | None = None,
        variant: str | None = None,
    ) -> JsonRpcResponse:
        self.calls.append(
            (
                "create_output_job_proposal",
                {
                    "path": path,
                    "prefix": prefix,
                    "include": include,
                    "name": name,
                    "manufacturing_plan": manufacturing_plan,
                    "output_dir": output_dir,
                    **({"variant": variant} if variant is not None else {}),
                    "proposal": proposal,
                    "rationale": rationale,
                },
            )
        )
        return JsonRpcResponse(
            "2.0",
            127,
            {
                "contract": "proposal_create_v1",
                "action": "propose_create_output_job",
                "project_root": path,
                "proposal_id": proposal or "proposal-output-job-create-test",
                "output_job": {"id": f"{include}-default", "prefix": prefix, "variant": variant},
            },
            None,
        )

    def update_output_job(
        self,
        path: str,
        output_job: str,
        name: str | None,
        output_dir: str | None,
        manufacturing_plan: str | None,
        clear_manufacturing_plan: bool | None,
        clear_output_dir: bool | None = None,
        variant: str | None = None,
        clear_variant: bool | None = None,
    ) -> JsonRpcResponse:
        self.calls.append(
            (
                "update_output_job",
                {
                    "path": path,
                    "output_job": output_job,
                    "name": name,
                    "manufacturing_plan": manufacturing_plan,
                    "clear_manufacturing_plan": clear_manufacturing_plan,
                    "output_dir": output_dir,
                    "clear_output_dir": clear_output_dir,
                    **({"variant": variant} if variant is not None else {}),
                    **({"clear_variant": clear_variant} if clear_variant is not None else {}),
                },
            )
        )
        return JsonRpcResponse(
            "2.0",
            132,
            {
                "action": "update_output_job",
                "project_root": path,
                "output_job": {
                    "id": output_job,
                    "name": name,
                    "manufacturing_plan": manufacturing_plan,
                    "variant": variant,
                },
                "created": False,
            },
            None,
        )

    def update_output_job_proposal(
        self,
        path: str,
        output_job: str,
        name: str | None,
        output_dir: str | None,
        manufacturing_plan: str | None,
        clear_manufacturing_plan: bool | None,
        clear_output_dir: bool | None = None,
        proposal: str | None = None,
        rationale: str | None = None,
        variant: str | None = None,
        clear_variant: bool | None = None,
    ) -> JsonRpcResponse:
        self.calls.append(
            (
                "update_output_job_proposal",
                {
                    "path": path,
                    "output_job": output_job,
                    "name": name,
                    "manufacturing_plan": manufacturing_plan,
                    "clear_manufacturing_plan": clear_manufacturing_plan,
                    "output_dir": output_dir,
                    "clear_output_dir": clear_output_dir,
                    **({"variant": variant} if variant is not None else {}),
                    **({"clear_variant": clear_variant} if clear_variant is not None else {}),
                    "proposal": proposal,
                    "rationale": rationale,
                },
            )
        )
        return JsonRpcResponse(
            "2.0",
            133,
            {
                "contract": "proposal_create_v1",
                "action": "propose_update_output_job",
                "project_root": path,
                "proposal_id": proposal or "proposal-output-job-test",
                "output_job": {"id": output_job, "name": name, "variant": variant},
            },
            None,
        )

    def delete_output_job_proposal(
        self,
        path: str,
        output_job: str,
        proposal: str | None = None,
        rationale: str | None = None,
    ) -> JsonRpcResponse:
        self.calls.append(
            (
                "delete_output_job_proposal",
                {
                    "path": path,
                    "output_job": output_job,
                    "proposal": proposal,
                    "rationale": rationale,
                },
            )
        )
        return JsonRpcResponse(
            "2.0",
            134,
            {
                "contract": "proposal_create_v1",
                "action": "propose_delete_output_job",
                "project_root": path,
                "proposal_id": proposal or "proposal-output-job-delete-test",
                "output_job": {"id": output_job},
            },
            None,
        )

    def run_output_job(
        self, path: str, output_job: str, output_dir: str | None = None
    ) -> JsonRpcResponse:
        self.calls.append(
            (
                "run_output_job",
                {
                    "path": path,
                    "output_job": output_job,
                    "output_dir": output_dir,
                },
            )
        )
        return JsonRpcResponse(
            "2.0",
            132,
            {
                "contract": "output_job_run_v1",
                "action": "run_output_job",
                "project_root": path,
                "output_job": {"id": output_job},
                "output_dir": output_dir,
                "artifact_report": {"generated_count": 1},
            },
            None,
        )

    def start_output_job_run(self, path: str, output_job: str) -> JsonRpcResponse:
        self.calls.append(
            (
                "start_output_job_run",
                {
                    "path": path,
                    "output_job": output_job,
                },
            )
        )
        return JsonRpcResponse(
            "2.0",
            132,
            {
                "contract": "output_job_run_lifecycle_v1",
                "action": "start_output_job_run",
                "project_root": path,
                "output_job": {"id": output_job},
                "output_job_run": {"run_id": "run-test", "status": "running"},
            },
            None,
        )

    def cancel_output_job_run(self, path: str, run: str) -> JsonRpcResponse:
        self.calls.append(
            (
                "cancel_output_job_run",
                {
                    "path": path,
                    "run": run,
                },
            )
        )
        return JsonRpcResponse(
            "2.0",
            132,
            {
                "contract": "output_job_run_lifecycle_v1",
                "action": "cancel_output_job_run",
                "project_root": path,
                "output_job_run": {"run_id": run, "status": "canceled"},
            },
            None,
        )

    def delete_output_job(self, path: str, output_job: str) -> JsonRpcResponse:
        self.calls.append(
            (
                "delete_output_job",
                {
                    "path": path,
                    "output_job": output_job,
                },
            )
        )
        return JsonRpcResponse(
            "2.0",
            133,
            {
                "action": "delete_output_job",
                "project_root": path,
                "output_job": {
                    "id": output_job,
                },
                "created": False,
            },
            None,
        )

    def generate_artifacts(
        self,
        path: str,
        output_dir: str | None,
        include: str | None,
        prefix: str | None,
        output_job: str | None = None,
    ) -> JsonRpcResponse:
        self.calls.append(
            (
                "generate_artifacts",
                {
                    "path": path,
                    "output_dir": output_dir,
                    "include": include,
                    "prefix": prefix,
                    "output_job": output_job,
                },
            )
        )
        if output_job is not None:
            return JsonRpcResponse(
                "2.0",
                138,
                {
                    "contract": "output_job_run_v1",
                    "action": "run_output_job",
                    "project_root": path,
                    "output_job": {"id": output_job},
                    "status": "succeeded",
                    "exit_code": 0,
                },
                None,
            )
        return JsonRpcResponse(
            "2.0",
            138,
            {
                "contract": "artifact_generate_v1",
                "action": "generate_artifacts",
                "project_root": path,
                "project_id": "project-test",
                "model_revision": "model-test",
                "output_dir": output_dir,
                "include": include.split(",") if include else [],
                "generated_count": 1,
                "generated": [
                    {
                        "include": include.split(",")[0],
                        "artifact_id": "artifact-test",
                        "kind": "gerber_set",
                        "project_id": "project-test",
                        "model_revision": "model-test",
                        "output_job": "output-job-test",
                        "variant": "variant-test",
                        "generator_version": "datum-test",
                        "file_count": 4,
                        "report": {
                            "artifact_metadata": {
                                "artifact_id": "artifact-test",
                                "kind": "gerber_set",
                                "project_id": "project-test",
                                "model_revision": "model-test",
                                "output_job": "output-job-test",
                                "variant": "variant-test",
                                "generator_version": "datum-test",
                                "output_dir": output_dir,
                                "validation_state": "not_validated",
                                "files": [
                                    {
                                        "path": "fabrication/board-F_Cu.gbr",
                                        "sha256": "sha256:abc123",
                                    }
                                ],
                            }
                        },
                    }
                ],
            },
            None,
        )

    def get_artifacts(self, path: str) -> JsonRpcResponse:
        self.calls.append(("get_artifacts", path))
        return JsonRpcResponse(
            "2.0",
            134,
            {
                "action": "artifact_metadata_list",
                "contract": "datum.artifact_metadata_list.v1",
                "project_root": path,
                "project_id": "project-test",
                "model_revision": "model-test",
                "artifact_count": 1,
                "artifacts": [
                    {
                        "artifact_id": "artifact-test",
                        "kind": "manufacturing_set",
                        "project_id": "project-test",
                        "model_revision": "model-test",
                        "generator_version": "datum-test",
                        "output_dir": "/tmp/fab",
                        "validation_state": "valid",
                        "files": [{"path": "fab/doa2526.gbr", "sha256": "sha256-test"}],
                    }
                ],
                "artifact_run_count": 1,
                "artifact_runs": [{"artifact_id": "artifact-test", "status": "succeeded"}],
            },
            None,
        )

    def show_artifact(self, path: str, artifact: str) -> JsonRpcResponse:
        self.calls.append(("show_artifact", {"path": path, "artifact": artifact}))
        return JsonRpcResponse(
            "2.0",
            135,
            {
                "contract": "artifact_metadata_v1",
                "project_root": path,
                "artifact": {
                    "artifact_id": artifact,
                    "kind": "manufacturing_set",
                    "project_id": "project-test",
                    "model_revision": "model-test",
                    "generator_version": "datum-test",
                    "output_dir": "/tmp/fab",
                    "validation_state": "valid",
                    "files": [{"path": "fab/doa2526.gbr", "sha256": "sha256-test"}],
                },
                "run_count": 1,
                "latest_run": {"artifact_id": artifact, "status": "succeeded"},
                "runs": [{"artifact_id": artifact, "status": "succeeded"}],
            },
            None,
        )

    def get_artifact_files(self, path: str, artifact: str) -> JsonRpcResponse:
        self.calls.append(("get_artifact_files", {"path": path, "artifact": artifact}))
        return JsonRpcResponse(
            "2.0",
            139,
            {
                "contract": "artifact_files_v1",
                "project_root": path,
                "project_id": "project-test",
                "model_revision": "model-test",
                "artifact_id": artifact,
                "kind": "manufacturing_set",
                "output_dir": "/tmp/fab",
                "generator_version": "datum-test",
                "validation_state": "valid",
                "file_count": 1,
                "files": [{"path": "fab/doa2526.gbr", "sha256": "sha256-test"}],
                "production_projection_count": 1,
                "production_projections": [
                    {
                        "projection_kind": "gerber_copper",
                        "projection_contract": "gerber_copper_projection_v1",
                        "model_revision": "model-test",
                        "byte_count": 42,
                        "sha256": "projection-sha256-test",
                    }
                ],
            },
            None,
        )

    def preview_artifact_file(
        self, path: str, artifact: str, artifact_dir: str | None, file: str
    ) -> JsonRpcResponse:
        self.calls.append(
            (
                "preview_artifact_file",
                {
                    "path": path,
                    "artifact": artifact,
                    "artifact_dir": artifact_dir,
                    "file": file,
                },
            )
        )
        return JsonRpcResponse(
            "2.0",
            140,
            {
                "contract": "artifact_file_preview_v1",
                "project_root": path,
                "artifact_id": artifact,
                "file": file,
                "file_path": f"{artifact_dir or '/tmp/fab'}/{file}",
                "hash_matches_metadata": True,
                "preview_kind": "gerber_rs274x",
                "preview_available": True,
                "inspection": {"geometry_count": 1},
            },
            None,
        )

    def compare_artifacts(self, path: str, before: str, after: str) -> JsonRpcResponse:
        self.calls.append(
            ("compare_artifacts", {"path": path, "before": before, "after": after})
        )
        return JsonRpcResponse(
            "2.0",
            136,
            {
                "contract": "artifact_metadata_compare_v1",
                "project_root": path,
                "before_artifact_id": before,
                "after_artifact_id": after,
                "equivalent": False,
                "files_equal": False,
            },
            None,
        )

    def validate_artifact(self, path: str, artifact: str) -> JsonRpcResponse:
        self.calls.append(("validate_artifact", {"path": path, "artifact": artifact}))
        return JsonRpcResponse(
            "2.0",
            137,
            {
                "contract": "artifact_metadata_validation_v1",
                "project_root": path,
                "artifact_id": artifact,
                "valid": True,
                "validation_state": "valid",
            },
            None,
        )

    def export_manufacturing_set(
        self, path: str, output_dir: str, prefix: str | None
    ) -> JsonRpcResponse:
        self.calls.append(
            (
                "export_manufacturing_set",
                {"path": path, "output_dir": output_dir, "prefix": prefix},
            )
        )
        return JsonRpcResponse(
            "2.0",
            130,
            {
                "action": "export_manufacturing_set",
                "project_root": path,
                "project_id": "project-test",
                "model_revision": "model-test",
                "output_dir": output_dir,
                "prefix": prefix,
                "artifact_manifest_path": f"{output_dir}/datum-artifact-manifest.json",
                "artifact_metadata": {
                    "artifact_id": "artifact-test",
                    "kind": "manufacturing_set",
                    "project_id": "project-test",
                    "model_revision": "model-test",
                    "generator_version": "datum-test",
                    "output_dir": output_dir,
                    "file_count": 4,
                },
                "output_job_run": {
                    "output_job_kind": "manufacturing_set",
                    "status": "succeeded",
                },
            },
            None,
        )

    def validate_manufacturing_set(
        self, path: str, output_dir: str, prefix: str | None
    ) -> JsonRpcResponse:
        self.calls.append(
            (
                "validate_manufacturing_set",
                {"path": path, "output_dir": output_dir, "prefix": prefix},
            )
        )
        return JsonRpcResponse(
            "2.0",
            131,
            {
                "action": "validate_manufacturing_set",
                "project_root": path,
                "output_dir": output_dir,
                "prefix": prefix,
                "valid": True,
                "artifact_validation_state": "valid",
                "artifact_file_hash_mismatch_count": 0,
                "artifact_manifest_path": f"{output_dir}/datum-artifact-manifest.json",
            },
            None,
        )

    def get_check_report(self) -> JsonRpcResponse:
        self.calls.append(("get_check_report", None))
        return JsonRpcResponse(
            "2.0",
            2,
            {
                "domain": "board",
                "summary": {
                    "status": "warning",
                    "errors": 0,
                    "warnings": 1,
                    "infos": 1,
                    "waived": 0,
                    "by_code": [
                        {"code": "partially_routed_net", "count": 1},
                        {"code": "net_without_copper", "count": 1},
                    ],
                },
                "diagnostics": [
                    {"kind": "partially_routed_net", "severity": "warning"},
                    {"kind": "net_without_copper", "severity": "info"},
                ],
            },
            None,
        )

    def get_board_summary(self) -> JsonRpcResponse:
        self.calls.append(("get_board_summary", None))
        return JsonRpcResponse(
            "2.0",
            20,
            {
                "name": "simple-demo",
                "layer_count": 3,
                "component_count": 1,
                "net_count": 2,
            },
            None,
        )

    def get_schematic_summary(self) -> JsonRpcResponse:
        self.calls.append(("get_schematic_summary", None))
        return JsonRpcResponse(
            "2.0",
            21,
            {
                "sheet_count": 1,
                "symbol_count": 1,
                "net_label_count": 3,
                "port_count": 1,
            },
            None,
        )

    def get_schematic_wires(self, path: str) -> JsonRpcResponse:
        self.calls.append(("get_schematic_wires", path))
        return JsonRpcResponse("2.0", 212, [{"uuid": "wire-test", "from": {"x": 1, "y": 2}, "to": {"x": 3, "y": 4}}], None)

    def get_schematic_junctions(self, path: str) -> JsonRpcResponse:
        self.calls.append(("get_schematic_junctions", path))
        return JsonRpcResponse("2.0", 214, [{"uuid": "junction-test", "position": {"x": 1, "y": 2}}], None)

    def get_schematic_labels(self, path: str) -> JsonRpcResponse:
        self.calls.append(("get_schematic_labels", path))
        return JsonRpcResponse("2.0", 217, [{"uuid": "label-test", "name": "VIN", "kind": "Global", "position": {"x": 1, "y": 2}}], None)

    def get_schematic_ports(self, path: str) -> JsonRpcResponse:
        self.calls.append(("get_schematic_ports", path))
        return JsonRpcResponse("2.0", 218, [{"uuid": "port-test", "name": "SUB_IN", "direction": "Input", "position": {"x": 1, "y": 2}}], None)

    def get_schematic_noconnects(self, path: str) -> JsonRpcResponse:
        self.calls.append(("get_schematic_noconnects", path))
        return JsonRpcResponse("2.0", 219, [{"uuid": "noconnect-test", "symbol": "symbol-test", "pin": "pin-test", "position": {"x": 1, "y": 2}}], None)

    def get_schematic_buses(self, path: str) -> JsonRpcResponse:
        self.calls.append(("get_schematic_buses", path))
        return JsonRpcResponse("2.0", 220, [{"uuid": "bus-test", "name": "DATA", "members": ["DATA0", "DATA1"]}], None)

    def get_schematic_bus_entries(self, path: str) -> JsonRpcResponse:
        self.calls.append(("get_schematic_bus_entries", path))
        return JsonRpcResponse("2.0", 221, [{"uuid": "bus-entry-test", "bus": "bus-test", "wire": "wire-test", "position": {"x": 1, "y": 2}}], None)

    def get_schematic_texts(self, path: str) -> JsonRpcResponse:
        self.calls.append(("get_schematic_texts", path))
        return JsonRpcResponse("2.0", 222, [{"uuid": "text-test", "text": "note", "position": {"x": 1, "y": 2}, "rotation": 90}], None)

    def get_schematic_drawings(self, path: str) -> JsonRpcResponse:
        self.calls.append(("get_schematic_drawings", path))
        return JsonRpcResponse("2.0", 223, [{"uuid": "drawing-test", "kind": "line", "from": {"x": 1, "y": 2}, "to": {"x": 3, "y": 4}}], None)

    def get_board_tracks(self, path: str) -> JsonRpcResponse:
        self.calls.append(("get_board_tracks", path))
        return JsonRpcResponse("2.0", 213, [{"uuid": "track-test", "net": "net-test", "from": {"x": 1, "y": 2}, "to": {"x": 3, "y": 4}, "width": 5, "layer": 1}], None)

    def get_board_vias(self, path: str) -> JsonRpcResponse:
        self.calls.append(("get_board_vias", path))
        return JsonRpcResponse("2.0", 215, [{"uuid": "via-test", "net": "net-test", "position": {"x": 1, "y": 2}, "drill": 3, "diameter": 4, "from_layer": 1, "to_layer": 2}], None)

    def get_board_pads(self, path: str) -> JsonRpcResponse:
        self.calls.append(("get_board_pads", path))
        return JsonRpcResponse("2.0", 216, [{"uuid": "pad-test", "package": "package-test", "name": "1", "position": {"x": 1, "y": 2}, "layer": 1}], None)

    def get_board_zones(self, path: str) -> JsonRpcResponse:
        self.calls.append(("get_board_zones", path))
        return JsonRpcResponse("2.0", 224, [{"uuid": "zone-test", "net": "net-test", "polygon": {"vertices": [{"x": 0, "y": 0}, {"x": 1, "y": 0}, {"x": 1, "y": 1}], "closed": True}, "layer": 1}], None)
    def get_board_texts(self, path: str) -> JsonRpcResponse:
        self.calls.append(("get_board_texts", path))
        return JsonRpcResponse("2.0", 225, [{"uuid": "board-text-test", "text": "REF**", "position": {"x": 1, "y": 2}, "layer": 1}], None)
    def get_board_keepouts(self, path: str) -> JsonRpcResponse:
        self.calls.append(("get_board_keepouts", path))
        return JsonRpcResponse("2.0", 226, [{"uuid": "keepout-test", "kind": "copper", "layers": [1], "polygon": {"vertices": [{"x": 0, "y": 0}, {"x": 1, "y": 0}, {"x": 1, "y": 1}], "closed": True}}], None)
    def get_board_outline(self, path: str) -> JsonRpcResponse:
        self.calls.append(("get_board_outline", path))
        return JsonRpcResponse("2.0", 227, {"vertices": [{"x": 0, "y": 0}, {"x": 1, "y": 0}, {"x": 1, "y": 1}], "closed": True}, None)
    def get_board_stackup(self, path: str) -> JsonRpcResponse:
        self.calls.append(("get_board_stackup", path))
        return JsonRpcResponse("2.0", 228, [{"id": 1, "name": "Top", "layer_type": "Copper", "thickness_nm": 35000}], None)
    def get_board_dimensions(self, path: str) -> JsonRpcResponse:
        self.calls.append(("get_board_dimensions", path))
        return JsonRpcResponse("2.0", 229, [{"uuid": "dimension-test", "from": {"x": 0, "y": 0}, "to": {"x": 1, "y": 1}, "layer": 41}], None)
    def get_board_nets(self, path: str) -> JsonRpcResponse:
        self.calls.append(("get_board_nets", path))
        return JsonRpcResponse("2.0", 230, [{"uuid": "net-test", "name": "GND", "class": "class-test"}], None)
    def get_board_net_classes(self, path: str) -> JsonRpcResponse:
        self.calls.append(("get_board_net_classes", path))
        return JsonRpcResponse("2.0", 231, [{"uuid": "class-test", "name": "Default", "clearance": 150000}], None)

    def get_sheets(self) -> JsonRpcResponse:
        self.calls.append(("get_sheets", None))
        return JsonRpcResponse(
            "2.0",
            32,
            [
                {"name": "Root", "symbols": 1, "ports": 1, "labels": 3, "buses": 1},
            ],
            None,
        )

    def get_net_info(self) -> JsonRpcResponse:
        self.calls.append(("get_net_info", None))
        return JsonRpcResponse(
            "2.0",
            22,
            [
                {"name": "GND", "tracks": 1, "vias": 1, "zones": 0},
                {"name": "VCC", "tracks": 0, "vias": 0, "zones": 0},
            ],
            None,
        )

    def get_unrouted(self) -> JsonRpcResponse:
        self.calls.append(("get_unrouted", None))
        return JsonRpcResponse(
            "2.0",
            31,
            [
                {
                    "net_name": "SIG",
                    "from": {"component": "R1", "pin": "1"},
                    "to": {"component": "R2", "pin": "1"},
                    "distance_nm": 20000000,
                }
            ],
            None,
        )

    def get_components(self) -> JsonRpcResponse:
        self.calls.append(("get_components", None))
        return JsonRpcResponse(
            "2.0",
            24,
            [
                {
                    "uuid": "comp-1",
                    "package_uuid": "00000000-0000-0000-0000-000000000000",
                    "reference": "R1",
                    "value": "10k",
                    "footprint": "Resistor_SMD:R_0603_1608Metric",
                }
            ],
            None,
        )

    def get_netlist(self) -> JsonRpcResponse:
        self.calls.append(("get_netlist", None))
        return JsonRpcResponse(
            "2.0",
            103,
            [
                {
                    "uuid": "11111111-1111-1111-1111-111111111111",
                    "name": "GND",
                    "class": "Default",
                    "pins": [{"component": "R1", "pin": "2"}],
                    "routed_pct": 1.0,
                    "labels": None,
                    "ports": None,
                    "sheets": None,
                    "semantic_class": None,
                }
            ],
            None,
        )

    def get_schematic_net_info(self) -> JsonRpcResponse:
        self.calls.append(("get_schematic_net_info", None))
        return JsonRpcResponse(
            "2.0",
            23,
            [
                {"name": "SCL", "labels": 1, "ports": 0},
                {"name": "VCC", "labels": 1, "ports": 0},
            ],
            None,
        )

    def get_labels(self) -> JsonRpcResponse:
        self.calls.append(("get_labels", None))
        return JsonRpcResponse(
            "2.0",
            25,
            [
                {"name": "SCL"},
                {"name": "VCC"},
                {"name": "SUB_IN"},
            ],
            None,
        )

    def get_symbols(self) -> JsonRpcResponse:
        self.calls.append(("get_symbols", None))
        return JsonRpcResponse(
            "2.0",
            29,
            [
                {"reference": "R1", "value": "10k"},
            ],
            None,
        )

    def get_symbol_fields(self, symbol_uuid: str) -> JsonRpcResponse:
        self.calls.append(("get_symbol_fields", symbol_uuid))
        return JsonRpcResponse(
            "2.0",
            104,
            [
                {"uuid": "f1", "symbol": symbol_uuid, "key": "Reference", "value": "R1"},
                {"uuid": "f2", "symbol": symbol_uuid, "key": "Value", "value": "10k"},
            ],
            None,
        )

    def get_ports(self) -> JsonRpcResponse:
        self.calls.append(("get_ports", None))
        return JsonRpcResponse(
            "2.0",
            26,
            [
                {"name": "SUB_IN"},
            ],
            None,
        )

    def get_buses(self) -> JsonRpcResponse:
        self.calls.append(("get_buses", None))
        return JsonRpcResponse(
            "2.0",
            27,
            [
                {"name": "DATA", "members": ["SCL", "SDA"]},
            ],
            None,
        )

    def get_bus_entries(self) -> JsonRpcResponse:
        self.calls.append(("get_bus_entries", None))
        return JsonRpcResponse(
            "2.0",
            105,
            [{"uuid": "be1", "sheet": "s1", "bus": "b1", "wire": None}],
            None,
        )

    def get_noconnects(self) -> JsonRpcResponse:
        self.calls.append(("get_noconnects", None))
        return JsonRpcResponse(
            "2.0",
            30,
            [
                {"symbol": "R1", "pin": "2"},
            ],
            None,
        )

    def get_hierarchy(self) -> JsonRpcResponse:
        self.calls.append(("get_hierarchy", None))
        return JsonRpcResponse(
            "2.0",
            28,
            {
                "instances": [{"name": "child"}],
                "links": [],
            },
            None,
        )

    def get_project_hierarchy(self, path: str | None = None) -> JsonRpcResponse:
        self.calls.append(("get_project_hierarchy", path))
        return JsonRpcResponse(
            "2.0",
            2801,
            {
                "instances": [
                    {
                        "uuid": "instance-1",
                        "definition": "definition-1",
                        "parent_sheet": "sheet-1",
                        "name": "Main Instance",
                    }
                ],
                "links": [],
            },
            None,
        )

    def export_route_path_proposal(
        self,
        path: str,
        net_uuid: str,
        from_anchor_pad_uuid: str,
        to_anchor_pad_uuid: str,
        candidate: str,
        policy: str | None,
        out: str,
    ) -> JsonRpcResponse:
        self.calls.append(
            (
                "export_route_path_proposal",
                {
                    "path": path,
                    "net_uuid": net_uuid,
                    "from_anchor_pad_uuid": from_anchor_pad_uuid,
                    "to_anchor_pad_uuid": to_anchor_pad_uuid,
                    "candidate": candidate,
                    "policy": policy,
                    "out": out,
                },
            )
        )
        return JsonRpcResponse(
            "2.0",
            116,
            {
                "action": "export_route_path_proposal",
                "contract": (
                    "m5_route_path_candidate_authored_copper_graph_policy_v1"
                    if candidate == "authored-copper-graph"
                    else "m5_route_path_candidate_v2"
                ),
                "path": out,
                "candidate": candidate,
                "policy": policy,
                "artifact_kind": "native_route_proposal_artifact",
            },
            None,
        )

    def route_apply(
        self,
        path: str,
        net_uuid: str,
        from_anchor_pad_uuid: str,
        to_anchor_pad_uuid: str,
        candidate: str,
        policy: str | None,
    ) -> JsonRpcResponse:
        self.calls.append(
            (
                "route_apply",
                {
                    "path": path,
                    "net_uuid": net_uuid,
                    "from_anchor_pad_uuid": from_anchor_pad_uuid,
                    "to_anchor_pad_uuid": to_anchor_pad_uuid,
                    "candidate": candidate,
                    "policy": policy,
                },
            )
        )
        return JsonRpcResponse(
            "2.0",
            117,
            {
                "action": "route_apply",
                "contract": (
                    "m5_route_path_candidate_authored_copper_graph_policy_v1"
                    if candidate == "authored-copper-graph"
                    else "m5_route_path_candidate_v2"
                ),
                "path": path,
                "candidate": candidate,
                "policy": policy,
                "proposal_actions": 1,
                "applied_actions": 0 if candidate == "authored-copper-graph" else 1,
            },
            None,
        )

    def route_proposal(
        self,
        path: str,
        net_uuid: str,
        from_anchor_pad_uuid: str,
        to_anchor_pad_uuid: str,
        profile: str | None,
    ) -> JsonRpcResponse:
        self.calls.append(
            (
                "route_proposal",
                {
                    "path": path,
                    "net_uuid": net_uuid,
                    "from_anchor_pad_uuid": from_anchor_pad_uuid,
                    "to_anchor_pad_uuid": to_anchor_pad_uuid,
                    "profile": profile,
                },
            )
        )
        return JsonRpcResponse(
            "2.0",
            118,
            {
                "action": "route_proposal",
                "path": path,
                "net_uuid": net_uuid,
                "selection_profile": profile or "default",
                "selected_candidate": "route-path-candidate",
                "selected_contract": "m5_route_path_candidate_v2",
                "selection_reason": "first_selectable_candidate",
                "evaluated_candidates": 2,
            },
            None,
        )

    def route_proposal_explain(
        self,
        path: str,
        net_uuid: str,
        from_anchor_pad_uuid: str,
        to_anchor_pad_uuid: str,
        profile: str | None,
    ) -> JsonRpcResponse:
        self.calls.append(
            (
                "route_proposal_explain",
                {
                    "path": path,
                    "net_uuid": net_uuid,
                    "from_anchor_pad_uuid": from_anchor_pad_uuid,
                    "to_anchor_pad_uuid": to_anchor_pad_uuid,
                    "profile": profile,
                },
            )
        )
        return JsonRpcResponse(
            "2.0",
            119,
            {
                "action": "route_proposal_explain",
                "path": path,
                "net_uuid": net_uuid,
                "selection_profile": profile or "default",
                "selected_candidate": "route-path-candidate",
                "selected_family": "route-path-candidate",
                "families": [
                    {
                        "family": "route-path-candidate",
                        "status": "selected",
                        "reason": "first_selectable_candidate",
                    },
                    {
                        "family": "authored-copper-graph",
                        "status": "rejected",
                        "reason": "policy_unavailable",
                    },
                ],
            },
            None,
        )

    def route_strategy_report(
        self,
        path: str,
        net_uuid: str,
        from_anchor_pad_uuid: str,
        to_anchor_pad_uuid: str,
        objective: str | None,
    ) -> JsonRpcResponse:
        self.calls.append(
            (
                "route_strategy_report",
                {
                    "path": path,
                    "net_uuid": net_uuid,
                    "from_anchor_pad_uuid": from_anchor_pad_uuid,
                    "to_anchor_pad_uuid": to_anchor_pad_uuid,
                    "objective": objective,
                },
            )
        )
        resolved = objective or "default"
        selected_candidate = (
            "authored-copper-graph"
            if resolved == "authored-copper-priority"
            else "route-path-candidate"
        )
        selected_policy = "plain" if resolved == "authored-copper-priority" else None
        return JsonRpcResponse(
            "2.0",
            119,
            {
                "action": "route_strategy_report",
                "path": path,
                "net_uuid": net_uuid,
                "objective": resolved,
                "recommended_profile": resolved,
                "recommendation_rule": (
                    f"objective {resolved} maps directly to selector profile {resolved} "
                    "using the accepted deterministic M6 objective/profile table"
                ),
                "selector_status": "deterministic_route_proposal_selected",
                "selector_rule": "profile default selects the first successful candidate",
                "selected_candidate": selected_candidate,
                "selected_policy": selected_policy,
                "selected_contract": (
                    "m5_route_path_candidate_authored_copper_graph_policy_v1"
                    if selected_candidate == "authored-copper-graph"
                    else "m5_route_path_candidate_v2"
                ),
                "selected_actions": 1,
                "next_step_command": "project route-proposal /tmp/demo --net ...",
            },
            None,
        )

    def route_strategy_compare(
        self,
        path: str,
        net_uuid: str,
        from_anchor_pad_uuid: str,
        to_anchor_pad_uuid: str,
    ) -> JsonRpcResponse:
        self.calls.append(
            (
                "route_strategy_compare",
                {
                    "path": path,
                    "net_uuid": net_uuid,
                    "from_anchor_pad_uuid": from_anchor_pad_uuid,
                    "to_anchor_pad_uuid": to_anchor_pad_uuid,
                },
            )
        )
        return JsonRpcResponse(
            "2.0",
            120,
            {
                "action": "route_strategy_compare",
                "path": path,
                "net_uuid": net_uuid,
                "comparison_rule": (
                    "compare accepted objectives/profiles in deterministic order "
                    "default > authored-copper-priority"
                ),
                "recommended_objective": "default",
                "recommended_profile": "default",
                "recommendation_reason": (
                    "recommended default because it yields a proposal while "
                    "preserving the baseline accepted selector order"
                ),
                "next_step_command": "project route-proposal /tmp/demo --net ...",
                "entries": [
                    {
                        "objective": "default",
                        "profile": "default",
                        "proposal_available": True,
                        "selector_status": "deterministic_route_proposal_selected",
                        "selected_candidate": "route-path-candidate",
                        "selected_policy": None,
                        "selected_contract": "m5_route_path_candidate_v2",
                        "selected_actions": 1,
                        "distinction": (
                            "baseline profile: preserves the accepted selector family order exactly"
                        ),
                    },
                    {
                        "objective": "authored-copper-priority",
                        "profile": "authored-copper-priority",
                        "proposal_available": True,
                        "selector_status": "deterministic_route_proposal_selected",
                        "selected_candidate": "authored-copper-graph",
                        "selected_policy": "plain",
                        "selected_contract": "m5_route_path_candidate_authored_copper_graph_policy_v1",
                        "selected_actions": 1,
                        "distinction": (
                            "reuse-priority profile: prepends the accepted authored-copper-graph policy family ahead of the unchanged default order"
                        ),
                    },
                ],
            },
            None,
        )

    def route_strategy_delta(
        self,
        path: str,
        net_uuid: str,
        from_anchor_pad_uuid: str,
        to_anchor_pad_uuid: str,
    ) -> JsonRpcResponse:
        self.calls.append(
            (
                "route_strategy_delta",
                {
                    "path": path,
                    "net_uuid": net_uuid,
                    "from_anchor_pad_uuid": from_anchor_pad_uuid,
                    "to_anchor_pad_uuid": to_anchor_pad_uuid,
                },
            )
        )
        return JsonRpcResponse(
            "2.0",
            121,
            {
                "action": "route_strategy_delta",
                "path": path,
                "net_uuid": net_uuid,
                "compared_objectives": ["default", "authored-copper-priority"],
                "compared_profiles": ["default", "authored-copper-priority"],
                "outcomes_match": False,
                "outcome_relation": "different",
                "delta_classification": "different_candidate_family",
                "recommendation_summary": (
                    "recommended default because it yields a proposal while "
                    "preserving the baseline accepted selector order"
                ),
                "material_difference": (
                    "the accepted profiles currently resolve to different candidate families, "
                    "so the choice changes whether the engine prefers baseline synthesis or "
                    "authored-copper reuse first"
                ),
                "recommended_objective": "default",
                "recommended_profile": "default",
                "profiles": [
                    {
                        "objective": "default",
                        "profile": "default",
                        "proposal_available": True,
                        "selected_candidate": "route-path-candidate",
                        "selected_policy": None,
                    },
                    {
                        "objective": "authored-copper-priority",
                        "profile": "authored-copper-priority",
                        "proposal_available": True,
                        "selected_candidate": "authored-copper-graph",
                        "selected_policy": "plain",
                    },
                ],
            },
            None,
        )

    def write_route_strategy_curated_fixture_suite(
        self, out_dir: str, manifest: str | None = None
    ) -> JsonRpcResponse:
        self.calls.append(
            (
                "write_route_strategy_curated_fixture_suite",
                {
                    "out_dir": out_dir,
                    "manifest": manifest,
                },
            )
        )
        return JsonRpcResponse(
            "2.0",
            121,
            {
                "action": "write_route_strategy_curated_fixture_suite",
                "suite_id": "m6_route_strategy_curated_fixture_suite_v1",
                "authoring_boundary": "generated_fixture_only",
                "write_path_policy": (
                    "direct project-shard writes are restricted to deterministic "
                    "regression fixture generation"
                ),
                "out_dir": out_dir,
                "requests_manifest_path": manifest
                or f"{out_dir}/route-strategy-batch-requests.json",
                "requests_manifest_kind": "native_route_strategy_batch_requests",
                "requests_manifest_version": 1,
                "total_fixtures": 4,
                "total_requests": 4,
                "fixtures": [
                    {
                        "request_id": "same-outcome-default",
                        "fixture_id": "same-outcome-default",
                        "project_root": f"{out_dir}/same-outcome-default",
                        "net_uuid": "00000000-0000-0000-0000-00000000c200",
                        "from_anchor_pad_uuid": "00000000-0000-0000-0000-00000000c205",
                        "to_anchor_pad_uuid": "00000000-0000-0000-0000-00000000c206",
                        "coverage_labels": [
                            "same_outcome",
                            "baseline_route_path_candidate",
                        ],
                    }
                ],
                "next_step_command": (
                    "project route-strategy-batch-evaluate --requests "
                    f"{manifest or f'{out_dir}/route-strategy-batch-requests.json'}"
                ),
            },
            None,
        )

    def capture_route_strategy_curated_baseline(
        self,
        out_dir: str,
        manifest: str | None = None,
        result: str | None = None,
    ) -> JsonRpcResponse:
        self.calls.append(
            (
                "capture_route_strategy_curated_baseline",
                {
                    "out_dir": out_dir,
                    "manifest": manifest,
                    "result": result,
                },
            )
        )
        result_path = result or f"{out_dir}/route-strategy-batch-result.json"
        return JsonRpcResponse(
            "2.0",
            122,
            {
                "action": "capture_route_strategy_curated_baseline",
                "suite_id": "m6_route_strategy_curated_fixture_suite_v1",
                "authoring_boundary": "generated_fixture_only",
                "write_path_policy": (
                    "direct project-shard writes are restricted to deterministic "
                    "regression fixture generation"
                ),
                "out_dir": out_dir,
                "requests_manifest_path": manifest
                or f"{out_dir}/route-strategy-batch-requests.json",
                "result_artifact_path": result_path,
                "requests_manifest_kind": "native_route_strategy_batch_requests",
                "requests_manifest_version": 1,
                "result_kind": "native_route_strategy_batch_result_artifact",
                "result_version": 1,
                "total_fixtures": 4,
                "total_requests": 4,
                "summary": {
                    "total_evaluated_requests": 4,
                    "recommendation_counts_by_profile": {"default": 4},
                    "delta_classification_counts": {
                        "same_outcome": 2,
                        "different_candidate_family": 1,
                        "no_proposal_under_any_profile": 1,
                    },
                    "same_outcome_count": 3,
                    "different_outcome_count": 1,
                    "proposal_available_count": 3,
                    "no_proposal_count": 1,
                },
                "next_inspect_command": (
                    "project inspect-route-strategy-batch-result "
                    f"{result_path}"
                ),
                "next_gate_example_command": (
                    "project gate-route-strategy-batch-result "
                    f"{result_path} {result_path} --policy strict_identical"
                ),
            },
            None,
        )

    def route_strategy_batch_evaluate(self, requests: str) -> JsonRpcResponse:
        self.calls.append(
            (
                "route_strategy_batch_evaluate",
                {
                    "requests": requests,
                },
            )
        )
        return JsonRpcResponse(
            "2.0",
            122,
            {
                "action": "route_strategy_batch_evaluate",
                "kind": "native_route_strategy_batch_result_artifact",
                "version": 1,
                "requests_manifest_path": requests,
                "requests_manifest_kind": "native_route_strategy_batch_requests",
                "requests_manifest_version": 1,
                "summary": {
                    "total_evaluated_requests": 2,
                    "recommendation_counts_by_profile": {"default": 2},
                    "delta_classification_counts": {
                        "different_candidate_family": 1,
                        "same_outcome": 1,
                    },
                    "same_outcome_count": 1,
                    "different_outcome_count": 1,
                    "proposal_available_count": 2,
                    "no_proposal_count": 0,
                },
                "results": [
                    {
                        "identity": {
                            "request_id": "request-a",
                            "fixture_id": "fixture-a",
                            "project_root": "/tmp/demo-a",
                            "net_uuid": "11111111-1111-1111-1111-111111111111",
                            "from_anchor_pad_uuid": "22222222-2222-2222-2222-222222222222",
                            "to_anchor_pad_uuid": "33333333-3333-3333-3333-333333333333",
                        },
                        "recommended_profile": "default",
                        "delta_classification": "different_candidate_family",
                        "outcomes_match": False,
                    },
                    {
                        "identity": {
                            "request_id": "request-b",
                            "fixture_id": "fixture-b",
                            "project_root": "/tmp/demo-b",
                            "net_uuid": "44444444-4444-4444-4444-444444444444",
                            "from_anchor_pad_uuid": "55555555-5555-5555-5555-555555555555",
                            "to_anchor_pad_uuid": "66666666-6666-6666-6666-666666666666",
                        },
                        "recommended_profile": "default",
                        "delta_classification": "same_outcome",
                        "outcomes_match": True,
                    },
                ],
            },
            None,
        )

    def inspect_route_strategy_batch_result(self, artifact: str) -> JsonRpcResponse:
        self.calls.append(("inspect_route_strategy_batch_result", artifact))
        return JsonRpcResponse(
            "2.0",
            123,
            {
                "action": "inspect_route_strategy_batch_result",
                "artifact_path": artifact,
                "kind": "native_route_strategy_batch_result_artifact",
                "source_version": 1,
                "version": 1,
                "requests_manifest_kind": "native_route_strategy_batch_requests",
                "requests_manifest_version": 1,
                "summary": {
                    "total_evaluated_requests": 2,
                    "recommendation_counts_by_profile": {"default": 2},
                    "delta_classification_counts": {
                        "different_candidate_family": 1,
                        "same_outcome": 1,
                    },
                    "same_outcome_count": 1,
                    "different_outcome_count": 1,
                    "proposal_available_count": 2,
                    "no_proposal_count": 0,
                },
                "results": [
                    {
                        "identity": {
                            "request_id": "request-a",
                            "fixture_id": "fixture-a",
                            "project_root": "/tmp/demo-a",
                            "net_uuid": "11111111-1111-1111-1111-111111111111",
                            "from_anchor_pad_uuid": "22222222-2222-2222-2222-222222222222",
                            "to_anchor_pad_uuid": "33333333-3333-3333-3333-333333333333",
                        },
                        "recommended_profile": "default",
                        "delta_classification": "different_candidate_family",
                        "outcomes_match": False,
                    }
                ],
                "malformed_entries": [],
            },
            None,
        )

    def validate_route_strategy_batch_result(self, artifact: str) -> JsonRpcResponse:
        self.calls.append(("validate_route_strategy_batch_result", artifact))
        return JsonRpcResponse(
            "2.0",
            124,
            {
                "action": "validate_route_strategy_batch_result",
                "artifact_path": artifact,
                "kind": "native_route_strategy_batch_result_artifact",
                "source_version": 1,
                "version": 1,
                "structurally_valid": True,
                "version_compatible": True,
                "missing_required_fields": [],
                "request_result_count_matches_summary": True,
                "recommendation_counts_match_summary": True,
                "delta_classification_counts_match_summary": True,
                "outcome_counts_match_summary": True,
                "proposal_counts_match_summary": True,
                "malformed_entries": [],
            },
            None,
        )

    def compare_route_strategy_batch_result(
        self, before: str, after: str
    ) -> JsonRpcResponse:
        self.calls.append(
            (
                "compare_route_strategy_batch_result",
                {"before": before, "after": after},
            )
        )
        return JsonRpcResponse(
            "2.0",
            125,
            {
                "action": "compare_route_strategy_batch_result",
                "comparison_classification": "per_request_outcomes_changed",
                "compatibility_rule": "artifacts are compatible only when both use kind native_route_strategy_batch_result_artifact, version 1, and the same requests manifest kind/version",
                "compatible_artifacts": True,
                "before_artifact": {
                    "artifact_path": before,
                    "kind": "native_route_strategy_batch_result_artifact",
                    "version": 1,
                    "requests_manifest_kind": "native_route_strategy_batch_requests",
                    "requests_manifest_version": 1,
                },
                "after_artifact": {
                    "artifact_path": after,
                    "kind": "native_route_strategy_batch_result_artifact",
                    "version": 1,
                    "requests_manifest_kind": "native_route_strategy_batch_requests",
                    "requests_manifest_version": 1,
                },
                "total_request_count_change": {"before": 2, "after": 2, "change": 0},
                "recommendation_distribution_changes": {
                    "default": {"before": 2, "after": 1, "change": -1},
                    "authored-copper-priority": {
                        "before": 0,
                        "after": 1,
                        "change": 1,
                    },
                },
                "delta_classification_distribution_changes": {
                    "different_candidate_family": {
                        "before": 1,
                        "after": 0,
                        "change": -1,
                    },
                    "same_outcome": {"before": 1, "after": 2, "change": 1},
                },
                "same_outcome_count_change": {"before": 1, "after": 2, "change": 1},
                "different_outcome_count_change": {
                    "before": 1,
                    "after": 0,
                    "change": -1,
                },
                "proposal_available_count_change": {
                    "before": 2,
                    "after": 2,
                    "change": 0,
                },
                "no_proposal_count_change": {"before": 0, "after": 0, "change": 0},
                "added_request_ids": [],
                "removed_request_ids": [],
                "common_request_ids": ["request-a", "request-b"],
                "changed_common_requests": [
                    {
                        "request_id": "request-a",
                        "recommendation_changed": True,
                        "delta_classification_changed": True,
                        "selected_live_outcome_changed": True,
                        "before_recommended_profile": "default",
                        "after_recommended_profile": "authored-copper-priority",
                        "before_delta_classification": "different_candidate_family",
                        "after_delta_classification": "same_outcome",
                        "before_selected_candidate": "route-path-candidate",
                        "after_selected_candidate": "authored-copper-graph",
                        "before_selected_policy": None,
                        "after_selected_policy": "plain",
                    }
                ],
            },
            None,
        )

    def gate_route_strategy_batch_result(
        self, before: str, after: str, policy: str | None = None
    ) -> JsonRpcResponse:
        self.calls.append(
            (
                "gate_route_strategy_batch_result",
                {"before": before, "after": after, "policy": policy},
            )
        )
        return JsonRpcResponse(
            "2.0",
            126,
            {
                "action": "gate_route_strategy_batch_result",
                "selected_gate_policy": policy or "strict_identical",
                "passed": False,
                "comparison_classification": "per_request_outcomes_changed",
                "pass_fail_reasons": [
                    "failed because strict_identical requires comparison_classification = identical"
                ],
                "threshold_facts": {
                    "changed_recommendations": 1,
                    "changed_delta_classifications": 1,
                    "changed_per_request_outcomes": 1,
                    "added_request_ids": 0,
                    "removed_request_ids": 0,
                },
                "changed_recommendations": 1,
                "changed_delta_classifications": 1,
                "changed_per_request_outcomes": 1,
                "comparison": {
                    "action": "compare_route_strategy_batch_result",
                    "comparison_classification": "per_request_outcomes_changed",
                    "compatible_artifacts": True,
                },
            },
            None,
        )

    def summarize_route_strategy_batch_results(
        self,
        dir: str | None = None,
        artifacts: list[str] | None = None,
        baseline: str | None = None,
        policy: str | None = None,
    ) -> JsonRpcResponse:
        self.calls.append(
            (
                "summarize_route_strategy_batch_results",
                {
                    "dir": dir,
                    "artifacts": artifacts,
                    "baseline": baseline,
                    "policy": policy,
                },
            )
        )
        return JsonRpcResponse(
            "2.0",
            127,
            {
                "action": "summarize_route_strategy_batch_results",
                "ordering_basis": "filesystem_modified_time_then_path",
                "baseline_artifact": baseline,
                "selected_gate_policy": policy,
                "summary": {
                    "total_artifacts": 2,
                    "structurally_valid_artifacts": 2,
                    "structurally_invalid_artifacts": 0,
                    "gate_passed_artifacts": 1,
                    "gate_failed_artifacts": 0,
                },
                "artifacts": [
                    {
                        "artifact_path": "/tmp/run-a.json",
                        "kind": "native_route_strategy_batch_result_artifact",
                        "version": 1,
                        "requests_manifest_kind": "native_route_strategy_batch_requests",
                        "requests_manifest_version": 1,
                        "file_modified_unix_seconds": 1710000000,
                        "run_order": 1,
                        "structurally_valid": True,
                        "request_count": 2,
                        "recommendation_distribution": {"default": 2},
                        "delta_classification_distribution": {"same_outcome": 2},
                        "validation_error": None,
                        "is_baseline": True,
                        "baseline_gate": None,
                    },
                    {
                        "artifact_path": "/tmp/run-b.json",
                        "kind": "native_route_strategy_batch_result_artifact",
                        "version": 1,
                        "requests_manifest_kind": "native_route_strategy_batch_requests",
                        "requests_manifest_version": 1,
                        "file_modified_unix_seconds": 1710000100,
                        "run_order": 2,
                        "structurally_valid": True,
                        "request_count": 2,
                        "recommendation_distribution": {"default": 2},
                        "delta_classification_distribution": {"same_outcome": 2},
                        "validation_error": None,
                        "is_baseline": False,
                        "baseline_gate": {
                            "selected_gate_policy": policy or "strict_identical",
                            "passed": True,
                            "comparison_classification": "identical",
                            "pass_fail_reasons": ["passed because the saved artifacts are identical"],
                        },
                    },
                ],
            },
            None,
        )

    def export_route_proposal(
        self,
        path: str,
        net_uuid: str,
        from_anchor_pad_uuid: str,
        to_anchor_pad_uuid: str,
        out: str,
        profile: str | None,
    ) -> JsonRpcResponse:
        self.calls.append(
            (
                "export_route_proposal",
                {
                    "path": path,
                    "net_uuid": net_uuid,
                    "from_anchor_pad_uuid": from_anchor_pad_uuid,
                    "to_anchor_pad_uuid": to_anchor_pad_uuid,
                    "profile": profile,
                    "out": out,
                },
            )
        )
        return JsonRpcResponse(
            "2.0",
            120,
            {
                "action": "export_route_proposal",
                "path": out,
                "selection_profile": profile or "default",
                "selected_candidate": "route-path-candidate",
                "selected_contract": "m5_route_path_candidate_v2",
                "artifact_kind": "native_route_proposal_artifact",
            },
            None,
        )

    def route_apply_selected(
        self,
        path: str,
        net_uuid: str,
        from_anchor_pad_uuid: str,
        to_anchor_pad_uuid: str,
        profile: str | None,
    ) -> JsonRpcResponse:
        self.calls.append(
            (
                "route_apply_selected",
                {
                    "path": path,
                    "net_uuid": net_uuid,
                    "from_anchor_pad_uuid": from_anchor_pad_uuid,
                    "to_anchor_pad_uuid": to_anchor_pad_uuid,
                    "profile": profile,
                },
            )
        )
        return JsonRpcResponse(
            "2.0",
            121,
            {
                "action": "route_apply_selected",
                "path": path,
                "selection_profile": profile or "default",
                "selected_candidate": "route-path-candidate",
                "selected_contract": "m5_route_path_candidate_v2",
                "proposal_actions": 1,
                "applied_actions": 1,
            },
            None,
        )

    def inspect_route_proposal_artifact(self, artifact: str) -> JsonRpcResponse:
        self.calls.append(("inspect_route_proposal_artifact", artifact))
        return JsonRpcResponse(
            "2.0",
            114,
            {
                "action": "inspect_route_proposal_artifact",
                "artifact_kind": "native_route_proposal_artifact",
                "contract": "m5_route_path_candidate_authored_copper_graph_policy_v1",
                "path": artifact,
                "actions": 2,
            },
            None,
        )

    def review_route_proposal(
        self,
        path: str | None = None,
        net_uuid: str | None = None,
        from_anchor_pad_uuid: str | None = None,
        to_anchor_pad_uuid: str | None = None,
        profile: str | None = None,
        artifact: str | None = None,
    ) -> JsonRpcResponse:
        self.calls.append(
            (
                "review_route_proposal",
                {
                    "path": path,
                    "net_uuid": net_uuid,
                    "from_anchor_pad_uuid": from_anchor_pad_uuid,
                    "to_anchor_pad_uuid": to_anchor_pad_uuid,
                    "profile": profile,
                    "artifact": artifact,
                },
            )
        )
        return JsonRpcResponse(
            "2.0",
            123,
            {
                "action": "review_route_proposal",
                "review_source": (
                    "route_proposal_artifact"
                    if artifact is not None
                    else "selected_route_proposal"
                ),
                "contract": "m5_route_path_candidate_v2",
                "actions": 1,
                "draw_track_actions": 1,
            },
            None,
        )
 
    def revalidate_route_proposal_artifact(
        self,
        path: str,
        artifact: str,
    ) -> JsonRpcResponse:
        self.calls.append(
            (
                "revalidate_route_proposal_artifact",
                {
                    "path": path,
                    "artifact": artifact,
                },
            )
        )
        return JsonRpcResponse(
            "2.0",
            122,
            {
                "action": "revalidate_route_proposal_artifact",
                "project_root": path,
                "artifact_path": artifact,
                "contract": "m5_route_path_candidate_authored_copper_graph_policy_v1",
                "artifact_actions": 2,
                "live_actions": 2,
                "matches_live": True,
                "drift_kind": None,
                "drift_message": None,
                "live_rebuild_error": None,
            },
            None,
        )

    def apply_route_proposal_artifact(self, path: str, artifact: str) -> JsonRpcResponse:
        self.calls.append(
            (
                "apply_route_proposal_artifact",
                {
                    "path": path,
                    "artifact": artifact,
                },
            )
        )
        return JsonRpcResponse(
            "2.0",
            115,
            {
                "action": "apply_route_proposal_artifact",
                "path": path,
                "artifact": artifact,
                "artifact_actions": 2,
                "applied_actions": 0,
            },
            None,
        )

    def get_connectivity_diagnostics(self) -> JsonRpcResponse:
        self.calls.append(("get_connectivity_diagnostics", None))
        return JsonRpcResponse(
            "2.0",
            3,
            [
                {"kind": "partially_routed_net", "severity": "warning"},
                {"kind": "net_without_copper", "severity": "info"},
            ],
            None,
        )

    def get_design_rules(self) -> JsonRpcResponse:
        self.calls.append(("get_design_rules", None))
        return JsonRpcResponse("2.0", 102, [], None)
