#!/usr/bin/env python3
"""End-to-end canonical MCP write parity against real native projects."""

from __future__ import annotations

import json
import os
import shlex
import subprocess
import tempfile
import unittest
import uuid
from pathlib import Path
from unittest.mock import patch

from server_runtime import EngineDaemonClient, StdioToolHost


REPO_ROOT = Path(__file__).resolve().parents[1]


def datum_cli_prefix() -> list[str]:
    configured = os.environ.get("DATUM_CLI_BIN")
    if configured:
        return shlex.split(configured)
    binary = REPO_ROOT / "target" / "debug" / "datum-eda"
    if binary.exists():
        return [str(binary)]
    return ["cargo", "run", "-q", "-p", "datum-eda-cli", "--"]


def run_cli_json(root: Path, *args: str):
    completed = subprocess.run(
        [*datum_cli_prefix(), "--format", "json", *args],
        cwd=REPO_ROOT,
        capture_output=True,
        text=True,
        check=False,
    )
    if completed.returncode != 0:
        detail = completed.stderr.strip() or completed.stdout.strip()
        raise AssertionError(f"datum-eda CLI failed: {detail}")
    return json.loads(completed.stdout)

def call_tool(host: StdioToolHost, name: str, arguments: dict):
    arguments = guarded_journal_arguments(host, name, arguments)
    response = host.handle_message(
        {
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {"name": name, "arguments": arguments},
        }
    )
    payload = response["result"]["content"][0]["json"]
    assert payload["ok"] is True, payload
    return payload

def guarded_journal_arguments(host: StdioToolHost, name: str, arguments: dict) -> dict:
    if name not in {"datum.journal.undo", "datum.journal.redo"}:
        return arguments
    if "expected_tip_transaction" in arguments:
        return arguments
    path = arguments["path"]
    list_response = host.handle_message(
        {
            "jsonrpc": "2.0",
            "id": 0,
            "method": "tools/call",
            "params": {"name": "datum.journal.list", "arguments": {"path": path}},
        }
    )
    payload = list_response["result"]["content"][0]["json"]
    assert payload["ok"] is True, payload
    latest = payload["transactions"][-1]["transaction_id"]
    return {**arguments, "expected_tip_transaction": latest}


def query_result(host: StdioToolHost, name: str, path: Path):
    return call_tool(host, name, {"path": str(path)})["result"]


def board_components(root: Path):
    return run_cli_json(root, "project", "query", str(root), "board-components")


def assert_latest_journal_operation(
    test: unittest.TestCase,
    host: StdioToolHost,
    path: str,
    reason: str,
    kind: str,
):
    journal = call_tool(host, "datum.journal.list", {"path": path})
    test.assertEqual(journal["contract"], "project_transaction_journal_list_v1")
    test.assertTrue(journal["can_undo"])
    latest = journal["transactions"][-1]
    test.assertEqual(latest["reason"], reason)
    test.assertEqual(latest["operations"], 1)
    record = call_tool(
        host,
        "datum.journal.show",
        {"path": path, "transaction": latest["transaction_id"]},
    )
    test.assertEqual(record["contract"], "project_transaction_journal_record_v1")
    operation = record["transaction"]["operations"][0]
    test.assertEqual(operation["kind"], kind)
    return operation


def seed_board_net(host: StdioToolHost, root: Path) -> str:
    net_class = call_tool(
        host,
        "datum.pcb.place_net_class",
        {
            "path": str(root),
            "name": "Default",
            "clearance_nm": 150000,
            "track_width_nm": 200000,
            "via_drill_nm": 300000,
            "via_diameter_nm": 600000,
        },
    )["net_class_uuid"]
    return call_tool(
        host,
        "datum.pcb.place_net",
        {"path": str(root), "name": "GND", "class": net_class},
    )["net_uuid"]


