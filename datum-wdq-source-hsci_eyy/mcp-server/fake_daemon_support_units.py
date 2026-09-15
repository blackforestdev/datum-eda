#!/usr/bin/env python3
"""Fake daemon client responses for Units-service MCP tests."""

from __future__ import annotations

from typing import Any

from server_runtime import JsonRpcResponse


class FakeDaemonClientUnitsMixin:
    def set_project_display_units(
        self,
        path: str,
        profile: dict[str, Any],
        expected_model_revision: str | None = None,
    ) -> JsonRpcResponse:
        self.calls.append(
            (
                "set_project_display_units",
                path,
                profile,
                expected_model_revision,
            )
        )
        return JsonRpcResponse(
            "2.0",
            170,
            {
                "status": "committed",
                "project_root": path,
                "profile": profile,
            },
            None,
        )

    def resolve_length(
        self,
        canonical_nm: int | None = None,
        expression: str | None = None,
        quantity: str | None = None,
        unit: str | None = None,
        system: str | None = None,
        field: str | None = None,
        project_id: str | None = None,
    ) -> JsonRpcResponse:
        self.calls.append(
            (
                "resolve_length",
                canonical_nm,
                expression,
                quantity,
                unit,
                system,
                field,
                project_id,
            )
        )
        return JsonRpcResponse(
            "2.0",
            171,
            {"ok": True, "canonical_nm": canonical_nm},
            None,
        )
