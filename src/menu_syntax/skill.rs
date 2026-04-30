//! Skill spec metadata extraction.
//!
//! Scripts and scriptlets register `/slug` skills via `menuSyntax` entries
//! with `family: "skill.v1"`. This module surfaces the typed Rust shape for
//! launcher consumers (the `:type:skill` filter and the inline
//! `Suggested skills` UI) without forcing every consumer to pull in
//! `serde_json::Value` parsing.
//!
//! Story: sdk-skill-spec-metadata. The runtime UI integration
//! (`:type:skill review` filtering, unknown-`!command` hint) lives in a
//! follow-up pass; this module is the pure data layer.

use serde_json::Value;

/// One `skill.v1` registration extracted from a `menuSyntax` array.
///
/// Mirrors the TypeScript `SkillHandlerSpec` from
/// `kit-init/types/menu-syntax.d.ts` — keep the field set in sync.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SkillSpec {
    pub slug: String,
    pub label: Option<String>,
    pub description: Option<String>,
    pub context_requirements: Vec<String>,
    pub accepts_capture_target: Option<String>,
}

/// Extract `SkillSpec` rows from a raw `menuSyntax` JSON value (array-shape
/// or single-object shape). Only entries with `family == "skill.v1"` are
/// returned. Entries missing or with empty `slug` are skipped silently —
/// the doctor surface ([[src/menu_syntax/doctor.rs]]) is what flags
/// authoring mistakes; this extractor is intentionally permissive so the
/// launcher can still surface valid sibling specs when one is malformed.
pub fn skill_specs_from_value(value: &Value) -> Vec<SkillSpec> {
    let entries: Vec<&Value> = match value {
        Value::Array(arr) => arr.iter().collect(),
        Value::Object(_) => vec![value],
        _ => return vec![],
    };
    let mut out = Vec::new();
    for entry in entries {
        let obj = match entry.as_object() {
            Some(o) => o,
            None => continue,
        };
        // Family check is case-insensitive: an author writing "SKILL.V1"
        // (autocorrect / copy-paste / capitalization mistake) should not
        // silently lose their skill registration. Closes Run 11 Pass 20 [?]
        // `skill-spec-family-filter-case-sensitive`. Other modules in the
        // codebase normalize case before comparison; this one now matches.
        let family_ok = obj
            .get("family")
            .and_then(|v| v.as_str())
            .map(|s| s.eq_ignore_ascii_case("skill.v1"))
            .unwrap_or(false);
        if !family_ok {
            continue;
        }
        let slug = match obj.get("slug").and_then(|v| v.as_str()) {
            Some(s) if !s.trim().is_empty() => s.trim().to_string(),
            _ => continue,
        };
        let label = obj.get("label").and_then(|v| v.as_str()).map(String::from);
        let description = obj
            .get("description")
            .and_then(|v| v.as_str())
            .map(String::from);
        let context_requirements = obj
            .get("contextRequirements")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|e| e.as_str())
                    .map(String::from)
                    .collect()
            })
            .unwrap_or_default();
        let accepts_capture_target = obj
            .get("acceptsCaptureTarget")
            .and_then(|v| v.as_str())
            .map(String::from);
        out.push(SkillSpec {
            slug,
            label,
            description,
            context_requirements,
            accepts_capture_target,
        });
    }
    // Dedupe by slug keeping the FIRST occurrence — declaration-order wins.
    // Closes Run 11 Pass 20 [?] `skill-spec-no-dedup-on-duplicate-slugs`. The
    // doctor surface ([[src/menu_syntax/doctor.rs]]) is responsible for
    // warning the author about duplicates; this extractor's job is to give
    // the runtime UI a clean Vec so `:type:skill` doesn't render two
    // identical rows.
    let mut seen: Vec<String> = Vec::with_capacity(out.len());
    out.retain(|s| {
        if seen.iter().any(|prev| prev == &s.slug) {
            false
        } else {
            seen.push(s.slug.clone());
            true
        }
    });
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn single_object_with_skill_family_extracts_spec() {
        let v = json!({
            "family": "skill.v1",
            "slug": "review",
            "label": "Review the current file",
            "description": "Run a code review pass on the file in front of you",
            "contextRequirements": ["currentFile"],
        });
        let specs = skill_specs_from_value(&v);
        assert_eq!(specs.len(), 1);
        assert_eq!(specs[0].slug, "review");
        assert_eq!(specs[0].label.as_deref(), Some("Review the current file"));
        assert_eq!(specs[0].context_requirements, vec!["currentFile"]);
    }

    #[test]
    fn array_form_extracts_only_skill_v1_entries() {
        let v = json!([
            { "family": "capture.v1", "targets": ["todo"] },
            { "family": "skill.v1", "slug": "review" },
            { "family": "command.v1", "head": "deploy" },
            { "family": "skill.v1", "slug": "summarize", "description": "summarize selection" },
        ]);
        let specs = skill_specs_from_value(&v);
        assert_eq!(specs.len(), 2);
        assert_eq!(specs[0].slug, "review");
        assert_eq!(specs[1].slug, "summarize");
        assert_eq!(specs[1].description.as_deref(), Some("summarize selection"));
    }

    #[test]
    fn missing_slug_is_skipped() {
        let v = json!([
            { "family": "skill.v1" },
            { "family": "skill.v1", "slug": "" },
            { "family": "skill.v1", "slug": "  " },
            { "family": "skill.v1", "slug": "valid" },
        ]);
        let specs = skill_specs_from_value(&v);
        assert_eq!(specs.len(), 1);
        assert_eq!(specs[0].slug, "valid");
    }

    #[test]
    fn slug_is_trimmed() {
        let v = json!({ "family": "skill.v1", "slug": "  review  " });
        let specs = skill_specs_from_value(&v);
        assert_eq!(specs[0].slug, "review");
    }

    #[test]
    fn context_requirements_filters_non_string_entries() {
        let v = json!({
            "family": "skill.v1",
            "slug": "review",
            "contextRequirements": ["currentFile", 42, null, "selection"],
        });
        let specs = skill_specs_from_value(&v);
        assert_eq!(
            specs[0].context_requirements,
            vec!["currentFile", "selection"]
        );
    }

    #[test]
    fn missing_optional_fields_default_to_none_or_empty() {
        let v = json!({ "family": "skill.v1", "slug": "minimal" });
        let specs = skill_specs_from_value(&v);
        let s = &specs[0];
        assert_eq!(s.slug, "minimal");
        assert!(s.label.is_none());
        assert!(s.description.is_none());
        assert!(s.context_requirements.is_empty());
        assert!(s.accepts_capture_target.is_none());
    }

    #[test]
    fn accepts_capture_target_passes_through() {
        let v = json!({
            "family": "skill.v1",
            "slug": "log-todo",
            "acceptsCaptureTarget": "todo",
        });
        let specs = skill_specs_from_value(&v);
        assert_eq!(specs[0].accepts_capture_target.as_deref(), Some("todo"));
    }

    #[test]
    fn non_object_entries_yield_empty() {
        assert!(skill_specs_from_value(&json!("string")).is_empty());
        assert!(skill_specs_from_value(&json!(42)).is_empty());
        assert!(skill_specs_from_value(&json!(null)).is_empty());
        assert!(skill_specs_from_value(&json!([1, 2, 3])).is_empty());
    }

    #[test]
    fn wrong_family_is_filtered() {
        let v = json!({ "family": "capture.v1", "slug": "review" });
        assert!(skill_specs_from_value(&v).is_empty());
    }

    #[test]
    fn family_match_is_case_insensitive() {
        // Closes Run 11 Pass 20 [?] skill-spec-family-filter-case-sensitive.
        for variant in ["skill.v1", "SKILL.V1", "Skill.V1", "skill.V1", "sKiLl.v1"] {
            let v = json!({ "family": variant, "slug": "review" });
            let specs = skill_specs_from_value(&v);
            assert_eq!(specs.len(), 1, "family `{variant}` should match");
            assert_eq!(specs[0].slug, "review");
        }
        // Falsifier guard: a typo with extra char must NOT match.
        let v = json!({ "family": "skill.v11", "slug": "review" });
        assert!(skill_specs_from_value(&v).is_empty());
    }

    #[test]
    fn duplicate_slugs_are_deduped_keeping_first() {
        // Closes Run 11 Pass 20 [?] skill-spec-no-dedup-on-duplicate-slugs.
        // Two `{family:"skill.v1",slug:"review"}` entries must collapse to
        // one in the returned Vec. The FIRST occurrence wins so
        // declaration-order is the authoring contract.
        let v = json!([
            { "family": "skill.v1", "slug": "review", "label": "First" },
            { "family": "skill.v1", "slug": "review", "label": "Second" },
        ]);
        let specs = skill_specs_from_value(&v);
        assert_eq!(specs.len(), 1);
        assert_eq!(specs[0].label.as_deref(), Some("First"));
    }

    #[test]
    fn dedup_preserves_distinct_slugs() {
        // Falsifier for over-dedup: distinct slugs must all surface.
        let v = json!([
            { "family": "skill.v1", "slug": "review" },
            { "family": "skill.v1", "slug": "summarize" },
            { "family": "skill.v1", "slug": "explain" },
        ]);
        let specs = skill_specs_from_value(&v);
        assert_eq!(specs.len(), 3);
        assert_eq!(
            specs.iter().map(|s| s.slug.as_str()).collect::<Vec<_>>(),
            vec!["review", "summarize", "explain"]
        );
    }

    #[test]
    fn dedup_treats_trimmed_slug_as_same() {
        // `slug: "  review  "` trims to "review" before dedup, so a
        // whitespace-padded duplicate still collapses to one entry.
        let v = json!([
            { "family": "skill.v1", "slug": "review" },
            { "family": "skill.v1", "slug": "  review  " },
        ]);
        let specs = skill_specs_from_value(&v);
        assert_eq!(specs.len(), 1);
        assert_eq!(specs[0].slug, "review");
    }
}
