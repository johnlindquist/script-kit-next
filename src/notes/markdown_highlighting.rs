use gpui_component::highlighter::{LanguageConfig, LanguageRegistry};
use std::sync::OnceLock;

use crate::protocol::ElementEditorRuntimeInfo;

static MARKDOWN_HIGHLIGHTER: OnceLock<()> = OnceLock::new();
const MARKDOWN_LANGUAGE: &str = "markdown";
const MARKDOWN_INLINE_LANGUAGE: &str = "markdown_inline";
const MARKDOWN_INJECTION_LANGUAGES: [&str; 3] = ["html", "toml", "yaml"];
const MARKDOWN_HIGHLIGHTS_QUERY: &str = include_str!("markdown_queries/markdown_highlights.scm");
const MARKDOWN_INJECTIONS_QUERY: &str = include_str!("markdown_queries/markdown_injections.scm");
const MARKDOWN_INLINE_HIGHLIGHTS_QUERY: &str =
    include_str!("markdown_queries/markdown_inline_highlights.scm");

pub fn register_markdown_highlighter() {
    MARKDOWN_HIGHLIGHTER.get_or_init(|| {
        let registry = LanguageRegistry::singleton();

        let markdown = LanguageConfig::new(
            MARKDOWN_LANGUAGE,
            tree_sitter_md::LANGUAGE.into(),
            MARKDOWN_INJECTION_LANGUAGES
                .iter()
                .map(|language| (*language).into())
                .collect(),
            MARKDOWN_HIGHLIGHTS_QUERY,
            MARKDOWN_INJECTIONS_QUERY,
            "",
        );
        registry.register(MARKDOWN_LANGUAGE, &markdown);

        let markdown_inline = LanguageConfig::new(
            MARKDOWN_INLINE_LANGUAGE,
            tree_sitter_md::INLINE_LANGUAGE.into(),
            Vec::new(),
            MARKDOWN_INLINE_HIGHLIGHTS_QUERY,
            "",
            "",
        );
        registry.register(MARKDOWN_INLINE_LANGUAGE, &markdown_inline);
    });
}

pub fn markdown_editor_runtime_info() -> ElementEditorRuntimeInfo {
    register_markdown_highlighter();
    let registry = LanguageRegistry::singleton();
    ElementEditorRuntimeInfo {
        owner: crate::components::notes_editor::NOTES_EDITOR_STYLE_OWNER.to_string(),
        language: MARKDOWN_LANGUAGE.to_string(),
        markdown_registered: registry.language(MARKDOWN_LANGUAGE).is_some(),
        markdown_inline_registered: registry.language(MARKDOWN_INLINE_LANGUAGE).is_some(),
        injection_languages: MARKDOWN_INJECTION_LANGUAGES
            .iter()
            .map(|language| (*language).to_string())
            .collect(),
        inline_markdown_injection_disabled: !MARKDOWN_INJECTION_LANGUAGES
            .iter()
            .any(|language| *language == MARKDOWN_INLINE_LANGUAGE)
            && !MARKDOWN_INJECTIONS_QUERY.contains(MARKDOWN_INLINE_LANGUAGE),
        highlight_query_fingerprint: stable_query_fingerprint(MARKDOWN_HIGHLIGHTS_QUERY),
        injection_query_fingerprint: stable_query_fingerprint(MARKDOWN_INJECTIONS_QUERY),
        inline_highlight_query_fingerprint: stable_query_fingerprint(
            MARKDOWN_INLINE_HIGHLIGHTS_QUERY,
        ),
        editor_scroll_metrics: None,
        markdown_link_highlight_ranges: None,
    }
}

fn stable_query_fingerprint(query: &str) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in query.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("fnv1a64:{hash:016x}")
}

#[cfg(test)]
mod tests {
    use super::{markdown_editor_runtime_info, register_markdown_highlighter};
    use gpui_component::highlighter::LanguageRegistry;
    use tree_sitter_highlight::HighlightConfiguration;

    #[test]
    fn test_registers_markdown_languages() {
        register_markdown_highlighter();
        let registry = LanguageRegistry::singleton();
        let markdown = registry.language("markdown");
        let markdown_inline = registry.language("markdown_inline");

        assert!(markdown.is_some(), "markdown language should be registered");
        assert!(
            markdown_inline.is_some(),
            "markdown_inline language should be registered"
        );

        let markdown = markdown.unwrap();
        assert!(
            !markdown.highlights.as_ref().is_empty(),
            "markdown highlight query should not be empty"
        );
        assert!(
            !markdown
                .injection_languages
                .iter()
                .any(|language| language.as_ref() == "markdown_inline"),
            "Editable markdown must not inject markdown_inline for every inline node; \
             gpui-component reparses injections while styling visible lines, which makes \
             long Notes scrolling lag"
        );
        assert!(
            !markdown.injections.as_ref().contains("markdown_inline"),
            "Editable markdown injection query should avoid per-inline markdown reparsing"
        );
    }

    #[test]
    fn test_markdown_queries_compile() {
        HighlightConfiguration::new(
            tree_sitter_md::LANGUAGE.into(),
            "markdown",
            super::MARKDOWN_HIGHLIGHTS_QUERY,
            super::MARKDOWN_INJECTIONS_QUERY,
            "",
        )
        .expect("markdown highlight query should compile");

        HighlightConfiguration::new(
            tree_sitter_md::INLINE_LANGUAGE.into(),
            "markdown_inline",
            super::MARKDOWN_INLINE_HIGHLIGHTS_QUERY,
            "",
            "",
        )
        .expect("markdown_inline highlight query should compile");
    }

    #[test]
    fn test_markdown_runtime_info_matches_editable_highlighter_contract() {
        let runtime = markdown_editor_runtime_info();
        assert_eq!(runtime.owner, "components.notes_editor");
        assert_eq!(runtime.language, "markdown");
        assert!(runtime.markdown_registered);
        assert!(runtime.markdown_inline_registered);
        assert_eq!(runtime.injection_languages, ["html", "toml", "yaml"]);
        assert!(runtime.inline_markdown_injection_disabled);
        assert!(runtime.highlight_query_fingerprint.starts_with("fnv1a64:"));
        assert!(runtime.injection_query_fingerprint.starts_with("fnv1a64:"));
        assert!(runtime
            .inline_highlight_query_fingerprint
            .starts_with("fnv1a64:"));
    }
}
