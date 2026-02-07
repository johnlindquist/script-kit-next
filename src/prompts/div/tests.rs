use super::*;

#[test]
fn test_render_context_from_theme() {
    let colors = theme::ColorScheme::dark_default();
    let ctx = RenderContext::from_theme(&colors);

    assert_eq!(ctx.text_primary, colors.text.primary);
    assert_eq!(ctx.text_secondary, colors.text.secondary);
    assert_eq!(ctx.accent_color, colors.accent.selected);
}

#[test]
fn test_render_simple_text() {
    let elements = parse_html("Hello World");
    let ctx = RenderContext::from_theme(&theme::ColorScheme::dark_default());

    // Should not panic
    let _ = render_elements(&elements, ctx);
}

#[test]
fn test_render_complex_html() {
    let html = r#"
        <h1>Title</h1>
        <p>A paragraph with <strong>bold</strong> and <em>italic</em> text.</p>
        <ul>
            <li>Item 1</li>
            <li>Item 2</li>
        </ul>
        <blockquote>A quote</blockquote>
        <pre><code>let x = 1;</code></pre>
        <hr>
        <a href="https://example.com">Link</a>
    "#;
    let elements = parse_html(html);
    let ctx = RenderContext::from_theme(&theme::ColorScheme::dark_default());

    // Should not panic
    let _ = render_elements(&elements, ctx);
}

#[test]
fn test_render_headers_different_sizes() {
    for level in 1..=6 {
        let html = format!("<h{}>Header {}</h{}>", level, level, level);
        let elements = parse_html(&html);
        let ctx = RenderContext::from_theme(&theme::ColorScheme::dark_default());

        // Should not panic
        let _ = render_elements(&elements, ctx);
    }
}

#[test]
fn test_render_nested_formatting() {
    let html = "<p><strong><em>Bold and italic</em></strong></p>";
    let elements = parse_html(html);
    let ctx = RenderContext::from_theme(&theme::ColorScheme::dark_default());

    // Should not panic
    let _ = render_elements(&elements, ctx);
}

#[test]
fn test_default_container_padding_follows_design_spacing() {
    for variant in [
        DesignVariant::Default,
        DesignVariant::Minimal,
        DesignVariant::Compact,
    ] {
        let expected = get_tokens(variant).spacing().padding_md;
        assert_eq!(default_container_padding(variant), expected);
    }
}

#[test]
fn test_collect_inline_segments_preserves_nested_inline_styles_when_html_contains_formatting() {
    let elements =
        parse_html("<p>Hello <strong>Bold</strong> <em>Italic</em> <code>const x = 1;</code></p>");
    let segments = collect_inline_segments(&elements);

    assert!(segments
        .iter()
        .any(|segment| segment.text == "Bold" && segment.style.bold));
    assert!(segments
        .iter()
        .any(|segment| segment.text == "Italic" && segment.style.italic));
    assert!(segments
        .iter()
        .any(|segment| segment.text == "const x = 1;" && segment.style.code));
}

#[test]
fn test_collect_inline_segments_preserves_link_target_when_html_contains_nested_link_text() {
    let elements =
        parse_html("<p>Open <a href=\"submit:continue\"><strong>Continue</strong></a></p>");
    let segments = collect_inline_segments(&elements);

    assert!(segments.iter().any(|segment| {
        segment.text == "Continue"
            && segment.style.bold
            && segment.style.link_href.as_deref() == Some("submit:continue")
    }));
}

#[test]
fn test_is_div_submit_key_handles_enter_return_escape_and_esc() {
    assert!(is_div_submit_key("enter"));
    assert!(is_div_submit_key("return"));
    assert!(is_div_submit_key("escape"));
    assert!(is_div_submit_key("esc"));
    assert!(!is_div_submit_key("tab"));
}
