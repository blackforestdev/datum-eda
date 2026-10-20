"""Offline regressions for the black native04 capture; no desktop or scratch input."""
import ast
from pathlib import Path
import struct
import subprocess
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import Mock, patch
import zlib

from native_capture_pixels import require_native_capture_pixels


def png(width=1344, height=806, *, rgba=False, nonblack=False):
    """Reproduce captured RGB8 format and failed image dimensions deterministically."""
    def chunk(kind, data):
        return (struct.pack(">I", len(data)) + kind + data
                + struct.pack(">I", zlib.crc32(kind + data)))
    channels = 4 if rgba else 3
    row = bytes(width * channels)
    pixels = bytearray((b"\0" + row) * height)
    if nonblack:
        pixels[1] = 1  # A single nonzero color sample suffices: no arbitrary budget.
    header = struct.pack(">IIBBBBB", width, height, 8, 6 if rgba else 2, 0, 0, 0)
    return (b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", header)
            + chunk(b"IDAT", zlib.compress(pixels)) + chunk(b"IEND", b""))


class NativeCapturePixelsTests(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name)

    def write(self, data):
        path = self.root / "capture.png"
        path.write_bytes(data)
        return path

    def test_actual_black_output_geometry_rejected_and_original_preserved(self):
        original = png()
        path = self.write(original)
        with self.assertRaisesRegex(RuntimeError, "wholly black RGB"):
            require_native_capture_pixels(path)
        self.assertEqual(path.read_bytes(), original)

    def test_rgb_and_rgba_valid_png_with_one_nonzero_color_sample_accepted(self):
        for rgba in [False, True]:
            with self.subTest(rgba=rgba):
                require_native_capture_pixels(self.write(png(rgba=rgba, nonblack=True)))

    def test_alpha_does_not_make_wholly_black_color_usable(self):
        with self.assertRaisesRegex(RuntimeError, "wholly black RGB"):
            require_native_capture_pixels(self.write(png(rgba=True)))

    def test_missing_malformed_truncated_and_corrupt_png_fail_honestly(self):
        valid = png(nonblack=True)
        for data in [b"not PNG", valid[:40], valid[:-8],
                     valid[:29] + bytes([valid[29] ^ 1]) + valid[30:]]:
            with self.subTest(length=len(data)):
                path = self.write(data)
                with self.assertRaisesRegex(RuntimeError, "Unusable native capture"):
                    require_native_capture_pixels(path)
                self.assertEqual(path.read_bytes(), data)
        with self.assertRaisesRegex(RuntimeError, "Unusable native capture"):
            require_native_capture_pixels(self.root / "missing.png")

    def test_decodable_non_png_is_rejected(self):
        with self.assertRaisesRegex(RuntimeError, "not PNG"):
            require_native_capture_pixels(self.write(b"P6\n1 1\n255\n\x01\x01\x01"))

    def capture(self, image, *, exit_code=0, stderr=""):
        # Execute the actual capture method without importing/starting GI or GUI.
        source = Path(__file__).with_name("prove_native_project_startup_gui.py")
        tree = ast.parse(source.read_text())
        owner = next(n for n in tree.body if isinstance(n, ast.ClassDef) and n.name == "StartupQA")
        method = next(n for n in owner.body if isinstance(n, ast.FunctionDef) and n.name == "capture")
        namespace = {"subprocess": subprocess,
                     "require_native_capture_pixels": require_native_capture_pixels}
        exec(compile(ast.Module(body=[method], type_ignores=[]), str(source), "exec"), namespace)
        qa = SimpleNamespace(root=self.root, capture_counts={}, actions=[], observations={},
                             env={"WAYLAND_DISPLAY": "owned-qa"}, focus=Mock(),
                             wait_presented=Mock(return_value={"presented_damage": 4}),
                             nodes=Mock(return_value=[]))
        def screenshooter(*args, **kwargs):
            if image is not None:
                (kwargs["cwd"] / "weston-shot.png").write_bytes(image)
            return subprocess.CompletedProcess(args[0], exit_code, "", stderr)
        with patch.object(subprocess, "run", side_effect=screenshooter):
            try:
                receipt = namespace["capture"](qa, "selected", "owned-window")
            finally:
                self.last_qa = qa
        return receipt

    def test_shared_capture_stops_before_public_walk_and_preserves_failed_png(self):
        original = png()
        with self.assertRaisesRegex(RuntimeError, "wholly black RGB"):
            self.capture(original)
        self.last_qa.nodes.assert_not_called()
        self.assertEqual((self.root / "selected.png").read_bytes(), original)
        self.assertEqual(len(self.last_qa.actions), 1)  # No retry.
        self.assertEqual(self.last_qa.actions[0]["exit"], 0)

    def test_shared_capture_valid_pixels_continue_to_public_observation(self):
        self.assertEqual(self.capture(png(nonblack=True)), {"presented_damage": 4})
        self.last_qa.nodes.assert_called_once_with()
        self.assertEqual(self.last_qa.observations, {"selected": []})

    def test_shared_capture_keeps_command_failure_and_original_output(self):
        original = png(nonblack=True)
        with self.assertRaisesRegex(AssertionError, "native capture failed: capture error"):
            self.capture(original, exit_code=1, stderr="capture error")
        self.last_qa.nodes.assert_not_called()
        self.assertEqual((self.root / "selected-native-capture/weston-shot.png").read_bytes(), original)


if __name__ == "__main__":
    unittest.main()
