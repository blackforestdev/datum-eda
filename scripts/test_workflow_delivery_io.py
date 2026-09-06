#!/usr/bin/env python3
"""Hermetic PM041 input primitive tests; not the complete N01-N20/P01-P07 suite."""

from pathlib import Path
import hashlib
import tempfile
import unittest

from workflow_delivery_io import (
    DeliveryInputError, canonical_json, normalized_path, parse_json,
    read_worktree_bytes, sha256, worktree_path,
)


class JsonInputs(unittest.TestCase):
    def test_nested_duplicate_keys_refuse(self):
        for raw in (b'{"a":1,"a":2}', b'{"a":{"b":1,"b":2}}'):
            with self.subTest(raw=raw), self.assertRaises(DeliveryInputError) as caught:
                parse_json(raw, "contract.json")
            self.assertEqual(caught.exception.code, "WDQ-CONTRACT")
            self.assertEqual(caught.exception.path, "contract.json")

    def test_nonfinite_numbers_refuse_including_exponent_overflow(self):
        for raw in (b'NaN', b'Infinity', b'-Infinity', b'1e9999', b'[-1e9999]'):
            with self.subTest(raw=raw), self.assertRaises(DeliveryInputError):
                parse_json(raw, "contract.json")

    def test_bad_encoding_syntax_and_lone_surrogates_refuse(self):
        for raw in (b'"\xff"', b'{', b'{} trailing', b'"\\ud800"', b'{"\\udfff":0}'):
            with self.subTest(raw=raw), self.assertRaises(DeliveryInputError):
                parse_json(raw, "contract.json")

    def test_types_preserved_for_shape_validator(self):
        result = parse_json(b'{"version":true,"count":1,"items":[]}', "x.json")
        self.assertIs(type(result["version"]), bool)
        self.assertIs(type(result["count"]), int)
        self.assertEqual(result["items"], [])

    def test_canonical_identity_exact_utf8_and_newline(self):
        result = canonical_json({"z": [2, 1], "a": "μ"})
        self.assertEqual(result, '{"a":"μ","z":[2,1]}\n'.encode())
        self.assertEqual(sha256(result), hashlib.sha256(result).hexdigest())
        self.assertEqual(result, canonical_json({"a": "μ", "z": [2, 1]}))
        self.assertNotEqual(result, canonical_json({"a": "μ", "z": [1, 2]}))

    def test_python_only_values_cannot_gain_canonical_identity(self):
        for value in ({1: "x"}, (1, 2), {"a": float("nan")}, b"bytes", "\ud800"):
            with self.subTest(value=repr(value)), self.assertRaises(DeliveryInputError):
                canonical_json(value)

    def test_blob_hash_is_raw_not_normalized_json(self):
        self.assertNotEqual(sha256(b'{"x":1}'), sha256(b'{ "x": 1 }'))
        self.assertEqual(
            canonical_json(parse_json(b'{"x":1}', "a")),
            canonical_json(parse_json(b'{ "x": 1 }', "b")),
        )


class PathInputs(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name) / "repo"
        self.root.mkdir()
        (self.root / "file.json").write_bytes(b'{"valid":true}\n')

    def test_absolute_parent_url_and_non_normalized_paths_refuse(self):
        for path in ("", ".", "../x", "a/../x", "/x", "a//b", "a/./b",
                     "a/", "https://example.org/x", "a\\b", "x\0y"):
            with self.subTest(path=path), self.assertRaises(DeliveryInputError):
                normalized_path(path)

    def test_unicode_space_names_are_literal_not_shell_arguments(self):
        path = "μ name $(never-execute).json"
        (self.root / path).write_bytes(b'{}')
        self.assertEqual(read_worktree_bytes(self.root, path), b'{}')

    def test_missing_files_and_directories_refuse_without_mutation(self):
        before = sorted(p.name for p in self.root.iterdir())
        for path in ("missing.json", "folder"):
            if path == "folder":
                (self.root / path).mkdir()
            with self.assertRaises(DeliveryInputError) as caught:
                read_worktree_bytes(self.root, path)
            self.assertEqual(caught.exception.code, "WDQ-ARTIFACT")
        self.assertEqual(before, ["file.json"])
        self.assertFalse((self.root / "missing.json").exists())
        self.assertEqual((self.root / "file.json").read_bytes(), b'{"valid":true}\n')

    def test_internal_symlink_resolves_but_escape_refuses(self):
        (self.root / "inside").symlink_to(self.root / "file.json")
        self.assertEqual(read_worktree_bytes(self.root, "inside"), b'{"valid":true}\n')
        outside = Path(self.temporary.name) / "outside"
        outside.write_bytes(b'private')
        (self.root / "escape").symlink_to(outside)
        with self.assertRaises(DeliveryInputError):
            read_worktree_bytes(self.root, "escape")

    def test_future_artifact_may_be_missing_but_not_escape(self):
        future = worktree_path(self.root, "proof/run.json", must_exist=False)
        self.assertEqual(future, self.root / "proof/run.json")
        self.assertFalse(future.exists())
        (self.root / "outside").symlink_to(Path(self.temporary.name), target_is_directory=True)
        with self.assertRaises(DeliveryInputError):
            worktree_path(self.root, "outside/new.json", must_exist=False)

    def test_broken_and_cyclic_symlinks_refuse(self):
        (self.root / "broken").symlink_to(self.root / "absent")
        (self.root / "cycle").symlink_to(self.root / "cycle")
        for name in ("broken", "cycle"):
            with self.subTest(name=name), self.assertRaises(DeliveryInputError):
                read_worktree_bytes(self.root, name)


if __name__ == "__main__":
    unittest.main()
