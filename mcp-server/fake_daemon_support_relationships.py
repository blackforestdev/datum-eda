#!/usr/bin/env python3
"""Fake daemon client relationship and variant responses for MCP tests."""

from __future__ import annotations

from server_runtime import JsonRpcResponse


def _component_instance_record(
    component_instance: str = "ci-test",
    symbol: str = "sym-test",
    package: str = "pkg-test",
    status: str = "bound",
) -> dict:
    return {
        "component_instance_id": component_instance,
        "id": component_instance,
        "project_id": "project-test",
        "model_revision": "model-rev-test",
        "status": status,
        "symbol": symbol,
        "symbol_id": symbol,
        "package": package,
        "package_id": package,
        "binding": {
            "symbol_id": symbol,
            "package_id": package,
            "status": status,
        },
    }


class FakeDaemonClientRelationshipsMixin:
    def get_component_instances(self, path: str) -> JsonRpcResponse:
        self.calls.append(("get_component_instances", path))
        component_instance = _component_instance_record()
        return JsonRpcResponse(
            "2.0",
            139,
            {
                "contract": "component_instances_query_v1",
                "project_root": path,
                "project_id": "project-test",
                "model_revision": "model-rev-test",
                "component_instance_count": 1,
                "component_instances": {"ci-test": component_instance},
                "instances": [component_instance],
            },
            None,
        )

    def bind_component_instance(
        self,
        path: str,
        symbol: str,
        package: str,
        component_instance: str | None = None,
    ) -> JsonRpcResponse:
        self.calls.append(("bind_component_instance", path, symbol, package, component_instance))
        component_instance_id = component_instance or "ci-created"
        return JsonRpcResponse(
            "2.0",
            138,
            {
                "action": "bind_component_instance",
                "project_id": "project-test",
                "model_revision": "model-rev-test",
                "component_instance": component_instance_id,
                "component_instance_id": component_instance_id,
                "status": "bound",
                "symbol": symbol,
                "symbol_id": symbol,
                "package": package,
                "package_id": package,
                "binding": {
                    "symbol_id": symbol,
                    "package_id": package,
                    "status": "bound",
                },
            },
            None,
        )

    def set_component_instance(
        self,
        path: str,
        component_instance: str,
        symbol: str,
        package: str,
    ) -> JsonRpcResponse:
        self.calls.append(("set_component_instance", path, component_instance, symbol, package))
        return JsonRpcResponse(
            "2.0",
            137,
            {
                "action": "set_component_instance",
                "project_id": "project-test",
                "model_revision": "model-rev-test",
                "component_instance": component_instance,
                "component_instance_id": component_instance,
                "status": "bound",
                "symbol": symbol,
                "symbol_id": symbol,
                "package": package,
                "package_id": package,
                "binding": {
                    "symbol_id": symbol,
                    "package_id": package,
                    "status": "bound",
                },
            },
            None,
        )

    def delete_component_instance(self, path: str, component_instance: str) -> JsonRpcResponse:
        self.calls.append(("delete_component_instance", path, component_instance))
        return JsonRpcResponse(
            "2.0",
            136,
            {
                "action": "delete_component_instance",
                "project_id": "project-test",
                "model_revision": "model-rev-test",
                "component_instance": component_instance,
                "component_instance_id": component_instance,
                "status": "deleted",
            },
            None,
        )

    def get_relationships(self, path: str) -> JsonRpcResponse:
        self.calls.append(("get_relationships", path))
        return JsonRpcResponse(
            "2.0",
            140,
            {
                "contract": "relationships_query_v1",
                "project_root": path,
                "project_id": "project-test",
                "model_revision": "model-rev-test",
                "relationship_count": 1,
                "relationships": [
                    {
                        "relationship_id": "rel-test",
                        "id": "rel-test",
                        "project_id": "project-test",
                        "model_revision": "model-rev-test",
                        "kind": "symbol_package_binding",
                        "source_id": "sym-test",
                        "target_id": "pkg-test",
                        "status": "implemented",
                    }
                ],
                "statuses": {"rel-test": "implemented"},
            },
            None,
        )

    def get_variants(self, path: str) -> JsonRpcResponse:
        self.calls.append(("get_variants", path))
        return JsonRpcResponse(
            "2.0",
            141,
            {
                "contract": "variants_query_v1",
                "project_root": path,
                "variant_count": 1,
                "populations": {"variant-test": {"obj-test": "not_applicable_for_variant"}},
            },
            None,
        )
