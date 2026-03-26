#!/usr/bin/env python3
"""Engine daemon JSON-RPC request/response and socket transport tests."""

from __future__ import annotations

import json
import os
import socket
import tempfile
import threading
import unittest

from server_runtime import EngineDaemonClient, JsonRpcResponse


class TestDaemonClient(unittest.TestCase):
    def test_builds_get_check_report_request(self) -> None:
        client = EngineDaemonClient()
        request = client.get_check_report_request()
        self.assertEqual(request.jsonrpc, "2.0")
        self.assertEqual(request.id, 1)
        self.assertEqual(request.method, "get_check_report")
        self.assertEqual(request.params, {})

    def test_request_ids_increment_across_methods(self) -> None:
        client = EngineDaemonClient()
        first = client.get_check_report_request()
        second = client.get_connectivity_diagnostics_request()
        third = client.run_erc_request()
        fourth = client.run_drc_request()
        self.assertEqual((first.id, second.id, third.id, fourth.id), (1, 2, 3, 4))

    def test_builds_open_project_request(self) -> None:
        client = EngineDaemonClient()
        request = client.open_project_request("/tmp/demo.kicad_pcb")
        self.assertEqual(request.method, "open_project")
        self.assertEqual(request.params, {"path": "/tmp/demo.kicad_pcb"})

    def test_builds_close_project_request(self) -> None:
        client = EngineDaemonClient()
        request = client.close_project_request()
        self.assertEqual(request.method, "close_project")
        self.assertEqual(request.params, {})

    def test_builds_m3_write_requests(self) -> None:
        client = EngineDaemonClient()
        save = client.save_request("/tmp/out.kicad_pcb")
        delete = client.delete_track_request("track-uuid")
        delete_via = client.delete_via_request("via-uuid")
        delete_component = client.delete_component_request("comp-uuid")
        move_component = client.move_component_request("comp-uuid", 15.0, 12.0, 90.0)
        rotate_component = client.rotate_component_request("comp-uuid", 180.0)
        set_value = client.set_value_request("comp-uuid", "22k")
        assign_part = client.assign_part_request("comp-uuid", "part-uuid")
        set_package = client.set_package_request("comp-uuid", "package-uuid")
        set_package_with_part = client.set_package_with_part_request(
            "comp-uuid", "package-uuid", "part-uuid"
        )
        set_net_class = client.set_net_class_request(
            "net-uuid", "power", 125000, 250000, 300000, 600000
        )
        set_reference = client.set_reference_request("comp-uuid", "R10")
        set_rule = client.set_design_rule_request(
            "ClearanceCopper",
            "All",
            {"Clearance": {"min": 125000}},
            10,
            "default clearance",
        )
        undo = client.undo_request()
        redo = client.redo_request()
        self.assertEqual(save.method, "save")
        self.assertEqual(save.params, {"path": "/tmp/out.kicad_pcb"})
        self.assertEqual(delete.method, "delete_track")
        self.assertEqual(delete.params, {"uuid": "track-uuid"})
        self.assertEqual(delete_via.method, "delete_via")
        self.assertEqual(delete_via.params, {"uuid": "via-uuid"})
        self.assertEqual(delete_component.method, "delete_component")
        self.assertEqual(delete_component.params, {"uuid": "comp-uuid"})
        self.assertEqual(move_component.method, "move_component")
        self.assertEqual(
            move_component.params,
            {"uuid": "comp-uuid", "x_mm": 15.0, "y_mm": 12.0, "rotation_deg": 90.0},
        )
        self.assertEqual(rotate_component.method, "rotate_component")
        self.assertEqual(
            rotate_component.params,
            {"uuid": "comp-uuid", "x_mm": 0.0, "y_mm": 0.0, "rotation_deg": 180.0},
        )
        self.assertEqual(set_value.method, "set_value")
        self.assertEqual(set_value.params, {"uuid": "comp-uuid", "value": "22k"})
        self.assertEqual(assign_part.method, "assign_part")
        self.assertEqual(
            assign_part.params, {"uuid": "comp-uuid", "part_uuid": "part-uuid"}
        )
        self.assertEqual(set_package.method, "set_package")
        self.assertEqual(
            set_package.params, {"uuid": "comp-uuid", "package_uuid": "package-uuid"}
        )
        self.assertEqual(set_package_with_part.method, "set_package_with_part")
        self.assertEqual(
            set_package_with_part.params,
            {
                "uuid": "comp-uuid",
                "package_uuid": "package-uuid",
                "part_uuid": "part-uuid",
            },
        )
        self.assertEqual(set_net_class.method, "set_net_class")
        self.assertEqual(
            set_net_class.params,
            {
                "net_uuid": "net-uuid",
                "class_name": "power",
                "clearance": 125000,
                "track_width": 250000,
                "via_drill": 300000,
                "via_diameter": 600000,
                "diffpair_width": 0,
                "diffpair_gap": 0,
            },
        )
        self.assertEqual(set_reference.method, "set_reference")
        self.assertEqual(
            set_reference.params, {"uuid": "comp-uuid", "reference": "R10"}
        )
        self.assertEqual(set_rule.method, "set_design_rule")
        self.assertEqual(
            set_rule.params,
            {
                "rule_type": "ClearanceCopper",
                "scope": "All",
                "parameters": {"Clearance": {"min": 125000}},
                "priority": 10,
                "name": "default clearance",
            },
        )
        self.assertEqual(undo.method, "undo")
        self.assertEqual(undo.params, {})
        self.assertEqual(redo.method, "redo")
        self.assertEqual(redo.params, {})

    def test_builds_search_pool_request(self) -> None:
        client = EngineDaemonClient()
        request = client.search_pool_request("sot23")
        self.assertEqual(request.method, "search_pool")
        self.assertEqual(request.params, {"query": "sot23"})

    def test_builds_get_part_and_get_package_requests(self) -> None:
        client = EngineDaemonClient()
        part = client.get_part_request("11111111-1111-1111-1111-111111111111")
        package = client.get_package_request("22222222-2222-2222-2222-222222222222")
        package_candidates = client.get_package_change_candidates_request(
            "33333333-3333-3333-3333-333333333333"
        )
        self.assertEqual(part.method, "get_part")
        self.assertEqual(part.params, {"uuid": "11111111-1111-1111-1111-111111111111"})
        self.assertEqual(package.method, "get_package")
        self.assertEqual(package.params, {"uuid": "22222222-2222-2222-2222-222222222222"})
        self.assertEqual(package_candidates.method, "get_package_change_candidates")
        self.assertEqual(
            package_candidates.params,
            {"uuid": "33333333-3333-3333-3333-333333333333"},
        )

    def test_builds_explain_violation_request(self) -> None:
        client = EngineDaemonClient()
        request = client.explain_violation_request("drc", 3)
        self.assertEqual(request.method, "explain_violation")
        self.assertEqual(request.params, {"domain": "drc", "index": 3})

    def test_builds_summary_requests(self) -> None:
        client = EngineDaemonClient()
        board = client.get_board_summary_request()
        schematic = client.get_schematic_summary_request()
        self.assertEqual(board.method, "get_board_summary")
        self.assertEqual(board.params, {})
        self.assertEqual(schematic.method, "get_schematic_summary")
        self.assertEqual(schematic.params, {})

    def test_builds_net_info_requests(self) -> None:
        client = EngineDaemonClient()
        board = client.get_net_info_request()
        unrouted = client.get_unrouted_request()
        schematic = client.get_schematic_net_info_request()
        self.assertEqual(board.method, "get_net_info")
        self.assertEqual(board.params, {})
        self.assertEqual(unrouted.method, "get_unrouted")
        self.assertEqual(unrouted.params, {})
        self.assertEqual(schematic.method, "get_schematic_net_info")
        self.assertEqual(schematic.params, {})

    def test_builds_component_and_schematic_object_requests(self) -> None:
        client = EngineDaemonClient()
        components = client.get_components_request()
        netlist = client.get_netlist_request()
        labels = client.get_labels_request()
        symbols = client.get_symbols_request()
        symbol_fields = client.get_symbol_fields_request("11111111-1111-1111-1111-111111111111")
        ports = client.get_ports_request()
        buses = client.get_buses_request()
        bus_entries = client.get_bus_entries_request()
        noconnects = client.get_noconnects_request()
        hierarchy = client.get_hierarchy_request()
        self.assertEqual(components.method, "get_components")
        self.assertEqual(netlist.method, "get_netlist")
        self.assertEqual(labels.method, "get_labels")
        self.assertEqual(symbols.method, "get_symbols")
        self.assertEqual(symbol_fields.method, "get_symbol_fields")
        self.assertEqual(ports.method, "get_ports")
        self.assertEqual(buses.method, "get_buses")
        self.assertEqual(bus_entries.method, "get_bus_entries")
        self.assertEqual(noconnects.method, "get_noconnects")
        self.assertEqual(hierarchy.method, "get_hierarchy")
        self.assertEqual(components.params, {})
        self.assertEqual(netlist.params, {})
        self.assertEqual(labels.params, {})
        self.assertEqual(symbols.params, {})
        self.assertEqual(
            symbol_fields.params,
            {"symbol_uuid": "11111111-1111-1111-1111-111111111111"},
        )
        self.assertEqual(ports.params, {})
        self.assertEqual(buses.params, {})
        self.assertEqual(bus_entries.params, {})
        self.assertEqual(noconnects.params, {})
        self.assertEqual(hierarchy.params, {})

    def test_builds_get_design_rules_request(self) -> None:
        client = EngineDaemonClient()
        request = client.get_design_rules_request()
        self.assertEqual(request.method, "get_design_rules")
        self.assertEqual(request.params, {})

    def test_response_decodes_success_payload(self) -> None:
        response = JsonRpcResponse.from_json(
            json.dumps(
                {
                    "jsonrpc": "2.0",
                    "id": 7,
                    "result": {
                        "domain": "board",
                        "summary": {"status": "warning"},
                        "diagnostics": [],
                    },
                    "error": None,
                }
            )
        )
        self.assertEqual(response.id, 7)
        self.assertIsNone(response.error)
        assert isinstance(response.result, dict)
        self.assertEqual(response.result["domain"], "board")

    def test_response_decodes_error_payload(self) -> None:
        response = JsonRpcResponse.from_json(
            json.dumps(
                {
                    "jsonrpc": "2.0",
                    "id": 9,
                    "result": None,
                    "error": {"code": -32001, "message": "no project open"},
                }
            )
        )
        self.assertIsNone(response.result)
        self.assertIsNotNone(response.error)
        assert response.error is not None
        self.assertEqual(response.error.code, -32001)
        self.assertEqual(response.error.message, "no project open")

    def test_call_requires_socket_configuration(self) -> None:
        client = EngineDaemonClient(socket_path=None)
        with self.assertRaisesRegex(RuntimeError, "EDA_ENGINE_SOCKET is not configured"):
            client.get_check_report()

    def test_get_check_report_round_trips_over_unix_socket(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            socket_path = os.path.join(tmp, "eda.sock")
            probe = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
            try:
                probe.bind(socket_path)
            except PermissionError as exc:
                self.skipTest(f"unix socket bind not permitted in this environment: {exc}")
            finally:
                probe.close()
                if os.path.exists(socket_path):
                    os.unlink(socket_path)
            ready = threading.Event()

            def serve_once() -> None:
                with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as server:
                    server.bind(socket_path)
                    server.listen(1)
                    ready.set()
                    conn, _ = server.accept()
                    with conn:
                        data = b""
                        while not data.endswith(b"\n"):
                            chunk = conn.recv(4096)
                            if not chunk:
                                break
                            data += chunk
                        request = json.loads(data.decode("utf-8").strip())
                        self.assertEqual(request["method"], "get_check_report")
                        response = json.dumps(
                            {
                                "jsonrpc": "2.0",
                                "id": request["id"],
                                "result": {
                                    "domain": "board",
                                    "summary": {
                                        "status": "warning",
                                        "errors": 0,
                                        "warnings": 1,
                                        "infos": 1,
                                        "waived": 0,
                                        "by_code": [
                                            {"code": "partially_routed_net", "count": 1},
                                            {"code": "net_without_copper", "count": 1},
                                        ],
                                    },
                                    "diagnostics": [
                                        {"kind": "partially_routed_net", "severity": "warning"},
                                        {"kind": "net_without_copper", "severity": "info"},
                                    ],
                                },
                                "error": None,
                            }
                        )
                        conn.sendall(response.encode("utf-8") + b"\n")

            thread = threading.Thread(target=serve_once)
            thread.start()
            ready.wait(timeout=2)

            client = EngineDaemonClient(socket_path=socket_path)
            response = client.get_check_report()
            self.assertIsNone(response.error)
            assert isinstance(response.result, dict)
            self.assertEqual(response.result["domain"], "board")
            self.assertEqual(response.result["summary"]["status"], "warning")
            self.assertEqual(
                response.result["summary"]["by_code"][0]["code"], "partially_routed_net"
            )
            thread.join(timeout=2)
            self.assertFalse(thread.is_alive())
