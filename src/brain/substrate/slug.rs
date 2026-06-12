//! Slug generation and deduplication for brain filenames.

use std::path::Path;

use crate::notes::metadata::slugify_note_ref;

/// Lowercase hyphenated slug from arbitrary text.
pub fn slugify(value: &str) -> String {
    slugify_note_ref(value)
}

/// Slug for fragment provenance source labels (e.g. `agent-chat`, `clipboard`).
pub fn source_slug(value: &str) -> String {
    let slug = slugify(value);
    if slug.is_empty() {
        "capture".to_string()
    } else {
        slug
    }
}

/// Return `base_slug`, or `base_slug-2`, `base_slug-3`, … when `exists` matches.
pub fn dedupe_slug(base_slug: &str, exists: impl Fn(&str) -> bool) -> String {
    let base = slugify(base_slug);
    let base = if base.is_empty() {
        "note".to_string()
    } else {
        base
    };

    if !exists(&base) {
        return base;
    }

    let mut suffix = 2;
    loop {
        let candidate = format!("{base}-{suffix}");
        if !exists(&candidate) {
            return candidate;
        }
        suffix += 1;
    }
}

/// Return `base_slug` when no file exists at `dir/{slug}.md`, else dedupe.
pub fn dedupe_slug_in_dir(dir: &Path, base_slug: &str) -> String {
    dedupe_slug(base_slug, |slug| dir.join(format!("{slug}.md")).exists())
}
