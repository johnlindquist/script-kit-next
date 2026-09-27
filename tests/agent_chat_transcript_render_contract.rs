const MAIN_VIEW_CHROME_SOURCE: &str = include_str!("../src/components/main_view_chrome.rs");

fn source_between<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    let start_index = source
        .find(start)
        .unwrap_or_else(|| panic!("missing start marker: {}", start));
    let source = &source[start_index..];
    let end_index = source
        .find(end)
        .unwrap_or_else(|| panic!("missing end marker after {}: {}", start, end));
    &source[..end_index]
}

#[test]
fn main_view_main_slot_is_a_flex_column_viewport() {
    let body = source_between(
        MAIN_VIEW_CHROME_SOURCE,
        "pub(crate) fn render_main_view_main_slot(",
        "\n}\n\npub(crate) fn main_view_input_text_inset_left",
    );

    assert!(
        body.contains(".flex_1()")
            && body.contains(".min_h(px(0.))")
            && body.contains(".w_full()")
            && body.contains(".overflow_hidden()"),
        "MainViewMain must remain a bounded viewport"
    );
    assert!(
        body.contains(".flex()") && body.contains(".flex_col()"),
        "MainViewMain must be a flex column so Agent Chat transcript descendants receive real height"
    );

    let flex = body.find(".flex()").expect("missing flex");
    let child = body.find(".child(main)").expect("missing child(main)");
    assert!(
        flex < child,
        "MainViewMain must become a flex container before mounting the Agent Chat body"
    );
}
