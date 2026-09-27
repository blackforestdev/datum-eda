"""Offline controls for client deduplication and honest incomplete observations."""
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import drm_clients as drm


def record(client=26, pdev="0000:00:02.0", counters="drm-total-system0: 1180 KiB\ndrm-resident-system0: 0\n"):
    return ("drm-driver: i915\n" + (f"drm-client-id: {client}\n" if client is not None else "")
            + f"drm-pdev: {pdev}\n" + counters)


def fixture(root, pid, descriptors):
    directory = root / str(pid)
    (directory / "fdinfo").mkdir(parents=True)
    (directory / "stat").write_text(f"{pid} (name with ) spaces) S " + "0 " * 18 + "12345\n")
    for fd, value in descriptors.items():
        (directory / "fdinfo" / str(fd)).write_text(value)


class Controls(unittest.TestCase):
    def test_identity_and_counter_classes(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            fixture(root, 100, {3: record(), 4: record(), 5: record(pdev="0000:01:00.0"), 6: "pos: 0\n"})
            fixture(root, 101, {3: record()})
            result = drm.collect([101, 100, 100], root)
            self.assertTrue(result["complete_enumeration"])
            self.assertEqual(len(result["clients"]), 2)
            client = result["clients"][0]
            self.assertEqual(len(client["other_fd_observations"]), 2)
            self.assertEqual(client["first_sample"]["starttime_ticks"], 12345)
            self.assertEqual(client["first_sample"]["memory_bytes"], {
                "drm-total-system0": 1208320, "drm-resident-system0": 0})
            parsed = drm.parse(record(counters="drm-total-local: 2 MiB\ndrm-shared-local: 1024 bytes\ndrm-memory-local: 512\ndrm-resident-local: 512\n"))
            self.assertEqual(parsed["memory_bytes"], {"drm-total-local": 2097152,
                "drm-shared-local": 1024, "drm-memory-local": 512, "drm-resident-local": 512})

    def test_unknown_and_malformed_are_not_zero(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            fixture(root, 100, {3: record(counters="drm-total-cycles-render: 123456\ndrm-cycles-render: 789\n"), 4: record(client=None)})
            result = drm.collect([100, 999], root)
            self.assertFalse(result["complete_enumeration"])
            self.assertFalse(result["memory_counters_available"])
            self.assertEqual(result["clients"][0]["first_sample"]["memory_bytes"], {})
            self.assertEqual(len(result["unidentified"]), 1)
            self.assertEqual(result["errors"][0]["pid"], 999)
        for invalid in ("drm-resident-local: -1", "drm-resident-local: 1 GB",
                        "drm-resident-: 1", "drm-client-id: 99", "drm-driver: duplicate"):
            with self.subTest(invalid=invalid), self.assertRaises(ValueError):
                drm.parse(record() + invalid + "\n")

    def test_closed_fd_and_process_change_remain_incomplete(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            fixture(root, 100, {3: record(), 4: record()})
            original = drm.read_at
            stat_reads = 0

            def racing_read(directory, name):
                nonlocal stat_reads
                if name == "4":
                    raise FileNotFoundError("descriptor closed before read")
                text = original(directory, name)
                if name == "stat":
                    stat_reads += 1
                    if stat_reads == 2:
                        return text.replace("12345", "54321")
                return text

            with patch.object(drm, "read_at", racing_read):
                result = drm.collect([100], root)
            self.assertFalse(result["complete_enumeration"])
            self.assertEqual(len(result["clients"]), 1)
            self.assertEqual(len(result["errors"]), 2)
            self.assertNotIn("identity_verified_after_read", result["processes"][0])


if __name__ == "__main__":
    unittest.main()
