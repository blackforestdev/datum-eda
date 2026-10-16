"""Reject weakened/missing canonical journal persistence ownership."""
from pathlib import Path
import tempfile
import unittest

from private_writer_journal_policy import JOURNAL_OWNERS, check_journal_persistence


ROOT = Path(__file__).resolve().parents[1]
STAGE, IO = JOURNAL_OWNERS


class JournalPolicyTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        for relative in JOURNAL_OWNERS:
            dest = self.root / relative
            dest.parent.mkdir(parents=True, exist_ok=True)
            dest.write_bytes((ROOT / relative).read_bytes())

    def tearDown(self):
        self.temp.cleanup()

    def change(self, relative, old, new):
        path = self.root / relative
        source = path.read_text()
        self.assertIn(old, source)
        path.write_text(source.replace(old, new, 1))

    def test_real_two_owner_split_passes(self):
        self.assertEqual(check_journal_persistence(self.root), [])

    def test_missing_io_owner_is_not_blessed_by_staging_owner(self):
        (self.root / IO).unlink()
        self.assertIn("missing engine journal persistence file", " ".join(check_journal_persistence(self.root)))

    def test_extra_write_in_either_owner_refuses(self):
        for relative in JOURNAL_OWNERS:
            with self.subTest(relative=relative):
                path = self.root / relative
                original = path.read_text()
                path.write_text(original + '\nstd::fs::write("private.json", payload)?;\n')
                self.assertTrue(check_journal_persistence(self.root))
                path.write_text(original)

    def test_aggregate_three_writes_cannot_move_between_owners(self):
        self.change(STAGE, "std::fs::write(", "private_write(")
        path = self.root / IO
        path.write_text(path.read_text() + '\nstd::fs::write("private.json", payload)?;\n')
        failures = check_journal_persistence(self.root)
        self.assertTrue(any(str(STAGE) in x and "found 1" in x for x in failures))
        self.assertTrue(any(str(IO) in x and "found 2" in x for x in failures))

    def test_missing_cursor_append_or_stage_owner_refuses(self):
        for relative, pattern in [
            (IO, "pub(crate) fn write_journal_cursor("),
            (IO, "pub(super) fn append_transaction_journal("),
            (STAGE, "pub(super) fn promote_staged_shard_writes("),
        ]:
            with self.subTest(pattern=pattern):
                original = (self.root / relative).read_text()
                self.change(relative, pattern, "fn private_owner(")
                self.assertTrue(check_journal_persistence(self.root))
                (self.root / relative).write_text(original)

    def test_changed_append_surface_and_missing_durability_refuse(self):
        self.change(IO, "std::fs::OpenOptions::new()", "private_append()")
        self.assertTrue(any("OpenOptions calls" in x for x in check_journal_persistence(self.root)))
        original = (ROOT / IO).read_text()
        (self.root / IO).write_text(original.replace("file.sync_all()?;", "private_sync()?;"))
        self.assertTrue(any("file.sync_all" in x for x in check_journal_persistence(self.root)))


if __name__ == "__main__":
    unittest.main()
