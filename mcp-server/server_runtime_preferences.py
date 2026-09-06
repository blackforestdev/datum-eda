"""Trusted MCP-to-daemon Global Preferences request adaptation."""

from __future__ import annotations

from typing import Any


def install_preferences_methods(client_cls: type) -> None:
    def install(name: str, payload_class: str, payload_kind: str, fields: list[str]) -> None:
        def method(self: Any, *args: Any) -> Any:
            values = dict(zip([*fields, "_transport_actor"], args, strict=True))
            actor = values.pop("_transport_actor")
            request = {
                "schema": {"name": name, "version": 1},
                "payload": {
                    "class": payload_class,
                    "request": {"kind": payload_kind, **values},
                },
            }
            return self.call(
                self.build_request(
                    "preferences.mcp_product",
                    {"request": request, "actor": actor},
                )
            )

        setattr(
            client_cls,
            name.replace("datum.preferences.", "preferences_").replace(".", "_"),
            method,
        )

    install("datum.preferences.describe", "query", "describe", [])
    install("datum.preferences.list", "query", "list", ["section"])
    install("datum.preferences.get", "query", "get", ["key"])
    install("datum.preferences.search", "query", "search", ["query"])
    install("datum.preferences.explain", "query", "explain", ["key"])
    install(
        "datum.preferences.preview_project_units_seed",
        "query",
        "preview_project_units_seed",
        ["source"],
    )
    install(
        "datum.preferences.proposal.prepare",
        "proposal",
        "prepare",
        ["mutation", "rationale"],
    )
    install(
        "datum.preferences.proposal.validate",
        "proposal",
        "validate",
        ["proposal"],
    )
    install(
        "datum.preferences.proposal.accept_apply",
        "proposal",
        "accept_and_apply",
        ["proposal"],
    )
    install(
        "datum.preferences.proposal.reject",
        "proposal",
        "reject",
        ["proposal"],
    )
