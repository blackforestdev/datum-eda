#!/usr/bin/env python3
"""GUI-free consumer proof for the Seam-1 frozen-identity selection switch (U3 repair).

Audit's U3-core verdict ran the committed ``gui_producer._consume_identity`` against a real frozen
baton and hit ``KeyError: 'coordinate'`` at the environments-array lookup: the frozen 21-field
``environments`` array carries NO per-entry ``coordinate`` key (its closed field set + the
``environment_sha256`` binding forbid injecting one), so the only coordinate->id projection is the
frozen.json ``selection`` aid. This proof exercises the ACTUAL consumer function (not the aid's bare
parseability -- audit's precise critique of the prior U3 unit test): it builds a baton through the
LANDED ``assemble.freeze_identity`` and drives ``_consume_identity`` to a successful row-binding with
the correct deterministic id, on the exact shape that reproduced the KeyError.

    python3 scripts/test_global_preferences_gui_identity_consumer.py
"""
from __future__ import annotations

import hashlib
from pathlib import Path
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parent))

from global_preferences_acceptance import assemble, gui_producer  # noqa: E402


def _native_environment(backend, scale, window):
    """A valid 21-field native environment entry (passes identity.validate_environments) whose id is
    the deterministic ``native-<backend>-<int-scale>-<WxH>`` derivation freeze_identity requires."""
    entry = {
        "os": "Linux", "architecture": "x86_64", "kernel": "6.12.0-test",
        "cpu": "Synthetic x86_64 @ 1.0GHz", "memory_mib": 4096, "filesystem": "ext4",
        "mount_options": "rw,relatime", "storage_device": "synthetic-nvme0n1",
        "gpu": "synthetic-gpu", "driver": "synthetic-driver", "display_backend": backend,
        "compositor_or_server": "synthetic-weston", "scale_factor": scale,
        "window_size": list(window), "locale": "C.UTF-8", "network_state": "offline",
        "power_mode": "ac", "background_load": "idle", "clock": "monotonic-synced",
        "tool_versions": {"weston": "test"},
    }
    entry["id"] = assemble.native_environment_id(entry)
    return entry


def _headless_environment():
    """A valid headless entry (display_backend None): present in the 21-field array but absent from the
    selection aid, so the consumer must ignore it when resolving a native coordinate."""
    return {
        "id": "headless", "os": "Linux", "architecture": "x86_64", "kernel": "6.12.0-test",
        "cpu": "Synthetic x86_64 @ 1.0GHz", "memory_mib": 4096, "filesystem": "ext4",
        "mount_options": "rw,relatime", "storage_device": "synthetic-nvme0n1",
        "gpu": None, "driver": None, "display_backend": None, "compositor_or_server": None,
        "scale_factor": None, "window_size": None, "locale": "C.UTF-8", "network_state": "offline",
        "power_mode": "ac", "background_load": "idle", "clock": "monotonic-synced",
        "tool_versions": {"python": "test"},
    }


ROLES = ("cli", "gui", "daemon", "engine", "gui-render", "mcp")


class GuiIdentityConsumerTests(unittest.TestCase):
    def setUp(self):
        # A real on-disk gui binary whose sha256 the frozen candidate pins (no-substitution check).
        self._tmp = tempfile.TemporaryDirectory()
        self.gui_binary = Path(self._tmp.name) / "datum-gui"
        self.gui_binary.write_bytes(b"synthetic gui binary bytes")
        self.gui_sha = hashlib.sha256(self.gui_binary.read_bytes()).hexdigest()
        self.coordinate = ("wayland", 1.0, (1024, 900))
        self.environments = [
            _headless_environment(),
            _native_environment("wayland", 1.0, (1024, 900)),
            _native_environment("x11", 1.0, (720, 760)),  # distractor: must not be selected
        ]
        candidate = {
            "revision": "0" * 40, "tree": "0" * 40, "cargo_lock_sha256": "0" * 64,
            "toolchains": {"rustc": "test", "cargo": "test", "python": "test"},
            "build_commands": [],
            "binaries": {role: {"path": f"/synthetic/{role}",
                                "sha256": (self.gui_sha if role == "gui"
                                           else hashlib.sha256(role.encode()).hexdigest())}
                         for role in ROLES},
        }
        input_manifest_ref = {"path": "scripts/synthetic-manifest", "sha256": "1" * 64}
        matrix_sha256 = "2" * 64
        # The LANDED freeze -- the identity every producer consumes.
        self.frozen = assemble.freeze_identity(
            candidate, self.environments, input_manifest_ref, matrix_sha256)

    def tearDown(self):
        self._tmp.cleanup()

    def test_frozen_environments_carry_no_coordinate_key(self):
        """The precondition that reproduced the committed KeyError: the 21-field array has no
        ``coordinate`` key, and the ``selection`` aid is the only place coordinates live."""
        for entry in self.frozen["environments"]:
            self.assertNotIn("coordinate", entry)
        self.assertTrue(self.frozen["selection"])
        for row in self.frozen["selection"]:
            self.assertEqual(set(row), {"coordinate", "id"})

    def test_consumer_binds_to_correct_id_without_keyerror(self):
        """Drive the ACTUAL consumer on the real frozen shape: the old ``e['coordinate']`` lookup
        KeyErrored here; the switch resolves via the selection aid to the deterministic native id."""
        candidate, binaries, input_manifest, matrix_sha256, environments = (
            gui_producer._consume_identity(self.frozen, self.gui_binary, self.coordinate))
        self.assertEqual(environments, {"native-wayland-1-1024x900": self.coordinate})
        self.assertEqual(binaries, {"gui": self.gui_sha})
        self.assertIs(candidate, self.frozen["candidate"])
        self.assertEqual(input_manifest, self.frozen["input_manifest"])
        self.assertEqual(matrix_sha256, self.frozen["matrix_sha256"])
        # The resolved id is a real id in the frozen 21-field array (the seam-1 one binding).
        self.assertIn("native-wayland-1-1024x900",
                      {entry["id"] for entry in self.frozen["environments"]})

    def test_resolved_environment_feeds_produce_case_one_match_rule(self):
        """produce_case selects the environment whose coordinate equals key[4] and requires exactly
        one -- demonstrate the consumer's returned mapping satisfies that binding for this row."""
        _, _, _, _, environments = gui_producer._consume_identity(
            self.frozen, self.gui_binary, self.coordinate)
        matches = [name for name, coord in environments.items() if coord == self.coordinate]
        self.assertEqual(matches, ["native-wayland-1-1024x900"])

    def test_unknown_coordinate_is_refused(self):
        """A coordinate with no selection row is refused (not silently bound to a wrong environment)."""
        with self.assertRaises(SystemExit):
            gui_producer._consume_identity(self.frozen, self.gui_binary, ("wayland", 2.0, (1024, 900)))

    def test_substituted_gui_binary_is_refused(self):
        """The frozen candidate pins the gui binary sha256; a different binary is refused."""
        other = Path(self._tmp.name) / "other-gui"
        other.write_bytes(b"a different binary")
        with self.assertRaises(SystemExit):
            gui_producer._consume_identity(self.frozen, other, self.coordinate)


if __name__ == "__main__":
    unittest.main()
