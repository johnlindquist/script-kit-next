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
            .contains(&MARKDOWN_INLINE_LANGUAGE)
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

// ── Resolved markdown capture styles (design contract) ────────────────────
//
// The editable Notes capture styling chain is:
// `NotesEditor::new_markdown_pair` → `register_markdown_highlighter()` →
// `InputState::code_editor("markdown")` → gpui-component LanguageRegistry →
// `markdown_highlights.scm` → SyntaxHighlighter → `cx.theme().highlight_theme`
// → highlighted `TextRun`s inside the Input. The highlight theme itself is
// built by the app (`crate::theme::gpui_integration::build_markdown_highlight_theme`)
// and installed by `sync_gpui_component_theme`, so this resolver reads the
// SAME deterministic source the renderer paints from — never screenshot
// samples, never copied literals.
//
// NOTE: capture styles carry color/weight/font-style ONLY. The vendored
// `ThemeStyle`/`gpui::HighlightStyle` have no font-size field ("uniformly
// sized" per gpui) — markdown titles paint at the shared editor font size,
// heavier weight, clipped by the fixed 20px line box.

/// Query capture applied to heading contents (`(atx_heading (inline) @title)`).
pub(crate) const MARKDOWN_TITLE_CAPTURE: &str = "title";
/// Query capture applied to the heading `#` markers (`atx_h*_marker`).
pub(crate) const MARKDOWN_HEADING_MARKER_CAPTURE: &str = "punctuation.special";
/// Query capture applied to bullet/list markers (`list_marker_minus` etc.).
pub(crate) const MARKDOWN_LIST_MARKER_CAPTURE: &str = "punctuation.list_marker";

/// One resolved syntax capture style, exactly as the Input's highlighter
/// receives it from the theme (`SyntaxColors::style`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct ResolvedSyntaxStyle {
    pub color: Option<gpui::Hsla>,
    pub font_weight: Option<f32>,
    pub italic: Option<bool>,
}

impl ResolvedSyntaxStyle {
    fn from_highlight(style: Option<gpui::HighlightStyle>) -> Self {
        match style {
            Some(style) => Self {
                color: style.color,
                font_weight: style.font_weight.map(|weight| weight.0),
                italic: style
                    .font_style
                    .map(|font_style| font_style == gpui::FontStyle::Italic),
            },
            None => Self {
                color: None,
                font_weight: None,
                italic: None,
            },
        }
    }
}

/// The three markdown capture styles the Notes editor paints for the
/// heading/list anatomy visible in the reference fixture.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct ResolvedNotesMarkdownStyles {
    pub title: ResolvedSyntaxStyle,
    pub heading_marker: ResolvedSyntaxStyle,
    pub list_marker: ResolvedSyntaxStyle,
}

/// Resolve the markdown capture styles for a Script Kit theme via the same
/// pure theme → gpui-component highlight-theme conversion the renderer
/// installs (`build_markdown_highlight_theme`). Pure: safe for the
/// checked-in design-contract exporter.
pub(crate) fn resolved_notes_markdown_styles(
    sk_theme: &crate::theme::Theme,
    is_dark: bool,
) -> ResolvedNotesMarkdownStyles {
    let highlight_theme =
        crate::theme::gpui_integration::build_markdown_highlight_theme(sk_theme, is_dark);
    let resolve =
        |name: &str| ResolvedSyntaxStyle::from_highlight(highlight_theme.style.syntax.style(name));
    ResolvedNotesMarkdownStyles {
        title: resolve(MARKDOWN_TITLE_CAPTURE),
        heading_marker: resolve(MARKDOWN_HEADING_MARKER_CAPTURE),
        list_marker: resolve(MARKDOWN_LIST_MARKER_CAPTURE),
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
    fn test_highlight_query_binds_the_contract_captures() {
        // The design contract exports styles for exactly these captures; if
        // the query stops binding them the resolved tokens become fiction.
        let query = super::MARKDOWN_HIGHLIGHTS_QUERY;
        assert!(query.contains("@title"), "query must capture @title");
        assert!(
            query.contains("@punctuation.special"),
            "query must capture @punctuation.special (heading markers)"
        );
        assert!(
            query.contains("@punctuation.list_marker"),
            "query must capture @punctuation.list_marker"
        );
    }

    #[test]
    fn test_stock_dark_markdown_styles_resolve_from_the_real_highlight_theme() {
        let theme = crate::theme::presets::all_presets()
            .into_iter()
            .find(|preset| preset.id == "script-kit-dark")
            .expect("script-kit-dark preset")
            .create_theme();
        let styles = super::resolved_notes_markdown_styles(&theme, true);

        let rgba = |color: Option<gpui::Hsla>| -> u32 {
            let rgba: gpui::Rgba = color.expect("capture resolves a color").into();
            let byte = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u32;
            (byte(rgba.r) << 24) | (byte(rgba.g) << 16) | (byte(rgba.b) << 8) | byte(rgba.a)
        };

        // Title: accent (#fbbf24), bold — the same authority as
        // build_markdown_highlight_theme's `syntax.title` override.
        assert_eq!(rgba(styles.title.color), 0xFBBF24FF);
        assert_eq!(styles.title.font_weight, Some(gpui::FontWeight::BOLD.0));
        // Heading `#` marker: text.muted (white in the stock preset), no
        // weight override — a SEPARATE capture from the title.
        assert_eq!(rgba(styles.heading_marker.color), 0xFFFFFFFF);
        assert_eq!(styles.heading_marker.font_weight, None);
        // List markers: accent, bold.
        assert_eq!(rgba(styles.list_marker.color), 0xFBBF24FF);
        assert_eq!(
            styles.list_marker.font_weight,
            Some(gpui::FontWeight::BOLD.0)
        );
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
