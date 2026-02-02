use gpui_component::highlighter::{LanguageConfig, LanguageRegistry};
use std::sync::OnceLock;

static MARKDOWN_HIGHLIGHTER: OnceLock<()> = OnceLock::new();

pub fn register_markdown_highlighter() {
    MARKDOWN_HIGHLIGHTER.get_or_init(|| {
        let registry = LanguageRegistry::singleton();

        let markdown = LanguageConfig::new(
            "markdown",
            tree_sitter_md::LANGUAGE.into(),
            vec![
                "markdown_inline".into(),
                "html".into(),
                "toml".into(),
                "yaml".into(),
            ],
            include_str!("markdown_queries/markdown_highlights.scm"),
            include_str!("markdown_queries/markdown_injections.scm"),
            "",
        );
        registry.register("markdown", &markdown);

        let markdown_inline = LanguageConfig::new(
            "markdown_inline",
            tree_sitter_md::INLINE_LANGUAGE.into(),
            Vec::new(),
            include_str!("markdown_queries/markdown_inline_highlights.scm"),
            "",
            "",
        );
        registry.register("markdown_inline", &markdown_inline);
    });
}

#[cfg(test)]
mod tests {
    use super::register_markdown_highlighter;
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
    }

    #[test]
    fn test_markdown_queries_compile() {
        HighlightConfiguration::new(
            tree_sitter_md::LANGUAGE.into(),
            "markdown",
            include_str!("markdown_queries/markdown_highlights.scm"),
            include_str!("markdown_queries/markdown_injections.scm"),
            "",
        )
        .expect("markdown highlight query should compile");

        HighlightConfiguration::new(
            tree_sitter_md::INLINE_LANGUAGE.into(),
            "markdown_inline",
            include_str!("markdown_queries/markdown_inline_highlights.scm"),
            "",
            "",
        )
        .expect("markdown_inline highlight query should compile");
    }
}
