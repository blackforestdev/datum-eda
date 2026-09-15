from __future__ import annotations
import unittest
from server_runtime import StdioToolHost
from test_support import FakeDaemonClient
class TestDispatchDatumTaxonomy(unittest.TestCase):
    def test_tools_call_dispatches_datum_context_get(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 301,
                "method": "tools/call",
                "params": {
                    "name": "datum.context.get",
                    "arguments": {
                        "session": "session-test",
                        "path": "/tmp/context.json",
                        "project_root": "/tmp/native-project",
                    },
                },
            }
        )
        self.assertEqual(
            daemon.calls,
            [
                (
                    "datum_context_get",
                    "session-test",
                    "/tmp/context.json",
                    "/tmp/native-project",
                )
            ],
        )
        payload = response["result"]["content"][0]["json"]
        self.assertTrue(payload["ok"])
        self.assertEqual(payload["schema"], {"name": "datum.context.get", "version": 1})
        self.assertEqual(payload["result"]["contract"], "datum_terminal_context_v1")
        self.assertEqual(
            payload["contract"],
            "datum_terminal_context_v1",
        )
        self.assertEqual(
            payload["actor_type"],
            "ExternalAgent",
        )
        self.assertEqual(
            payload["visible_artifact_ids"],
            [],
        )
        self.assertEqual(payload["visible_output_job_ids"], [])
        self.assertEqual(payload["visible_artifact_file_paths"], [])
    def test_tools_call_dispatches_datum_context_refresh(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 303,
                "method": "tools/call",
                "params": {
                    "name": "datum.context.refresh",
                    "arguments": {
                        "session": "session-test",
                        "path": "/tmp/context.json",
                        "project_root": "/tmp/native-project",
                    },
                },
            }
        )
        self.assertEqual(
            daemon.calls,
            [
                (
                    "datum_context_refresh",
                    "session-test",
                    "/tmp/context.json",
                    "/tmp/native-project",
                )
            ],
        )
        payload = response["result"]["content"][0]["json"]
        self.assertTrue(payload["ok"])
        self.assertEqual(payload["schema"], {"name": "datum.context.refresh", "version": 1})
        self.assertTrue(payload["result"]["refreshed"])
        self.assertTrue(payload["refreshed"])
    def test_canonical_datum_check_and_artifact_tools_return_target_envelopes(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        check_response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 304,
                "method": "tools/call",
                "params": {
                    "name": "datum.check.run",
                    "arguments": {"path": "/tmp/native-project"},
                },
            }
        )
        check_payload = check_response["result"]["content"][0]["json"]
        self.assertTrue(check_payload["ok"])
        self.assertEqual(check_payload["schema"], {"name": "datum.check.run", "version": 1})
        self.assertEqual(check_payload["context"]["project_id"], "project-test")
        self.assertEqual(check_payload["context"]["model_revision"], "model-test")
        self.assertEqual(check_payload["result"]["check_run_id"], "check-run-test")
        self.assertEqual(check_payload["result"]["profile_id"], "native-combined")
        self.assertEqual(check_payload["result"]["status"], "error")
        self.assertEqual(check_payload["result"]["finding_count"], 1)
        self.assertEqual(
            check_payload["result"]["profile_basis"]["profile_id"],
            "native-combined",
        )
        self.assertEqual(
            check_payload["result"]["coverage"][0]["rule_id"],
            "process_aperture_policy",
        )
        self.assertEqual(check_payload["result"]["coverage"][0]["status"], "evaluated")
        finding = check_payload["result"]["findings"][0]
        self.assertEqual(finding["finding_id"], "finding-test")
        self.assertEqual(finding["fingerprint"], "sha256:finding-test")
        self.assertEqual((finding["domain"], finding["rule_id"]), ("standards", "process_aperture_policy"))
        self.assertEqual(finding["standards_basis"], "datum.process_aperture_and_geometry.current")
        self.assertEqual((finding["rule_revision"], finding["import_key"]), ("v1", "kicad:board:/pads/0"))
        self.assertEqual(finding["primary_target"]["object_id"], "pad-test")
        self.assertIn("process mask/paste profile", finding["explanation"])
        self.assertEqual(
            finding["suggested_next_action"],
            "Generate and review a standards repair proposal.",
        )
        self.assertEqual((finding["proposal_refs"], check_payload["result"]["raw"]["action"]), (["proposal-test"], "native_project_check_run"))
        self.assertEqual(check_payload["check_run_id"], "check-run-test")
        artifact_response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 305,
                "method": "tools/call",
                "params": {
                    "name": "datum.artifact.files",
                    "arguments": {
                        "path": "/tmp/native-project",
                        "artifact": "artifact-test",
                    },
                },
            }
        )
        artifact_payload = artifact_response["result"]["content"][0]["json"]
        self.assertTrue(artifact_payload["ok"])
        self.assertEqual(
            artifact_payload["schema"], {"name": "datum.artifact.files", "version": 1}
        )
        self.assertEqual(artifact_payload["context"]["project_id"], "project-test")
        self.assertEqual(artifact_payload["context"]["model_revision"], "model-test")
        self.assertEqual(artifact_payload["result"]["artifact_id"], "artifact-test")
        self.assertEqual(artifact_payload["result"]["kind"], "manufacturing_set")
        self.assertEqual(artifact_payload["result"]["output_dir"], "/tmp/fab")
        self.assertEqual(artifact_payload["result"]["validation_state"], "valid")
        self.assertEqual(artifact_payload["result"]["file_count"], 1)
        artifact_file = artifact_payload["result"]["files"][0]
        self.assertEqual(artifact_file["path"], "fab/doa2526.gbr")
        self.assertEqual(artifact_file["sha256"], "sha256-test")
        self.assertEqual(artifact_file["raw"]["path"], "fab/doa2526.gbr")
        projection = artifact_payload["result"]["production_projections"][0]
        self.assertEqual(projection["projection_kind"], "gerber_copper")
        self.assertEqual(
            artifact_payload["result"]["raw"]["contract"],
            "artifact_files_v1",
        )
        self.assertEqual(artifact_payload["artifact_id"], "artifact-test")
    def test_canonical_datum_nonartifact_families_return_normalized_results(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        proposal_response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 308,
                "method": "tools/call",
                "params": {
                    "name": "datum.proposal.list",
                    "arguments": {"path": "/tmp/native-project"},
                },
            }
        )
        proposal_payload = proposal_response["result"]["content"][0]["json"]
        self.assertTrue(proposal_payload["ok"])
        self.assertEqual(proposal_payload["result"]["proposal_count"], 1)
        proposal = proposal_payload["result"]["proposals"][0]
        self.assertEqual(proposal["proposal_id"], "proposal-test")
        self.assertEqual(proposal["status"], "draft")
        self.assertEqual(proposal["kind"], "standards_repair")
        self.assertEqual(proposal["operation_count"], 1)
        self.assertEqual(proposal["review"]["status"], "draft")
        self.assertEqual(
            proposal_payload["result"]["raw"]["contract"],
            "proposals_query_v1",
        )
        journal_response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 309,
                "method": "tools/call",
                "params": {
                    "name": "datum.journal.list",
                    "arguments": {"path": "/tmp/native-project"},
                },
            }
        )
        journal_payload = journal_response["result"]["content"][0]["json"]
        self.assertTrue(journal_payload["ok"])
        self.assertEqual(journal_payload["result"]["transaction_count"], 1)
        self.assertTrue(journal_payload["result"]["can_undo"])
        transaction = journal_payload["result"]["transactions"][0]
        self.assertEqual(transaction["transaction_id"], "txn-test")
        self.assertEqual(transaction["operation_count"], 2)
        self.assertEqual(transaction["operations"][0]["operation_id"], "op-bind-ci")
        component_response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 310,
                "method": "tools/call",
                "params": {
                    "name": "datum.query.component_instances",
                    "arguments": {"path": "/tmp/native-project"},
                },
            }
        )
        component_payload = component_response["result"]["content"][0]["json"]
        self.assertTrue(component_payload["ok"])
        self.assertEqual(component_payload["result"]["component_instance_count"], 1)
        component = component_payload["result"]["component_instances"][0]
        self.assertEqual(component["component_instance_id"], "ci-test")
        self.assertEqual(component["symbol"], "sym-test")
        self.assertEqual(component["package"], "pkg-test")
        self.assertEqual(component["binding"]["status"], "bound")
        import_response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 311,
                "method": "tools/call",
                "params": {
                    "name": "datum.query.import_map",
                    "arguments": {"path": "/tmp/native-project"},
                },
            }
        )
        import_payload = import_response["result"]["content"][0]["json"]
        self.assertTrue(import_payload["ok"])
        self.assertEqual(import_payload["result"]["import_map_count"], 1)
        entry = import_payload["result"]["entries"][0]
        self.assertEqual(entry["import_id"], "kicad:board:root")
        self.assertEqual(entry["source_hash"], "fixture-source-hash")
        self.assertEqual(entry["target_kind"], "board")
    def test_canonical_production_proposal_aliases_return_target_envelopes(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        cases = [
            (
                "datum.proposal.create_panel_projection",
                {"path": "/tmp/native-project", "key": "panel-a", "proposal": "proposal-panel-create"},
                "propose_create_panel_projection",
                "proposal-panel-create",
            ),
            (
                "datum.proposal.update_panel_projection",
                {
                    "path": "/tmp/native-project",
                    "panel_projection": "panel-test",
                    "name": "Panel A",
                    "proposal": "proposal-panel-update",
                },
                "propose_update_panel_projection",
                "proposal-panel-update",
            ),
            (
                "datum.proposal.delete_panel_projection",
                {
                    "path": "/tmp/native-project",
                    "panel_projection": "panel-test",
                    "proposal": "proposal-panel-delete",
                },
                "propose_delete_panel_projection",
                "proposal-panel-delete",
            ),
            (
                "datum.proposal.create_manufacturing_plan",
                {"path": "/tmp/native-project", "prefix": "release-a", "proposal": "proposal-plan-create"},
                "propose_create_manufacturing_plan",
                "proposal-plan-create",
            ),
            (
                "datum.proposal.update_manufacturing_plan",
                {
                    "path": "/tmp/native-project",
                    "manufacturing_plan": "plan-test",
                    "name": "Release A",
                    "proposal": "proposal-plan-update",
                },
                "propose_update_manufacturing_plan",
                "proposal-plan-update",
            ),
            (
                "datum.proposal.delete_manufacturing_plan",
                {
                    "path": "/tmp/native-project",
                    "manufacturing_plan": "plan-test",
                    "proposal": "proposal-plan-delete",
                },
                "propose_delete_manufacturing_plan",
                "proposal-plan-delete",
            ),
            (
                "datum.proposal.create_output_job",
                {
                    "path": "/tmp/native-project",
                    "prefix": "release-a",
                    "include": "gerber-set",
                    "proposal": "proposal-job-create",
                },
                "propose_create_output_job",
                "proposal-job-create",
            ),
            (
                "datum.proposal.update_output_job",
                {
                    "path": "/tmp/native-project",
                    "output_job": "job-test",
                    "name": "Release CAM",
                    "proposal": "proposal-job-update",
                },
                "propose_update_output_job",
                "proposal-job-update",
            ),
            (
                "datum.proposal.delete_output_job",
                {
                    "path": "/tmp/native-project",
                    "output_job": "job-test",
                    "proposal": "proposal-job-delete",
                },
                "propose_delete_output_job",
                "proposal-job-delete",
            ),
        ]
        for index, (tool_name, arguments, expected_action, expected_proposal) in enumerate(
            cases,
            start=313,
        ):
            response = host.handle_message(
                {
                    "jsonrpc": "2.0",
                    "id": index,
                    "method": "tools/call",
                    "params": {"name": tool_name, "arguments": arguments},
                }
            )
            payload = response["result"]["content"][0]["json"]
            self.assertTrue(payload["ok"])
            self.assertEqual(payload["schema"], {"name": tool_name, "version": 1})
            self.assertEqual(payload["result"]["action"], expected_action)
            self.assertEqual(payload["result"]["proposal_id"], expected_proposal)
            self.assertEqual(payload["result"]["raw"]["contract"], "proposal_create_v1")
            self.assertEqual(payload["result"]["raw"]["action"], expected_action)
            self.assertEqual(payload["proposal_id"], expected_proposal)
    def test_canonical_datum_failures_return_target_error_envelopes(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        check_response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 306,
                "method": "tools/call",
                "params": {
                    "name": "datum.check.run",
                    "arguments": {},
                },
            }
        )
        check_payload = check_response["result"]["content"][0]["json"]
        self.assertFalse(check_payload["ok"])
        self.assertEqual(check_payload["schema"], {"name": "datum.check.run", "version": 1})
        self.assertEqual(check_payload["context"]["project_id"], None)
        self.assertEqual(check_payload["error"]["code"], "tool_call_failed")
        self.assertIn("path", check_payload["error"]["message"])
        self.assertEqual(check_payload["error"]["details"]["exception_type"], "KeyError")
        artifact_response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 307,
                "method": "tools/call",
                "params": {
                    "name": "datum.artifact.validate",
                    "arguments": {"path": "/tmp/native-project"},
                },
            }
        )
        artifact_payload = artifact_response["result"]["content"][0]["json"]
        self.assertFalse(artifact_payload["ok"])
        self.assertEqual(
            artifact_payload["schema"], {"name": "datum.artifact.validate", "version": 1}
        )
        self.assertEqual(artifact_payload["context"]["model_revision"], None)
        self.assertEqual(artifact_payload["error"]["code"], "tool_call_failed")
        self.assertIn("artifact", artifact_payload["error"]["message"])
        self.assertEqual(artifact_payload["error"]["details"]["exception_type"], "KeyError")
    def test_canonical_datum_envelope_reserved_keys_cannot_be_overwritten(self) -> None:
        daemon = FakeDaemonClient()
        original_get_check_run = daemon.get_check_run
        def conflicting_check_run(path, profile=None):
            response = original_get_check_run(path, profile)
            response.result["ok"] = "legacy-ok"
            response.result["schema"] = {"name": "legacy.schema", "version": 999}
            response.result["context"] = {"project_id": "legacy-project"}
            response.result["result"] = {"legacy": True}
            response.result["error"] = {"code": "legacy_error"}
            return response
        daemon.get_check_run = conflicting_check_run
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 312,
                "method": "tools/call",
                "params": {
                    "name": "datum.check.run",
                    "arguments": {"path": "/tmp/native-project"},
                },
            }
        )
        payload = response["result"]["content"][0]["json"]
        self.assertTrue(payload["ok"])
        self.assertEqual(payload["schema"], {"name": "datum.check.run", "version": 1})
        self.assertEqual(payload["context"]["project_id"], "project-test")
        self.assertEqual(payload["result"]["check_run_id"], "check-run-test")
        self.assertNotIn("error", payload)
        self.assertEqual(payload["result"]["raw"]["ok"], "legacy-ok")

    def test_tools_call_dispatches_datum_query_aliases(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        for tool_name, expected_call in [
            ("datum.check.run", "get_check_run"),
            ("datum.check.list", "get_check_runs"),
            ("datum.check.show", "show_check_run"),
            ("datum.check.profiles", "get_check_profiles"),
            ("datum.check.fill_zones", "fill_zones"),
            ("datum.check.repair_standards", "generate_standards_repair_proposals"),
            ("datum.check.waive", "waive_finding"),
            ("datum.check.accept_deviation", "accept_deviation"),
            ("datum.check.explain_violation", "explain_violation"),
            ("datum.query.board_summary", "get_board_summary"),
            ("datum.query.components", "get_components"),
            ("datum.query.netlist", "get_netlist"),
            ("datum.query.schematic_summary", "get_schematic_summary"),
            ("datum.query.schematic_wires", "get_schematic_wires"),
            ("datum.query.schematic_junctions", "get_schematic_junctions"),
            ("datum.query.board_tracks", "get_board_tracks"),
            ("datum.query.board_vias", "get_board_vias"),
            ("datum.query.board_pads", "get_board_pads"),
            *[(f"datum.query.board_{p.split(':')[0]}", f"get_board_{p.split(':')[1]}") for p in "zones:zones texts:texts keepouts:keepouts outline:outline stackup:stackup dimensions:dimensions nets:nets net_classes:net_classes".split()],
            ("datum.query.schematic_labels", "get_schematic_labels"),
            ("datum.query.schematic_ports", "get_schematic_ports"),
            ("datum.query.schematic_noconnects", "get_schematic_noconnects"),
            ("datum.query.schematic_buses", "get_schematic_buses"),
            ("datum.query.schematic_bus_entries", "get_schematic_bus_entries"),
            ("datum.query.schematic_texts", "get_schematic_texts"),
            ("datum.query.schematic_drawings", "get_schematic_drawings"),
            *[(f"datum.query.{p.split(':')[0]}", p.split(':')[1]) for p in "sheets:get_sheets symbols:get_symbols symbol_fields:get_symbol_fields labels:get_labels ports:get_ports buses:get_buses bus_entries:get_bus_entries noconnects:get_noconnects hierarchy:get_project_hierarchy schematic_nets:get_schematic_net_info connectivity_diagnostics:get_connectivity_diagnostics design_rules:get_design_rules".split()],
            ("datum.query.source_shards", "get_source_shards"),
            ("datum.query.zone_fills", "get_zone_fills"),
            ("datum.query.component_instances", "get_component_instances"),
            ("datum.query.relationships", "get_relationships"),
            ("datum.query.variants", "get_variants"),
            ("datum.query.import_map", "get_import_map"),
            ("datum.query.panel_projections", "get_panel_projections"),
            ("datum.query.manufacturing_plans", "get_manufacturing_plans"),
            ("datum.query.output_jobs", "get_output_jobs"),
            ("datum.pcb.place_component", "place_board_component"),
            ("datum.pcb.generate_board_components", "generate_board_components"),
            ("datum.pcb.move_component", "move_board_component"),
            ("datum.pcb.rotate_component", "rotate_board_component"),
            ("datum.pcb.flip_component", "flip_board_component"),
            ("datum.pcb.delete_component", "delete_board_component"),
            ("datum.pcb.set_component_reference", "set_board_component_reference"),
            ("datum.pcb.set_component_value", "set_board_component_value"),
            ("datum.pcb.set_component_part", "set_board_component_part"),
            ("datum.pcb.set_component_package", "set_board_component_package"),
            ("datum.pcb.lock_component", "lock_board_component"),
            ("datum.pcb.unlock_component", "unlock_board_component"),
            ("datum.pcb.draw_track", "draw_board_track"),
            ("datum.pcb.edit_track", "edit_board_track"),
            ("datum.pcb.delete_track", "delete_board_track"),
            ("datum.pcb.place_via", "place_board_via"),
            ("datum.pcb.edit_via", "edit_board_via"),
            ("datum.pcb.delete_via", "delete_board_via"),
            ("datum.pcb.place_zone", "place_board_zone"),
            ("datum.pcb.edit_zone", "edit_board_zone"),
            ("datum.pcb.delete_zone", "delete_board_zone"),
            ("datum.pcb.place_pad", "place_board_pad"),
            ("datum.pcb.edit_pad", "edit_board_pad"),
            ("datum.pcb.delete_pad", "delete_board_pad"),
            ("datum.pcb.set_pad_net", "set_board_pad_net"),
            ("datum.pcb.clear_pad_net", "clear_board_pad_net"),
            ("datum.pcb.place_net", "place_board_net"),
            ("datum.pcb.edit_net", "edit_board_net"),
            ("datum.pcb.delete_net", "delete_board_net"),
            ("datum.pcb.set_board_name", "set_board_name"),
            ("datum.pcb.set_outline", "set_board_outline"),
            ("datum.pcb.set_stackup", "set_board_stackup"),
            ("datum.pcb.add_default_top_stackup", "add_default_top_stackup"),
            ("datum.pcb.place_keepout", "place_board_keepout"),
            ("datum.pcb.edit_keepout", "edit_board_keepout"),
            ("datum.pcb.delete_keepout", "delete_board_keepout"),
            ("datum.pcb.place_dimension", "place_board_dimension"),
            ("datum.pcb.edit_dimension", "edit_board_dimension"),
            ("datum.pcb.delete_dimension", "delete_board_dimension"),
            ("datum.pcb.place_text", "place_board_text"),
            ("datum.pcb.edit_text", "edit_board_text"),
            ("datum.pcb.delete_text", "delete_board_text"),
            ("datum.pcb.place_net_class", "place_board_net_class"),
            ("datum.pcb.edit_net_class", "edit_board_net_class"),
            ("datum.pcb.delete_net_class", "delete_board_net_class"),
            ("datum.component_instance.bind", "bind_component_instance"),
            ("datum.component_instance.set", "set_component_instance"),
            ("datum.component_instance.delete", "delete_component_instance"),
            ("datum.library.list_objects", "get_pool_library_objects"), ("datum.library.show_object", "show_pool_library_object"), ("datum.library.pool_models", "get_pool_model_blobs"), ("datum.library.gc_pool_models", "gc_pool_model_blobs"), ("datum.library.create_object", "create_pool_library_object"), ("datum.library.create_unit", "create_pool_unit"), ("datum.library.set_unit_pin", "set_pool_unit_pin"), ("datum.library.create_symbol", "create_pool_symbol"), ("datum.library.add_symbol_line", "add_pool_symbol_line"), ("datum.library.add_symbol_rect", "add_pool_symbol_rect"), ("datum.library.add_symbol_circle", "add_pool_symbol_circle"), ("datum.library.add_symbol_arc", "add_pool_symbol_arc"), ("datum.library.add_symbol_polygon", "add_pool_symbol_polygon"), ("datum.library.add_symbol_text", "add_pool_symbol_text"), ("datum.library.set_symbol_pin_anchor", "set_pool_symbol_pin_anchor"), ("datum.library.create_entity", "create_pool_entity"), ("datum.library.create_padstack", "create_pool_padstack"), ("datum.library.create_package", "create_pool_package"), ("datum.library.create_footprint", "create_pool_footprint"), ("datum.library.set_footprint_pad", "set_pool_footprint_pad"), ("datum.library.set_footprint_courtyard_rect", "set_pool_footprint_courtyard_rect"), ("datum.library.set_footprint_courtyard_polygon", "set_pool_footprint_courtyard_polygon"), ("datum.library.add_footprint_silkscreen_line", "add_pool_footprint_silkscreen_line"), ("datum.library.set_package_pad", "set_pool_package_pad"), ("datum.library.set_package_courtyard_rect", "set_pool_package_courtyard_rect"), ("datum.library.set_package_courtyard_polygon", "set_pool_package_courtyard_polygon"), ("datum.library.add_package_silkscreen_line", "add_pool_package_silkscreen_line"), ("datum.library.add_package_silkscreen_rect", "add_pool_package_silkscreen_rect"), ("datum.library.add_package_silkscreen_polygon", "add_pool_package_silkscreen_polygon"), ("datum.library.add_package_silkscreen_circle", "add_pool_package_silkscreen_circle"), ("datum.library.add_package_silkscreen_arc", "add_pool_package_silkscreen_arc"), ("datum.library.add_package_silkscreen_text", "add_pool_package_silkscreen_text"), ("datum.library.create_part", "create_pool_part"), ("datum.library.set_part_metadata", "set_pool_part_metadata"), ("datum.library.set_part_parametric", "set_pool_part_parametric"), ("datum.library.set_part_orderable_mpns", "set_pool_part_orderable_mpns"), ("datum.library.set_part_tags", "set_pool_part_tags"), ("datum.library.set_part_behavioural_models", "set_pool_part_behavioural_models"), ("datum.library.attach_part_model", "attach_pool_part_model"), ("datum.library.detach_part_model", "detach_pool_part_model"), ("datum.library.set_part_thermal", "set_pool_part_thermal"), ("datum.library.set_part_pad_map_entry", "set_pool_part_pad_map_entry"), ("datum.library.set_part_pad_map", "set_pool_part_pad_map"), ("datum.library.create_pin_pad_map", "create_pool_pin_pad_map"), ("datum.library.set_pin_pad_map", "set_pool_pin_pad_map"), ("datum.library.set_object", "set_pool_library_object"), ("datum.library.delete_object", "delete_pool_library_object"),
            ("datum.library.add_footprint_silkscreen_rect", "add_pool_footprint_silkscreen_rect"), ("datum.library.add_footprint_silkscreen_circle", "add_pool_footprint_silkscreen_circle"), ("datum.library.add_footprint_silkscreen_polygon", "add_pool_footprint_silkscreen_polygon"),
            ("datum.schematic.create_sheet", "create_sheet"), ("datum.schematic.delete_sheet", "delete_sheet"), ("datum.schematic.rename_sheet", "rename_sheet"), ("datum.schematic.create_sheet_definition", "create_sheet_definition"), ("datum.schematic.create_sheet_instance", "create_sheet_instance"), ("datum.schematic.delete_sheet_instance", "delete_sheet_instance"), ("datum.schematic.move_sheet_instance", "move_sheet_instance"), ("datum.schematic.bind_sheet_instance_port", "bind_sheet_instance_port"), ("datum.schematic.unbind_sheet_instance_port", "unbind_sheet_instance_port"), ("datum.schematic.draw_wire", "draw_wire"),
            ("datum.schematic.delete_wire", "delete_wire"),
            ("datum.schematic.place_junction", "place_junction"),
            ("datum.schematic.delete_junction", "delete_junction"),
            ("datum.schematic.place_noconnect", "place_noconnect"),
            ("datum.schematic.delete_noconnect", "delete_noconnect"),
            ("datum.schematic.place_label", "place_label"),
            ("datum.schematic.rename_label", "rename_label"),
            ("datum.schematic.delete_label", "delete_label"),
            ("datum.schematic.place_port", "place_port"),
            ("datum.schematic.edit_port", "edit_port"),
            ("datum.schematic.delete_port", "delete_port"),
            ("datum.schematic.create_bus", "create_bus"),
            ("datum.schematic.edit_bus_members", "edit_bus_members"),
            ("datum.schematic.delete_bus", "delete_bus"),
            ("datum.schematic.place_bus_entry", "place_bus_entry"),
            ("datum.schematic.delete_bus_entry", "delete_bus_entry"),
            ("datum.schematic.place_text", "place_schematic_text"),
            ("datum.schematic.edit_text", "edit_schematic_text"),
            ("datum.schematic.delete_text", "delete_schematic_text"),
            *[(f"datum.schematic.{name}", name) for name in "place_drawing_line place_drawing_rect place_drawing_circle place_drawing_arc edit_drawing_line edit_drawing_rect edit_drawing_circle edit_drawing_arc delete_drawing".split()],
            *[(f"datum.schematic.{name}", name) for name in "place_symbol move_symbol rotate_symbol mirror_symbol delete_symbol set_symbol_reference set_symbol_value set_symbol_display_mode set_symbol_hidden_power_behavior set_symbol_unit clear_symbol_unit set_symbol_gate clear_symbol_gate set_symbol_entity clear_symbol_entity set_symbol_part clear_symbol_part set_symbol_lib_id clear_symbol_lib_id set_pin_override clear_pin_override add_symbol_field edit_symbol_field delete_symbol_field".split()],
            ("datum.proposal.create", "create_proposal"),
            ("datum.proposal.create_panel_projection", "create_panel_projection_proposal"),
            ("datum.proposal.update_panel_projection", "update_panel_projection_proposal"),
            ("datum.proposal.delete_panel_projection", "delete_panel_projection_proposal"),
            ("datum.proposal.create_manufacturing_plan", "create_manufacturing_plan_proposal"),
            ("datum.proposal.update_manufacturing_plan", "update_manufacturing_plan_proposal"),
            ("datum.proposal.delete_manufacturing_plan", "delete_manufacturing_plan_proposal"),
            ("datum.proposal.create_output_job", "create_output_job_proposal"),
            ("datum.proposal.update_output_job", "update_output_job_proposal"),
            ("datum.proposal.delete_output_job", "delete_output_job_proposal"),
            ("datum.proposal.list", "get_proposals"),
            ("datum.proposal.show", "show_proposal"),
            ("datum.proposal.preview", "preview_proposal"),
            ("datum.proposal.validate", "validate_proposal"),
            ("datum.proposal.review", "review_proposal"),
            ("datum.proposal.defer", "defer_proposal"),
            ("datum.proposal.reject", "reject_proposal"),
            ("datum.proposal.accept_apply", "accept_apply_proposal"),
            ("datum.proposal.apply", "apply_proposal"),
            ("datum.journal.list", "get_journal_list"),
            ("datum.journal.show", "get_journal_transaction"),
            ("datum.journal.undo", "journal_undo"),
            ("datum.journal.redo", "journal_redo"),
            ("datum.artifact.generate", "generate_artifacts"),
            ("datum.artifact.list", "get_artifacts"),
            ("datum.artifact.show", "show_artifact"),
            ("datum.artifact.files", "get_artifact_files"),
            ("datum.artifact.preview", "preview_artifact_file"),
            ("datum.artifact.compare", "compare_artifacts"),
            ("datum.artifact.validate", "validate_artifact"),
            ("datum.artifact.start_output_job_run", "start_output_job_run"),
            ("datum.artifact.cancel_output_job_run", "cancel_output_job_run"),
            ("datum.artifact.export_manufacturing_set", "export_manufacturing_set"),
            ("datum.artifact.validate_manufacturing_set", "validate_manufacturing_set"),
        ]:
            arguments = {"path": "/tmp/native-project"}
            if tool_name == "datum.proposal.create":
                arguments["batch"] = "/tmp/batch.json"
                arguments["rationale"] = "review batch"
            if tool_name == "datum.proposal.create_panel_projection": arguments["key"] = "panel-a"
            if tool_name == "datum.proposal.update_panel_projection": arguments["panel_projection"] = "panel-test"
            if tool_name == "datum.proposal.delete_panel_projection":
                arguments["panel_projection"] = "panel-test"
            if tool_name == "datum.proposal.create_manufacturing_plan":
                arguments["prefix"] = "release-a"
            if tool_name == "datum.proposal.update_manufacturing_plan":
                arguments["manufacturing_plan"] = "plan-test"
            if tool_name == "datum.proposal.delete_manufacturing_plan":
                arguments["manufacturing_plan"] = "plan-test"
            if tool_name == "datum.proposal.create_output_job": arguments.update({"prefix": "release-a", "include": "gerber-set"})
            if tool_name == "datum.proposal.update_output_job":
                arguments["output_job"] = "job-test"
            if tool_name == "datum.proposal.delete_output_job":
                arguments["output_job"] = "job-test"
            if tool_name.startswith("datum.proposal.") and tool_name != "datum.proposal.list":
                arguments["proposal"] = "proposal-test"
            if tool_name == "datum.proposal.review":
                arguments["status"] = "accepted"
            if tool_name == "datum.journal.show": arguments["transaction"] = "txn-test"
            if tool_name in {"datum.journal.undo", "datum.journal.redo"}: arguments["expected_tip_transaction"] = "txn-tip"
            if tool_name == "datum.artifact.generate":
                arguments["output_dir"] = "/tmp/fab"
                arguments["include"] = "gerber-set"
                arguments["prefix"] = "doa2526"
            if tool_name in {"datum.artifact.show", "datum.artifact.files", "datum.artifact.preview"}: arguments["artifact"] = "artifact-test"
            if tool_name == "datum.artifact.preview": arguments["file"] = "fab/doa2526.gbr"
            if tool_name == "datum.artifact.compare": arguments.update({"before": "artifact-before", "after": "artifact-after"})
            if tool_name == "datum.artifact.validate":
                arguments["artifact"] = "artifact-test"
            if tool_name == "datum.artifact.start_output_job_run":
                arguments["output_job"] = "job-test"
            if tool_name == "datum.artifact.cancel_output_job_run":
                arguments["run"] = "run-test"
            if tool_name in {"datum.artifact.export_manufacturing_set", "datum.artifact.validate_manufacturing_set"}: arguments.update({"output_dir": "/tmp/fab", "prefix": "doa2526"})
            if tool_name in {"datum.check.waive", "datum.check.accept_deviation"}: arguments.update({"fingerprint": "finding-fingerprint", "rationale": "documented exception"})
            if tool_name == "datum.check.explain_violation": arguments.update({"domain": "drc", "fingerprint": "sha256:finding"})
            if tool_name == "datum.check.show": arguments["check_run"] = "check-run-test"
            if tool_name == "datum.query.symbol_fields": arguments["symbol_uuid"] = "symbol-1"
            if tool_name == "datum.component_instance.bind": arguments.update({"symbol": "sym-test", "package": "pkg-test"})
            if tool_name == "datum.component_instance.set": arguments.update({"component_instance": "ci-test", "symbol": "sym-next", "package": "pkg-next"})
            if tool_name == "datum.component_instance.delete": arguments["component_instance"] = "ci-test"
            if tool_name == "datum.library.list_objects": arguments.update({"kind": "symbols", "include_payload": True})
            if tool_name in {"datum.library.show_object", "datum.library.pool_models", "datum.library.gc_pool_models"}: arguments.update({"datum.library.show_object": {"object": "symbol-test", "kind": "symbols"}, "datum.library.pool_models": {"pool": "pool", "role": "spice", "sha256": "abc123"}, "datum.library.gc_pool_models": {"pool": "pool", "role": "spice", "sha256": "abc123", "apply": True}}[tool_name])
            if tool_name == "datum.library.create_object": arguments.update({"pool": "pool", "kind": "symbols", "object": "symbol-test", "from_json": "/tmp/symbol.json"})
            if tool_name in {"datum.library.create_unit", "datum.library.set_unit_pin", "datum.library.create_symbol", "datum.library.add_symbol_line", "datum.library.add_symbol_rect", "datum.library.add_symbol_circle", "datum.library.add_symbol_arc", "datum.library.add_symbol_polygon", "datum.library.add_symbol_text", "datum.library.set_symbol_pin_anchor", "datum.library.create_entity", "datum.library.create_padstack", "datum.library.create_package", "datum.library.create_footprint", "datum.library.set_footprint_pad", "datum.library.set_footprint_courtyard_rect", "datum.library.set_footprint_courtyard_polygon", "datum.library.add_footprint_silkscreen_line", "datum.library.set_package_pad", "datum.library.set_package_courtyard_rect", "datum.library.set_package_courtyard_polygon", "datum.library.add_package_silkscreen_line", "datum.library.add_package_silkscreen_rect", "datum.library.add_package_silkscreen_polygon", "datum.library.add_package_silkscreen_circle", "datum.library.add_package_silkscreen_arc", "datum.library.add_package_silkscreen_text", "datum.library.add_package_model_3d", "datum.library.set_package_body_heights", "datum.library.create_part", "datum.library.set_part_metadata", "datum.library.set_part_parametric", "datum.library.set_part_orderable_mpns", "datum.library.set_part_tags", "datum.library.set_part_behavioural_models", "datum.library.attach_part_model", "datum.library.detach_part_model", "datum.library.set_part_thermal", "datum.library.set_part_pad_map_entry", "datum.library.set_part_pad_map", "datum.library.create_pin_pad_map", "datum.library.set_pin_pad_map", "datum.library.set_object", "datum.library.delete_object"}: arguments.update({"pool": "pool", "unit": "unit-test", "name": "OpAmpUnit", "manufacturer": "Datum"} if tool_name == "datum.library.create_unit" else {"pool": "pool", "unit": "unit-test", "pin": "pin-test", "name": "OUT", "direction": "Output", "swap_group": 1} if tool_name == "datum.library.set_unit_pin" else {"pool": "pool", "symbol": "symbol-test", "unit": "unit-test", "name": "OpAmpSymbol"} if tool_name == "datum.library.create_symbol" else {"pool": "pool", "symbol": "symbol-test", "from_x_nm": 0, "from_y_nm": 0, "to_x_nm": 1000, "to_y_nm": 0, "width_nm": 100} if tool_name == "datum.library.add_symbol_line" else {"pool": "pool", "symbol": "symbol-test", "min_x_nm": 0, "min_y_nm": 0, "max_x_nm": 1000, "max_y_nm": 2000, "width_nm": 100} if tool_name == "datum.library.add_symbol_rect" else {"pool": "pool", "symbol": "symbol-test", "center_x_nm": 500, "center_y_nm": 600, "radius_nm": 250, "width_nm": 100} if tool_name == "datum.library.add_symbol_circle" else {"pool": "pool", "symbol": "symbol-test", "x_nm": 1000, "y_nm": 2000, "radius_nm": 3000, "start_angle": 0, "end_angle": 900, "width_nm": 150000} if tool_name == "datum.library.add_symbol_arc" else {"pool": "pool", "symbol": "symbol-test", "vertices": "0,0;1000,0;1000,1000", "closed": True, "width_nm": 150000} if tool_name == "datum.library.add_symbol_polygon" else {"pool": "pool", "symbol": "symbol-test", "text": "REF**", "x_nm": 100, "y_nm": 200, "rotation": 900} if tool_name == "datum.library.add_symbol_text" else {"pool": "pool", "symbol": "symbol-test", "pin": "pin-test", "x_nm": 100, "y_nm": 200} if tool_name == "datum.library.set_symbol_pin_anchor" else {"pool": "pool", "entity": "entity-test", "gate": "gate-test", "unit": "unit-test", "symbol": "symbol-test", "name": "DualOpAmp", "prefix": "U", "manufacturer": "Datum", "gate_name": "A"} if tool_name == "datum.library.create_entity" else {"pool": "pool", "padstack": "padstack-test", "name": "RoundViaPad", "aperture": "circle", "diameter_nm": 1200000, "drill_nm": 600000} if tool_name == "datum.library.create_padstack" else {"pool": "pool", "package": "package-test", "name": "SOT23", "pad": "pad-test", "padstack": "padstack-test", "pad_name": "1", "x_nm": 1000, "y_nm": 2000, "layer": 1} if tool_name == "datum.library.create_package" else {"pool": "pool", "footprint": "footprint-test", "package": "package-test", "name": "SOT23_LandPattern"} if tool_name == "datum.library.create_footprint" else {"pool": "pool", "footprint": "footprint-test", "pad": "pad-test", "padstack": "padstack-test", "pad_name": "2", "x_nm": 1000, "y_nm": 2000, "layer": 1} if tool_name == "datum.library.set_footprint_pad" else {"pool": "pool", "footprint": "footprint-test", "min_x_nm": 1000, "min_y_nm": 2000, "max_x_nm": 3000, "max_y_nm": 4000} if tool_name == "datum.library.set_footprint_courtyard_rect" else {"pool": "pool", "footprint": "footprint-test", "vertices": "0,0;1000,0;1000,1000"} if tool_name == "datum.library.set_footprint_courtyard_polygon" else {"pool": "pool", "footprint": "footprint-test", "from_x_nm": 1000, "from_y_nm": 2000, "to_x_nm": 3000, "to_y_nm": 4000, "width_nm": 150000} if tool_name == "datum.library.add_footprint_silkscreen_line" else {"pool": "pool", "package": "package-test", "pad": "pad-test", "padstack": "padstack-test", "pad_name": "2", "x_nm": 1000, "y_nm": 2000, "layer": 1} if tool_name == "datum.library.set_package_pad" else {"pool": "pool", "package": "package-test", "min_x_nm": 1000, "min_y_nm": 2000, "max_x_nm": 3000, "max_y_nm": 4000} if tool_name == "datum.library.set_package_courtyard_rect" else {"pool": "pool", "package": "package-test", "vertices": "0,0;1000,0;1000,1000"} if tool_name == "datum.library.set_package_courtyard_polygon" else {"pool": "pool", "package": "package-test", "from_x_nm": 1000, "from_y_nm": 2000, "to_x_nm": 3000, "to_y_nm": 4000, "width_nm": 150000} if tool_name == "datum.library.add_package_silkscreen_line" else {"pool": "pool", "package": "package-test", "min_x_nm": 1000, "min_y_nm": 2000, "max_x_nm": 3000, "max_y_nm": 4000, "width_nm": 150000} if tool_name == "datum.library.add_package_silkscreen_rect" else {"pool": "pool", "package": "package-test", "vertices": "0,0;1000,0;1000,1000", "closed": True, "width_nm": 150000} if tool_name == "datum.library.add_package_silkscreen_polygon" else {"pool": "pool", "package": "package-test", "center_x_nm": 1000, "center_y_nm": 2000, "radius_nm": 3000, "width_nm": 150000} if tool_name == "datum.library.add_package_silkscreen_circle" else {"pool": "pool", "package": "package-test", "x_nm": 1000, "y_nm": 2000, "radius_nm": 3000, "start_angle": 0, "end_angle": 900, "width_nm": 150000} if tool_name == "datum.library.add_package_silkscreen_arc" else {"pool": "pool", "package": "package-test", "text": "REF**", "x_nm": 1000, "y_nm": 2000, "rotation": 90} if tool_name == "datum.library.add_package_silkscreen_text" else {"pool": "pool", "package": "package-test", "model_path": "models/pkg.step", "transform_json": None} if tool_name == "datum.library.add_package_model_3d" else {"pool": "pool", "package": "package-test", "body_height_nm": 1000000, "body_height_mounted_nm": 1200000, "clear": False} if tool_name == "datum.library.set_package_body_heights" else {"pool": "pool", "part": "part-test", "entity": "entity-test", "package": "package-test", "mpn": "OPA1656ID", "manufacturer": "Texas Instruments", "value": "OPA1656", "description": "", "datasheet": "", "lifecycle": "Active"} if tool_name == "datum.library.create_part" else {"pool": "pool", "part": "part-test", "mpn": "OPA1656ID", "manufacturer": "Texas Instruments"} if tool_name == "datum.library.set_part_metadata" else {"pool": "pool", "part": "part-test", "mode": "merge", "params": {"gbw": "53MHz"}} if tool_name == "datum.library.set_part_parametric" else {"pool": "pool", "part": "part-test", "mode": "merge", "mpns": ["OPA1656ID", "OPA1656IDR"]} if tool_name == "datum.library.set_part_orderable_mpns" else {"pool": "pool", "part": "part-test", "mode": "merge", "tags": ["audio", "opamp"]} if tool_name == "datum.library.set_part_tags" else {"pool": "pool", "part": "part-test", "mode": "merge", "models": ["{\"kind\":\"spice\",\"path\":\"models/opamp.lib\"}"]} if tool_name == "datum.library.set_part_behavioural_models" else {"pool": "pool", "part": "part-test", "source": "models/opamp.lib", "role": "simulation", "model_names": ["OPA1656"]} if tool_name == "datum.library.attach_part_model" else {"pool": "pool", "part": "part-test", "attachment": "attachment-test"} if tool_name == "datum.library.detach_part_model" else {"pool": "pool", "part": "part-test", "theta_ja_c_per_w": "42.5", "thermal_reference": "JEDEC"} if tool_name == "datum.library.set_part_thermal" else {"pool": "pool", "part": "part-test", "pad": "pad-test", "gate": "gate-test", "pin": "pin-test"} if tool_name == "datum.library.set_part_pad_map_entry" else {"pool": "pool", "part": "part-test", "mode": "replace", "entries": [{"pad": "pad-test", "gate": "gate-test", "pin": "pin-test"}]} if tool_name == "datum.library.set_part_pad_map" else {"pool": "pool", "map": "map-test", "part": "part-test", "entries": [{"pin": "pin-test", "pad": "pad-test"}], "set_default": True} if tool_name == "datum.library.create_pin_pad_map" else {"pool": "pool", "map": "map-test", "mode": "replace", "entries": [{"pin": "pin-test", "pad": "pad-test"}]} if tool_name == "datum.library.set_pin_pad_map" else {"pool": "pool", "kind": "symbols", "object": "symbol-test", "from_json": "/tmp/symbol-edited.json"} if tool_name == "datum.library.set_object" else {"kind": "symbols", "object": "symbol-test"})
            if tool_name in {"datum.library.add_footprint_silkscreen_rect", "datum.library.add_footprint_silkscreen_circle", "datum.library.add_footprint_silkscreen_polygon"}: arguments.update({"datum.library.add_footprint_silkscreen_rect": {"pool": "pool", "footprint": "footprint-test", "min_x_nm": 1000, "min_y_nm": 2000, "max_x_nm": 3000, "max_y_nm": 4000, "width_nm": 150000}, "datum.library.add_footprint_silkscreen_circle": {"pool": "pool", "footprint": "footprint-test", "center_x_nm": 1000, "center_y_nm": 2000, "radius_nm": 3000, "width_nm": 150000}, "datum.library.add_footprint_silkscreen_polygon": {"pool": "pool", "footprint": "footprint-test", "vertices": "0,0;1000,0;1000,1000", "closed": True, "width_nm": 150000}}[tool_name])
            if tool_name in ("datum.schematic.create_sheet", "datum.schematic.delete_sheet", "datum.schematic.rename_sheet", "datum.schematic.create_sheet_definition", "datum.schematic.create_sheet_instance", "datum.schematic.delete_sheet_instance", "datum.schematic.move_sheet_instance", "datum.schematic.bind_sheet_instance_port", "datum.schematic.unbind_sheet_instance_port"): arguments = {"datum.schematic.create_sheet": {"path": "/tmp/native-project", "name": "Aux", "sheet": "sheet-1"}, "datum.schematic.delete_sheet": {"path": "/tmp/native-project", "sheet": "sheet-1"}, "datum.schematic.rename_sheet": {"path": "/tmp/native-project", "sheet": "sheet-1", "name": "Renamed"}, "datum.schematic.create_sheet_definition": {"path": "/tmp/native-project", "root_sheet": "sheet-1", "name": "Main Definition", "definition": "definition-1"}, "datum.schematic.create_sheet_instance": {"path": "/tmp/native-project", "definition": "definition-1", "parent_sheet": "sheet-1", "name": "Main Instance", "x_nm": 100, "y_nm": 200, "instance": "instance-1"}, "datum.schematic.delete_sheet_instance": {"path": "/tmp/native-project", "instance": "instance-1"}, "datum.schematic.move_sheet_instance": {"path": "/tmp/native-project", "instance": "instance-1", "x_nm": 300, "y_nm": 400}, "datum.schematic.bind_sheet_instance_port": {"path": "/tmp/native-project", "instance": "instance-1", "port": "port-1"}, "datum.schematic.unbind_sheet_instance_port": {"path": "/tmp/native-project", "instance": "instance-1", "port": "port-1"}}[tool_name]
            if tool_name == "datum.schematic.draw_wire": arguments = {"path": "/tmp/native-project", "sheet": "sheet-1", "from_x_nm": 0, "from_y_nm": 0, "to_x_nm": 1000, "to_y_nm": 0}
            if tool_name == "datum.schematic.delete_wire": arguments = {"path": "/tmp/native-project", "wire": "wire-1"}
            if tool_name == "datum.schematic.place_junction": arguments = {"path": "/tmp/native-project", "sheet": "sheet-1", "x_nm": 100, "y_nm": 200}
            if tool_name == "datum.schematic.delete_junction": arguments = {"path": "/tmp/native-project", "junction": "junction-1"}
            if tool_name == "datum.schematic.place_noconnect": arguments = {"path": "/tmp/native-project", "sheet": "sheet-1", "symbol": "symbol-1", "pin": "pin-1", "x_nm": 100, "y_nm": 200}
            if tool_name == "datum.schematic.delete_noconnect": arguments = {"path": "/tmp/native-project", "noconnect": "noconnect-1"}
            if tool_name == "datum.schematic.place_label": arguments = {"path": "/tmp/native-project", "sheet": "sheet-1", "name": "VCC", "x_nm": 100, "y_nm": 200, "kind": "power"}
            if tool_name == "datum.schematic.rename_label": arguments = {"path": "/tmp/native-project", "label": "label-1", "name": "VEE"}
            if tool_name == "datum.schematic.delete_label": arguments = {"path": "/tmp/native-project", "label": "label-1"}
            if tool_name == "datum.schematic.place_port": arguments = {"path": "/tmp/native-project", "sheet": "sheet-1", "name": "OUT", "direction": "output", "x_nm": 100, "y_nm": 200}
            if tool_name == "datum.schematic.edit_port": arguments = {"path": "/tmp/native-project", "port": "port-1", "direction": "input"}
            if tool_name == "datum.schematic.delete_port": arguments = {"path": "/tmp/native-project", "port": "port-1"}
            if tool_name == "datum.schematic.create_bus": arguments = {"path": "/tmp/native-project", "sheet": "sheet-1", "name": "DATA", "members": ["DATA0", "DATA1"]}
            if tool_name == "datum.schematic.edit_bus_members": arguments = {"path": "/tmp/native-project", "bus": "bus-1", "members": ["DATA0", "DATA1", "DATA2"]}
            if tool_name == "datum.schematic.delete_bus": arguments = {"path": "/tmp/native-project", "bus": "bus-1"}
            if tool_name == "datum.schematic.place_bus_entry": arguments = {"path": "/tmp/native-project", "sheet": "sheet-1", "bus": "bus-1", "x_nm": 100, "y_nm": 200, "wire": "wire-1"}
            if tool_name == "datum.schematic.delete_bus_entry": arguments = {"path": "/tmp/native-project", "bus_entry": "bus-entry-1"}
            if tool_name == "datum.schematic.place_text": arguments = {"path": "/tmp/native-project", "sheet": "sheet-1", "text": "note", "x_nm": 100, "y_nm": 200, "rotation_deg": 90}
            if tool_name == "datum.schematic.edit_text": arguments = {"path": "/tmp/native-project", "text": "text-1", "value": "new note"}
            if tool_name == "datum.schematic.delete_text": arguments = {"path": "/tmp/native-project", "text": "text-1"}
            if tool_name == "datum.schematic.place_drawing_line": arguments = {"path": "/tmp/native-project", "sheet": "sheet-1", "from_x_nm": 0, "from_y_nm": 0, "to_x_nm": 100, "to_y_nm": 0}
            if tool_name == "datum.schematic.place_drawing_rect": arguments = {"path": "/tmp/native-project", "sheet": "sheet-1", "min_x_nm": 0, "min_y_nm": 0, "max_x_nm": 100, "max_y_nm": 100}
            if tool_name == "datum.schematic.place_drawing_circle": arguments = {"path": "/tmp/native-project", "sheet": "sheet-1", "center_x_nm": 50, "center_y_nm": 50, "radius_nm": 25}
            if tool_name == "datum.schematic.place_drawing_arc": arguments = {"path": "/tmp/native-project", "sheet": "sheet-1", "center_x_nm": 50, "center_y_nm": 50, "radius_nm": 25, "start_angle_mdeg": 0, "end_angle_mdeg": 90000}
            if tool_name == "datum.schematic.edit_drawing_line": arguments = {"path": "/tmp/native-project", "drawing": "drawing-1", "to_x_nm": 200, "to_y_nm": 0}
            if tool_name == "datum.schematic.edit_drawing_rect": arguments = {"path": "/tmp/native-project", "drawing": "drawing-1", "max_x_nm": 200, "max_y_nm": 100}
            if tool_name == "datum.schematic.edit_drawing_circle": arguments = {"path": "/tmp/native-project", "drawing": "drawing-1", "radius_nm": 50}
            if tool_name == "datum.schematic.edit_drawing_arc": arguments = {"path": "/tmp/native-project", "drawing": "drawing-1", "end_angle_mdeg": 180000}
            if tool_name == "datum.schematic.delete_drawing": arguments = {"path": "/tmp/native-project", "drawing": "drawing-1"}
            arguments = {"datum.schematic.place_symbol": {"path": "/tmp/native-project", "sheet": "sheet-1", "reference": "U1", "value": "OPA", "x_nm": 100, "y_nm": 200, "lib_id": "Device:R", "rotation_deg": 90, "mirrored": True}, "datum.schematic.move_symbol": {"path": "/tmp/native-project", "symbol": "symbol-1", "x_nm": 200, "y_nm": 300}, "datum.schematic.rotate_symbol": {"path": "/tmp/native-project", "symbol": "symbol-1", "rotation_deg": 180}, "datum.schematic.mirror_symbol": {"path": "/tmp/native-project", "symbol": "symbol-1"}, "datum.schematic.delete_symbol": {"path": "/tmp/native-project", "symbol": "symbol-1"}, "datum.schematic.set_symbol_reference": {"path": "/tmp/native-project", "symbol": "symbol-1", "reference": "U2"}, "datum.schematic.set_symbol_value": {"path": "/tmp/native-project", "symbol": "symbol-1", "value": "OPA1656"}, "datum.schematic.set_symbol_display_mode": {"path": "/tmp/native-project", "symbol": "symbol-1", "mode": "library-default"}, "datum.schematic.set_symbol_hidden_power_behavior": {"path": "/tmp/native-project", "symbol": "symbol-1", "behavior": "explicit"}, "datum.schematic.set_symbol_unit": {"path": "/tmp/native-project", "symbol": "symbol-1", "unit": "A"}, "datum.schematic.clear_symbol_unit": {"path": "/tmp/native-project", "symbol": "symbol-1"}, "datum.schematic.set_symbol_gate": {"path": "/tmp/native-project", "symbol": "symbol-1", "gate": "gate-1"}, "datum.schematic.clear_symbol_gate": {"path": "/tmp/native-project", "symbol": "symbol-1"}, "datum.schematic.set_symbol_entity": {"path": "/tmp/native-project", "symbol": "symbol-1", "entity": "entity-1"}, "datum.schematic.clear_symbol_entity": {"path": "/tmp/native-project", "symbol": "symbol-1"}, "datum.schematic.set_symbol_part": {"path": "/tmp/native-project", "symbol": "symbol-1", "part": "part-1"}, "datum.schematic.clear_symbol_part": {"path": "/tmp/native-project", "symbol": "symbol-1"}, "datum.schematic.set_symbol_lib_id": {"path": "/tmp/native-project", "symbol": "symbol-1", "lib_id": "Device:R"}, "datum.schematic.clear_symbol_lib_id": {"path": "/tmp/native-project", "symbol": "symbol-1"}, "datum.schematic.set_pin_override": {"path": "/tmp/native-project", "symbol": "symbol-1", "pin": "pin-1", "visible": False, "x_nm": 10}, "datum.schematic.clear_pin_override": {"path": "/tmp/native-project", "symbol": "symbol-1", "pin": "pin-1"}, "datum.schematic.add_symbol_field": {"path": "/tmp/native-project", "symbol": "symbol-1", "key": "MPN", "value": "ABC", "hidden": True, "x_nm": 10}, "datum.schematic.edit_symbol_field": {"path": "/tmp/native-project", "field": "field-1", "value": "DEF", "visible": True}, "datum.schematic.delete_symbol_field": {"path": "/tmp/native-project", "field": "field-1"}}.get(tool_name, arguments)
            if tool_name == "datum.pcb.place_component": arguments = {"path": "/tmp/native-project", "part": "part-1", "package": "pkg-1", "reference": "U1", "value": "OPA", "x_nm": 15000000, "y_nm": 12000000, "layer": 1}
            if tool_name == "datum.pcb.generate_board_components": arguments = {"path": "/tmp/native-project", "as_proposal": True, "proposal": "proposal-1", "rationale": "review handoff", "origin_x_nm": 1000, "origin_y_nm": 2000, "pitch_nm": 3000, "layer": 1}
            if tool_name == "datum.pcb.move_component": arguments = {"path": "/tmp/native-project", "component": "comp-1", "x_nm": 15000000, "y_nm": 12000000}
            if tool_name == "datum.pcb.rotate_component": arguments = {"path": "/tmp/native-project", "component": "comp-1", "rotation_deg": 90}
            if tool_name == "datum.pcb.flip_component": arguments = {"path": "/tmp/native-project", "component": "comp-1", "layer": 2}
            if tool_name == "datum.pcb.delete_component": arguments = {"path": "/tmp/native-project", "component": "comp-1"}
            if tool_name == "datum.pcb.set_component_reference": arguments = {"path": "/tmp/native-project", "component": "comp-1", "reference": "U2"}
            if tool_name == "datum.pcb.set_component_value": arguments = {"path": "/tmp/native-project", "component": "comp-1", "value": "OPA1656"}
            if tool_name == "datum.pcb.set_component_part": arguments = {"path": "/tmp/native-project", "component": "comp-1", "part": "part-2"}
            if tool_name == "datum.pcb.set_component_package": arguments = {"path": "/tmp/native-project", "component": "comp-1", "package": "pkg-2"}
            if tool_name in {"datum.pcb.lock_component", "datum.pcb.unlock_component"}: arguments = {"path": "/tmp/native-project", "component": "comp-1"}
            if tool_name == "datum.pcb.draw_track": arguments = {"path": "/tmp/native-project", "net": "net-1", "from_x_nm": 0, "from_y_nm": 0, "to_x_nm": 1000, "to_y_nm": 0, "width_nm": 150, "layer": 1}
            if tool_name == "datum.pcb.edit_track": arguments = {"path": "/tmp/native-project", "track": "track-1", "width_nm": 200}
            if tool_name == "datum.pcb.delete_track": arguments = {"path": "/tmp/native-project", "track": "track-1"}
            if tool_name == "datum.pcb.place_via": arguments = {"path": "/tmp/native-project", "net": "net-1", "x_nm": 100, "y_nm": 200, "drill_nm": 300, "diameter_nm": 600, "from_layer": 1, "to_layer": 2}
            if tool_name == "datum.pcb.edit_via": arguments = {"path": "/tmp/native-project", "via": "via-1", "diameter_nm": 700}
            if tool_name == "datum.pcb.delete_via": arguments = {"path": "/tmp/native-project", "via": "via-1"}
            if tool_name == "datum.pcb.place_zone": arguments = {"path": "/tmp/native-project", "net": "net-1", "vertices": ["0,0", "1000,0", "1000,1000"], "layer": 1, "thermal_gap_nm": 250, "thermal_spoke_width_nm": 200}
            if tool_name == "datum.pcb.edit_zone": arguments = {"path": "/tmp/native-project", "zone": "zone-1", "vertices": ["0,0", "2000,0", "2000,1000"], "layer": 2, "priority": 5, "thermal_relief": False, "thermal_gap_nm": 0, "thermal_spoke_width_nm": 0}
            if tool_name == "datum.pcb.delete_zone": arguments = {"path": "/tmp/native-project", "zone": "zone-1"}
            if tool_name == "datum.pcb.place_pad": arguments = {"path": "/tmp/native-project", "package": "pkg-1", "name": "1", "x_nm": 10, "y_nm": 20, "layer": 1, "shape": "circle", "diameter_nm": 600, "net": "net-1"}
            if tool_name == "datum.pcb.edit_pad": arguments = {"path": "/tmp/native-project", "pad": "pad-1", "diameter_nm": 700}
            if tool_name == "datum.pcb.delete_pad": arguments = {"path": "/tmp/native-project", "pad": "pad-1"}
            if tool_name == "datum.pcb.set_pad_net": arguments = {"path": "/tmp/native-project", "pad": "pad-1", "net": "net-1"}
            if tool_name == "datum.pcb.clear_pad_net": arguments = {"path": "/tmp/native-project", "pad": "pad-1"}
            if tool_name == "datum.pcb.place_net": arguments = {"path": "/tmp/native-project", "name": "VCC", "class": "class-1", "impedance_target_ohms": "50.0", "impedance_tolerance_pct": "10", "controlled_dielectric_layer": 2}
            if tool_name == "datum.pcb.edit_net": arguments = {"path": "/tmp/native-project", "net": "net-1", "name": "VEE", "impedance_tolerance_pct": "7.5", "clear_controlled_impedance": True}
            if tool_name == "datum.pcb.delete_net": arguments = {"path": "/tmp/native-project", "net": "net-1"}
            if tool_name == "datum.pcb.set_board_name": arguments = {"path": "/tmp/native-project", "name": "Main Board"}
            if tool_name == "datum.pcb.set_outline": arguments = {"path": "/tmp/native-project", "vertices": ["0,0", "1000,0", "1000,1000"]}
            if tool_name == "datum.pcb.set_stackup": arguments = {"path": "/tmp/native-project", "layers": ["1:F.Cu:copper:35000", "2:B.Cu:copper:35000"]}
            if tool_name == "datum.pcb.add_default_top_stackup": arguments = {"path": "/tmp/native-project"}
            if tool_name == "datum.pcb.place_keepout": arguments = {"path": "/tmp/native-project", "vertices": ["0,0", "1000,0", "1000,1000"], "layers": [1, 2], "kind": "route"}
            if tool_name == "datum.pcb.edit_keepout": arguments = {"path": "/tmp/native-project", "keepout": "keepout-1", "kind": "copper"}
            if tool_name == "datum.pcb.delete_keepout": arguments = {"path": "/tmp/native-project", "keepout": "keepout-1"}
            if tool_name == "datum.pcb.place_dimension": arguments = {"path": "/tmp/native-project", "from_x_nm": 0, "from_y_nm": 0, "to_x_nm": 1000, "to_y_nm": 0, "layer": 1, "text": "1mm"}
            if tool_name == "datum.pcb.edit_dimension": arguments = {"path": "/tmp/native-project", "dimension": "dimension-1", "clear_text": True}
            if tool_name == "datum.pcb.delete_dimension": arguments = {"path": "/tmp/native-project", "dimension": "dimension-1"}
            if tool_name == "datum.pcb.place_text": arguments = {"path": "/tmp/native-project", "text": "REF**", "x_nm": 10, "y_nm": 20, "layer": 1, "mirrored": True}
            if tool_name == "datum.pcb.edit_text": arguments = {"path": "/tmp/native-project", "text": "text-1", "value": "REV A", "mirrored": False}
            if tool_name == "datum.pcb.delete_text": arguments = {"path": "/tmp/native-project", "text": "text-1"}
            if tool_name == "datum.pcb.place_net_class": arguments = {"path": "/tmp/native-project", "name": "Signal", "clearance_nm": 150, "track_width_nm": 150, "via_drill_nm": 300, "via_diameter_nm": 600}
            if tool_name == "datum.pcb.edit_net_class": arguments = {"path": "/tmp/native-project", "net_class": "class-1", "track_width_nm": 200}
            if tool_name == "datum.pcb.delete_net_class": arguments = {"path": "/tmp/native-project", "net_class": "class-1"}
            host.handle_message({"jsonrpc": "2.0", "id": 302, "method": "tools/call", "params": {"name": tool_name, "arguments": arguments}})
            self.assertEqual(daemon.calls[-1][0], expected_call)
