#!/usr/bin/env python3
"""MCP methods that author native pool/library content through the CLI bridge."""

from __future__ import annotations

import os
from typing import Any

LIBRARY_AUTHORING_CLI_METHODS = {
    "add_pool_package_model_3d",
    "add_pool_package_silkscreen_arc",
    "add_pool_package_silkscreen_circle",
    "add_pool_package_silkscreen_line",
    "add_pool_package_silkscreen_polygon",
    "add_pool_package_silkscreen_rect",
    "add_pool_package_silkscreen_text",
    "add_pool_footprint_silkscreen_circle",
    "add_pool_footprint_silkscreen_line",
    "add_pool_footprint_silkscreen_polygon",
    "add_pool_footprint_silkscreen_rect",
    "add_pool_symbol_arc",
    "add_pool_symbol_circle",
    "add_pool_symbol_line",
    "add_pool_symbol_polygon",
    "add_pool_symbol_rect",
    "add_pool_symbol_text",
    "attach_pool_part_model",
    "create_pool_entity",
    "create_pool_footprint",
    "create_pool_library_object",
    "create_pool_package",
    "create_pool_padstack",
    "create_pool_part",
    "create_pool_pin_pad_map",
    "create_pool_symbol",
    "create_pool_unit",
    "delete_pool_library_object",
    "detach_pool_part_model",
    "generate_ipc7351b_soic",
    "set_pool_library_object",
    "set_pool_footprint_courtyard_polygon",
    "set_pool_footprint_courtyard_rect",
    "set_pool_footprint_pad",
    "set_pool_package_body_heights",
    "set_pool_package_courtyard_polygon",
    "set_pool_package_courtyard_rect",
    "set_pool_package_pad",
    "set_pool_part_behavioural_models",
    "set_pool_part_metadata",
    "set_pool_part_orderable_mpns",
    "set_pool_part_packaging_options",
    "set_pool_part_pad_map",
    "set_pool_part_pad_map_entry",
    "set_pool_part_parametric",
    "set_pool_part_supply_chain",
    "set_pool_part_tags",
    "set_pool_part_thermal",
    "set_pool_pin_pad_map",
    "set_pool_symbol_pin_anchor",
    "set_pool_unit_pin",
}


def cli_run_kwargs_for_method(method: str) -> dict[str, Any]:
    run_kwargs: dict[str, Any] = {"capture_output": True, "text": True, "check": False}
    if method in LIBRARY_AUTHORING_CLI_METHODS:
        env = dict(os.environ)
        env["DATUM_COMMIT_SOURCE"] = "tool"
        env["DATUM_TOOL_SURFACE"] = "mcp"
        run_kwargs["env"] = env
    return run_kwargs
