from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import hardcoded_visual_inventory as inventory


def calls(source: str, path: str = "src/example.rs") -> list[inventory.VisualCall]:
    return inventory.scan_source(path, source)


class HardcodedVisualScannerTests(unittest.TestCase):
    def test_all_color_constructor_names_are_scanned(self) -> None:
        source = "rgb(1); rgba(2); hsl(3); hsla(4);"
        self.assertCountEqual(
            [call.signature for call in calls(source)],
            ["rgb(1)", "rgba(2)", "hsl(3)", "hsla(4)"],
        )

    def test_comments_and_strings_are_ignored(self) -> None:
        source = r'''
// rgb(0xff0000)
/* rgba(0x00ff00ff) */
const FIXTURE: &str = r#"hsla(0.5, 1.0, 0.5, 1.0)"#;
fn live() { let _ = gpui::rgb(0x12_34_56); }
'''
        self.assertEqual([call.signature for call in calls(source)], ["rgb(0x123456)"])

    def test_cfg_test_scopes_are_ignored(self) -> None:
        source = '''
#[cfg(test)]
mod tests {
    fn fixture() { let _ = rgb(0xabcdef); }
}
#[cfg(test)]
fn test_helper() { let _ = rgba(0x123456ff); }
fn live() { let _ = rgba(0x010203ff); }
'''
        self.assertEqual([call.signature for call in calls(source)], ["rgba(0x010203ff)"])

    def test_cfg_any_test_scope_is_still_scanned(self) -> None:
        source = '''
#[cfg(any(target_os = "macos", test))]
fn live_on_macos() { let _ = rgba(0x123456ff); }
'''
        self.assertEqual([call.signature for call in calls(source)], ["rgba(0x123456ff)"])

    def test_external_test_module_paths_are_ignored(self) -> None:
        source = "fn fixture() { rgba(0x123456ff); }"
        for path in [
            "src/components/form_fields/tests.rs",
            "src/components/form_fields_tests.rs",
            "src/theme/lightweight_colors_test.rs",
            "src/components/button/tests/render.rs",
        ]:
            with self.subTest(path=path):
                self.assertEqual(calls(source, path), [])

    def test_grouped_and_signed_numeric_literals_are_scanned(self) -> None:
        source = (
            "rgba((0xff0000ff)); hsl(-0.1, 1., 0.5); "
            "rgb(+(0x123456)); rgba((0xff000000) | alpha);"
        )
        self.assertCountEqual(
            [call.signature for call in calls(source)],
            [
                "rgba((0xff0000ff))",
                "hsl(-0.1,1.,0.5)",
                "rgb(+(0x123456))",
                "rgba((0xff000000)|alpha)",
            ],
        )

    def test_token_derived_calls_are_ignored(self) -> None:
        source = '''
fn render(theme: Theme) {
    let _ = rgb(theme.colors.text.primary);
    let _ = rgba((theme.colors.ui.border << 8) | 0x40);
}
'''
        self.assertEqual(calls(source), [])

    def test_narrow_token_owner_path_is_ignored(self) -> None:
        self.assertEqual(inventory.TOKEN_OWNER_PATHS, frozenset({"src/theme/helpers.rs"}))
        self.assertEqual(calls("fn fallback() { rgba(0x10182080); }", "src/theme/helpers.rs"), [])


class HardcodedVisualRatchetTests(unittest.TestCase):
    def test_new_literal_fails(self) -> None:
        additions = inventory.added_calls(calls("fn f() { rgb(0xff0000); }"), [])
        self.assertEqual(additions, [("src/example.rs", "rgb(0xff0000)", 1)])

    def test_duplicate_addition_uses_multiset_counts(self) -> None:
        baseline = calls("fn f() { rgb(0xff0000); }")
        current = calls("fn f() { rgb(0xff0000); rgb(0xff0000); }")
        self.assertEqual(inventory.added_calls(current, baseline)[0][2], 1)

    def test_literal_replacement_fails(self) -> None:
        baseline = calls("fn f() { rgb(0xff0000); }")
        current = calls("fn f() { rgb(0x00ff00); }")
        self.assertEqual(inventory.added_calls(current, baseline)[0][1], "rgb(0x00ff00)")

    def test_non_first_literal_replacement_fails(self) -> None:
        baseline = calls("fn f() { hsla(0., 1., 0.5, 1.); }")
        current = calls("fn f() { hsla(0., 0.5, 0.5, 1.); }")
        self.assertEqual(
            inventory.added_calls(current, baseline)[0][1],
            "hsla(0.,0.5,0.5,1.)",
        )

    def test_literal_formatting_only_change_is_normalized(self) -> None:
        baseline = calls("fn f() { rgba(0xFF_00_00_FF); }")
        current = calls("fn f() { rgba( 0xff0000ff ); }")
        self.assertEqual(inventory.added_calls(current, baseline), [])

    def test_constructor_namespace_only_change_is_normalized(self) -> None:
        baseline = calls("fn f() { rgba(0xff0000ff); }")
        current = calls("fn f() { gpui::rgba(0xff0000ff); }")
        self.assertEqual(inventory.added_calls(current, baseline), [])

    def test_removal_is_allowed(self) -> None:
        baseline = calls("fn f() { rgb(0xff0000); rgba(0x00000080); }")
        current = calls("fn f() { rgb(0xff0000); }")
        self.assertEqual(inventory.added_calls(current, baseline), [])

    def test_rename_maps_to_prior_file_identity(self) -> None:
        baseline = calls("fn f() { rgb(0xff0000); }", "src/old.rs")
        current = calls("fn f() { rgb(0xff0000); }", "src/new.rs")
        self.assertEqual(
            inventory.added_calls(current, baseline, {"src/new.rs": "src/old.rs"}),
            [],
        )


if __name__ == "__main__":
    unittest.main()
