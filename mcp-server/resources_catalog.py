#!/usr/bin/env python3
"""Stable, discovery-scoped Datum MCP resources and URI templates."""

from __future__ import annotations

import json
import re
from typing import Any

from discovery_scope import DiscoveryScope


_ID = re.compile(r"^[A-Za-z0-9][A-Za-z0-9._:@+-]{0,255}$")

RESOURCE_TEMPLATES = [
    {
        "uriTemplate": "datum://context/pinned/{context_id}",
        "name": "Pinned Datum context",
        "description": "A stable terminal context snapshot by context identity.",
        "mimeType": "application/json",
    },
    {
        "uriTemplate": "datum://model/revision/{revision}",
        "name": "Datum model revision",
        "description": "The discovery snapshot for an exact model revision.",
        "mimeType": "application/json",
    },
    {
        "uriTemplate": "datum://selection/{context_id}",
        "name": "Datum selection",
        "description": "The selected objects captured by a terminal context.",
        "mimeType": "application/json",
    },
    {
        "uriTemplate": "datum://check/{fingerprint}",
        "name": "Datum check finding",
        "description": "A visible check finding by stable fingerprint.",
        "mimeType": "application/json",
    },
    {
        "uriTemplate": "datum://proposal/{proposal_id}",
        "name": "Datum proposal",
        "description": "A proposal visible in the current context.",
        "mimeType": "application/json",
    },
    {
        "uriTemplate": "datum://artifact/{artifact_id}",
        "name": "Datum artifact",
        "description": "An artifact visible in the current context.",
        "mimeType": "application/json",
    },
    {
        "uriTemplate": "datum://render/board/{revision}.svg",
        "name": "Datum board render",
        "description": "A board SVG for an exact model revision when published.",
        "mimeType": "image/svg+xml",
    },
    {
        "uriTemplate": "datum://render/schematic/{revision}.svg",
        "name": "Datum schematic render",
        "description": "A schematic SVG for an exact model revision when published.",
        "mimeType": "image/svg+xml",
    },
    {
        "uriTemplate": "datum://object/{kind}/{object_id}",
        "name": "Datum object",
        "description": "A stable Datum object identity; resolve with typed query tools.",
        "mimeType": "application/json",
    },
    {
        "uriTemplate": "datum://objects/{kind}{?cursor,limit}",
        "name": "Datum object page",
        "description": "A cursor-paginated collection of stable Datum object identities.",
        "mimeType": "application/json",
    },
]


class DatumResourceCatalog:
    def __init__(self, scope: DiscoveryScope) -> None:
        self._scope = scope

    def list_resources(self) -> list[dict[str, str]]:
        document = self._scope.document
        resources = [
            _resource("datum://project/current", "Current Datum project"),
            _resource("datum://context/live", "Live Datum context"),
            _resource("datum://checks/current", "Current Datum checks"),
        ]
        context_id = self._scope.context_id
        if context_id:
            resources.extend(
                [
                    _resource(
                        f"datum://context/pinned/{context_id}", "Pinned Datum context"
                    ),
                    _resource(f"datum://selection/{context_id}", "Datum selection"),
                ]
            )
        revision = document.get("model_revision")
        if _stable_id(revision):
            resources.append(
                _resource(f"datum://model/revision/{revision}", "Datum model revision")
            )
        for key, prefix, label in (
            ("visible_finding_fingerprints", "check", "Datum check finding"),
            ("visible_proposal_ids", "proposal", "Datum proposal"),
            ("visible_artifact_ids", "artifact", "Datum artifact"),
        ):
            for value in _stable_ids(document.get(key)):
                resources.append(_resource(f"datum://{prefix}/{value}", label))
        return resources

    def read(self, uri: str) -> dict[str, Any]:
        document = self._scope.document
        if uri == "datum://project/current":
            value = {
                key: document.get(key)
                for key in (
                    "project_root",
                    "project_id",
                    "project_name",
                    "board_id",
                    "board_name",
                    "scene_id",
                    "model_revision",
                    "source_revision",
                )
            }
        elif uri == "datum://context/live":
            value = document
        elif uri == "datum://checks/current":
            value = {
                "latest_check_run_id": document.get("latest_check_run_id"),
                "visible_check_run_ids": document.get("visible_check_run_ids", []),
                "visible_finding_fingerprints": document.get(
                    "visible_finding_fingerprints", []
                ),
                "check_status": document.get("check_status"),
            }
        elif self._matches_current(uri, "context/pinned", self._scope.context_id):
            value = document
        elif self._matches_current(uri, "model/revision", document.get("model_revision")):
            value = document
        elif self._matches_current(uri, "selection", self._scope.context_id):
            value = document.get("selection_context", {})
        else:
            value = self._read_visible_identity(uri)
        return {
            "contents": [
                {
                    "uri": uri,
                    "mimeType": "application/json",
                    "text": json.dumps(value, sort_keys=True, separators=(",", ":")),
                }
            ]
        }

    def supports(self, uri: str) -> bool:
        try:
            self.read(uri)
        except ValueError:
            return False
        return True

    def _matches_current(self, uri: str, prefix: str, expected: Any) -> bool:
        return _stable_id(expected) and uri == f"datum://{prefix}/{expected}"

    def _read_visible_identity(self, uri: str) -> dict[str, Any]:
        document = self._scope.document
        for prefix, key in (
            ("check", "visible_finding_fingerprints"),
            ("proposal", "visible_proposal_ids"),
            ("artifact", "visible_artifact_ids"),
        ):
            marker = f"datum://{prefix}/"
            if uri.startswith(marker):
                identity = uri[len(marker) :]
                if _stable_id(identity) and identity in _stable_ids(document.get(key)):
                    return {"kind": prefix, "id": identity, "context_id": self._scope.context_id}
        raise ValueError(f"resource is unavailable in this discovery scope: {uri}")


def _resource(uri: str, name: str) -> dict[str, str]:
    return {"uri": uri, "name": name, "mimeType": "application/json"}


def _stable_id(value: Any) -> bool:
    return isinstance(value, str) and _ID.fullmatch(value) is not None


def _stable_ids(value: Any) -> list[str]:
    if not isinstance(value, list):
        return []
    return [item for item in value if _stable_id(item)]