class TestNativeWriteParity(unittest.TestCase):
    def test_pcb_component_tools_call_writes_model_and_journal(self) -> None:
        with tempfile.TemporaryDirectory(prefix="datum-mcp-component-") as tmp:
            root = Path(tmp)
            run_cli_json(root, "project", "new", str(root), "--name", "Component Parity")
            env = {"DATUM_CLI_BIN": " ".join(shlex.quote(part) for part in datum_cli_prefix())}
            with patch.dict(os.environ, env, clear=False):
                host = StdioToolHost(EngineDaemonClient())
                part = "11111111-1111-4111-8111-111111111111"
                package = "22222222-2222-4222-8222-222222222222"
                component = call_tool(
                    host,
                    "datum.pcb.place_component",
                    {
                        "path": str(root),
                        "part": part,
                        "package": package,
                        "reference": "U1",
                        "value": "OPA",
                        "x_nm": 1000,
                        "y_nm": 2000,
                        "layer": 1,
                    },
                )["component_uuid"]
                operation = assert_latest_journal_operation(
                    self, host, str(root), "place board component", "create_board_package"
                )
                self.assertEqual(operation["package_id"], component)
                components = board_components(root)
                self.assertEqual(len(components), 1)
                self.assertEqual(components[0]["uuid"], component)
                self.assertEqual(components[0]["position"], {"x": 1000, "y": 2000})

                self.assertEqual(
                    call_tool(
                        host,
                        "datum.pcb.move_component",
                        {"path": str(root), "component": component, "x_nm": 3000, "y_nm": 4000},
                    )["action"],
                    "move_board_component",
                )
                operation = assert_latest_journal_operation(
                    self, host, str(root), "move board component", "set_board_package_position"
                )
                self.assertEqual(operation["package_id"], component)
                self.assertEqual(board_components(root)[0]["position"], {"x": 3000, "y": 4000})

                call_tool(
                    host,
                    "datum.pcb.rotate_component",
                    {"path": str(root), "component": component, "rotation_deg": 90},
                )
                operation = assert_latest_journal_operation(
                    self, host, str(root), "rotate board component", "set_board_package_rotation"
                )
                self.assertEqual(operation["package_id"], component)
                self.assertEqual(board_components(root)[0]["rotation"], 90)

                call_tool(
                    host,
                    "datum.pcb.flip_component",
                    {"path": str(root), "component": component, "layer": 2},
                )
                operation = assert_latest_journal_operation(
                    self, host, str(root), "set board component layer", "set_component_side"
                )
                self.assertEqual(operation["package_id"], component)
                self.assertEqual(board_components(root)[0]["layer"], 2)

                call_tool(
                    host,
                    "datum.pcb.set_component_reference",
                    {"path": str(root), "component": component, "reference": "U2"},
                )
                operation = assert_latest_journal_operation(
                    self, host, str(root), "set board component reference", "set_board_package_reference"
                )
                self.assertEqual(operation["package_id"], component)
                self.assertEqual(board_components(root)[0]["reference"], "U2")

                call_tool(
                    host,
                    "datum.pcb.set_component_value",
                    {"path": str(root), "component": component, "value": "OPA1656"},
                )
                operation = assert_latest_journal_operation(
                    self, host, str(root), "set board component value", "set_board_package_value"
                )
                self.assertEqual(operation["package_id"], component)
                self.assertEqual(board_components(root)[0]["value"], "OPA1656")

                self.assertEqual(
                    call_tool(
                        host,
                        "datum.pcb.delete_component",
                        {"path": str(root), "component": component},
                    )["action"],
                    "delete_board_component",
                )
                operation = assert_latest_journal_operation(
                    self, host, str(root), "delete board component", "delete_board_package"
                )
                self.assertEqual(operation["package_id"], component)
                self.assertEqual(board_components(root), [])
                self.assertEqual(call_tool(host, "datum.journal.undo", {"path": str(root)})["status"], "applied")
                self.assertEqual(board_components(root)[0]["uuid"], component)
                self.assertEqual(call_tool(host, "datum.journal.redo", {"path": str(root)})["status"], "applied")
                self.assertEqual(board_components(root), [])

    def test_schematic_draw_wire_tools_call_writes_model_and_journal(self) -> None:
        with tempfile.TemporaryDirectory(prefix="datum-mcp-wire-") as tmp:
            root = Path(tmp)
            run_cli_json(root, "project", "new", str(root), "--name", "Wire Parity")
            env = {"DATUM_CLI_BIN": " ".join(shlex.quote(part) for part in datum_cli_prefix())}
            with patch.dict(os.environ, env, clear=False):
                host = StdioToolHost(EngineDaemonClient())
                sheet = call_tool(
                    host,
                    "datum.schematic.create_sheet",
                    {"path": str(root), "name": "Main"},
                )["sheet_uuid"]
                wire = call_tool(
                    host,
                    "datum.schematic.draw_wire",
                    {
                        "path": str(root),
                        "sheet": sheet,
                        "from_x_nm": 10,
                        "from_y_nm": 20,
                        "to_x_nm": 30,
                        "to_y_nm": 40,
                    },
                )["wire_uuid"]

                wires = query_result(host, "datum.query.schematic_wires", root)
                self.assertEqual(len(wires), 1)
                self.assertEqual(wires[0]["uuid"], wire)
                self.assertEqual(wires[0]["sheet"], sheet)
                self.assertEqual(wires[0]["from"], {"x": 10, "y": 20})
                self.assertEqual(wires[0]["to"], {"x": 30, "y": 40})
                operation = assert_latest_journal_operation(
                    self, host, str(root), "draw schematic wire", "create_schematic_wire"
                )
                self.assertEqual(operation["sheet_id"], sheet)
                self.assertEqual(operation["wire_id"], wire)
                self.assertEqual(operation["wire"]["uuid"], wire)
                self.assertEqual(operation["wire"]["from"], {"x": 10, "y": 20})
                self.assertEqual(operation["wire"]["to"], {"x": 30, "y": 40})
                undo = call_tool(host, "datum.journal.undo", {"path": str(root)})
                self.assertEqual(undo["action"], "undo")
                self.assertEqual(undo["status"], "applied")
                self.assertEqual(
                    query_result(host, "datum.query.schematic_wires", root), []
                )
                redo = call_tool(host, "datum.journal.redo", {"path": str(root)})
                self.assertEqual(redo["action"], "redo")
                self.assertEqual(redo["status"], "applied")
                redone_wires = query_result(host, "datum.query.schematic_wires", root)
                self.assertEqual(len(redone_wires), 1)
                self.assertEqual(redone_wires[0]["uuid"], wire)

    def test_schematic_junction_tools_call_writes_deletes_and_replays(self) -> None:
        with tempfile.TemporaryDirectory(prefix="datum-mcp-junction-") as tmp:
            root = Path(tmp)
            run_cli_json(root, "project", "new", str(root), "--name", "Junction Parity")
            env = {"DATUM_CLI_BIN": " ".join(shlex.quote(part) for part in datum_cli_prefix())}
            with patch.dict(os.environ, env, clear=False):
                host = StdioToolHost(EngineDaemonClient())
                sheet = call_tool(host, "datum.schematic.create_sheet", {"path": str(root), "name": "Main"})["sheet_uuid"]
                junction = call_tool(
                    host,
                    "datum.schematic.place_junction",
                    {"path": str(root), "sheet": sheet, "x_nm": 50, "y_nm": 60},
                )["junction_uuid"]
                junctions = query_result(host, "datum.query.schematic_junctions", root)
                self.assertEqual(len(junctions), 1)
                self.assertEqual(junctions[0]["uuid"], junction)
                self.assertEqual(junctions[0]["sheet"], sheet)
                self.assertEqual(junctions[0]["position"], {"x": 50, "y": 60})
                operation = assert_latest_journal_operation(
                    self, host, str(root), "place schematic junction", "place_schematic_marker"
                )
                self.assertEqual(operation["sheet_id"], sheet)
                self.assertEqual(operation["marker_id"], junction)
                self.assertEqual(operation["marker_kind"], "Junction")
                self.assertEqual(operation["marker"]["position"], {"x": 50, "y": 60})
                self.assertEqual(call_tool(host, "datum.journal.undo", {"path": str(root)})["status"], "applied")
                self.assertEqual(query_result(host, "datum.query.schematic_junctions", root), [])
                self.assertEqual(call_tool(host, "datum.journal.redo", {"path": str(root)})["status"], "applied")
                self.assertEqual(query_result(host, "datum.query.schematic_junctions", root)[0]["uuid"], junction)
                self.assertEqual(
                    call_tool(host, "datum.schematic.delete_junction", {"path": str(root), "junction": junction})["action"],
                    "delete_junction",
                )
                self.assertEqual(query_result(host, "datum.query.schematic_junctions", root), [])
                operation = assert_latest_journal_operation(
                    self, host, str(root), "delete schematic junction", "delete_schematic_junction"
                )
                self.assertEqual(operation["junction_id"], junction)

    def test_schematic_label_tools_call_writes_deletes_and_replays(self) -> None:
        with tempfile.TemporaryDirectory(prefix="datum-mcp-label-") as tmp:
            root = Path(tmp)
            run_cli_json(root, "project", "new", str(root), "--name", "Label Parity")
            env = {"DATUM_CLI_BIN": " ".join(shlex.quote(part) for part in datum_cli_prefix())}
            with patch.dict(os.environ, env, clear=False):
                host = StdioToolHost(EngineDaemonClient())
                sheet = call_tool(host, "datum.schematic.create_sheet", {"path": str(root), "name": "Main"})["sheet_uuid"]
                label = call_tool(
                    host,
                    "datum.schematic.place_label",
                    {"path": str(root), "sheet": sheet, "name": "VIN", "kind": "global", "x_nm": 123, "y_nm": 456},
                )["label_uuid"]
                labels = query_result(host, "datum.query.schematic_labels", root)
                self.assertEqual(len(labels), 1)
                self.assertEqual(labels[0]["uuid"], label)
                self.assertEqual(labels[0]["sheet"], sheet)
                self.assertEqual(labels[0]["name"], "VIN")
                self.assertEqual(labels[0]["kind"], "Global")
                self.assertEqual(labels[0]["position"], {"x": 123, "y": 456})
                operation = assert_latest_journal_operation(
                    self, host, str(root), "place schematic label", "create_schematic_label"
                )
                self.assertEqual(operation["sheet_id"], sheet)
                self.assertEqual(operation["label_id"], label)
                self.assertEqual(operation["label"]["name"], "VIN")
                self.assertEqual(operation["label"]["position"], {"x": 123, "y": 456})
                self.assertEqual(call_tool(host, "datum.journal.undo", {"path": str(root)})["status"], "applied")
                self.assertEqual(query_result(host, "datum.query.schematic_labels", root), [])
                self.assertEqual(call_tool(host, "datum.journal.redo", {"path": str(root)})["status"], "applied")
                self.assertEqual(query_result(host, "datum.query.schematic_labels", root)[0]["uuid"], label)
                self.assertEqual(
                    call_tool(host, "datum.schematic.delete_label", {"path": str(root), "label": label})["action"],
                    "delete_label",
                )
                self.assertEqual(query_result(host, "datum.query.schematic_labels", root), [])
                operation = assert_latest_journal_operation(
                    self, host, str(root), "delete schematic label", "delete_schematic_label"
                )
                self.assertEqual(operation["label_id"], label)

    def test_schematic_port_tools_call_writes_deletes_and_replays(self) -> None:
        with tempfile.TemporaryDirectory(prefix="datum-mcp-port-") as tmp:
            root = Path(tmp)
            run_cli_json(root, "project", "new", str(root), "--name", "Port Parity")
            env = {"DATUM_CLI_BIN": " ".join(shlex.quote(part) for part in datum_cli_prefix())}
            with patch.dict(os.environ, env, clear=False):
                host = StdioToolHost(EngineDaemonClient())
                sheet = call_tool(host, "datum.schematic.create_sheet", {"path": str(root), "name": "Main"})["sheet_uuid"]
                port = call_tool(
                    host,
                    "datum.schematic.place_port",
                    {"path": str(root), "sheet": sheet, "name": "SUB_IN", "direction": "input", "x_nm": 11, "y_nm": 22},
                )["port_uuid"]
                ports = query_result(host, "datum.query.schematic_ports", root)
                self.assertEqual(len(ports), 1)
                self.assertEqual(ports[0]["uuid"], port)
                self.assertEqual(ports[0]["sheet"], sheet)
                self.assertEqual(ports[0]["name"], "SUB_IN")
                self.assertEqual(ports[0]["direction"], "Input")
                self.assertEqual(ports[0]["position"], {"x": 11, "y": 22})
                operation = assert_latest_journal_operation(
                    self, host, str(root), "place schematic port", "create_schematic_port"
                )
                self.assertEqual(operation["sheet_id"], sheet)
                self.assertEqual(operation["port_id"], port)
                self.assertEqual(operation["port"]["direction"], "Input")
                self.assertEqual(call_tool(host, "datum.journal.undo", {"path": str(root)})["status"], "applied")
                self.assertEqual(query_result(host, "datum.query.schematic_ports", root), [])
                self.assertEqual(call_tool(host, "datum.journal.redo", {"path": str(root)})["status"], "applied")
                self.assertEqual(query_result(host, "datum.query.schematic_ports", root)[0]["uuid"], port)
                self.assertEqual(
                    call_tool(host, "datum.schematic.delete_port", {"path": str(root), "port": port})["action"],
                    "delete_port",
                )
                self.assertEqual(query_result(host, "datum.query.schematic_ports", root), [])
                operation = assert_latest_journal_operation(
                    self, host, str(root), "delete schematic port", "delete_schematic_port"
                )
                self.assertEqual(operation["port_id"], port)

    def test_schematic_noconnect_tools_call_writes_deletes_and_replays(self) -> None:
        with tempfile.TemporaryDirectory(prefix="datum-mcp-noconnect-") as tmp:
            root = Path(tmp)
            run_cli_json(root, "project", "new", str(root), "--name", "NoConnect Parity")
            env = {"DATUM_CLI_BIN": " ".join(shlex.quote(part) for part in datum_cli_prefix())}
            with patch.dict(os.environ, env, clear=False):
                host = StdioToolHost(EngineDaemonClient())
                sheet = call_tool(host, "datum.schematic.create_sheet", {"path": str(root), "name": "Main"})["sheet_uuid"]
                symbol = str(uuid.uuid4())
                pin = str(uuid.uuid4())
                noconnect = call_tool(
                    host,
                    "datum.schematic.place_noconnect",
                    {"path": str(root), "sheet": sheet, "symbol": symbol, "pin": pin, "x_nm": 70, "y_nm": 80},
                )["noconnect_uuid"]
                markers = query_result(host, "datum.query.schematic_noconnects", root)
                self.assertEqual(len(markers), 1)
                self.assertEqual(markers[0]["uuid"], noconnect)
                self.assertEqual(markers[0]["sheet"], sheet)
                self.assertEqual(markers[0]["symbol"], symbol)
                self.assertEqual(markers[0]["pin"], pin)
                self.assertEqual(markers[0]["position"], {"x": 70, "y": 80})
                operation = assert_latest_journal_operation(
                    self, host, str(root), "place schematic noconnect", "place_schematic_marker"
                )
                self.assertEqual(operation["sheet_id"], sheet)
                self.assertEqual(operation["marker_id"], noconnect)
                self.assertEqual(operation["marker_kind"], "NoConnect")
                self.assertEqual(operation["marker"]["symbol"], symbol)
                self.assertEqual(operation["marker"]["pin"], pin)
                self.assertEqual(call_tool(host, "datum.journal.undo", {"path": str(root)})["status"], "applied")
                self.assertEqual(query_result(host, "datum.query.schematic_noconnects", root), [])
                self.assertEqual(call_tool(host, "datum.journal.redo", {"path": str(root)})["status"], "applied")
                self.assertEqual(query_result(host, "datum.query.schematic_noconnects", root)[0]["uuid"], noconnect)
                self.assertEqual(
                    call_tool(host, "datum.schematic.delete_noconnect", {"path": str(root), "noconnect": noconnect})["action"],
                    "delete_noconnect",
                )
                self.assertEqual(query_result(host, "datum.query.schematic_noconnects", root), [])
                operation = assert_latest_journal_operation(
                    self, host, str(root), "delete schematic noconnect", "delete_schematic_no_connect"
                )
                self.assertEqual(operation["noconnect_id"], noconnect)

    def test_schematic_bus_tools_call_writes_deletes_and_replays(self) -> None:
        with tempfile.TemporaryDirectory(prefix="datum-mcp-bus-") as tmp:
            root = Path(tmp)
            run_cli_json(root, "project", "new", str(root), "--name", "Bus Parity")
            env = {"DATUM_CLI_BIN": " ".join(shlex.quote(part) for part in datum_cli_prefix())}
            with patch.dict(os.environ, env, clear=False):
                host = StdioToolHost(EngineDaemonClient())
                sheet = call_tool(host, "datum.schematic.create_sheet", {"path": str(root), "name": "Main"})["sheet_uuid"]
                bus = call_tool(
                    host,
                    "datum.schematic.create_bus",
                    {"path": str(root), "sheet": sheet, "name": "DATA", "members": ["DATA0", "DATA1"]},
                )["bus_uuid"]
                buses = query_result(host, "datum.query.schematic_buses", root)
                self.assertEqual(len(buses), 1)
                self.assertEqual(buses[0]["uuid"], bus)
                self.assertEqual(buses[0]["sheet"], sheet)
                self.assertEqual(buses[0]["name"], "DATA")
                self.assertEqual(buses[0]["members"], ["DATA0", "DATA1"])
                operation = assert_latest_journal_operation(
                    self, host, str(root), "create schematic bus", "create_schematic_bus"
                )
                self.assertEqual(operation["sheet_id"], sheet)
                self.assertEqual(operation["bus_id"], bus)
                self.assertEqual(operation["bus"]["members"], ["DATA0", "DATA1"])
                self.assertEqual(
                    call_tool(
                        host,
                        "datum.schematic.edit_bus_members",
                        {"path": str(root), "bus": bus, "members": ["DATA0", "DATA1", "DATA2"]},
                    )["action"],
                    "edit_bus_members",
                )
                self.assertEqual(query_result(host, "datum.query.schematic_buses", root)[0]["members"], ["DATA0", "DATA1", "DATA2"])
                operation = assert_latest_journal_operation(
                    self, host, str(root), "edit schematic bus members", "set_schematic_bus"
                )
                self.assertEqual(operation["bus_id"], bus)
                self.assertEqual(operation["bus"]["members"], ["DATA0", "DATA1", "DATA2"])
                self.assertEqual(call_tool(host, "datum.journal.undo", {"path": str(root)})["status"], "applied")
                self.assertEqual(query_result(host, "datum.query.schematic_buses", root)[0]["members"], ["DATA0", "DATA1"])
                self.assertEqual(call_tool(host, "datum.journal.redo", {"path": str(root)})["status"], "applied")
                self.assertEqual(query_result(host, "datum.query.schematic_buses", root)[0]["members"], ["DATA0", "DATA1", "DATA2"])
                entry = call_tool(
                    host,
                    "datum.schematic.place_bus_entry",
                    {"path": str(root), "sheet": sheet, "bus": bus, "x_nm": 100, "y_nm": 200},
                )["bus_entry_uuid"]
                entries = query_result(host, "datum.query.schematic_bus_entries", root)
                self.assertEqual(len(entries), 1)
                self.assertEqual(entries[0]["uuid"], entry)
                self.assertEqual(entries[0]["sheet"], sheet)
                self.assertEqual(entries[0]["bus"], bus)
                self.assertIsNone(entries[0]["wire"])
                self.assertEqual(entries[0]["position"], {"x": 100, "y": 200})
                operation = assert_latest_journal_operation(
                    self, host, str(root), "place schematic bus entry", "create_schematic_bus_entry"
                )
                self.assertEqual(operation["bus_entry_id"], entry)
                self.assertEqual(operation["bus_entry"]["bus"], bus)
                self.assertEqual(call_tool(host, "datum.journal.undo", {"path": str(root)})["status"], "applied")
                self.assertEqual(query_result(host, "datum.query.schematic_bus_entries", root), [])
                self.assertEqual(call_tool(host, "datum.journal.redo", {"path": str(root)})["status"], "applied")
                self.assertEqual(query_result(host, "datum.query.schematic_bus_entries", root)[0]["uuid"], entry)
                blocked = host.handle_message(
                    {
                        "jsonrpc": "2.0",
                        "id": 1,
                        "method": "tools/call",
                        "params": {"name": "datum.schematic.delete_bus", "arguments": {"path": str(root), "bus": bus}},
                    }
                )["result"]["content"][0]["json"]
                self.assertFalse(blocked["ok"])
                self.assertIn("still referenced by bus entry", blocked["error"]["message"])
                self.assertEqual(
                    call_tool(host, "datum.schematic.delete_bus_entry", {"path": str(root), "bus_entry": entry})["action"],
                    "delete_bus_entry",
                )
                self.assertEqual(query_result(host, "datum.query.schematic_bus_entries", root), [])
                operation = assert_latest_journal_operation(
                    self, host, str(root), "delete schematic bus entry", "delete_schematic_bus_entry"
                )
                self.assertEqual(operation["bus_entry_id"], entry)
                self.assertEqual(
                    call_tool(host, "datum.schematic.delete_bus", {"path": str(root), "bus": bus})["action"],
                    "delete_bus",
                )
                self.assertEqual(query_result(host, "datum.query.schematic_buses", root), [])
                operation = assert_latest_journal_operation(
                    self, host, str(root), "delete schematic bus", "delete_schematic_bus"
                )
                self.assertEqual(operation["bus_id"], bus)
                self.assertEqual(call_tool(host, "datum.journal.undo", {"path": str(root)})["status"], "applied")
                self.assertEqual(query_result(host, "datum.query.schematic_buses", root)[0]["uuid"], bus)
                self.assertEqual(call_tool(host, "datum.journal.redo", {"path": str(root)})["status"], "applied")
                self.assertEqual(query_result(host, "datum.query.schematic_buses", root), [])

    def test_schematic_text_tools_call_writes_deletes_and_replays(self) -> None:
        with tempfile.TemporaryDirectory(prefix="datum-mcp-text-") as tmp:
            root = Path(tmp)
            run_cli_json(root, "project", "new", str(root), "--name", "Text Parity")
            env = {"DATUM_CLI_BIN": " ".join(shlex.quote(part) for part in datum_cli_prefix())}
            with patch.dict(os.environ, env, clear=False):
                host = StdioToolHost(EngineDaemonClient())
                sheet = call_tool(host, "datum.schematic.create_sheet", {"path": str(root), "name": "Main"})["sheet_uuid"]
                text = call_tool(
                    host,
                    "datum.schematic.place_text",
                    {"path": str(root), "sheet": sheet, "text": "note", "x_nm": 10, "y_nm": 20, "rotation_deg": 90},
                )["text_uuid"]
                texts = query_result(host, "datum.query.schematic_texts", root)
                self.assertEqual(len(texts), 1)
                self.assertEqual(texts[0]["uuid"], text)
                self.assertEqual(texts[0]["sheet"], sheet)
                self.assertEqual(texts[0]["text"], "note")
                self.assertEqual(texts[0]["position"], {"x": 10, "y": 20})
                self.assertEqual(texts[0]["rotation"], 90)
                operation = assert_latest_journal_operation(
                    self, host, str(root), "place schematic text", "create_schematic_text"
                )
                self.assertEqual(operation["text_id"], text)
                self.assertEqual(operation["text"]["text"], "note")
                self.assertEqual(
                    call_tool(
                        host,
                        "datum.schematic.edit_text",
                        {"path": str(root), "text": text, "value": "new note", "x_nm": 30, "y_nm": 40},
                    )["action"],
                    "edit_text",
                )
                texts = query_result(host, "datum.query.schematic_texts", root)
                self.assertEqual(texts[0]["text"], "new note")
                self.assertEqual(texts[0]["position"], {"x": 30, "y": 40})
                operation = assert_latest_journal_operation(
                    self, host, str(root), "edit schematic text", "set_schematic_text"
                )
                self.assertEqual(operation["text_id"], text)
                self.assertEqual(call_tool(host, "datum.journal.undo", {"path": str(root)})["status"], "applied")
                self.assertEqual(query_result(host, "datum.query.schematic_texts", root)[0]["text"], "note")
                self.assertEqual(call_tool(host, "datum.journal.redo", {"path": str(root)})["status"], "applied")
                self.assertEqual(query_result(host, "datum.query.schematic_texts", root)[0]["text"], "new note")
                self.assertEqual(
                    call_tool(host, "datum.schematic.delete_text", {"path": str(root), "text": text})["action"],
                    "delete_text",
                )
                self.assertEqual(query_result(host, "datum.query.schematic_texts", root), [])
                operation = assert_latest_journal_operation(
                    self, host, str(root), "delete schematic text", "delete_schematic_text"
                )
                self.assertEqual(operation["text_id"], text)

    def test_schematic_drawing_line_tools_call_writes_deletes_and_replays(self) -> None:
        with tempfile.TemporaryDirectory(prefix="datum-mcp-drawing-") as tmp:
            root = Path(tmp)
            run_cli_json(root, "project", "new", str(root), "--name", "Drawing Parity")
            env = {"DATUM_CLI_BIN": " ".join(shlex.quote(part) for part in datum_cli_prefix())}
            with patch.dict(os.environ, env, clear=False):
                host = StdioToolHost(EngineDaemonClient())
                sheet = call_tool(host, "datum.schematic.create_sheet", {"path": str(root), "name": "Main"})["sheet_uuid"]
                drawing = call_tool(
                    host,
                    "datum.schematic.place_drawing_line",
                    {"path": str(root), "sheet": sheet, "from_x_nm": 0, "from_y_nm": 0, "to_x_nm": 100, "to_y_nm": 0},
                )["drawing_uuid"]
                drawings = query_result(host, "datum.query.schematic_drawings", root)
                self.assertEqual(len(drawings), 1)
                self.assertEqual(drawings[0]["uuid"], drawing)
                self.assertEqual(drawings[0]["sheet"], sheet)
                self.assertEqual(drawings[0]["kind"], "line")
                self.assertEqual(drawings[0]["from"], {"x": 0, "y": 0})
                self.assertEqual(drawings[0]["to"], {"x": 100, "y": 0})
                operation = assert_latest_journal_operation(
                    self, host, str(root), "place schematic drawing line", "create_schematic_drawing"
                )
                self.assertEqual(operation["drawing_id"], drawing)
                self.assertEqual(
                    call_tool(
                        host,
                        "datum.schematic.edit_drawing_line",
                        {"path": str(root), "drawing": drawing, "to_x_nm": 200, "to_y_nm": 0},
                    )["action"],
                    "edit_drawing_line",
                )
                self.assertEqual(query_result(host, "datum.query.schematic_drawings", root)[0]["to"], {"x": 200, "y": 0})
                operation = assert_latest_journal_operation(
                    self, host, str(root), "edit schematic drawing line", "set_schematic_drawing"
                )
                self.assertEqual(operation["drawing_id"], drawing)
                self.assertEqual(call_tool(host, "datum.journal.undo", {"path": str(root)})["status"], "applied")
                self.assertEqual(query_result(host, "datum.query.schematic_drawings", root)[0]["to"], {"x": 100, "y": 0})
                self.assertEqual(call_tool(host, "datum.journal.redo", {"path": str(root)})["status"], "applied")
                self.assertEqual(query_result(host, "datum.query.schematic_drawings", root)[0]["to"], {"x": 200, "y": 0})
                self.assertEqual(
                    call_tool(host, "datum.schematic.delete_drawing", {"path": str(root), "drawing": drawing})["action"],
                    "delete_drawing",
                )
                self.assertEqual(query_result(host, "datum.query.schematic_drawings", root), [])
                operation = assert_latest_journal_operation(
                    self, host, str(root), "delete schematic drawing", "delete_schematic_drawing"
                )
                self.assertEqual(operation["drawing_id"], drawing)

if __name__ == "__main__":
    unittest.main()
