#!/usr/bin/env python3
"""Canonical native pool-library MCP tools/call alias dispatch tests."""

from __future__ import annotations

import unittest

from server_runtime import StdioToolHost
from test_support import FakeDaemonClient


class TestDispatchLibraryAliases(unittest.TestCase):
    def test_tools_call_dispatches_canonical_library_aliases(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        for tool_name, arguments, expected_method in [
            (
                "datum.library.create_object",
                {
                    "path": "/tmp/native-project",
                    "pool": "pool",
                    "kind": "symbols",
                    "object": "symbol-test",
                    "from_json": "/tmp/symbol.json",
                },
                "create_pool_library_object",
            ),
            (
                "datum.library.delete_object",
                {
                    "path": "/tmp/native-project",
                    "kind": "symbols",
                    "object": "symbol-test",
                },
                "delete_pool_library_object",
            ),
            (
                "datum.library.create_unit",
                {
                    "path": "/tmp/native-project",
                    "pool": "pool",
                    "unit": "unit-test",
                    "name": "OpAmpUnit",
                    "manufacturer": "Datum",
                },
                "create_pool_unit",
            ),
            (
                "datum.library.create_symbol",
                {
                    "path": "/tmp/native-project",
                    "pool": "pool",
                    "symbol": "symbol-test",
                    "unit": "unit-test",
                    "name": "OpAmpSymbol",
                },
                "create_pool_symbol",
            ),
            (
                "datum.library.add_symbol_line",
                {"path": "/tmp/native-project", "pool": "pool", "symbol": "symbol-test", "from_x_nm": 0, "from_y_nm": 0, "to_x_nm": 1000, "to_y_nm": 0, "width_nm": 100},
                "add_pool_symbol_line",
            ),
            (
                "datum.library.add_symbol_rect",
                {"path": "/tmp/native-project", "pool": "pool", "symbol": "symbol-test", "min_x_nm": 0, "min_y_nm": 0, "max_x_nm": 1000, "max_y_nm": 2000, "width_nm": 100},
                "add_pool_symbol_rect",
            ),
            (
                "datum.library.add_symbol_circle",
                {"path": "/tmp/native-project", "pool": "pool", "symbol": "symbol-test", "center_x_nm": 500, "center_y_nm": 600, "radius_nm": 250, "width_nm": 100},
                "add_pool_symbol_circle",
            ),
            (
                "datum.library.add_symbol_text",
                {"path": "/tmp/native-project", "pool": "pool", "symbol": "symbol-test", "text": "REF**", "x_nm": 1000, "y_nm": 2000, "rotation": 90},
                "add_pool_symbol_text",
            ),
            (
                "datum.library.set_symbol_pin_anchor",
                {"path": "/tmp/native-project", "pool": "pool", "symbol": "symbol-test", "pin": "pin-test", "x_nm": 100, "y_nm": 200},
                "set_pool_symbol_pin_anchor",
            ),
            (
                "datum.library.create_entity",
                {
                    "path": "/tmp/native-project",
                    "pool": "pool",
                    "entity": "entity-test",
                    "gate": "gate-test",
                    "unit": "unit-test",
                    "symbol": "symbol-test",
                    "name": "DualOpAmp",
                    "prefix": "U",
                    "manufacturer": "Datum",
                    "gate_name": "A",
                },
                "create_pool_entity",
            ),
            (
                "datum.library.create_padstack",
                {
                    "path": "/tmp/native-project",
                    "pool": "pool",
                    "padstack": "padstack-test",
                    "name": "RoundViaPad",
                    "aperture": "circle",
                    "diameter_nm": 1200000,
                    "drill_nm": 600000,
                },
                "create_pool_padstack",
            ),
            (
                "datum.library.create_package",
                {
                    "path": "/tmp/native-project",
                    "pool": "pool",
                    "package": "package-test",
                    "name": "SOT23",
                    "pad": "pad-test",
                    "padstack": "padstack-test",
                    "pad_name": "1",
                    "x_nm": 1000,
                    "y_nm": 2000,
                    "layer": 1,
                },
                "create_pool_package",
            ),
            (
                "datum.library.create_part",
                {
                    "path": "/tmp/native-project",
                    "pool": "pool",
                    "part": "part-test",
                    "entity": "entity-test",
                    "package": "package-test",
                    "mpn": "OPA1656ID",
                    "manufacturer": "Texas Instruments",
                    "value": "OPA1656",
                    "description": "",
                    "datasheet": "",
                    "lifecycle": "Active",
                },
                "create_pool_part",
            ),
            (
                "datum.library.set_unit_pin",
                {
                    "path": "/tmp/native-project",
                    "pool": "pool",
                    "unit": "unit-test",
                    "pin": "pin-test",
                    "name": "OUT",
                    "direction": "Output",
                    "swap_group": 1,
                },
                "set_pool_unit_pin",
            ),
            (
                "datum.library.set_part_metadata",
                {"path": "/tmp/native-project", "pool": "pool", "part": "part-test", "manufacturer_jep106": 123, "value": "OPA1656", "lifecycle": "Active"},
                "set_pool_part_metadata",
            ),
            (
                "datum.library.set_part_parametric",
                {"path": "/tmp/native-project", "pool": "pool", "part": "part-test", "mode": "merge", "params": {"noise": "2.9nV/rtHz"}},
                "set_pool_part_parametric",
            ),
            (
                "datum.library.set_part_orderable_mpns",
                {"path": "/tmp/native-project", "pool": "pool", "part": "part-test", "mode": "merge", "mpns": ["OPA1656ID", "OPA1656IDR"]},
                "set_pool_part_orderable_mpns",
            ),
            (
                "datum.library.set_part_tags",
                {"path": "/tmp/native-project", "pool": "pool", "part": "part-test", "mode": "merge", "tags": ["audio", "opamp"]},
                "set_pool_part_tags",
            ),
            (
                "datum.library.set_part_packaging_options",
                {"path": "/tmp/native-project", "pool": "pool", "part": "part-test", "mode": "merge", "options": ["kind=tape_reel;qty=2500"]},
                "set_pool_part_packaging_options",
            ),
            (
                "datum.library.set_part_supply_chain",
                {
                    "path": "/tmp/native-project",
                    "pool": "pool",
                    "part": "part-test",
                    "checked_at": "2026-06-21T12:34:56Z",
                    "offers": ["{\"supplier\":\"DigiKey\",\"sku\":\"296-OPA1656ID-ND\"}"],
                },
                "set_pool_part_supply_chain",
            ),
            (
                "datum.library.set_part_behavioural_models",
                {"path": "/tmp/native-project", "pool": "pool", "part": "part-test", "mode": "merge", "models": ["{\"kind\":\"spice\",\"path\":\"models/opamp.lib\"}"]},
                "set_pool_part_behavioural_models",
            ),
            (
                "datum.library.attach_part_model",
                {"path": "/tmp/native-project", "pool": "pool", "part": "part-test", "source": "models/opamp.lib", "role": "simulation", "model_names": ["OPA1656"]},
                "attach_pool_part_model",
            ),
            (
                "datum.library.detach_part_model",
                {"path": "/tmp/native-project", "pool": "pool", "part": "part-test", "model": "model-test"},
                "detach_pool_part_model",
            ),
            (
                "datum.library.set_part_thermal",
                {"path": "/tmp/native-project", "pool": "pool", "part": "part-test", "theta_ja_c_per_w": "42.5", "clear": True},
                "set_pool_part_thermal",
            ),
            (
                "datum.library.set_part_pad_map_entry",
                {
                    "path": "/tmp/native-project",
                    "pool": "pool",
                    "part": "part-test",
                    "pad": "pad-test",
                    "gate": "gate-test",
                    "pin": "pin-test",
                },
                "set_pool_part_pad_map_entry",
            ),
            (
                "datum.library.set_part_pad_map",
                {
                    "path": "/tmp/native-project",
                    "pool": "pool",
                    "part": "part-test",
                    "mode": "replace",
                    "entries": [{"pad": "pad-test", "gate": "gate-test", "pin": "pin-test"}],
                },
                "set_pool_part_pad_map",
            ),
            (
                "datum.library.set_package_pad",
                {
                    "path": "/tmp/native-project",
                    "pool": "pool",
                    "package": "package-test",
                    "pad": "pad-test",
                    "padstack": "padstack-test",
                    "pad_name": "2",
                    "x_nm": 1000,
                    "y_nm": 2000,
                    "layer": 1,
                },
                "set_pool_package_pad",
            ),
            (
                "datum.library.set_package_courtyard_rect",
                {
                    "path": "/tmp/native-project",
                    "pool": "pool",
                    "package": "package-test",
                    "min_x_nm": 1000,
                    "min_y_nm": 2000,
                    "max_x_nm": 3000,
                    "max_y_nm": 4000,
                },
                "set_pool_package_courtyard_rect",
            ),
            (
                "datum.library.set_package_courtyard_polygon",
                {
                    "path": "/tmp/native-project",
                    "pool": "pool",
                    "package": "package-test",
                    "vertices": "0,0;1000,0;1000,1000",
                },
                "set_pool_package_courtyard_polygon",
            ),
            (
                "datum.library.add_package_silkscreen_line",
                {
                    "path": "/tmp/native-project",
                    "pool": "pool",
                    "package": "package-test",
                    "from_x_nm": 1000,
                    "from_y_nm": 2000,
                    "to_x_nm": 3000,
                    "to_y_nm": 4000,
                    "width_nm": 150000,
                },
                "add_pool_package_silkscreen_line",
            ),
            (
                "datum.library.add_package_silkscreen_rect",
                {"path": "/tmp/native-project", "pool": "pool", "package": "package-test", "min_x_nm": 1000, "min_y_nm": 2000, "max_x_nm": 3000, "max_y_nm": 4000, "width_nm": 150000},
                "add_pool_package_silkscreen_rect",
            ),
            (
                "datum.library.add_package_silkscreen_polygon",
                {"path": "/tmp/native-project", "pool": "pool", "package": "package-test", "vertices": "0,0;1000,0;1000,1000", "closed": False, "width_nm": 150000},
                "add_pool_package_silkscreen_polygon",
            ),
            (
                "datum.library.add_package_silkscreen_circle",
                {"path": "/tmp/native-project", "pool": "pool", "package": "package-test", "center_x_nm": 1000, "center_y_nm": 2000, "radius_nm": 3000, "width_nm": 150000},
                "add_pool_package_silkscreen_circle",
            ),
            (
                "datum.library.add_package_silkscreen_arc",
                {"path": "/tmp/native-project", "pool": "pool", "package": "package-test", "x_nm": 1000, "y_nm": 2000, "radius_nm": 3000, "start_angle": 0, "end_angle": 900, "width_nm": 150000},
                "add_pool_package_silkscreen_arc",
            ),
            (
                "datum.library.add_symbol_arc",
                {"path": "/tmp/native-project", "pool": "pool", "symbol": "symbol-test", "x_nm": 1000, "y_nm": 2000, "radius_nm": 3000, "start_angle": 0, "end_angle": 900, "width_nm": 150000},
                "add_pool_symbol_arc",
            ),
            (
                "datum.library.add_symbol_polygon",
                {"path": "/tmp/native-project", "pool": "pool", "symbol": "symbol-test", "vertices": "0,0;1000,0;1000,1000", "closed": True, "width_nm": 150000},
                "add_pool_symbol_polygon",
            ),
            (
                "datum.library.add_package_silkscreen_text",
                {"path": "/tmp/native-project", "pool": "pool", "package": "package-test", "text": "REF**", "x_nm": 1000, "y_nm": 2000, "rotation": 90},
                "add_pool_package_silkscreen_text",
            ),
            (
                "datum.library.add_package_model_3d",
                {"path": "/tmp/native-project", "pool": "pool", "package": "package-test", "model_path": "models/pkg.step", "format": "wrl", "scale": "1.0"},
                "add_pool_package_model_3d",
            ),
            (
                "datum.library.set_package_body_heights",
                {"path": "/tmp/native-project", "pool": "pool", "package": "package-test", "body_height_nm": 1000000, "body_height_mounted_nm": 1200000, "clear": False},
                "set_pool_package_body_heights",
            ),
            (
                "datum.library.set_object",
                {
                    "path": "/tmp/native-project",
                    "pool": "pool",
                    "kind": "symbols",
                    "object": "symbol-test",
                    "from_json": "/tmp/symbol-edited.json",
                },
                "set_pool_library_object",
            ),
        ]:
            response = host.handle_message(
                {
                    "jsonrpc": "2.0",
                    "id": 221,
                    "method": "tools/call",
                    "params": {"name": tool_name, "arguments": arguments},
                }
            )
            self.assertEqual(daemon.calls[-1][0], expected_method)
            payload = response["result"]["content"][0]["json"]
            self.assertTrue(payload["ok"])
            self.assertEqual(payload["schema"], {"name": tool_name, "version": 1})
            self.assertEqual(payload["result"]["object_uuid"], daemon.calls[-1][3] if expected_method in {"create_pool_unit", "set_pool_unit_pin", "create_pool_symbol", "add_pool_symbol_line", "add_pool_symbol_rect", "add_pool_symbol_circle", "add_pool_symbol_arc", "add_pool_symbol_polygon", "add_pool_symbol_text", "set_pool_symbol_pin_anchor", "create_pool_entity", "create_pool_padstack", "create_pool_package", "set_pool_package_pad", "set_pool_package_courtyard_rect", "set_pool_package_courtyard_polygon", "add_pool_package_silkscreen_line", "add_pool_package_silkscreen_rect", "add_pool_package_silkscreen_polygon", "add_pool_package_silkscreen_circle", "add_pool_package_silkscreen_arc", "add_pool_package_silkscreen_text", "add_pool_package_model_3d", "set_pool_package_body_heights", "create_pool_part", "set_pool_part_metadata", "set_pool_part_parametric", "set_pool_part_orderable_mpns", "set_pool_part_tags", "set_pool_part_packaging_options", "set_pool_part_supply_chain", "set_pool_part_behavioural_models", "attach_pool_part_model", "detach_pool_part_model", "set_pool_part_thermal", "set_pool_part_pad_map_entry", "set_pool_part_pad_map"} else daemon.calls[-1][4])
