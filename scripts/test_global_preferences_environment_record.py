"""Validator-grounded tests for the GP-CM05E Phase-A U1 environment-record producer.

The environment record is readiness-bearing: it binds every measurement/case/native observation
and the Phase-B runner via environment-sha256. These tests assert the producer's output is accepted
by the SAME validator the acceptance runner uses (validate_environments) and that the digest matches
report.py's binding formula -- never a hand-rolled shape."""
import unittest

from global_preferences_acceptance.environment_record import (
    ENVIRONMENT_FIELDS, build_environments, environment_sha256, headless_environment,
    native_environment,
)
from global_preferences_acceptance.identity import validate_environments
from global_preferences_acceptance.inputs import EvidenceError, canonical, digest

TARGET = "."
NATIVE = [
    {"coordinate": c, "gpu": "Intel HD Graphics P630", "driver": "Mesa 25.0.7",
     "compositor": "owned Weston", "background_load": "loadavg 1.0/1.0/1.0; quiet window"}
    for c in ("wayland/1.0/1024x900", "wayland/1.0/720x760", "x11/1.0/1024x900", "x11/1.0/720x760")
]


class EnvironmentRecordTests(unittest.TestCase):
    def test_headless_environment_passes_the_acceptance_validator(self):
        env = headless_environment(TARGET)
        self.assertEqual(set(env), set(ENVIRONMENT_FIELDS))
        coords = validate_environments([env])
        self.assertEqual(coords, {"headless-linux-x86_64": None})

    def test_headless_native_fields_are_none(self):
        env = headless_environment(TARGET)
        for field in ("gpu", "driver", "display_backend", "compositor_or_server",
                      "scale_factor", "window_size"):
            self.assertIsNone(env[field], field)

    def test_required_facts_are_never_unknown(self):
        env = headless_environment(TARGET)
        for field in ("kernel", "cpu", "filesystem", "mount_options", "storage_device", "locale",
                      "network_state", "power_mode", "background_load", "clock"):
            self.assertNotIn(env[field].lower(), {"unknown", "unavailable"}, field)
        self.assertGreaterEqual(env["memory_mib"], 1)
        self.assertTrue(env["tool_versions"] and "python" in env["tool_versions"])

    def test_full_environment_list_passes_with_unique_coordinates(self):
        envs = build_environments(TARGET, NATIVE)
        self.assertEqual(len(envs), 5)
        coords = validate_environments(envs)
        self.assertEqual(coords["native-wayland-1-1024x900"], ("wayland", 1.0, (1024, 900)))
        self.assertEqual(len({None, *(v for v in coords.values() if v)}), 5)

    def test_environment_sha256_matches_report_binding_formula(self):
        envs = build_environments(TARGET, NATIVE)
        # report.py binds via digest(canonical(report["environments"])); U1 must match exactly.
        self.assertEqual(environment_sha256(envs), digest(canonical(envs)))

    def test_native_entry_has_all_fields_and_native_coordinate(self):
        env = native_environment(TARGET, "wayland/1.0/1024x900", gpu="g", driver="d",
                                 compositor="c", background_load="loadavg 1/1/1")
        self.assertEqual(set(env), set(ENVIRONMENT_FIELDS))
        self.assertEqual(env["display_backend"], "wayland")
        self.assertEqual(env["scale_factor"], 1.0)
        self.assertEqual(env["window_size"], [1024, 900])
        validate_environments([headless_environment(TARGET), env])

    def test_duplicate_native_coordinate_is_rejected(self):
        dup = [NATIVE[0], dict(NATIVE[0])]
        with self.assertRaises(EvidenceError):
            build_environments(TARGET, dup)

    def test_unsupported_coordinate_is_rejected(self):
        bad = [{"coordinate": "wayland/3.0/1024x900", "gpu": "g", "driver": "d",
                "compositor": "c", "background_load": "l"}]
        with self.assertRaises(EvidenceError):
            build_environments(TARGET, bad)


if __name__ == "__main__":
    unittest.main()
