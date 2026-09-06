"""Trusted MCP adapter for the engine-owned Project-genesis service."""

from __future__ import annotations

from typing import Any


def install_project_genesis_method(client_cls: type) -> None:
    def project_new(
        self: Any,
        request_id: str,
        destination: str,
        project_name: str,
        project_id: str | None,
        units_source: dict[str, Any],
        transport_actor: dict[str, Any],
    ) -> Any:
        request = {
            "request_id": request_id,
            "destination": destination,
            "project_name": project_name,
            "project_id": project_id,
            "units_source": units_source,
        }
        return self.call(
            self.build_request(
                "project.genesis",
                {"request": request, "actor": transport_actor},
            )
        )

    setattr(client_cls, "project_new", project_new)
