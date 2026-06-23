#!/usr/bin/env python3
"""Native pool-library MCP tools/call dispatch tests."""

from __future__ import annotations

import unittest

from server_runtime import StdioToolHost
from test_support import FakeDaemonClient


class TestDispatchLibrary(unittest.TestCase):
    def test_tools_call_dispatches_pool_library_reads(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        for tool_name, arguments, expected_call in [
            (
                "get_pool_library_objects",
                {
                    "path": "/tmp/native-project",
                    "kind": "symbols",
                    "include_payload": True,
                },
                ("get_pool_library_objects", "/tmp/native-project", None, "symbols", None, True),
            ),
            (
                "show_pool_library_object",
                {
                    "path": "/tmp/native-project",
                    "object": "symbol-test",
                    "kind": "symbols",
                },
                ("show_pool_library_object", "/tmp/native-project", "symbol-test", None, "symbols"),
            ),
            (
                "get_pool_model_blobs",
                {
                    "path": "/tmp/native-project",
                    "pool": "pool",
                    "role": "spice",
                    "sha256": "abc123",
                },
                ("get_pool_model_blobs", "/tmp/native-project", "pool", "spice", "abc123"),
            ),
            (
                "gc_pool_model_blobs",
                {
                    "path": "/tmp/native-project",
                    "pool": "pool",
                    "role": "spice",
                    "sha256": "abc123",
                    "apply": True,
                },
                ("gc_pool_model_blobs", "/tmp/native-project", "pool", "spice", "abc123", True),
            ),
        ]:
            response = host.handle_message(
                {
                    "jsonrpc": "2.0",
                    "id": 218,
                    "method": "tools/call",
                    "params": {"name": tool_name, "arguments": arguments},
                }
            )
            self.assertEqual(daemon.calls[-1], expected_call)
            payload = response["result"]["content"][0]["json"]
            self.assertIn(payload["contract"], {
                "native_project_library_objects_query_v1",
                "native_project_pool_models_query_v1",
                "native_project_pool_model_gc_v1",
            })

    def test_tools_call_dispatches_canonical_library_read_aliases(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        for tool_name, arguments, expected_method in [
            (
                "datum.library.list_objects",
                {"path": "/tmp/native-project", "kind": "symbols", "include_payload": True},
                "get_pool_library_objects",
            ),
            (
                "datum.library.show_object",
                {"path": "/tmp/native-project", "object": "symbol-test", "kind": "symbols"},
                "show_pool_library_object",
            ),
            (
                "datum.library.pool_models",
                {"path": "/tmp/native-project", "pool": "pool", "role": "spice"},
                "get_pool_model_blobs",
            ),
            (
                "datum.library.gc_pool_models",
                {"path": "/tmp/native-project", "pool": "pool", "role": "spice", "apply": True},
                "gc_pool_model_blobs",
            ),
        ]:
            response = host.handle_message(
                {
                    "jsonrpc": "2.0",
                    "id": 219,
                    "method": "tools/call",
                    "params": {"name": tool_name, "arguments": arguments},
                }
            )
            self.assertEqual(daemon.calls[-1][0], expected_method)
            payload = response["result"]["content"][0]["json"]
            self.assertTrue(payload["ok"])
            self.assertEqual(payload["schema"], {"name": tool_name, "version": 1})
            self.assertEqual(
                payload["result"].get(
                    "object_count",
                    payload["result"].get("model_count", payload["result"].get("planned_count")),
                ),
                1,
            )

    def test_tools_call_dispatches_pool_library_writes(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        for tool_name, arguments, expected_call in [
            (
                "create_pool_library_object",
                {
                    "path": "/tmp/native-project",
                    "pool": "pool",
                    "kind": "symbols",
                    "object": "symbol-test",
                    "from_json": "/tmp/symbol.json",
                },
                (
                    "create_pool_library_object",
                    "/tmp/native-project",
                    "pool",
                    "symbols",
                    "symbol-test",
                    "/tmp/symbol.json",
                ),
            ),
            (
                "delete_pool_library_object",
                {
                    "path": "/tmp/native-project",
                    "pool": "pool",
                    "kind": "symbols",
                    "object": "symbol-test",
                },
                (
                    "delete_pool_library_object",
                    "/tmp/native-project",
                    "pool",
                    "symbols",
                    "symbol-test",
                ),
            ),
            (
                "create_pool_unit",
                {
                    "path": "/tmp/native-project",
                    "pool": "pool",
                    "unit": "unit-test",
                    "name": "OpAmpUnit",
                    "manufacturer": "Datum",
                },
                (
                    "create_pool_unit",
                    "/tmp/native-project",
                    "pool",
                    "unit-test",
                    "OpAmpUnit",
                    "Datum",
                ),
            ),
            (
                "create_pool_symbol",
                {
                    "path": "/tmp/native-project",
                    "pool": "pool",
                    "symbol": "symbol-test",
                    "unit": "unit-test",
                    "name": "OpAmpSymbol",
                },
                (
                    "create_pool_symbol",
                    "/tmp/native-project",
                    "pool",
                    "symbol-test",
                    "unit-test",
                    "OpAmpSymbol",
                ),
            ),
            (
                "add_pool_symbol_line",
                {
                    "path": "/tmp/native-project",
                    "pool": "pool",
                    "symbol": "symbol-test",
                    "from_x_nm": 0,
                    "from_y_nm": 0,
                    "to_x_nm": 1000,
                    "to_y_nm": 0,
                    "width_nm": 100,
                },
                (
                    "add_pool_symbol_line",
                    "/tmp/native-project",
                    "pool",
                    "symbol-test",
                    0,
                    0,
                    1000,
                    0,
                    100,
                ),
            ),
            (
                "set_pool_symbol_pin_anchor",
                {"path": "/tmp/native-project", "pool": "pool", "symbol": "symbol-test", "pin": "pin-test", "x_nm": 100, "y_nm": 200},
                ("set_pool_symbol_pin_anchor", "/tmp/native-project", "pool", "symbol-test", "pin-test", 100, 200),
            ),
            (
                "add_pool_symbol_rect",
                {"path": "/tmp/native-project", "pool": "pool", "symbol": "symbol-test", "min_x_nm": 0, "min_y_nm": 0, "max_x_nm": 1000, "max_y_nm": 2000, "width_nm": 100},
                ("add_pool_symbol_rect", "/tmp/native-project", "pool", "symbol-test", 0, 0, 1000, 2000, 100),
            ),
            (
                "add_pool_symbol_circle",
                {"path": "/tmp/native-project", "pool": "pool", "symbol": "symbol-test", "center_x_nm": 500, "center_y_nm": 600, "radius_nm": 250, "width_nm": 100},
                ("add_pool_symbol_circle", "/tmp/native-project", "pool", "symbol-test", 500, 600, 250, 100),
            ),
            (
                "add_pool_symbol_text",
                {"path": "/tmp/native-project", "pool": "pool", "symbol": "symbol-test", "text": "REF**", "x_nm": 1000, "y_nm": 2000, "rotation": 90},
                ("add_pool_symbol_text", "/tmp/native-project", "pool", "symbol-test", "REF**", 1000, 2000, 90),
            ),
            (
                "create_pool_entity",
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
                (
                    "create_pool_entity",
                    "/tmp/native-project",
                    "pool",
                    "entity-test",
                    "gate-test",
                    "unit-test",
                    "symbol-test",
                    "DualOpAmp",
                    "U",
                    "Datum",
                    "A",
                ),
            ),
            (
                "create_pool_padstack",
                {
                    "path": "/tmp/native-project",
                    "pool": "pool",
                    "padstack": "padstack-test",
                    "name": "RoundViaPad",
                    "aperture": "circle",
                    "diameter_nm": 1200000,
                    "drill_nm": 600000,
                },
                (
                    "create_pool_padstack",
                    "/tmp/native-project",
                    "pool",
                    "padstack-test",
                    "RoundViaPad",
                    "circle",
                    1200000,
                    None,
                    None,
                    600000,
                ),
            ),
            (
                "create_pool_package",
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
                (
                    "create_pool_package",
                    "/tmp/native-project",
                    "pool",
                    "package-test",
                    "SOT23",
                    "pad-test",
                    "padstack-test",
                    "1",
                    1000,
                    2000,
                    1,
                ),
            ),
            (
                "create_pool_part",
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
                (
                    "create_pool_part",
                    "/tmp/native-project",
                    "pool",
                    "part-test",
                    "entity-test",
                    "package-test",
                    "OPA1656ID",
                    "Texas Instruments",
                    "OPA1656",
                    "",
                    "",
                    "Active",
                ),
            ),
            (
                "set_pool_unit_pin",
                {
                    "path": "/tmp/native-project",
                    "pool": "pool",
                    "unit": "unit-test",
                    "pin": "pin-test",
                    "name": "OUT",
                    "direction": "Output",
                    "swap_group": 1,
                },
                (
                    "set_pool_unit_pin",
                    "/tmp/native-project",
                    "pool",
                    "unit-test",
                    "pin-test",
                    "OUT",
                    "Output",
                    1,
                ),
            ),
            (
                "set_pool_part_metadata",
                {"path": "/tmp/native-project", "pool": "pool", "part": "part-test", "mpn": "OPA1656ID", "manufacturer": "Texas Instruments", "manufacturer_jep106": 123},
                ("set_pool_part_metadata", "/tmp/native-project", "pool", "part-test", "OPA1656ID", "Texas Instruments", 123, None, None, None, None),
            ),
            (
                "set_pool_part_parametric",
                {"path": "/tmp/native-project", "pool": "pool", "part": "part-test", "mode": "replace", "params": {"gbw": "53MHz", "slew_rate": "24V/us"}},
                ("set_pool_part_parametric", "/tmp/native-project", "pool", "part-test", "replace", {"gbw": "53MHz", "slew_rate": "24V/us"}),
            ),
            (
                "set_pool_part_orderable_mpns",
                {"path": "/tmp/native-project", "pool": "pool", "part": "part-test", "mode": "replace", "mpns": ["OPA1656ID", "OPA1656IDR"]},
                ("set_pool_part_orderable_mpns", "/tmp/native-project", "pool", "part-test", "replace", ["OPA1656ID", "OPA1656IDR"]),
            ),
            (
                "set_pool_part_tags",
                {"path": "/tmp/native-project", "pool": "pool", "part": "part-test", "mode": "replace", "tags": ["audio", "opamp"]},
                ("set_pool_part_tags", "/tmp/native-project", "pool", "part-test", "replace", ["audio", "opamp"]),
            ),
            (
                "set_pool_part_packaging_options",
                {"path": "/tmp/native-project", "pool": "pool", "part": "part-test", "mode": "replace", "options": ["kind=tape_reel;qty=2500", "{\"kind\":\"tray\",\"quantity\":90}"]},
                ("set_pool_part_packaging_options", "/tmp/native-project", "pool", "part-test", "replace", ["kind=tape_reel;qty=2500", "{\"kind\":\"tray\",\"quantity\":90}"]),
            ),
            (
                "set_pool_part_supply_chain",
                {
                    "path": "/tmp/native-project",
                    "pool": "pool",
                    "part": "part-test",
                    "clear": True,
                    "checked_at": "2026-06-21T12:34:56Z",
                    "offers": ["{\"supplier\":\"DigiKey\",\"sku\":\"296-OPA1656ID-ND\"}"],
                },
                ("set_pool_part_supply_chain", "/tmp/native-project", "pool", "part-test", True, "2026-06-21T12:34:56Z", ["{\"supplier\":\"DigiKey\",\"sku\":\"296-OPA1656ID-ND\"}"]),
            ),
            (
                "set_pool_part_behavioural_models",
                {"path": "/tmp/native-project", "pool": "pool", "part": "part-test", "mode": "replace", "models": ["{\"kind\":\"spice\",\"path\":\"models/opamp.lib\"}", "not-json"]},
                ("set_pool_part_behavioural_models", "/tmp/native-project", "pool", "part-test", "replace", ["{\"kind\":\"spice\",\"path\":\"models/opamp.lib\"}", "not-json"]),
            ),
            (
                "attach_pool_part_model",
                {
                    "path": "/tmp/native-project",
                    "pool": "pool",
                    "part": "part-test",
                    "source": "models/opamp.lib",
                    "role": "simulation",
                    "dialect": "spice",
                    "model_names": ["OPA1656", "OPA1656_ALT"],
                    "encrypted": True,
                    "encryption_scheme": "{\"kind\":\"aes\"}",
                    "vendor": "Texas Instruments",
                    "fetched_at": "2026-06-21T12:34:56Z",
                    "format_metadata_json": "{\"temperature\":\"25C\"}",
                },
                ("attach_pool_part_model", "/tmp/native-project", "pool", "part-test", "models/opamp.lib", "simulation", "spice", ["OPA1656", "OPA1656_ALT"], True, "{\"kind\":\"aes\"}", "Texas Instruments", "2026-06-21T12:34:56Z", "{\"temperature\":\"25C\"}"),
            ),
            (
                "detach_pool_part_model",
                {"path": "/tmp/native-project", "pool": "pool", "part": "part-test", "attachment": "attachment-test"},
                ("detach_pool_part_model", "/tmp/native-project", "pool", "part-test", "attachment-test", None),
            ),
            (
                "set_pool_part_thermal",
                {
                    "path": "/tmp/native-project",
                    "pool": "pool",
                    "part": "part-test",
                    "theta_ja_c_per_w": 42.5,
                    "theta_jc_top_c_per_w": "8.2",
                    "max_junction_c": 150,
                    "thermal_reference": "JEDEC JESD51",
                },
                ("set_pool_part_thermal", "/tmp/native-project", "pool", "part-test", 42.5, "8.2", None, None, 150, "JEDEC JESD51", False),
            ),
            (
                "set_pool_part_pad_map_entry",
                {
                    "path": "/tmp/native-project",
                    "pool": "pool",
                    "part": "part-test",
                    "pad": "pad-test",
                    "gate": "gate-test",
                    "pin": "pin-test",
                },
                (
                    "set_pool_part_pad_map_entry",
                    "/tmp/native-project",
                    "pool",
                    "part-test",
                    "pad-test",
                    "gate-test",
                    "pin-test",
                ),
            ),
            (
                "set_pool_part_pad_map",
                {
                    "path": "/tmp/native-project",
                    "pool": "pool",
                    "part": "part-test",
                    "mode": "replace",
                    "entries": [{"pad": "pad-test", "gate": "gate-test", "pin": "pin-test"}],
                },
                (
                    "set_pool_part_pad_map",
                    "/tmp/native-project",
                    "pool",
                    "part-test",
                    "replace",
                    [{"pad": "pad-test", "gate": "gate-test", "pin": "pin-test"}],
                ),
            ),
            (
                "set_pool_package_pad",
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
                (
                    "set_pool_package_pad",
                    "/tmp/native-project",
                    "pool",
                    "package-test",
                    "pad-test",
                    "padstack-test",
                    "2",
                    1000,
                    2000,
                    1,
                ),
            ),
            (
                "set_pool_package_courtyard_rect",
                {
                    "path": "/tmp/native-project",
                    "pool": "pool",
                    "package": "package-test",
                    "min_x_nm": 1000,
                    "min_y_nm": 2000,
                    "max_x_nm": 3000,
                    "max_y_nm": 4000,
                },
                (
                    "set_pool_package_courtyard_rect",
                    "/tmp/native-project",
                    "pool",
                    "package-test",
                    1000,
                    2000,
                    3000,
                    4000,
                ),
            ),
            (
                "set_pool_package_courtyard_polygon",
                {
                    "path": "/tmp/native-project",
                    "pool": "pool",
                    "package": "package-test",
                    "vertices": "0,0;1000,0;1000,1000",
                },
                (
                    "set_pool_package_courtyard_polygon",
                    "/tmp/native-project",
                    "pool",
                    "package-test",
                    "0,0;1000,0;1000,1000",
                ),
            ),
            (
                "add_pool_package_silkscreen_line",
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
                (
                    "add_pool_package_silkscreen_line",
                    "/tmp/native-project",
                    "pool",
                    "package-test",
                    1000,
                    2000,
                    3000,
                    4000,
                    150000,
                ),
            ),
            (
                "add_pool_package_silkscreen_rect",
                {
                    "path": "/tmp/native-project", "pool": "pool", "package": "package-test",
                    "min_x_nm": 1000, "min_y_nm": 2000, "max_x_nm": 3000, "max_y_nm": 4000, "width_nm": 150000,
                },
                (
                    "add_pool_package_silkscreen_rect", "/tmp/native-project", "pool", "package-test",
                    1000, 2000, 3000, 4000, 150000,
                ),
            ),
            (
                "add_pool_package_silkscreen_polygon",
                {"path": "/tmp/native-project", "pool": "pool", "package": "package-test", "vertices": "0,0;1000,0;1000,1000", "closed": True, "width_nm": 150000},
                ("add_pool_package_silkscreen_polygon", "/tmp/native-project", "pool", "package-test", "0,0;1000,0;1000,1000", True, 150000),
            ),
            (
                "add_pool_package_silkscreen_circle",
                {"path": "/tmp/native-project", "pool": "pool", "package": "package-test", "center_x_nm": 1000, "center_y_nm": 2000, "radius_nm": 3000, "width_nm": 150000},
                ("add_pool_package_silkscreen_circle", "/tmp/native-project", "pool", "package-test", 1000, 2000, 3000, 150000),
            ),
            (
                "add_pool_package_silkscreen_arc",
                {"path": "/tmp/native-project", "pool": "pool", "package": "package-test", "x_nm": 1000, "y_nm": 2000, "radius_nm": 3000, "start_angle": 0, "end_angle": 900, "width_nm": 150000},
                ("add_pool_package_silkscreen_arc", "/tmp/native-project", "pool", "package-test", 1000, 2000, 3000, 0, 900, 150000),
            ),
            (
                "add_pool_symbol_arc",
                {"path": "/tmp/native-project", "pool": "pool", "symbol": "symbol-test", "x_nm": 1000, "y_nm": 2000, "radius_nm": 3000, "start_angle": 0, "end_angle": 900, "width_nm": 150000},
                ("add_pool_symbol_arc", "/tmp/native-project", "pool", "symbol-test", 1000, 2000, 3000, 0, 900, 150000),
            ),
            (
                "add_pool_symbol_polygon",
                {"path": "/tmp/native-project", "pool": "pool", "symbol": "symbol-test", "vertices": "0,0;1000,0;1000,1000", "closed": True, "width_nm": 150000},
                ("add_pool_symbol_polygon", "/tmp/native-project", "pool", "symbol-test", "0,0;1000,0;1000,1000", True, 150000),
            ),
            (
                "add_pool_package_silkscreen_text",
                {"path": "/tmp/native-project", "pool": "pool", "package": "package-test", "text": "REF**", "x_nm": 1000, "y_nm": 2000, "rotation": 90},
                ("add_pool_package_silkscreen_text", "/tmp/native-project", "pool", "package-test", "REF**", 1000, 2000, 90),
            ),
            (
                "add_pool_package_model_3d",
                {"path": "/tmp/native-project", "pool": "pool", "package": "package-test", "model_path": "models/pkg.step", "transform_json": "{\"scale\":1}", "format": "step", "tx_nm": 10, "ty_nm": 20, "tz_nm": 30, "roll_tenths_deg": 1, "pitch_tenths_deg": 2, "yaw_tenths_deg": 3, "scale": 1.25},
                ("add_pool_package_model_3d", "/tmp/native-project", "pool", "package-test", "models/pkg.step", "{\"scale\":1}", "step", 10, 20, 30, 1, 2, 3, 1.25),
            ),
            (
                "set_pool_package_body_heights",
                {"path": "/tmp/native-project", "pool": "pool", "package": "package-test", "body_height_nm": 1000000, "body_height_mounted_nm": 1200000, "clear": False},
                ("set_pool_package_body_heights", "/tmp/native-project", "pool", "package-test", 1000000, 1200000, False),
            ),
            (
                "set_pool_library_object",
                {
                    "path": "/tmp/native-project",
                    "pool": "pool",
                    "kind": "symbols",
                    "object": "symbol-test",
                    "from_json": "/tmp/symbol-edited.json",
                },
                (
                    "set_pool_library_object",
                    "/tmp/native-project",
                    "pool",
                    "symbols",
                    "symbol-test",
                    "/tmp/symbol-edited.json",
                ),
            ),
        ]:
            response = host.handle_message(
                {
                    "jsonrpc": "2.0",
                    "id": 220,
                    "method": "tools/call",
                    "params": {"name": tool_name, "arguments": arguments},
                }
            )
            self.assertEqual(daemon.calls[-1], expected_call)
            payload = response["result"]["content"][0]["json"]
            self.assertEqual(payload["object_uuid"], expected_call[3] if expected_call[0] in {"create_pool_unit", "set_pool_unit_pin", "create_pool_symbol", "add_pool_symbol_line", "add_pool_symbol_rect", "add_pool_symbol_circle", "add_pool_symbol_arc", "add_pool_symbol_polygon", "add_pool_symbol_text", "set_pool_symbol_pin_anchor", "create_pool_entity", "create_pool_padstack", "create_pool_package", "set_pool_package_pad", "set_pool_package_courtyard_rect", "set_pool_package_courtyard_polygon", "add_pool_package_silkscreen_line", "add_pool_package_silkscreen_rect", "add_pool_package_silkscreen_polygon", "add_pool_package_silkscreen_circle", "add_pool_package_silkscreen_arc", "add_pool_package_silkscreen_text", "add_pool_package_model_3d", "set_pool_package_body_heights", "create_pool_part", "set_pool_part_metadata", "set_pool_part_parametric", "set_pool_part_orderable_mpns", "set_pool_part_tags", "set_pool_part_packaging_options", "set_pool_part_supply_chain", "set_pool_part_behavioural_models", "attach_pool_part_model", "detach_pool_part_model", "set_pool_part_thermal", "set_pool_part_pad_map_entry", "set_pool_part_pad_map"} else expected_call[4])
