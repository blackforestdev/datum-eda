#!/usr/bin/env python3
"""Proposal MCP tools/call dispatch tests."""

from __future__ import annotations

import unittest

from server_runtime import StdioToolHost
from test_support import FakeDaemonClient


class TestDispatchProposals(unittest.TestCase):
    def test_tools_call_dispatches_create_proposal(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {"jsonrpc": "2.0", "id": 203, "method": "tools/call", "params": {"name": "create_proposal", "arguments": {"path": "/tmp/native-project", "batch": "/tmp/batch.json", "rationale": "review batch", "proposal": "proposal-test", "source": "assistant", "checks_run": ["check-test"], "finding_fingerprints": ["sha256:test"]}}}
        )
        self.assertEqual(daemon.calls, [("create_proposal", "/tmp/native-project", "/tmp/batch.json", "review batch", "proposal-test", "assistant", ["check-test"], ["sha256:test"])])
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["contract"], "proposal_create_v1")
        self.assertEqual(
            payload["validation"]["policy"],
            "accepted_revision_guarded_source_policy_v1",
        )

    def test_tools_call_dispatches_create_draw_wire_proposal(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {"jsonrpc": "2.0", "id": 212, "method": "tools/call", "params": {"name": "create_draw_wire_proposal", "arguments": {"path": "/tmp/native-project", "sheet": "sheet-uuid", "from_x_nm": 0, "from_y_nm": 10, "to_x_nm": 100, "to_y_nm": 110, "proposal": "proposal-wire", "rationale": "review wire"}}}
        )
        self.assertEqual(daemon.calls, [("create_draw_wire_proposal", "/tmp/native-project", "sheet-uuid", 0, 10, 100, 110, "proposal-wire", "review wire")])
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["contract"], "proposal_create_v1")
        self.assertEqual(payload["action"], "propose_draw_wire")

    def test_tools_call_dispatches_create_place_label_proposal(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {"jsonrpc": "2.0", "id": 213, "method": "tools/call", "params": {"name": "create_place_label_proposal", "arguments": {"path": "/tmp/native-project", "sheet": "sheet-uuid", "name": "VCC", "x_nm": 100, "y_nm": 200, "kind": "power", "proposal": "proposal-label", "rationale": "review label"}}}
        )
        self.assertEqual(daemon.calls, [("create_place_label_proposal", "/tmp/native-project", "sheet-uuid", "VCC", 100, 200, "power", "proposal-label", "review label")])
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["contract"], "proposal_create_v1")
        self.assertEqual(payload["action"], "propose_place_label")

    def test_tools_call_dispatches_create_place_symbol_proposal(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {"jsonrpc": "2.0", "id": 214, "method": "tools/call", "params": {"name": "create_place_symbol_proposal", "arguments": {"path": "/tmp/native-project", "sheet": "sheet-uuid", "reference": "U1", "value": "OPA", "x_nm": 100, "y_nm": 200, "lib_id": "Device:R", "rotation_deg": 90, "mirrored": True, "proposal": "proposal-symbol", "rationale": "review symbol"}}}
        )
        self.assertEqual(daemon.calls, [("create_place_symbol_proposal", "/tmp/native-project", "sheet-uuid", "U1", "OPA", 100, 200, "Device:R", 90, True, "proposal-symbol", "review symbol")])
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["contract"], "proposal_create_v1")
        self.assertEqual(payload["action"], "propose_place_symbol")

    def test_tools_call_dispatches_create_board_component_replacement_proposal(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {"jsonrpc": "2.0", "id": 218, "method": "tools/call", "params": {"name": "create_board_component_replacement_proposal", "arguments": {"path": "/tmp/native-project", "component": "component-uuid", "package": "package-uuid", "part": "part-uuid", "value": "10k", "proposal": "proposal-replace", "rationale": "review replacement"}}}
        )
        self.assertEqual(daemon.calls, [("create_board_component_replacement_proposal", "/tmp/native-project", "component-uuid", "package-uuid", "part-uuid", "10k", "proposal-replace", "review replacement")])
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["contract"], "proposal_create_v1")
        self.assertEqual(payload["action"], "propose_board_component_replacement")

    def test_tools_call_dispatches_create_board_component_replacements_proposal(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        replacements = [
            {"component": "component-u1", "package": "package-u1", "part": "part-u1", "value": "10k"},
            {"component": "component-u2", "part": "part-u2", "value": "22k"},
        ]
        response = host.handle_message(
            {"jsonrpc": "2.0", "id": 220, "method": "tools/call", "params": {"name": "create_board_component_replacements_proposal", "arguments": {"path": "/tmp/native-project", "replacements": replacements, "proposal": "proposal-replacements", "rationale": "review replacements"}}}
        )
        self.assertEqual(daemon.calls, [("create_board_component_replacements_proposal", "/tmp/native-project", replacements, "proposal-replacements", "review replacements")])
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["contract"], "proposal_create_v1")
        self.assertEqual(payload["action"], "propose_board_component_replacement")
        self.assertEqual(payload["replacement_count"], 2)

    def test_tools_call_dispatches_create_board_component_replacement_plan_proposal(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        selections = [
            {"uuid": "component-u1", "package_uuid": "package-u1", "part_uuid": "part-u1", "value": "10k"},
            {"uuid": "component-u2", "part_uuid": "part-u2"},
        ]
        response = host.handle_message(
            {"jsonrpc": "2.0", "id": 222, "method": "tools/call", "params": {"name": "create_board_component_replacement_plan_proposal", "arguments": {"path": "/tmp/native-project", "selections": selections, "proposal": "proposal-plan", "rationale": "review plan"}}}
        )
        self.assertEqual(daemon.calls, [("create_board_component_replacement_plan_proposal", "/tmp/native-project", selections, "proposal-plan", "review plan")])
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["contract"], "proposal_create_v1")
        self.assertEqual(payload["action"], "propose_board_component_replacement")
        self.assertEqual(payload["selection_count"], 2)

    def test_tools_call_dispatches_create_pool_library_object_proposal(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {"jsonrpc": "2.0", "id": 224, "method": "tools/call", "params": {"name": "datum.proposal.create_pool_library_object", "arguments": {"path": "/tmp/native-project", "kind": "symbols", "object": "symbol-uuid", "from_json": "/tmp/symbol.json", "pool": "pool", "proposal": "proposal-library", "rationale": "review library object"}}}
        )
        self.assertEqual(daemon.calls, [("create_pool_library_object_proposal", "/tmp/native-project", "symbols", "symbol-uuid", "/tmp/symbol.json", "pool", "proposal-library", "review library object")])
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["contract"], "proposal_create_v1")
        self.assertEqual(payload["action"], "create_pool_library_object_proposal")
        self.assertEqual(payload["object_uuid"], "symbol-uuid")

    def test_tools_call_dispatches_create_pool_unit_proposal(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {"jsonrpc": "2.0", "id": 225, "method": "tools/call", "params": {"name": "datum.proposal.create_pool_unit", "arguments": {"path": "/tmp/native-project", "unit": "unit-uuid", "name": "OpAmpUnit", "manufacturer": "Datum Semi", "pool": "pool", "proposal": "proposal-unit", "rationale": "review unit"}}}
        )
        self.assertEqual(daemon.calls, [("create_pool_unit_proposal", "/tmp/native-project", "unit-uuid", "OpAmpUnit", "Datum Semi", "pool", "proposal-unit", "review unit")])
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["contract"], "proposal_create_v1")
        self.assertEqual(payload["action"], "create_pool_unit_proposal")
        self.assertEqual(payload["unit_uuid"], "unit-uuid")

    def test_tools_call_dispatches_create_pool_symbol_proposal(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {"jsonrpc": "2.0", "id": 226, "method": "tools/call", "params": {"name": "datum.proposal.create_pool_symbol", "arguments": {"path": "/tmp/native-project", "symbol": "symbol-uuid", "unit": "unit-uuid", "name": "OpAmpSymbol", "pool": "pool", "proposal": "proposal-symbol", "rationale": "review symbol"}}}
        )
        self.assertEqual(daemon.calls, [("create_pool_symbol_proposal", "/tmp/native-project", "symbol-uuid", "unit-uuid", "OpAmpSymbol", "pool", "proposal-symbol", "review symbol")])
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["contract"], "proposal_create_v1")
        self.assertEqual(payload["action"], "create_pool_symbol_proposal")
        self.assertEqual(payload["symbol_uuid"], "symbol-uuid")
        self.assertEqual(payload["unit_uuid"], "unit-uuid")

    def test_tools_call_dispatches_create_pool_entity_proposal(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {"jsonrpc": "2.0", "id": 227, "method": "tools/call", "params": {"name": "datum.proposal.create_pool_entity", "arguments": {"path": "/tmp/native-project", "entity": "entity-uuid", "gate": "gate-uuid", "unit": "unit-uuid", "symbol": "symbol-uuid", "name": "OpAmp", "prefix": "U", "manufacturer": "Datum Semi", "gate_name": "A", "pool": "pool", "proposal": "proposal-entity", "rationale": "review entity"}}}
        )
        self.assertEqual(daemon.calls, [("create_pool_entity_proposal", "/tmp/native-project", "entity-uuid", "gate-uuid", "unit-uuid", "symbol-uuid", "OpAmp", "U", "Datum Semi", "A", "pool", "proposal-entity", "review entity")])
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["contract"], "proposal_create_v1")
        self.assertEqual(payload["action"], "create_pool_entity_proposal")
        self.assertEqual(payload["entity_uuid"], "entity-uuid")
        self.assertEqual(payload["symbol_uuid"], "symbol-uuid")

    def test_tools_call_dispatches_create_pool_padstack_proposal(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {"jsonrpc": "2.0", "id": 228, "method": "tools/call", "params": {"name": "datum.proposal.create_pool_padstack", "arguments": {"path": "/tmp/native-project", "padstack": "padstack-uuid", "name": "RoundViaPad", "aperture": "circle", "diameter_nm": 1200000, "drill_nm": 600000, "pool": "pool", "proposal": "proposal-padstack", "rationale": "review padstack"}}}
        )
        self.assertEqual(daemon.calls, [("create_pool_padstack_proposal", "/tmp/native-project", "padstack-uuid", "RoundViaPad", "circle", 1200000, None, None, 600000, "pool", "proposal-padstack", "review padstack")])
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["contract"], "proposal_create_v1")
        self.assertEqual(payload["action"], "create_pool_padstack_proposal")
        self.assertEqual(payload["padstack_uuid"], "padstack-uuid")

    def test_tools_call_dispatches_create_pool_package_proposal(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {"jsonrpc": "2.0", "id": 229, "method": "tools/call", "params": {"name": "datum.proposal.create_pool_package", "arguments": {"path": "/tmp/native-project", "package": "package-uuid", "name": "SOT23", "pad": "pad-uuid", "padstack": "padstack-uuid", "pad_name": "1", "x_nm": 1000, "y_nm": 2000, "layer": 1, "pool": "pool", "proposal": "proposal-package", "rationale": "review package"}}}
        )
        self.assertEqual(daemon.calls, [("create_pool_package_proposal", "/tmp/native-project", "package-uuid", "SOT23", "pad-uuid", "padstack-uuid", "1", 1000, 2000, 1, "pool", "proposal-package", "review package")])
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["contract"], "proposal_create_v1")
        self.assertEqual(payload["action"], "create_pool_package_proposal")
        self.assertEqual(payload["package_uuid"], "package-uuid")

    def test_tools_call_dispatches_body_only_create_pool_package_proposal(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {"jsonrpc": "2.0", "id": 230, "method": "tools/call", "params": {"name": "datum.proposal.create_pool_package", "arguments": {"path": "/tmp/native-project", "package": "package-uuid", "name": "SOT23", "pool": "pool", "proposal": "proposal-package", "rationale": "review package body"}}}
        )
        self.assertEqual(daemon.calls, [("create_pool_package_proposal", "/tmp/native-project", "package-uuid", "SOT23", None, None, "1", 0, 0, 1, "pool", "proposal-package", "review package body")])
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["contract"], "proposal_create_v1")
        self.assertEqual(payload["action"], "create_pool_package_proposal")
        self.assertEqual(payload["package_uuid"], "package-uuid")

    def test_tools_call_dispatches_create_pool_footprint_proposal(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {"jsonrpc": "2.0", "id": 233, "method": "tools/call", "params": {"name": "datum.proposal.create_pool_footprint", "arguments": {"path": "/tmp/native-project", "footprint": "footprint-uuid", "package": "package-uuid", "name": "SOT23_LandPattern", "pool": "pool", "proposal": "proposal-footprint", "rationale": "review footprint"}}}
        )
        self.assertEqual(daemon.calls, [("create_pool_footprint_proposal", "/tmp/native-project", "footprint-uuid", "package-uuid", "SOT23_LandPattern", "pool", "proposal-footprint", "review footprint")])
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["contract"], "proposal_create_v1")
        self.assertEqual(payload["action"], "create_pool_footprint_proposal")
        self.assertEqual(payload["footprint_uuid"], "footprint-uuid")

    def test_tools_call_dispatches_set_pool_package_pad_proposal(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {"jsonrpc": "2.0", "id": 230, "method": "tools/call", "params": {"name": "datum.proposal.set_pool_package_pad", "arguments": {"path": "/tmp/native-project", "package": "package-uuid", "pad": "pad-uuid", "padstack": "padstack-uuid", "pad_name": "2", "x_nm": 3000, "y_nm": 4000, "layer": 1, "pool": "pool", "proposal": "proposal-package-pad", "rationale": "review package pad"}}}
        )
        self.assertEqual(daemon.calls, [("set_pool_package_pad_proposal", "/tmp/native-project", "package-uuid", "pad-uuid", "padstack-uuid", "2", 3000, 4000, 1, "pool", "proposal-package-pad", "review package pad")])
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["contract"], "proposal_create_v1")
        self.assertEqual(payload["action"], "set_pool_package_pad_proposal")
        self.assertEqual(payload["package_uuid"], "package-uuid")

    def test_tools_call_dispatches_set_pool_footprint_pad_proposal(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {"jsonrpc": "2.0", "id": 234, "method": "tools/call", "params": {"name": "datum.proposal.set_pool_footprint_pad", "arguments": {"path": "/tmp/native-project", "footprint": "footprint-uuid", "pad": "pad-uuid", "padstack": "padstack-uuid", "pad_name": "2", "x_nm": 3000, "y_nm": 4000, "layer": 1, "pool": "pool", "proposal": "proposal-footprint-pad", "rationale": "review footprint pad"}}}
        )
        self.assertEqual(daemon.calls, [("set_pool_footprint_pad_proposal", "/tmp/native-project", "footprint-uuid", "pad-uuid", "padstack-uuid", "2", 3000, 4000, 1, "pool", "proposal-footprint-pad", "review footprint pad")])
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["contract"], "proposal_create_v1")
        self.assertEqual(payload["action"], "set_pool_footprint_pad_proposal")
        self.assertEqual(payload["footprint_uuid"], "footprint-uuid")

    def test_tools_call_dispatches_create_pool_pin_pad_map_proposal(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        entries = ["pin-uuid:pad-uuid"]
        response = host.handle_message(
            {"jsonrpc": "2.0", "id": 239, "method": "tools/call", "params": {"name": "datum.proposal.create_pool_pin_pad_map", "arguments": {"path": "/tmp/native-project", "map": "map-uuid", "part": "part-uuid", "footprint": "footprint-uuid", "entries": entries, "set_default": True, "pool": "pool", "proposal": "proposal-pin-pad-map", "rationale": "review pin pad map"}}}
        )
        self.assertEqual(daemon.calls, [("create_pool_pin_pad_map_proposal", "/tmp/native-project", "map-uuid", "part-uuid", entries, "footprint-uuid", True, "pool", "proposal-pin-pad-map", "review pin pad map")])
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["contract"], "proposal_create_v1")
        self.assertEqual(payload["action"], "create_pool_pin_pad_map_proposal")
        self.assertEqual(payload["map_uuid"], "map-uuid")
        self.assertEqual(payload["entry_count"], 1)

    def test_tools_call_dispatches_set_pool_pin_pad_map_proposal(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        entries = ["pad-uuid:gate-uuid:pin-uuid"]
        response = host.handle_message(
            {"jsonrpc": "2.0", "id": 240, "method": "tools/call", "params": {"name": "datum.proposal.set_pool_pin_pad_map", "arguments": {"path": "/tmp/native-project", "map": "map-uuid", "mode": "replace", "entries": entries, "pool": "pool", "proposal": "proposal-pin-pad-map-set", "rationale": "review pin pad map update"}}}
        )
        self.assertEqual(daemon.calls, [("set_pool_pin_pad_map_proposal", "/tmp/native-project", "map-uuid", "replace", entries, "pool", "proposal-pin-pad-map-set", "review pin pad map update")])
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["contract"], "proposal_create_v1")
        self.assertEqual(payload["action"], "set_pool_pin_pad_map_proposal")
        self.assertEqual(payload["map_uuid"], "map-uuid")
        self.assertEqual(payload["mode"], "replace")

    def test_tools_call_dispatches_set_pool_footprint_courtyard_rect_proposal(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {"jsonrpc": "2.0", "id": 235, "method": "tools/call", "params": {"name": "datum.proposal.set_pool_footprint_courtyard_rect", "arguments": {"path": "/tmp/native-project", "footprint": "footprint-uuid", "min_x_nm": 1000, "min_y_nm": 2000, "max_x_nm": 3000, "max_y_nm": 4000, "pool": "pool", "proposal": "proposal-footprint-courtyard-rect", "rationale": "review footprint courtyard"}}}
        )
        self.assertEqual(daemon.calls, [("set_pool_footprint_courtyard_rect_proposal", "/tmp/native-project", "footprint-uuid", 1000, 2000, 3000, 4000, "pool", "proposal-footprint-courtyard-rect", "review footprint courtyard")])
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["contract"], "proposal_create_v1")
        self.assertEqual(payload["action"], "set_pool_footprint_courtyard_rect_proposal")
        self.assertEqual(payload["footprint_uuid"], "footprint-uuid")

    def test_tools_call_dispatches_set_pool_footprint_courtyard_polygon_proposal(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {"jsonrpc": "2.0", "id": 236, "method": "tools/call", "params": {"name": "datum.proposal.set_pool_footprint_courtyard_polygon", "arguments": {"path": "/tmp/native-project", "footprint": "footprint-uuid", "vertices": "0,0;1000,0;1000,2000;0,2000", "pool": "pool", "proposal": "proposal-footprint-courtyard-poly", "rationale": "review footprint courtyard"}}}
        )
        self.assertEqual(daemon.calls, [("set_pool_footprint_courtyard_polygon_proposal", "/tmp/native-project", "footprint-uuid", "0,0;1000,0;1000,2000;0,2000", "pool", "proposal-footprint-courtyard-poly", "review footprint courtyard")])
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["contract"], "proposal_create_v1")
        self.assertEqual(payload["action"], "set_pool_footprint_courtyard_polygon_proposal")
        self.assertEqual(payload["footprint_uuid"], "footprint-uuid")

    def test_tools_call_dispatches_add_pool_footprint_silkscreen_line_proposal(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {"jsonrpc": "2.0", "id": 237, "method": "tools/call", "params": {"name": "datum.proposal.add_pool_footprint_silkscreen_line", "arguments": {"path": "/tmp/native-project", "footprint": "footprint-uuid", "from_x_nm": 1000, "from_y_nm": 2000, "to_x_nm": 3000, "to_y_nm": 4000, "width_nm": 150000, "pool": "pool", "proposal": "proposal-footprint-silk-line", "rationale": "review footprint silkscreen line"}}}
        )
        self.assertEqual(daemon.calls, [("add_pool_footprint_silkscreen_line_proposal", "/tmp/native-project", "footprint-uuid", 1000, 2000, 3000, 4000, 150000, "pool", "proposal-footprint-silk-line", "review footprint silkscreen line")])
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["contract"], "proposal_create_v1")
        self.assertEqual(payload["action"], "add_pool_footprint_silkscreen_line_proposal")
        self.assertEqual(payload["footprint_uuid"], "footprint-uuid")

    def test_tools_call_dispatches_add_pool_footprint_silkscreen_shape_proposals(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        cases = [
            (
                "datum.proposal.add_pool_footprint_silkscreen_rect",
                {"path": "/tmp/native-project", "footprint": "footprint-uuid", "min_x_nm": 1000, "min_y_nm": 2000, "max_x_nm": 3000, "max_y_nm": 4000, "width_nm": 150000, "pool": "pool", "proposal": "proposal-footprint-silk-rect", "rationale": "review footprint silkscreen rect"},
                ("add_pool_footprint_silkscreen_rect_proposal", "/tmp/native-project", "footprint-uuid", 1000, 2000, 3000, 4000, 150000, "pool", "proposal-footprint-silk-rect", "review footprint silkscreen rect"),
                "add_pool_footprint_silkscreen_rect_proposal",
            ),
            (
                "datum.proposal.add_pool_footprint_silkscreen_circle",
                {"path": "/tmp/native-project", "footprint": "footprint-uuid", "center_x_nm": 5000, "center_y_nm": 6000, "radius_nm": 7000, "width_nm": 150000, "pool": "pool", "proposal": "proposal-footprint-silk-circle", "rationale": "review footprint silkscreen circle"},
                ("add_pool_footprint_silkscreen_circle_proposal", "/tmp/native-project", "footprint-uuid", 5000, 6000, 7000, 150000, "pool", "proposal-footprint-silk-circle", "review footprint silkscreen circle"),
                "add_pool_footprint_silkscreen_circle_proposal",
            ),
            (
                "datum.proposal.add_pool_footprint_silkscreen_polygon",
                {"path": "/tmp/native-project", "footprint": "footprint-uuid", "vertices": "0,0;1000,0;1000,1000", "closed": True, "width_nm": 150000, "pool": "pool", "proposal": "proposal-footprint-silk-polygon", "rationale": "review footprint silkscreen polygon"},
                ("add_pool_footprint_silkscreen_polygon_proposal", "/tmp/native-project", "footprint-uuid", "0,0;1000,0;1000,1000", True, 150000, "pool", "proposal-footprint-silk-polygon", "review footprint silkscreen polygon"),
                "add_pool_footprint_silkscreen_polygon_proposal",
            ),
        ]
        for tool_name, arguments, expected_call, expected_action in cases:
            response = host.handle_message(
                {"jsonrpc": "2.0", "id": 238, "method": "tools/call", "params": {"name": tool_name, "arguments": arguments}}
            )
            self.assertEqual(daemon.calls[-1], expected_call)
            payload = response["result"]["content"][0]["json"]
            self.assertEqual(payload["contract"], "proposal_create_v1")
            self.assertEqual(payload["action"], expected_action)
            self.assertEqual(payload["footprint_uuid"], "footprint-uuid")

    def test_tools_call_dispatches_set_pool_package_courtyard_rect_proposal(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {"jsonrpc": "2.0", "id": 231, "method": "tools/call", "params": {"name": "datum.proposal.set_pool_package_courtyard_rect", "arguments": {"path": "/tmp/native-project", "package": "package-uuid", "min_x_nm": 1000, "min_y_nm": 2000, "max_x_nm": 3000, "max_y_nm": 4000, "pool": "pool", "proposal": "proposal-courtyard-rect", "rationale": "review courtyard"}}}
        )
        self.assertEqual(daemon.calls, [("set_pool_package_courtyard_rect_proposal", "/tmp/native-project", "package-uuid", 1000, 2000, 3000, 4000, "pool", "proposal-courtyard-rect", "review courtyard")])
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["contract"], "proposal_create_v1")
        self.assertEqual(payload["action"], "set_pool_package_courtyard_rect_proposal")
        self.assertEqual(payload["package_uuid"], "package-uuid")

    def test_tools_call_dispatches_set_pool_package_courtyard_polygon_proposal(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {"jsonrpc": "2.0", "id": 232, "method": "tools/call", "params": {"name": "datum.proposal.set_pool_package_courtyard_polygon", "arguments": {"path": "/tmp/native-project", "package": "package-uuid", "vertices": "0,0;1000,0;1000,2000;0,2000", "pool": "pool", "proposal": "proposal-courtyard-poly", "rationale": "review courtyard"}}}
        )
        self.assertEqual(daemon.calls, [("set_pool_package_courtyard_polygon_proposal", "/tmp/native-project", "package-uuid", "0,0;1000,0;1000,2000;0,2000", "pool", "proposal-courtyard-poly", "review courtyard")])
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["contract"], "proposal_create_v1")
        self.assertEqual(payload["action"], "set_pool_package_courtyard_polygon_proposal")
        self.assertEqual(payload["package_uuid"], "package-uuid")

    def test_tools_call_dispatches_datum_alias_for_place_symbol_proposal(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {"jsonrpc": "2.0", "id": 215, "method": "tools/call", "params": {"name": "datum.proposal.create_place_symbol", "arguments": {"path": "/tmp/native-project", "sheet": "sheet-uuid", "reference": "U2", "value": "BUF", "x_nm": 300, "y_nm": 400}}}
        )
        self.assertEqual(daemon.calls, [("create_place_symbol_proposal", "/tmp/native-project", "sheet-uuid", "U2", "BUF", 300, 400, None, None, None, "proposal-symbol-test", None)])
        self.assertEqual(response["result"]["content"][0]["json"]["action"], "propose_place_symbol")

    def test_tools_call_dispatches_datum_alias_for_board_component_replacement_proposal(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {"jsonrpc": "2.0", "id": 219, "method": "tools/call", "params": {"name": "datum.proposal.create_board_component_replacement", "arguments": {"path": "/tmp/native-project", "component": "component-uuid", "value": "22k"}}}
        )
        self.assertEqual(daemon.calls, [("create_board_component_replacement_proposal", "/tmp/native-project", "component-uuid", None, None, "22k", "proposal-component-replacement-test", None)])
        self.assertEqual(response["result"]["content"][0]["json"]["action"], "propose_board_component_replacement")

    def test_tools_call_dispatches_datum_alias_for_board_component_replacements_proposal(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        replacements = [{"component": "component-u1", "value": "22k"}]
        response = host.handle_message(
            {"jsonrpc": "2.0", "id": 221, "method": "tools/call", "params": {"name": "datum.proposal.create_board_component_replacements", "arguments": {"path": "/tmp/native-project", "replacements": replacements}}}
        )
        self.assertEqual(daemon.calls, [("create_board_component_replacements_proposal", "/tmp/native-project", replacements, "proposal-component-replacements-test", None)])
        self.assertEqual(response["result"]["content"][0]["json"]["action"], "propose_board_component_replacement")

    def test_tools_call_dispatches_datum_alias_for_board_component_replacement_plan_proposal(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        selections = [{"uuid": "component-u1", "part_uuid": "part-u1"}]
        response = host.handle_message(
            {"jsonrpc": "2.0", "id": 223, "method": "tools/call", "params": {"name": "datum.proposal.create_board_component_replacement_plan", "arguments": {"path": "/tmp/native-project", "selections": selections}}}
        )
        self.assertEqual(daemon.calls, [("create_board_component_replacement_plan_proposal", "/tmp/native-project", selections, "proposal-component-replacement-plan-test", None)])
        self.assertEqual(response["result"]["content"][0]["json"]["action"], "propose_board_component_replacement")

    def test_tools_call_dispatches_get_proposals(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 204,
                "method": "tools/call",
                "params": {
                    "name": "get_proposals",
                    "arguments": {"path": "/tmp/native-project"},
                },
            }
        )
        self.assertEqual(daemon.calls, [("get_proposals", "/tmp/native-project")])
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["contract"], "proposals_query_v1")
        self.assertEqual(payload["proposal_count"], 1)

    def test_tools_call_dispatches_review_proposal(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {"jsonrpc": "2.0", "id": 205, "method": "tools/call", "params": {"name": "review_proposal", "arguments": {"path": "/tmp/native-project", "proposal": "proposal-test", "status": "accepted"}}}
        )
        self.assertEqual(daemon.calls, [("review_proposal", "/tmp/native-project", "proposal-test", "accepted")])
        self.assertEqual(response["result"]["content"][0]["json"]["status"], "accepted")

    def test_tools_call_dispatches_show_proposal(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {"jsonrpc": "2.0", "id": 207, "method": "tools/call", "params": {"name": "show_proposal", "arguments": {"path": "/tmp/native-project", "proposal": "proposal-test"}}}
        )
        self.assertEqual(daemon.calls, [("show_proposal", "/tmp/native-project", "proposal-test")])
        self.assertEqual(response["result"]["content"][0]["json"]["contract"], "proposal_show_v1")

    def test_tools_call_dispatches_preview_proposal(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {"jsonrpc": "2.0", "id": 216, "method": "tools/call", "params": {"name": "preview_proposal", "arguments": {"path": "/tmp/native-project", "proposal": "proposal-test"}}}
        )
        self.assertEqual(daemon.calls, [("preview_proposal", "/tmp/native-project", "proposal-test")])
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["contract"], "proposal_preview_v1")
        self.assertEqual(payload["diff"]["created"], ["object-test"])
        self.assertEqual(
            payload["validation"]["approval_path"],
            "draft_review_accept_then_apply",
        )

    def test_tools_call_dispatches_validate_proposal(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {"jsonrpc": "2.0", "id": 208, "method": "tools/call", "params": {"name": "validate_proposal", "arguments": {"path": "/tmp/native-project", "proposal": "proposal-test"}}}
        )
        self.assertEqual(daemon.calls, [("validate_proposal", "/tmp/native-project", "proposal-test")])
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["contract"], "proposal_validation_v1")
        self.assertEqual(payload["policy"], "accepted_revision_guarded_source_policy_v1")
        self.assertEqual(payload["approval_path"], "draft_review_accept_then_apply")
        self.assertEqual(payload["acceptance_required"], True)
        self.assertEqual(payload["current_revision_required"], True)
        self.assertEqual(payload["revision_guard_required"], True)
        self.assertEqual(payload["check_source_evidence_required"], True)

    def test_tools_call_dispatches_defer_proposal(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {"jsonrpc": "2.0", "id": 209, "method": "tools/call", "params": {"name": "defer_proposal", "arguments": {"path": "/tmp/native-project", "proposal": "proposal-test"}}}
        )
        self.assertEqual(daemon.calls, [("defer_proposal", "/tmp/native-project", "proposal-test")])
        self.assertEqual(response["result"]["content"][0]["json"]["status"], "deferred")

    def test_tools_call_dispatches_reject_proposal(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {"jsonrpc": "2.0", "id": 210, "method": "tools/call", "params": {"name": "reject_proposal", "arguments": {"path": "/tmp/native-project", "proposal": "proposal-test"}}}
        )
        self.assertEqual(daemon.calls, [("reject_proposal", "/tmp/native-project", "proposal-test")])
        self.assertEqual(response["result"]["content"][0]["json"]["status"], "rejected")

    def test_tools_call_dispatches_accept_apply_proposal(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {"jsonrpc": "2.0", "id": 211, "method": "tools/call", "params": {"name": "accept_apply_proposal", "arguments": {"path": "/tmp/native-project", "proposal": "proposal-test"}}}
        )
        self.assertEqual(daemon.calls, [("accept_apply_proposal", "/tmp/native-project", "proposal-test")])
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["status"], "applied")
        self.assertEqual(payload["policy"], "accepted_revision_guarded_source_policy_v1")
        self.assertEqual(payload["validation"]["status"], "accepted")
        self.assertEqual(payload["validation"]["can_apply"], True)

    def test_tools_call_dispatches_apply_proposal(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {"jsonrpc": "2.0", "id": 206, "method": "tools/call", "params": {"name": "apply_proposal", "arguments": {"path": "/tmp/native-project", "proposal": "proposal-test"}}}
        )
        self.assertEqual(daemon.calls, [("apply_proposal", "/tmp/native-project", "proposal-test")])
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["status"], "applied")
        self.assertEqual(payload["approval_path"], "draft_review_accept_then_apply")
        self.assertEqual(payload["validation"]["status"], "accepted")
        self.assertEqual(payload["validation"]["can_apply"], True)
