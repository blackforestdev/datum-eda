"""Fake Global Preferences daemon responses for MCP dispatch tests."""

from __future__ import annotations

from typing import Any

from server_runtime import JsonRpcResponse
class FakeDaemonClientPreferencesMixin:
    pass


def _install_fake(name: str, payload_class: str, payload_kind: str, fields: list[str]) -> None:
    def method(self: Any, *args: Any) -> JsonRpcResponse:
        values = dict(zip([*fields, "_transport_actor"], args, strict=True))
        actor = values.pop("_transport_actor")
        request = {
            "schema": {"name": name, "version": 1},
            "payload": {
                "class": payload_class,
                "request": {"kind": payload_kind, **values},
            },
        }
        params = {"request": request, "actor": actor}
        self.calls.append(("preferences.mcp_product", params))
        return JsonRpcResponse(
            "2.0",
            901,
            {
                "ok": True,
                "schema": request["schema"],
                "context": {"scope": "global_this_device"},
                "result": request["payload"],
            },
            None,
        )

    setattr(
        FakeDaemonClientPreferencesMixin,
        name.replace("datum.preferences.", "preferences_").replace(".", "_"),
        method,
    )


_install_fake("datum.preferences.describe", "query", "describe", [])
_install_fake("datum.preferences.list", "query", "list", ["section"])
_install_fake("datum.preferences.get", "query", "get", ["key"])
_install_fake("datum.preferences.search", "query", "search", ["query"])
_install_fake("datum.preferences.explain", "query", "explain", ["key"])
_install_fake("datum.preferences.preview_project_units_seed", "query", "preview_project_units_seed", ["source"])
_install_fake("datum.preferences.proposal.prepare", "proposal", "prepare", ["mutation", "rationale"])
_install_fake("datum.preferences.proposal.validate", "proposal", "validate", ["proposal"])
_install_fake("datum.preferences.proposal.accept_apply", "proposal", "accept_and_apply", ["proposal"])
_install_fake("datum.preferences.proposal.reject", "proposal", "reject", ["proposal"])
