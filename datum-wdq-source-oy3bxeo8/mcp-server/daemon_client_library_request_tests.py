#!/usr/bin/env python3
"""EngineDaemonClient native pool-library CLI bridge tests."""

from __future__ import annotations

import subprocess
import unittest
from unittest.mock import ANY, patch

from server_runtime import EngineDaemonClient


class TestDaemonClientLibraryRequests(unittest.TestCase):
    @patch("server_runtime.subprocess.run")
    def test_lists_pool_library_objects_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout='{"contract":"native_project_library_objects_query_v1","object_count":1}',
            stderr="",
        )
        response = EngineDaemonClient().get_pool_library_objects(
            "/tmp/native-project",
            "pool",
            "symbols",
            "symbol-test",
            True,
        )
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "project",
                "query",
                "/tmp/native-project",
                "pool-library-objects",
                "--pool",
                "pool",
                "--kind",
                "symbols",
                "--object",
                "symbol-test",
                "--include-payload",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["object_count"], 1)

    @patch("server_runtime.subprocess.run")
    def test_shows_pool_library_object_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout='{"contract":"native_project_library_objects_query_v1","object_count":1}',
            stderr="",
        )
        response = EngineDaemonClient().show_pool_library_object(
            "/tmp/native-project",
            "symbol-test",
            None,
            "symbols",
        )
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "project",
                "query",
                "/tmp/native-project",
                "pool-library-objects",
                "--object",
                "symbol-test",
                "--include-payload",
                "--kind",
                "symbols",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["object_count"], 1)

    @patch("server_runtime.subprocess.run")
    def test_gets_pool_models_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout='{"contract":"native_project_pool_models_query_v1","model_count":1}',
            stderr="",
        )
        response = EngineDaemonClient().get_pool_model_blobs(
            "/tmp/native-project",
            "pool",
            "spice",
            "abc123",
        )
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "project",
                "query",
                "/tmp/native-project",
                "pool-models",
                "--pool",
                "pool",
                "--role",
                "spice",
                "--sha256",
                "abc123",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["model_count"], 1)

    @patch("server_runtime.subprocess.run")
    def test_gcs_pool_models_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout='{"contract":"native_project_pool_model_gc_v1","deleted_count":1}',
            stderr="",
        )
        response = EngineDaemonClient().gc_pool_model_blobs(
            "/tmp/native-project",
            "pool",
            "spice",
            "abc123",
            True,
        )
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "project",
                "gc-pool-models",
                "/tmp/native-project",
                "--pool",
                "pool",
                "--role",
                "spice",
                "--sha256",
                "abc123",
                "--apply",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["deleted_count"], 1)

    @patch("server_runtime.subprocess.run")
    def test_creates_pool_library_object_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout='{"action":"create_pool_library_object","object_uuid":"symbol-test"}',
            stderr="",
        )
        response = EngineDaemonClient().create_pool_library_object(
            "/tmp/native-project",
            "pool",
            "symbols",
            "symbol-test",
            "/tmp/symbol.json",
        )
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "project",
                "create-pool-library-object",
                "/tmp/native-project",
                "--pool",
                "pool",
                "--kind",
                "symbols",
                "--object",
                "symbol-test",
                "--from-json",
                "/tmp/symbol.json",
            ],
            capture_output=True,
            text=True,
            check=False,
            env=ANY,
        )
        self.assertEqual(response.result["object_uuid"], "symbol-test")

    @patch("server_runtime.subprocess.run")
    def test_creates_pool_unit_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout='{"action":"create_unit","object_uuid":"unit-test"}',
            stderr="",
        )
        response = EngineDaemonClient().create_pool_unit(
            "/tmp/native-project",
            "pool",
            "unit-test",
            "OpAmpUnit",
            "Datum",
        )
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "project",
                "create-pool-unit",
                "/tmp/native-project",
                "--pool",
                "pool",
                "--unit",
                "unit-test",
                "--name",
                "OpAmpUnit",
                "--manufacturer",
                "Datum",
            ],
            capture_output=True,
            text=True,
            check=False,
            env=ANY,
        )
        self.assertEqual(response.result["object_uuid"], "unit-test")

    @patch("server_runtime.subprocess.run")
    def test_creates_pool_symbol_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout='{"action":"create_symbol","object_uuid":"symbol-test"}',
            stderr="",
        )
        response = EngineDaemonClient().create_pool_symbol(
            "/tmp/native-project",
            "pool",
            "symbol-test",
            "unit-test",
            "OpAmpSymbol",
        )
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "project",
                "create-pool-symbol",
                "/tmp/native-project",
                "--pool",
                "pool",
                "--symbol",
                "symbol-test",
                "--unit",
                "unit-test",
                "--name",
                "OpAmpSymbol",
            ],
            capture_output=True,
            text=True,
            check=False,
            env=ANY,
        )
        self.assertEqual(response.result["object_uuid"], "symbol-test")

    @patch("server_runtime.subprocess.run")
    def test_creates_pool_entity_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout='{"action":"create_entity","object_uuid":"entity-test"}',
            stderr="",
        )
        response = EngineDaemonClient().create_pool_entity(
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
        )
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "project",
                "create-pool-entity",
                "/tmp/native-project",
                "--pool",
                "pool",
                "--entity",
                "entity-test",
                "--gate",
                "gate-test",
                "--unit",
                "unit-test",
                "--symbol",
                "symbol-test",
                "--name",
                "DualOpAmp",
                "--prefix",
                "U",
                "--manufacturer",
                "Datum",
                "--gate-name",
                "A",
            ],
            capture_output=True,
            text=True,
            check=False,
            env=ANY,
        )
        self.assertEqual(response.result["object_uuid"], "entity-test")

    @patch("server_runtime.subprocess.run")
    def test_creates_pool_padstack_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout='{"action":"create_padstack","object_uuid":"padstack-test"}',
            stderr="",
        )
        response = EngineDaemonClient().create_pool_padstack(
            "/tmp/native-project",
            "pool",
            "padstack-test",
            "RoundViaPad",
            "circle",
            1200000,
            None,
            None,
            600000,
        )
        run_mock.assert_called_once_with(
            [
                "datum-eda", "--format", "json", "project", "create-pool-padstack",
                "/tmp/native-project", "--pool", "pool", "--padstack", "padstack-test",
                "--name", "RoundViaPad", "--aperture", "circle", "--diameter-nm", "1200000",
                "--drill-nm", "600000",
            ],
            capture_output=True,
            text=True,
            check=False,
            env=ANY,
        )
        self.assertEqual(response.result["object_uuid"], "padstack-test")

    @patch("server_runtime.subprocess.run")
    def test_creates_pool_package_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout='{"action":"create_package","object_uuid":"package-test"}',
            stderr="",
        )
        response = EngineDaemonClient().create_pool_package(
            "/tmp/native-project", "pool", "package-test", "SOT23", "pad-test",
            "padstack-test", "1", 1000, 2000, 1,
        )
        run_mock.assert_called_once_with(
            [
                "datum-eda", "--format", "json", "project", "create-pool-package",
                "/tmp/native-project", "--pool", "pool", "--package", "package-test",
                "--name", "SOT23", "--pad", "pad-test", "--padstack", "padstack-test",
                "--pad-name", "1", "--x-nm", "1000", "--y-nm", "2000", "--layer", "1",
            ],
            capture_output=True,
            text=True,
            check=False,
            env=ANY,
        )
        self.assertEqual(response.result["object_uuid"], "package-test")

    @patch("server_runtime.subprocess.run")
    def test_creates_body_only_pool_package_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout='{"action":"create_package","object_uuid":"package-test"}',
            stderr="",
        )
        response = EngineDaemonClient().create_pool_package(
            "/tmp/native-project", "pool", "package-test", "SOT23",
        )
        run_mock.assert_called_once_with(
            [
                "datum-eda", "--format", "json", "project", "create-pool-package",
                "/tmp/native-project", "--pool", "pool", "--package", "package-test",
                "--name", "SOT23",
            ],
            capture_output=True,
            text=True,
            check=False,
            env=ANY,
        )
        self.assertEqual(response.result["object_uuid"], "package-test")

    @patch("server_runtime.subprocess.run")
    def test_creates_pool_part_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(args=[], returncode=0, stdout='{"action":"create_part","object_uuid":"part-test"}', stderr="")
        response = EngineDaemonClient().create_pool_part("/tmp/native-project", "pool", "part-test", "entity-test", "package-test", "OPA1656ID", "Texas Instruments", "OPA1656", "", "", "Active")
        run_mock.assert_called_once_with(["datum-eda", "--format", "json", "project", "create-pool-part", "/tmp/native-project", "--pool", "pool", "--part", "part-test", "--entity", "entity-test", "--package", "package-test", "--mpn", "OPA1656ID", "--manufacturer", "Texas Instruments", "--value", "OPA1656", "--description", "", "--datasheet", "", "--lifecycle", "Active"], capture_output=True, text=True, check=False, env=ANY)
        self.assertEqual(response.result["object_uuid"], "part-test")

    @patch("server_runtime.subprocess.run")
    def test_generates_ipc7351b_soic_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(args=[], returncode=0, stdout='{"action":"generate_ipc7351b_soic","object_uuid":"footprint-test"}', stderr="")
        response = EngineDaemonClient().generate_ipc7351b_soic(
            "/tmp/native-project",
            "footprint-test",
            "package-test",
            "padstack-test",
            ["pad-1", "pad-2", "pad-3", "pad-4"],
            "SOIC-4_TEST",
            4,
            1270000,
            4900000,
            3900000,
            6000000,
            600000,
            400000,
            pool="pool",
            density="nominal",
            mask_expansion_nm=50000,
            paste_reduction_nm=50000,
            name="SOIC-4_TEST",
        )
        run_mock.assert_called_once_with(
            [
                "datum-eda", "--format", "json", "project", "generate-ipc7351b-soic",
                "/tmp/native-project", "--pool", "pool", "--footprint", "footprint-test",
                "--package", "package-test", "--padstack", "padstack-test",
                "--pad", "pad-1", "--pad", "pad-2", "--pad", "pad-3", "--pad", "pad-4",
                "--package-code", "SOIC-4_TEST", "--pin-count", "4", "--pitch-nm", "1270000",
                "--body-length-nm", "4900000", "--body-width-nm", "3900000",
                "--lead-span-nm", "6000000", "--terminal-length-nm", "600000",
                "--terminal-width-nm", "400000", "--density", "nominal",
                "--mask-expansion-nm", "50000", "--paste-reduction-nm", "50000",
                "--name", "SOIC-4_TEST",
            ],
            capture_output=True,
            text=True,
            check=False,
            env=ANY,
        )
        self.assertEqual(response.result["object_uuid"], "footprint-test")

    @patch("server_runtime.subprocess.run")
    def test_sets_pool_part_metadata_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(args=[], returncode=0, stdout='{"action":"set_part_metadata","object_uuid":"part-test"}', stderr="")
        response = EngineDaemonClient().set_pool_part_metadata("/tmp/native-project", "pool", "part-test", mpn="OPA1656ID", manufacturer_jep106=123, lifecycle="Active")
        run_mock.assert_called_once_with(["datum-eda", "--format", "json", "project", "set-pool-part-metadata", "/tmp/native-project", "--pool", "pool", "--part", "part-test", "--mpn", "OPA1656ID", "--manufacturer-jep106", "123", "--lifecycle", "Active"], capture_output=True, text=True, check=False, env=ANY)
        self.assertEqual(response.result["object_uuid"], "part-test")

    @patch("server_runtime.subprocess.run")
    def test_sets_pool_part_parametric_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(args=[], returncode=0, stdout='{"action":"set_part_parametric","object_uuid":"part-test"}', stderr="")
        response = EngineDaemonClient().set_pool_part_parametric("/tmp/native-project", "pool", "part-test", "replace", {"gbw": "53MHz", "slew_rate": "24V/us"})
        run_mock.assert_called_once_with(["datum-eda", "--format", "json", "project", "set-pool-part-parametric", "/tmp/native-project", "--pool", "pool", "--part", "part-test", "--mode", "replace", "--param", "gbw=53MHz", "--param", "slew_rate=24V/us"], capture_output=True, text=True, check=False, env=ANY)
        self.assertEqual(response.result["object_uuid"], "part-test")

    @patch("server_runtime.subprocess.run")
    def test_sets_pool_part_orderable_mpns_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(args=[], returncode=0, stdout='{"action":"set_part_orderable_mpns","object_uuid":"part-test"}', stderr="")
        response = EngineDaemonClient().set_pool_part_orderable_mpns("/tmp/native-project", "pool", "part-test", "replace", ["OPA1656ID", "OPA1656IDR"])
        run_mock.assert_called_once_with(["datum-eda", "--format", "json", "project", "set-pool-part-orderable-mpns", "/tmp/native-project", "--pool", "pool", "--part", "part-test", "--mode", "replace", "--mpn", "OPA1656ID", "--mpn", "OPA1656IDR"], capture_output=True, text=True, check=False, env=ANY)
        self.assertEqual(response.result["object_uuid"], "part-test")

    @patch("server_runtime.subprocess.run")
    def test_sets_pool_part_tags_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(args=[], returncode=0, stdout='{"action":"set_part_tags","object_uuid":"part-test"}', stderr="")
        response = EngineDaemonClient().set_pool_part_tags("/tmp/native-project", "pool", "part-test", "replace", ["audio", "opamp"])
        run_mock.assert_called_once_with(["datum-eda", "--format", "json", "project", "set-pool-part-tags", "/tmp/native-project", "--pool", "pool", "--part", "part-test", "--mode", "replace", "--tag", "audio", "--tag", "opamp"], capture_output=True, text=True, check=False, env=ANY)
        self.assertEqual(response.result["object_uuid"], "part-test")

    @patch("server_runtime.subprocess.run")
    def test_sets_pool_part_packaging_options_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(args=[], returncode=0, stdout='{"action":"set_part_packaging_options","object_uuid":"part-test"}', stderr="")
        response = EngineDaemonClient().set_pool_part_packaging_options("/tmp/native-project", "pool", "part-test", "replace", ["kind=tape_reel;qty=2500", "{\"kind\":\"tray\",\"quantity\":90}"])
        run_mock.assert_called_once_with(["datum-eda", "--format", "json", "project", "set-pool-part-packaging-options", "/tmp/native-project", "--pool", "pool", "--part", "part-test", "--mode", "replace", "--option", "kind=tape_reel;qty=2500", "--option", "{\"kind\":\"tray\",\"quantity\":90}"], capture_output=True, text=True, check=False, env=ANY)
        self.assertEqual(response.result["object_uuid"], "part-test")

    @patch("server_runtime.subprocess.run")
    def test_sets_pool_part_supply_chain_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(args=[], returncode=0, stdout='{"action":"set_part_supply_chain","object_uuid":"part-test"}', stderr="")
        response = EngineDaemonClient().set_pool_part_supply_chain(
            "/tmp/native-project",
            "pool",
            "part-test",
            clear=True,
            checked_at="2026-06-21T12:34:56Z",
            offers=["{\"supplier\":\"DigiKey\",\"sku\":\"296-OPA1656ID-ND\"}", "{\"supplier\":\"Mouser\",\"sku\":\"595-OPA1656ID\"}"],
        )
        run_mock.assert_called_once_with(["datum-eda", "--format", "json", "project", "set-pool-part-supply-chain", "/tmp/native-project", "--pool", "pool", "--part", "part-test", "--clear", "--checked-at", "2026-06-21T12:34:56Z", "--offer", "{\"supplier\":\"DigiKey\",\"sku\":\"296-OPA1656ID-ND\"}", "--offer", "{\"supplier\":\"Mouser\",\"sku\":\"595-OPA1656ID\"}"], capture_output=True, text=True, check=False, env=ANY)
        self.assertEqual(response.result["object_uuid"], "part-test")

    @patch("server_runtime.subprocess.run")
    def test_sets_pool_part_behavioural_models_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(args=[], returncode=0, stdout='{"action":"set_part_behavioural_models","object_uuid":"part-test"}', stderr="")
        response = EngineDaemonClient().set_pool_part_behavioural_models("/tmp/native-project", "pool", "part-test", "replace", ["{\"kind\":\"spice\",\"path\":\"models/opamp.lib\"}", "not-json"])
        run_mock.assert_called_once_with(["datum-eda", "--format", "json", "project", "set-pool-part-behavioural-models", "/tmp/native-project", "--pool", "pool", "--part", "part-test", "--mode", "replace", "--model", "{\"kind\":\"spice\",\"path\":\"models/opamp.lib\"}", "--model", "not-json"], capture_output=True, text=True, check=False, env=ANY)
        self.assertEqual(response.result["object_uuid"], "part-test")

    @patch("server_runtime.subprocess.run")
    def test_attaches_pool_part_model_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(args=[], returncode=0, stdout='{"action":"attach_part_model","object_uuid":"part-test"}', stderr="")
        response = EngineDaemonClient().attach_pool_part_model(
            "/tmp/native-project",
            "pool",
            "part-test",
            "models/opamp.lib",
            "simulation",
            dialect="spice",
            model_names=["OPA1656", "OPA1656_ALT"],
            encrypted=True,
            encryption_scheme="{\"kind\":\"aes\"}",
            vendor="Texas Instruments",
            fetched_at="2026-06-21T12:34:56Z",
            format_metadata_json="{\"temperature\":\"25C\"}",
        )
        run_mock.assert_called_once_with(["datum-eda", "--format", "json", "project", "attach-pool-part-model", "/tmp/native-project", "--pool", "pool", "--part", "part-test", "--source", "models/opamp.lib", "--role", "simulation", "--dialect", "spice", "--model-name", "OPA1656", "--model-name", "OPA1656_ALT", "--encrypted", "--encryption-scheme", "{\"kind\":\"aes\"}", "--vendor", "Texas Instruments", "--fetched-at", "2026-06-21T12:34:56Z", "--format-metadata-json", "{\"temperature\":\"25C\"}"], capture_output=True, text=True, check=False, env=ANY)
        self.assertEqual(response.result["object_uuid"], "part-test")

    @patch("server_runtime.subprocess.run")
    def test_detaches_pool_part_model_by_attachment_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(args=[], returncode=0, stdout='{"action":"detach_part_model","object_uuid":"part-test"}', stderr="")
        response = EngineDaemonClient().detach_pool_part_model("/tmp/native-project", "pool", "part-test", attachment="attachment-test")
        run_mock.assert_called_once_with(["datum-eda", "--format", "json", "project", "detach-pool-part-model", "/tmp/native-project", "--pool", "pool", "--part", "part-test", "--attachment", "attachment-test"], capture_output=True, text=True, check=False, env=ANY)
        self.assertEqual(response.result["object_uuid"], "part-test")

    @patch("server_runtime.subprocess.run")
    def test_detaches_pool_part_model_by_model_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(args=[], returncode=0, stdout='{"action":"detach_part_model","object_uuid":"part-test"}', stderr="")
        response = EngineDaemonClient().detach_pool_part_model("/tmp/native-project", "pool", "part-test", model="model-test")
        run_mock.assert_called_once_with(["datum-eda", "--format", "json", "project", "detach-pool-part-model", "/tmp/native-project", "--pool", "pool", "--part", "part-test", "--model", "model-test"], capture_output=True, text=True, check=False, env=ANY)
        self.assertEqual(response.result["object_uuid"], "part-test")

    @patch("server_runtime.subprocess.run")
    def test_sets_pool_part_thermal_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(args=[], returncode=0, stdout='{"action":"set_part_thermal","object_uuid":"part-test"}', stderr="")
        response = EngineDaemonClient().set_pool_part_thermal(
            "/tmp/native-project",
            "pool",
            "part-test",
            theta_ja_c_per_w=42.5,
            theta_jc_top_c_per_w="8.2",
            theta_jc_bot_c_per_w=None,
            theta_jb_c_per_w="12",
            max_junction_c=150,
            thermal_reference="JEDEC JESD51",
            clear=True,
        )
        run_mock.assert_called_once_with(["datum-eda", "--format", "json", "project", "set-pool-part-thermal", "/tmp/native-project", "--pool", "pool", "--part", "part-test", "--theta-ja-c-per-w", "42.5", "--theta-jc-top-c-per-w", "8.2", "--theta-jb-c-per-w", "12", "--max-junction-c", "150", "--thermal-reference", "JEDEC JESD51", "--clear"], capture_output=True, text=True, check=False, env=ANY)
        self.assertEqual(response.result["object_uuid"], "part-test")

    @patch("server_runtime.subprocess.run")
    def test_sets_pool_part_pad_map_entry_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(args=[], returncode=0, stdout='{"action":"set_part_pad_map_entry","object_uuid":"part-test"}', stderr="")
        response = EngineDaemonClient().set_pool_part_pad_map_entry("/tmp/native-project", "pool", "part-test", "pad-test", "gate-test", "pin-test")
        run_mock.assert_called_once_with(["datum-eda", "--format", "json", "project", "set-pool-part-pad-map-entry", "/tmp/native-project", "--pool", "pool", "--part", "part-test", "--pad", "pad-test", "--gate", "gate-test", "--pin", "pin-test"], capture_output=True, text=True, check=False, env=ANY)
        self.assertEqual(response.result["object_uuid"], "part-test")

    @patch("server_runtime.subprocess.run")
    def test_sets_pool_part_pad_map_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(args=[], returncode=0, stdout='{"action":"set_part_pad_map","object_uuid":"part-test"}', stderr="")
        response = EngineDaemonClient().set_pool_part_pad_map("/tmp/native-project", "pool", "part-test", "replace", [{"pad": "pad-test", "gate": "gate-test", "pin": "pin-test"}])
        run_mock.assert_called_once_with(["datum-eda", "--format", "json", "project", "set-pool-part-pad-map", "/tmp/native-project", "--pool", "pool", "--part", "part-test", "--mode", "replace", "--entry", "pad-test:gate-test:pin-test"], capture_output=True, text=True, check=False, env=ANY)
        self.assertEqual(response.result["object_uuid"], "part-test")

    @patch("server_runtime.subprocess.run")
    def test_creates_pool_pin_pad_map_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(args=[], returncode=0, stdout='{"action":"create_pin_pad_map","object_uuid":"map-test"}', stderr="")
        response = EngineDaemonClient().create_pool_pin_pad_map("/tmp/native-project", "pool", "map-test", "part-test", [{"pad": "pad-test", "gate": "gate-test", "pin": "pin-test"}], "footprint-test", True)
        args, kwargs = run_mock.call_args
        self.assertEqual(args[0], ["datum-eda", "--format", "json", "project", "create-pool-pin-pad-map", "/tmp/native-project", "--pool", "pool", "--map", "map-test", "--part", "part-test", "--footprint", "footprint-test", "--set-default", "--entry", "pad-test:gate-test:pin-test"])
        self.assertEqual(kwargs["env"]["DATUM_TOOL_SURFACE"], "mcp")
        self.assertEqual(response.result["object_uuid"], "map-test")

    @patch("server_runtime.subprocess.run")
    def test_sets_pool_pin_pad_map_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(args=[], returncode=0, stdout='{"action":"set_pin_pad_map","object_uuid":"map-test"}', stderr="")
        response = EngineDaemonClient().set_pool_pin_pad_map("/tmp/native-project", "pool", "map-test", "replace", [{"pad": "pad-test", "gate": "gate-test", "pin": "pin-test"}])
        args, kwargs = run_mock.call_args
        self.assertEqual(args[0], ["datum-eda", "--format", "json", "project", "set-pool-pin-pad-map", "/tmp/native-project", "--pool", "pool", "--map", "map-test", "--mode", "replace", "--entry", "pad-test:gate-test:pin-test"])
        self.assertEqual(kwargs["env"]["DATUM_COMMIT_SOURCE"], "tool")
        self.assertEqual(response.result["object_uuid"], "map-test")

    @patch("server_runtime.subprocess.run")
    def test_sets_pool_unit_pin_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout='{"action":"set_unit_pin","object_uuid":"unit-test"}',
            stderr="",
        )
        response = EngineDaemonClient().set_pool_unit_pin(
            "/tmp/native-project", "pool", "unit-test", "pin-test", "OUT", "Output", 1,
        )
        run_mock.assert_called_once_with(
            [
                "datum-eda", "--format", "json", "project", "set-pool-unit-pin",
                "/tmp/native-project", "--pool", "pool", "--unit", "unit-test",
                "--pin", "pin-test", "--name", "OUT", "--direction", "Output",
                "--swap-group", "1",
            ],
            capture_output=True,
            text=True,
            check=False,
            env=ANY,
        )
        self.assertEqual(response.result["object_uuid"], "unit-test")

    @patch("server_runtime.subprocess.run")
    def test_sets_pool_package_pad_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout='{"action":"set_package_pad","object_uuid":"package-test"}',
            stderr="",
        )
        response = EngineDaemonClient().set_pool_package_pad(
            "/tmp/native-project", "pool", "package-test", "pad-test", "padstack-test", "2", 1000, 2000, 1,
        )
        run_mock.assert_called_once_with(
            [
                "datum-eda", "--format", "json", "project", "set-pool-package-pad",
                "/tmp/native-project", "--pool", "pool", "--package", "package-test",
                "--pad", "pad-test", "--padstack", "padstack-test", "--pad-name", "2",
                "--x-nm", "1000", "--y-nm", "2000", "--layer", "1",
            ],
            capture_output=True,
            text=True,
            check=False,
            env=ANY,
        )
        self.assertEqual(response.result["object_uuid"], "package-test")

    @patch("server_runtime.subprocess.run")
    def test_sets_pool_package_courtyard_rect_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout='{"action":"set_package_courtyard_rect","object_uuid":"package-test"}',
            stderr="",
        )
        response = EngineDaemonClient().set_pool_package_courtyard_rect(
            "/tmp/native-project", "pool", "package-test", 1000, 2000, 3000, 4000,
        )
        run_mock.assert_called_once_with(
            [
                "datum-eda", "--format", "json", "project", "set-pool-package-courtyard-rect",
                "/tmp/native-project", "--pool", "pool", "--package", "package-test",
                "--min-x-nm", "1000", "--min-y-nm", "2000",
                "--max-x-nm", "3000", "--max-y-nm", "4000",
            ],
            capture_output=True,
            text=True,
            check=False,
            env=ANY,
        )
        self.assertEqual(response.result["object_uuid"], "package-test")

    @patch("server_runtime.subprocess.run")
    def test_sets_pool_package_courtyard_polygon_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(args=[], returncode=0, stdout='{"action":"set_package_courtyard_polygon","object_uuid":"package-test"}', stderr="")
        response = EngineDaemonClient().set_pool_package_courtyard_polygon("/tmp/native-project", "pool", "package-test", "0,0;1000,0;1000,1000")
        run_mock.assert_called_once_with(["datum-eda", "--format", "json", "project", "set-pool-package-courtyard-polygon", "/tmp/native-project", "--pool", "pool", "--package", "package-test", "--vertices", "0,0;1000,0;1000,1000"], capture_output=True, text=True, check=False, env=ANY)
        self.assertEqual(response.result["object_uuid"], "package-test")

    @patch("server_runtime.subprocess.run")
    def test_adds_pool_package_silkscreen_line_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout='{"action":"add_package_silkscreen_line","object_kind":"footprints","object_uuid":"footprint-test"}',
            stderr="",
        )
        response = EngineDaemonClient().add_pool_package_silkscreen_line(
            "/tmp/native-project", "pool", "package-test", 1000, 2000, 3000, 4000, 150000,
        )
        run_mock.assert_called_once_with(
            [
                "datum-eda", "--format", "json", "project", "add-pool-package-silkscreen-line",
                "/tmp/native-project", "--pool", "pool", "--package", "package-test",
                "--from-x-nm", "1000", "--from-y-nm", "2000",
                "--to-x-nm", "3000", "--to-y-nm", "4000", "--width-nm", "150000",
            ],
            capture_output=True,
            text=True,
            check=False,
            env=ANY,
        )
        self.assertEqual(response.result["object_kind"], "footprints")
        self.assertEqual(response.result["object_uuid"], "footprint-test")

    @patch("server_runtime.subprocess.run")
    def test_adds_pool_footprint_silkscreen_shapes_via_cli(self, run_mock) -> None:
        client = EngineDaemonClient()

        run_mock.return_value = subprocess.CompletedProcess(args=[], returncode=0, stdout='{"action":"add_footprint_silkscreen_rect","object_uuid":"footprint-test"}', stderr="")
        response = client.add_pool_footprint_silkscreen_rect("/tmp/native-project", "pool", "footprint-test", 1000, 2000, 3000, 4000, 150000)
        self.assertEqual(run_mock.call_args.args[0], ["datum-eda", "--format", "json", "project", "add-pool-footprint-silkscreen-rect", "/tmp/native-project", "--pool", "pool", "--footprint", "footprint-test", "--min-x-nm", "1000", "--min-y-nm", "2000", "--max-x-nm", "3000", "--max-y-nm", "4000", "--width-nm", "150000"])
        self.assertEqual(response.result["object_uuid"], "footprint-test")

        run_mock.reset_mock()
        run_mock.return_value = subprocess.CompletedProcess(args=[], returncode=0, stdout='{"action":"add_footprint_silkscreen_circle","object_uuid":"footprint-test"}', stderr="")
        response = client.add_pool_footprint_silkscreen_circle("/tmp/native-project", "pool", "footprint-test", 5000, 6000, 7000, 150000)
        self.assertEqual(run_mock.call_args.args[0], ["datum-eda", "--format", "json", "project", "add-pool-footprint-silkscreen-circle", "/tmp/native-project", "--pool", "pool", "--footprint", "footprint-test", "--center-x-nm", "5000", "--center-y-nm", "6000", "--radius-nm", "7000", "--width-nm", "150000"])
        self.assertEqual(response.result["object_uuid"], "footprint-test")

        run_mock.reset_mock()
        run_mock.return_value = subprocess.CompletedProcess(args=[], returncode=0, stdout='{"action":"add_footprint_silkscreen_polygon","object_uuid":"footprint-test"}', stderr="")
        response = client.add_pool_footprint_silkscreen_polygon("/tmp/native-project", "pool", "footprint-test", "0,0;1000,0;1000,1000", True, 150000)
        self.assertEqual(run_mock.call_args.args[0], ["datum-eda", "--format", "json", "project", "add-pool-footprint-silkscreen-polygon", "/tmp/native-project", "--pool", "pool", "--footprint", "footprint-test", "--vertices", "0,0;1000,0;1000,1000", "--closed", "true", "--width-nm", "150000"])
        self.assertEqual(response.result["object_uuid"], "footprint-test")

    @patch("server_runtime.subprocess.run")
    def test_adds_pool_symbol_line_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout='{"action":"add_symbol_line","object_uuid":"symbol-test"}',
            stderr="",
        )
        response = EngineDaemonClient().add_pool_symbol_line(
            "/tmp/native-project", "pool", "symbol-test", 0, 0, 1000, 0, 100,
        )
        run_mock.assert_called_once_with(
            [
                "datum-eda", "--format", "json", "project", "add-pool-symbol-line",
                "/tmp/native-project", "--pool", "pool", "--symbol", "symbol-test",
                "--from-x-nm", "0", "--from-y-nm", "0",
                "--to-x-nm", "1000", "--to-y-nm", "0", "--width-nm", "100",
            ],
            capture_output=True,
            text=True,
            check=False,
            env=ANY,
        )
        self.assertEqual(response.result["object_uuid"], "symbol-test")

    @patch("server_runtime.subprocess.run")
    def test_adds_pool_symbol_rect_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(args=[], returncode=0, stdout='{"action":"add_symbol_rect","object_uuid":"symbol-test"}', stderr="")
        response = EngineDaemonClient().add_pool_symbol_rect("/tmp/native-project", "pool", "symbol-test", 0, 0, 1000, 2000, 100)
        run_mock.assert_called_once_with(["datum-eda", "--format", "json", "project", "add-pool-symbol-rect", "/tmp/native-project", "--pool", "pool", "--symbol", "symbol-test", "--min-x-nm", "0", "--min-y-nm", "0", "--max-x-nm", "1000", "--max-y-nm", "2000", "--width-nm", "100"], capture_output=True, text=True, check=False, env=ANY)
        self.assertEqual(response.result["object_uuid"], "symbol-test")

    @patch("server_runtime.subprocess.run")
    def test_adds_pool_symbol_circle_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(args=[], returncode=0, stdout='{"action":"add_symbol_circle","object_uuid":"symbol-test"}', stderr="")
        response = EngineDaemonClient().add_pool_symbol_circle("/tmp/native-project", "pool", "symbol-test", 500, 600, 250, 100)
        run_mock.assert_called_once_with(["datum-eda", "--format", "json", "project", "add-pool-symbol-circle", "/tmp/native-project", "--pool", "pool", "--symbol", "symbol-test", "--center-x-nm", "500", "--center-y-nm", "600", "--radius-nm", "250", "--width-nm", "100"], capture_output=True, text=True, check=False, env=ANY)
        self.assertEqual(response.result["object_uuid"], "symbol-test")

    @patch("server_runtime.subprocess.run")
    def test_adds_pool_symbol_polygon_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(args=[], returncode=0, stdout='{"action":"add_symbol_polygon","object_uuid":"symbol-test"}', stderr="")
        response = EngineDaemonClient().add_pool_symbol_polygon("/tmp/native-project", "pool", "symbol-test", "0,0;1000,0;1000,1000", True, 150000)
        run_mock.assert_called_once_with(["datum-eda", "--format", "json", "project", "add-pool-symbol-polygon", "/tmp/native-project", "--pool", "pool", "--symbol", "symbol-test", "--vertices", "0,0;1000,0;1000,1000", "--closed", "true", "--width-nm", "150000"], capture_output=True, text=True, check=False, env=ANY)
        self.assertEqual(response.result["object_uuid"], "symbol-test")

    @patch("server_runtime.subprocess.run")
    def test_adds_pool_symbol_text_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(args=[], returncode=0, stdout='{"action":"add_symbol_text","object_uuid":"symbol-test"}', stderr="")
        response = EngineDaemonClient().add_pool_symbol_text("/tmp/native-project", "pool", "symbol-test", "REF**", 1000, 2000, 90)
        run_mock.assert_called_once_with(["datum-eda", "--format", "json", "project", "add-pool-symbol-text", "/tmp/native-project", "--pool", "pool", "--symbol", "symbol-test", "--text", "REF**", "--x-nm", "1000", "--y-nm", "2000", "--rotation", "90"], capture_output=True, text=True, check=False, env=ANY)
        self.assertEqual(response.result["object_uuid"], "symbol-test")

    @patch("server_runtime.subprocess.run")
    def test_sets_pool_symbol_pin_anchor_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(args=[], returncode=0, stdout='{"action":"set_symbol_pin_anchor","object_uuid":"symbol-test"}', stderr="")
        response = EngineDaemonClient().set_pool_symbol_pin_anchor("/tmp/native-project", "pool", "symbol-test", "pin-test", 100, 200)
        run_mock.assert_called_once_with(["datum-eda", "--format", "json", "project", "set-pool-symbol-pin-anchor", "/tmp/native-project", "--pool", "pool", "--symbol", "symbol-test", "--pin", "pin-test", "--x-nm", "100", "--y-nm", "200"], capture_output=True, text=True, check=False, env=ANY)
        self.assertEqual(response.result["object_uuid"], "symbol-test")

    @patch("server_runtime.subprocess.run")
    def test_adds_pool_package_silkscreen_rect_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[], returncode=0,
            stdout='{"action":"add_package_silkscreen_rect","object_kind":"footprints","object_uuid":"footprint-test"}', stderr="",
        )
        response = EngineDaemonClient().add_pool_package_silkscreen_rect(
            "/tmp/native-project", "pool", "package-test", 1000, 2000, 3000, 4000, 150000,
        )
        run_mock.assert_called_once_with(
            [
                "datum-eda", "--format", "json", "project", "add-pool-package-silkscreen-rect",
                "/tmp/native-project", "--pool", "pool", "--package", "package-test",
                "--min-x-nm", "1000", "--min-y-nm", "2000",
                "--max-x-nm", "3000", "--max-y-nm", "4000", "--width-nm", "150000",
            ],
            capture_output=True, text=True, check=False,
            env=ANY,
        )
        self.assertEqual(response.result["object_kind"], "footprints")
        self.assertEqual(response.result["object_uuid"], "footprint-test")

    @patch("server_runtime.subprocess.run")
    def test_adds_pool_package_silkscreen_polygon_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(args=[], returncode=0, stdout='{"action":"add_package_silkscreen_polygon","object_kind":"footprints","object_uuid":"footprint-test"}', stderr="")
        response = EngineDaemonClient().add_pool_package_silkscreen_polygon("/tmp/native-project", "pool", "package-test", "0,0;1000,0;1000,1000", True, 150000)
        run_mock.assert_called_once_with(["datum-eda", "--format", "json", "project", "add-pool-package-silkscreen-polygon", "/tmp/native-project", "--pool", "pool", "--package", "package-test", "--vertices", "0,0;1000,0;1000,1000", "--closed", "true", "--width-nm", "150000"], capture_output=True, text=True, check=False, env=ANY)
        self.assertEqual(response.result["object_kind"], "footprints")
        self.assertEqual(response.result["object_uuid"], "footprint-test")

    @patch("server_runtime.subprocess.run")
    def test_adds_pool_package_silkscreen_circle_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[], returncode=0,
            stdout='{"action":"add_package_silkscreen_circle","object_kind":"footprints","object_uuid":"footprint-test"}', stderr="",
        )
        response = EngineDaemonClient().add_pool_package_silkscreen_circle(
            "/tmp/native-project", "pool", "package-test", 1000, 2000, 3000, 150000,
        )
        run_mock.assert_called_once_with(
            [
                "datum-eda", "--format", "json", "project", "add-pool-package-silkscreen-circle",
                "/tmp/native-project", "--pool", "pool", "--package", "package-test",
                "--center-x-nm", "1000", "--center-y-nm", "2000",
                "--radius-nm", "3000", "--width-nm", "150000",
            ],
            capture_output=True, text=True, check=False,
            env=ANY,
        )
        self.assertEqual(response.result["object_kind"], "footprints")
        self.assertEqual(response.result["object_uuid"], "footprint-test")

    @patch("server_runtime.subprocess.run")
    def test_adds_pool_package_silkscreen_arc_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(args=[], returncode=0, stdout='{"action":"add_package_silkscreen_arc","object_kind":"footprints","object_uuid":"footprint-test"}', stderr="")
        response = EngineDaemonClient().add_pool_package_silkscreen_arc("/tmp/native-project", "pool", "package-test", 1000, 2000, 3000, 0, 900, 150000)
        run_mock.assert_called_once_with(["datum-eda", "--format", "json", "project", "add-pool-package-silkscreen-arc", "/tmp/native-project", "--pool", "pool", "--package", "package-test", "--x-nm", "1000", "--y-nm", "2000", "--radius-nm", "3000", "--start-angle", "0", "--end-angle", "900", "--width-nm", "150000"], capture_output=True, text=True, check=False, env=ANY)
        self.assertEqual(response.result["object_kind"], "footprints")
        self.assertEqual(response.result["object_uuid"], "footprint-test")

    @patch("server_runtime.subprocess.run")
    def test_adds_pool_symbol_arc_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(args=[], returncode=0, stdout='{"action":"add_symbol_arc","object_uuid":"symbol-test"}', stderr="")
        response = EngineDaemonClient().add_pool_symbol_arc("/tmp/native-project", "pool", "symbol-test", 1000, 2000, 3000, 0, 900, 150000)
        run_mock.assert_called_once_with(["datum-eda", "--format", "json", "project", "add-pool-symbol-arc", "/tmp/native-project", "--pool", "pool", "--symbol", "symbol-test", "--x-nm", "1000", "--y-nm", "2000", "--radius-nm", "3000", "--start-angle", "0", "--end-angle", "900", "--width-nm", "150000"], capture_output=True, text=True, check=False, env=ANY)
        self.assertEqual(response.result["object_uuid"], "symbol-test")

    @patch("server_runtime.subprocess.run")
    def test_adds_pool_package_silkscreen_text_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(args=[], returncode=0, stdout='{"action":"add_package_silkscreen_text","object_kind":"footprints","object_uuid":"footprint-test"}', stderr="")
        response = EngineDaemonClient().add_pool_package_silkscreen_text("/tmp/native-project", "pool", "package-test", "REF**", 1000, 2000, 90)
        run_mock.assert_called_once_with(["datum-eda", "--format", "json", "project", "add-pool-package-silkscreen-text", "/tmp/native-project", "--pool", "pool", "--package", "package-test", "--text", "REF**", "--x-nm", "1000", "--y-nm", "2000", "--rotation", "90"], capture_output=True, text=True, check=False, env=ANY)
        self.assertEqual(response.result["object_kind"], "footprints")
        self.assertEqual(response.result["object_uuid"], "footprint-test")

    @patch("server_runtime.subprocess.run")
    def test_adds_pool_package_model_3d_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(args=[], returncode=0, stdout='{"action":"add_package_model_3d","object_uuid":"package-test"}', stderr="")
        response = EngineDaemonClient().add_pool_package_model_3d("/tmp/native-project", "pool", "package-test", "models/pkg.step", "{\"scale\":1}")
        run_mock.assert_called_once_with(["datum-eda", "--format", "json", "project", "add-pool-package-model-3d", "/tmp/native-project", "--pool", "pool", "--package", "package-test", "--model-path", "models/pkg.step", "--transform-json", "{\"scale\":1}"], capture_output=True, text=True, check=False, env=ANY)
        self.assertEqual(response.result["object_uuid"], "package-test")

    @patch("server_runtime.subprocess.run")
    def test_adds_pool_package_model_3d_without_transform_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(args=[], returncode=0, stdout='{"action":"add_package_model_3d","object_uuid":"package-test"}', stderr="")
        EngineDaemonClient().add_pool_package_model_3d("/tmp/native-project", "pool", "package-test", "models/pkg.step", None)
        run_mock.assert_called_once_with(["datum-eda", "--format", "json", "project", "add-pool-package-model-3d", "/tmp/native-project", "--pool", "pool", "--package", "package-test", "--model-path", "models/pkg.step"], capture_output=True, text=True, check=False, env=ANY)

    @patch("server_runtime.subprocess.run")
    def test_adds_pool_package_model_3d_geometry_options_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(args=[], returncode=0, stdout='{"action":"add_package_model_3d","object_uuid":"package-test"}', stderr="")
        response = EngineDaemonClient().add_pool_package_model_3d("/tmp/native-project", "pool", "package-test", "models/pkg.step", None, "step", 10, 20, 30, 1, 2, 3, 1.25)
        run_mock.assert_called_once_with(["datum-eda", "--format", "json", "project", "add-pool-package-model-3d", "/tmp/native-project", "--pool", "pool", "--package", "package-test", "--model-path", "models/pkg.step", "--format", "step", "--tx-nm", "10", "--ty-nm", "20", "--tz-nm", "30", "--roll-tenths-deg", "1", "--pitch-tenths-deg", "2", "--yaw-tenths-deg", "3", "--scale", "1.25"], capture_output=True, text=True, check=False, env=ANY)
        self.assertEqual(response.result["object_uuid"], "package-test")

    @patch("server_runtime.subprocess.run")
    def test_sets_pool_package_body_heights_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(args=[], returncode=0, stdout='{"action":"set_package_body_heights","object_uuid":"package-test"}', stderr="")
        response = EngineDaemonClient().set_pool_package_body_heights("/tmp/native-project", "pool", "package-test", 1000000, 1200000, False)
        run_mock.assert_called_once_with(["datum-eda", "--format", "json", "project", "set-pool-package-body-heights", "/tmp/native-project", "--pool", "pool", "--package", "package-test", "--body-height-nm", "1000000", "--body-height-mounted-nm", "1200000"], capture_output=True, text=True, check=False, env=ANY)
        self.assertEqual(response.result["object_uuid"], "package-test")

    @patch("server_runtime.subprocess.run")
    def test_clears_pool_package_body_heights_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(args=[], returncode=0, stdout='{"action":"set_package_body_heights","object_uuid":"package-test"}', stderr="")
        EngineDaemonClient().set_pool_package_body_heights("/tmp/native-project", "pool", "package-test", None, None, True)
        run_mock.assert_called_once_with(["datum-eda", "--format", "json", "project", "set-pool-package-body-heights", "/tmp/native-project", "--pool", "pool", "--package", "package-test", "--clear"], capture_output=True, text=True, check=False, env=ANY)

    @patch("server_runtime.subprocess.run")
    def test_deletes_pool_library_object_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout='{"action":"delete_pool_library_object","object_uuid":"symbol-test"}',
            stderr="",
        )
        response = EngineDaemonClient().delete_pool_library_object(
            "/tmp/native-project",
            "pool",
            "symbols",
            "symbol-test",
        )
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "project",
                "delete-pool-library-object",
                "/tmp/native-project",
                "--pool",
                "pool",
                "--kind",
                "symbols",
                "--object",
                "symbol-test",
            ],
            capture_output=True,
            text=True,
            check=False,
            env=ANY,
        )
        self.assertEqual(response.result["object_uuid"], "symbol-test")

    @patch("server_runtime.subprocess.run")
    def test_sets_pool_library_object_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout='{"action":"set_pool_library_object","object_uuid":"symbol-test"}',
            stderr="",
        )
        response = EngineDaemonClient().set_pool_library_object(
            "/tmp/native-project",
            "pool",
            "symbols",
            "symbol-test",
            "/tmp/symbol-edited.json",
        )
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "project",
                "set-pool-library-object",
                "/tmp/native-project",
                "--pool",
                "pool",
                "--kind",
                "symbols",
                "--object",
                "symbol-test",
                "--from-json",
                "/tmp/symbol-edited.json",
            ],
            capture_output=True,
            text=True,
            check=False,
            env=ANY,
        )
        self.assertEqual(response.result["object_uuid"], "symbol-test")
