#!/usr/bin/env python3
"""Hermetic regressions for the libghostty-vt dependency pin."""

from __future__ import annotations

import copy
import importlib.util
import json
from pathlib import Path
import tarfile
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location("build_libghostty_vt", ROOT / "scripts/build_libghostty_vt.py")
assert SPEC and SPEC.loader
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


class LibghosttyPinTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.pin = json.loads((ROOT / "third_party/libghostty-vt/lock.json").read_text(encoding="utf-8"))

    def fixture(self) -> tuple[tempfile.TemporaryDirectory[str], Path, dict[str, object]]:
        temporary = tempfile.TemporaryDirectory()
        root = Path(temporary.name)
        notice = root / "third_party/libghostty-vt/LICENSE"
        notice.parent.mkdir(parents=True)
        notice.write_bytes((ROOT / "third_party/libghostty-vt/LICENSE").read_bytes())
        return temporary, root, copy.deepcopy(self.pin)

    def test_real_pin_passes(self) -> None:
        MODULE.load_pin(ROOT)

    def test_closed_root_rejects_extra_key(self) -> None:
        temporary, root, pin = self.fixture()
        with temporary:
            pin["floating_version"] = "main"
            with self.assertRaisesRegex(MODULE.PinError, "keys differ"):
                MODULE.validate_pin(root, pin)

    def test_full_commit_is_required(self) -> None:
        temporary, root, pin = self.fixture()
        with temporary:
            pin["source"]["commit"] = "794515ba"
            with self.assertRaisesRegex(MODULE.PinError, "full lowercase Git"):
                MODULE.validate_pin(root, pin)

    def test_build_args_cannot_drift_from_properties(self) -> None:
        temporary, root, pin = self.fixture()
        with temporary:
            pin["build"]["args"][-1] = "-Dvt-features=-kitty-graphics"
            with self.assertRaisesRegex(MODULE.PinError, "build.args"):
                MODULE.validate_pin(root, pin)

    def test_feature_inventory_cannot_shrink(self) -> None:
        temporary, root, pin = self.fixture()
        with temporary:
            pin["features"].remove("kitty_graphics")
            with self.assertRaisesRegex(MODULE.PinError, "exact complete"):
                MODULE.validate_pin(root, pin)

    def test_retained_license_must_match_upstream_hash(self) -> None:
        temporary, root, pin = self.fixture()
        with temporary:
            (root / "third_party/libghostty-vt/LICENSE").write_text("not the license", encoding="utf-8")
            with self.assertRaisesRegex(MODULE.PinError, "MIT notice"):
                MODULE.validate_pin(root, pin)

    def test_download_checksum_mismatch_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            source = Path(temporary) / "source"
            destination = Path(temporary) / "destination"
            source.write_bytes(b"pinned bytes")
            with self.assertRaisesRegex(MODULE.PinError, "checksum mismatch"):
                MODULE.fetch(source.as_uri(), "0" * 64, destination)
            self.assertFalse(destination.exists())

    def test_extraction_replaces_mutated_cached_source(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            upstream = root / "upstream"
            upstream.mkdir()
            (upstream / "source.zig").write_text("pinned", encoding="utf-8")
            archive = root / "source.tar.gz"
            with tarfile.open(archive, "w:gz") as bundle:
                bundle.add(upstream, arcname="ghostty-pin")
            destination = root / "extracted"
            MODULE.extract_fresh(archive, destination, strip_first=True)
            (destination / "source.zig").write_text("tampered", encoding="utf-8")
            (destination / "injected.zig").write_text("unexpected", encoding="utf-8")
            MODULE.extract_fresh(archive, destination, strip_first=True)
            self.assertEqual((destination / "source.zig").read_text(encoding="utf-8"), "pinned")
            self.assertFalse((destination / "injected.zig").exists())


if __name__ == "__main__":
    unittest.main()
