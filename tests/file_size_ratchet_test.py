from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import file_size_ratchet as ratchet


class ProductionPathTests(unittest.TestCase):
    def test_includes_production_rust_and_excludes_test_modules(self) -> None:
        self.assertTrue(ratchet.is_production_rust("src/actions/dialog.rs"))
        for path in [
            "tests/dialog.rs",
            "src/actions/tests/dialog.rs",
            "src/actions/tests.rs",
            "src/actions/dialog_test.rs",
            "src/actions/dialog_tests.rs",
            "src/actions/dialog_test/support.rs",
            "src/actions/dialog_random_tests/mod.rs",
            "src/actions/test_dialog.rs",
            "src/actions/dialog.md",
        ]:
            self.assertFalse(ratchet.is_production_rust(path), path)


class FileSizeRatchetTests(unittest.TestCase):
    def test_current_grandfathered_file_must_match_limit(self) -> None:
        counts = {"src/large.rs": 2_500}
        self.assertEqual(ratchet.evaluate(counts, {"src/large.rs": 2_500}), [])

    def test_new_critical_file_fails(self) -> None:
        errors = ratchet.evaluate({"src/new.rs": 2_001}, {})
        self.assertIn("new critical file", errors[0])

    def test_grandfathered_file_growth_fails(self) -> None:
        errors = ratchet.evaluate(
            {"src/large.rs": 2_501},
            {"src/large.rs": 2_500},
        )
        self.assertIn("critical file grew", errors[0])

    def test_grandfather_ceiling_must_follow_source_shrinkage(self) -> None:
        errors = ratchet.evaluate(
            {"src/large.rs": 2_400},
            {"src/large.rs": 2_500},
        )
        self.assertIn("lower its grandfather limit", errors[0])

    def test_stale_entry_fails_after_file_shrinks_below_threshold(self) -> None:
        errors = ratchet.evaluate(
            {"src/large.rs": 1_999},
            {"src/large.rs": 2_500},
        )
        self.assertIn("remove it from the allowlist", errors[0])

    def test_stale_entry_fails_after_file_is_removed(self) -> None:
        errors = ratchet.evaluate({}, {"src/large.rs": 2_500})
        self.assertIn("no longer exists", errors[0])

    def test_allowlist_entry_may_be_removed_after_split(self) -> None:
        self.assertEqual(
            ratchet.evaluate(
                {"src/large/mod.rs": 900},
                {},
                {"src/large.rs": 2_500},
            ),
            [],
        )

    def test_allowlist_entry_may_not_be_added(self) -> None:
        errors = ratchet.evaluate(
            {"src/large.rs": 2_500},
            {"src/large.rs": 2_500},
            {},
        )
        self.assertTrue(any("may only shrink" in error for error in errors))

    def test_grandfather_limit_may_decrease_but_not_increase(self) -> None:
        counts = {"src/large.rs": 2_400}
        self.assertEqual(
            ratchet.evaluate(counts, {"src/large.rs": 2_400}, {"src/large.rs": 2_500}),
            [],
        )
        errors = ratchet.evaluate(
            counts,
            {"src/large.rs": 2_600},
            {"src/large.rs": 2_500},
        )
        self.assertTrue(any("may not increase" in error for error in errors))

    def test_invalid_base_revision_is_not_treated_as_first_introduction(self) -> None:
        with self.assertRaisesRegex(RuntimeError, "invalid base revision"):
            ratchet.revision_allowlist("definitely-not-a-real-file-size-ratchet-revision")


if __name__ == "__main__":
    unittest.main()
