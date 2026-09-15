"""Units-service methods installed on the MCP engine client."""

from __future__ import annotations

from typing import Any, Callable


def install_units_methods(client_cls: type, append_optional: Callable) -> None:
    def set_project_display_units(
        self,
        path: str,
        profile: dict[str, Any],
        expected_model_revision: str | None = None,
    ):
        return self.call(
            self.build_request(
                "native.write",
                {
                    "project_root": path,
                    "verb": "datum.project.set_display_units",
                    "params": {"profile": profile},
                    "actor": "datum-mcp",
                    "source": "tool",
                    "reason": "Set Project Working Units through MCP",
                    "expected_model_revision": expected_model_revision,
                    "dry_run": False,
                },
            )
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
    ):
        request = self.build_request(
            "resolve_length",
            {
                "canonical_nm": canonical_nm,
                "expression": expression,
                "quantity": quantity,
                "unit": unit,
                "system": system,
                "field": field,
                "project_id": project_id,
            },
        )
        args = ["units", "resolve-length"]
        append_optional(args, "canonical-nm", canonical_nm)
        append_optional(args, "expression", expression)
        append_optional(args, "quantity", quantity)
        append_optional(args, "unit", unit)
        append_optional(args, "system", system)
        append_optional(args, "field", field)
        append_optional(args, "project-id", project_id)
        return self._run_cli_json_allowing_statuses(request, args, {0, 2})

    setattr(client_cls, "set_project_display_units", set_project_display_units)
    setattr(client_cls, "resolve_length", resolve_length)
