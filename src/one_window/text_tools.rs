//! Local display-only text tools. No provider, file or system pasteboard calls.
use sk_one_window::quick_text::Tool;
/// The first four UUID-v4 bytes are random; version/variant bits are elsewhere.
pub(super) fn random_word() -> u32 {
    let id = uuid::Uuid::new_v4();
    let b = id.as_bytes();
    u32::from_be_bytes([b[0], b[1], b[2], b[3]])
}
pub(super) fn result(tool: Tool, text: &str) -> String {
    match tool {
        Tool::Slug => crate::intent::script::slugify(text),
        Tool::Reverse => sk_one_window::quick_text::reverse(text),
        Tool::ReadingTime => sk_one_window::quick_text::reading_time(text),
        Tool::Whitespace => sk_one_window::quick_text::whitespace(text),
        Tool::Base64 => {
            use base64::Engine as _;
            base64::engine::general_purpose::STANDARD.encode(text.as_bytes())
        }
        Tool::PlainMarkdown => plain_markdown(text),
    }
}
fn plain_markdown(text: &str) -> String {
    use pulldown_cmark::{Event, Parser, TagEnd};
    let mut output = String::new();
    let newline = |o: &mut String| {
        if !o.is_empty() && !o.ends_with('\n') {
            o.push('\n');
        }
    };
    for event in Parser::new_ext(
        text,
        pulldown_cmark::Options::ENABLE_STRIKETHROUGH | pulldown_cmark::Options::ENABLE_TASKLISTS,
    ) {
        match event {
            Event::Text(s) | Event::Code(s) => output.push_str(&s),
            Event::SoftBreak | Event::HardBreak | Event::Rule => newline(&mut output),
            Event::TaskListMarker(true) => output.push_str("Done: "),
            Event::TaskListMarker(false) => output.push_str("To do: "),
            Event::End(
                TagEnd::Paragraph
                | TagEnd::Heading(_)
                | TagEnd::Item
                | TagEnd::CodeBlock
                | TagEnd::BlockQuote(_),
            ) => newline(&mut output),
            _ => (),
        }
    }
    output.truncate(output.trim_end_matches('\n').len());
    output
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn preserves_link_words_and_code_without_formatting_or_fetching() {
        assert_eq!(
            plain_markdown("# Invite\n\n**Meet** [here](https://example.invalid/) with `tea`."),
            "Invite\nMeet here with tea."
        );
        assert_eq!(result(Tool::Base64, "Café"), "Q2Fmw6k=");
    }
}
