use std::fs;

#[test]
fn prompt_footer_uses_footer_button_builder() {
    let content =
        fs::read_to_string("src/components/prompt_footer.rs").expect("read prompt_footer.rs");

    assert!(
        !content.contains("FooterButtonColors"),
        "PromptFooter should not depend on FooterButtonColors"
    );
    assert!(
        content.contains("FooterButton::new("),
        "PromptFooter should construct FooterButton with new()"
    );
    assert!(
        content.contains(".shortcut("),
        "PromptFooter should use the shortcut() builder method"
    );
    assert!(
        content.contains(".id("),
        "PromptFooter should assign an id on FooterButton"
    );
}
