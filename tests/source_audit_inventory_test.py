from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import source_audit_inventory as inventory


def sites(source: str, path: str = "tests/example.rs") -> list[inventory.ReaderSite]:
    return inventory.scan_source(path, source)


def targets(source: str, category: str = "app-source-audit") -> list[str]:
    return [site.target for site in sites(source) if site.category == category]


class RustReaderScannerTests(unittest.TestCase):
    def test_multiline_include_delimiters_and_raw_strings(self) -> None:
        source = '''
const A: &str = include_str!(
    "../src/a.rs"
);
const B: &str = include_str! {
    r#"../src/b.rs"#
};
'''
        self.assertEqual(targets(source), ["src/a.rs", "src/b.rs"])

    def test_multiline_read_to_string(self) -> None:
        source = '''
fn check() {
    let _ = std::fs::read_to_string(
        "src/multiline.rs"
    );
}
'''
        self.assertEqual(targets(source), ["src/multiline.rs"])

    def test_comments_and_embedded_rust_fixture_strings_are_ignored(self) -> None:
        source = r'''
// std::fs::read_to_string("src/comment.rs");
/* include_str!("../src/block.rs"); */
const FIXTURE: &str = r#"std::fs::read_to_string("src/fixture.rs")"#;
'''
        self.assertEqual(sites(source), [])

    def test_raw_path_string_is_a_real_target(self) -> None:
        source = 'fn t() { let _ = std::fs::read_to_string(r#"src/raw.rs"#); }'
        self.assertEqual(targets(source), ["src/raw.rs"])

    def test_concat_manifest_dir_is_resolved(self) -> None:
        source = '''
const SOURCE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/concat.rs"
));
'''
        self.assertEqual(targets(source), ["src/concat.rs"])

    def test_path_and_pathbuf_constructors_and_joins_are_resolved(self) -> None:
        source = '''
fn t() {
    let _ = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src").join("buf.rs")
    );
    let _ = std::fs::read_to_string(Path::new("src").join("path.rs"));
}
'''
        self.assertEqual(targets(source), ["src/buf.rs", "src/path.rs"])

    def test_let_and_const_bindings_are_resolved(self) -> None:
        source = '''
const ROOT: &str = "src";
fn t() {
    let path = Path::new(ROOT).join("bound.rs");
    let _ = std::fs::read_to_string(path);
}
'''
        self.assertEqual(targets(source), ["src/bound.rs"])

    def test_same_binding_name_does_not_cross_function_scopes(self) -> None:
        source = '''
fn fixture() {
    let path = "tests/fixtures/case.json";
    let _ = std::fs::read_to_string(path);
}
fn production() {
    let path = "src/real.rs";
    let _ = std::fs::read_to_string(path);
}
'''
        found = sites(source)
        self.assertEqual(
            [(site.category, site.target) for site in found],
            [
                ("fixture-golden-reader", "tests/fixtures/case.json"),
                ("app-source-audit", "src/real.rs"),
            ],
        )

    def test_local_reader_wrapper_calls_are_sites(self) -> None:
        source = '''
fn load(path: &str) -> String {
    std::fs::read_to_string(path).unwrap()
}
fn t() {
    let _ = load("src/wrapped.rs");
}
'''
        wrapped = [site for site in sites(source) if site.reader == "wrapper:load"]
        self.assertEqual([(site.category, site.target) for site in wrapped], [("app-source-audit", "src/wrapped.rs")])

    def test_imported_conventional_reader_wrapper_is_recognized(self) -> None:
        source = '''
use super::read_source as read;
fn t() { let _ = read("src/imported.rs"); }
'''
        self.assertEqual(targets(source), ["src/imported.rs"])

    def test_scan_roots_read_dir_walkdir_and_glob_are_recognized(self) -> None:
        source = '''
const ROOTS: &[&str] = &["src", "tests"];
fn collect(path: &Path) { let _ = std::fs::read_dir(path); }
fn t() {
    for root in ROOTS { collect(Path::new(root)); }
    let _ = WalkDir::new("src/walk");
    let _ = glob::glob("src/**/*.rs");
}
'''
        found = targets(source)
        self.assertIn("src", found)
        self.assertIn("src/walk", found)
        self.assertIn("src/**/*.rs", found)

    def test_mixed_source_and_fixture_file_has_both_categories(self) -> None:
        source = '''
const SOURCE: &str = include_str!("../src/mixed.rs");
const CASE: &str = include_str!("fixtures/mixed.json");
'''
        found = sites(source)
        self.assertEqual({site.category for site in found}, {"app-source-audit", "fixture-golden-reader"})

    def test_absolute_tmp_reads_are_generated_artifacts(self) -> None:
        found = sites('fn t() { let _ = std::fs::read_to_string("/tmp/result.json"); }')
        self.assertEqual(found[0].category, "generated-runtime-artifact-reader")

    def test_new_dynamic_reader_is_guarded(self) -> None:
        baseline: list[inventory.ReaderSite] = []
        current = sites("fn load(path: &Path) { let _ = std::fs::read_to_string(path); }")
        self.assertEqual(current[0].category, "unresolved-reader")
        self.assertEqual(len(inventory.added_guarded_sites(current, baseline)), 1)


class SignatureRatchetTests(unittest.TestCase):
    def test_addition_inside_existing_audit_file_fails(self) -> None:
        baseline = sites('const A: &str = include_str!("../src/a.rs");')
        current = sites(
            'const A: &str = include_str!("../src/a.rs");\n'
            'const B: &str = include_str!("../src/b.rs");'
        )
        additions = inventory.added_guarded_sites(current, baseline)
        self.assertEqual(additions[0][1][1], "src/b.rs")

    def test_removing_one_target_cannot_launder_a_different_addition(self) -> None:
        baseline = sites('const A: &str = include_str!("../src/a.rs");')
        current = sites('const B: &str = include_str!("../src/b.rs");')
        additions = inventory.added_guarded_sites(current, baseline)
        self.assertEqual([(signature[1], count) for _, signature, count in additions], [("src/b.rs", 1)])

    def test_duplicate_signature_uses_multiset_counts(self) -> None:
        baseline = sites('const A: &str = include_str!("../src/a.rs");')
        current = sites(
            'const A: &str = include_str!("../src/a.rs");\n'
            'const B: &str = include_str!("../src/a.rs");'
        )
        additions = inventory.added_guarded_sites(current, baseline)
        self.assertEqual(additions[0][2], 1)

    def test_rename_maps_current_file_to_prior_identity(self) -> None:
        baseline = sites('const A: &str = include_str!("../src/a.rs");', "tests/old.rs")
        current = sites('const A: &str = include_str!("../src/a.rs");', "tests/new.rs")
        self.assertEqual(
            inventory.added_guarded_sites(current, baseline, {"tests/new.rs": "tests/old.rs"}),
            [],
        )


if __name__ == "__main__":
    unittest.main()
