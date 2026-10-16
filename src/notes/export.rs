//! A note as HTML or plain text, for "Copy as HTML" and "Copy as Plain
//! Text", plus its word and character counts.

use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};

fn options() -> Options {
    Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TASKLISTS
}

fn escape(text: &str, out: &mut String) {
    for ch in text.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(ch),
        }
    }
}

fn heading_tag(level: HeadingLevel) -> &'static str {
    match level {
        HeadingLevel::H1 => "h1",
        HeadingLevel::H2 => "h2",
        HeadingLevel::H3 => "h3",
        HeadingLevel::H4 => "h4",
        HeadingLevel::H5 => "h5",
        HeadingLevel::H6 => "h6",
    }
}

/// The note's Markdown as an HTML fragment (headings, emphasis, links,
/// lists and checklists, quotes, code, tables, rules, images).
pub fn markdown_to_html(markdown: &str) -> String {
    let mut out = String::with_capacity(markdown.len() * 2);
    let mut in_table_head = false;
    for event in Parser::new_ext(markdown, options()) {
        match event {
            Event::Start(tag) => match tag {
                Tag::Paragraph => out.push_str("<p>"),
                Tag::Heading { level, .. } => {
                    out.push('<');
                    out.push_str(heading_tag(level));
                    out.push('>');
                }
                Tag::BlockQuote(_) => out.push_str("<blockquote>\n"),
                Tag::CodeBlock(kind) => {
                    out.push_str("<pre><code");
                    if let CodeBlockKind::Fenced(lang) = kind {
                        let lang = lang.split_whitespace().next().unwrap_or_default();
                        if !lang.is_empty() {
                            out.push_str(" class=\"language-");
                            escape(lang, &mut out);
                            out.push('"');
                        }
                    }
                    out.push('>');
                }
                Tag::List(Some(1)) => out.push_str("<ol>\n"),
                Tag::List(Some(start)) => out.push_str(&format!("<ol start=\"{start}\">\n")),
                Tag::List(None) => out.push_str("<ul>\n"),
                Tag::Item => out.push_str("<li>"),
                Tag::Emphasis => out.push_str("<em>"),
                Tag::Strong => out.push_str("<strong>"),
                Tag::Strikethrough => out.push_str("<del>"),
                Tag::Link { dest_url, .. } => {
                    out.push_str("<a href=\"");
                    escape(&dest_url, &mut out);
                    out.push_str("\">");
                }
                Tag::Image { dest_url, .. } => {
                    out.push_str("<img src=\"");
                    escape(&dest_url, &mut out);
                    out.push_str("\" alt=\"");
                }
                Tag::Table(_) => out.push_str("<table>\n"),
                Tag::TableHead => {
                    in_table_head = true;
                    out.push_str("<thead><tr>");
                }
                Tag::TableRow => out.push_str("<tr>"),
                Tag::TableCell => out.push_str(if in_table_head { "<th>" } else { "<td>" }),
                _ => {}
            },
            Event::End(tag) => match tag {
                TagEnd::Paragraph => out.push_str("</p>\n"),
                TagEnd::Heading(level) => {
                    out.push_str("</");
                    out.push_str(heading_tag(level));
                    out.push_str(">\n");
                }
                TagEnd::BlockQuote(_) => out.push_str("</blockquote>\n"),
                TagEnd::CodeBlock => out.push_str("</code></pre>\n"),
                TagEnd::List(true) => out.push_str("</ol>\n"),
                TagEnd::List(false) => out.push_str("</ul>\n"),
                TagEnd::Item => out.push_str("</li>\n"),
                TagEnd::Emphasis => out.push_str("</em>"),
                TagEnd::Strong => out.push_str("</strong>"),
                TagEnd::Strikethrough => out.push_str("</del>"),
                TagEnd::Link => out.push_str("</a>"),
                TagEnd::Image => out.push_str("\" />"),
                TagEnd::Table => out.push_str("</tbody></table>\n"),
                TagEnd::TableHead => {
                    in_table_head = false;
                    out.push_str("</tr></thead><tbody>\n");
                }
                TagEnd::TableRow => out.push_str("</tr>\n"),
                TagEnd::TableCell => out.push_str(if in_table_head { "</th>" } else { "</td>" }),
                _ => {}
            },
            Event::Text(text) => escape(&text, &mut out),
            Event::Code(code) => {
                out.push_str("<code>");
                escape(&code, &mut out);
                out.push_str("</code>");
            }
            Event::SoftBreak => out.push('\n'),
            Event::HardBreak => out.push_str("<br />\n"),
            Event::Rule => out.push_str("<hr />\n"),
            Event::TaskListMarker(done) => out.push_str(if done {
                "<input type=\"checkbox\" checked disabled /> "
            } else {
                "<input type=\"checkbox\" disabled /> "
            }),
            Event::Html(html) | Event::InlineHtml(html) => escape(&html, &mut out),
            _ => {}
        }
    }
    out.trim_end().to_string()
}

/// The note's text without Markdown syntax: headings and paragraphs as
/// lines, list items with "• " (or "☐ "/"☑ " for checklists), links as their
/// text, code as written.
pub fn markdown_to_plain_text(markdown: &str) -> String {
    let mut out = String::with_capacity(markdown.len());
    let mut lists: Vec<Option<u64>> = Vec::new();
    let mut pending_bullet = false;
    let mut in_table = false;
    let newline = |out: &mut String| {
        if !out.is_empty() && !out.ends_with('\n') {
            out.push('\n');
        }
    };
    for event in Parser::new_ext(markdown, options()) {
        match event {
            Event::Start(Tag::List(start)) => {
                newline(&mut out);
                lists.push(start);
            }
            Event::End(TagEnd::List(_)) => {
                lists.pop();
                newline(&mut out);
                if lists.is_empty() {
                    out.push('\n');
                }
            }
            Event::Start(Tag::Item) => {
                newline(&mut out);
                out.push_str(&"  ".repeat(lists.len().saturating_sub(1)));
                match lists.last_mut() {
                    Some(Some(number)) => {
                        out.push_str(&format!("{number}. "));
                        *number += 1;
                    }
                    _ => pending_bullet = true,
                }
            }
            Event::TaskListMarker(done) => {
                pending_bullet = false;
                out.push_str(if done { "\u{2611} " } else { "\u{2610} " });
            }
            Event::Text(text) | Event::Code(text) | Event::InlineHtml(text) | Event::Html(text) => {
                if pending_bullet {
                    out.push_str("\u{2022} ");
                    pending_bullet = false;
                }
                out.push_str(&text);
            }
            Event::SoftBreak | Event::HardBreak => out.push('\n'),
            Event::End(
                TagEnd::Paragraph
                | TagEnd::Heading(_)
                | TagEnd::CodeBlock
                | TagEnd::TableHead
                | TagEnd::TableRow,
            ) => {
                newline(&mut out);
                if lists.is_empty() && !in_table {
                    out.push('\n');
                }
            }
            Event::Start(Tag::Table(_)) => in_table = true,
            Event::End(TagEnd::Table) => {
                in_table = false;
                out.push('\n');
            }
            Event::End(TagEnd::TableCell) => out.push('\t'),
            Event::Rule => {
                newline(&mut out);
                out.push_str("\u{2014}\n\n");
            }
            _ => {}
        }
    }
    let lines: Vec<&str> = out
        .lines()
        .map(|line| line.trim_end_matches([' ', '\t']))
        .collect();
    let mut text = lines.join("\n");
    while text.contains("\n\n\n") {
        text = text.replace("\n\n\n", "\n\n");
    }
    text.trim().to_string()
}

/// Words and characters in the note's text (Markdown syntax not counted).
pub fn text_counts(markdown: &str) -> (usize, usize) {
    let text = markdown_to_plain_text(markdown);
    let words = text.split_whitespace().count();
    let characters = text.chars().filter(|ch| *ch != '\n').count();
    (words, characters)
}

#[cfg(test)]
mod tests {
    use super::*;

    const NOTE: &str = "# Plan\n\nShip **today** with [docs](https://x.dev?a=1&b=2).\n\n- [ ] write\n- [x] test\n\n1. one\n2. two\n\n> careful <here>\n\n```rust\nlet a = 1;\n```\n\n| A | B |\n|---|---|\n| 1 | 2 |\n";

    #[test]
    fn notes_become_html() {
        let html = markdown_to_html(NOTE);
        assert!(html.starts_with("<h1>Plan</h1>"), "{html}");
        assert!(html.contains("<strong>today</strong>"));
        assert!(html.contains("<a href=\"https://x.dev?a=1&amp;b=2\">docs</a>"));
        assert!(html.contains("<input type=\"checkbox\" disabled /> write"));
        assert!(html.contains("<input type=\"checkbox\" checked disabled /> test"));
        assert!(html.contains("<ol>\n<li>one</li>"));
        assert!(html.contains("<blockquote>\n<p>careful &lt;here&gt;</p>"));
        assert!(html.contains("<pre><code class=\"language-rust\">let a = 1;\n</code></pre>"));
        assert!(html.contains("<thead><tr><th>A</th><th>B</th></tr></thead><tbody>"));
        assert!(html.contains("<tr><td>1</td><td>2</td></tr>"));
    }

    #[test]
    fn notes_become_plain_text() {
        let text = markdown_to_plain_text(NOTE);
        assert_eq!(
            text,
            "Plan\n\nShip today with docs.\n\n\u{2610} write\n\u{2611} test\n\n1. one\n2. two\n\ncareful <here>\n\nlet a = 1;\n\nA\tB\n1\t2"
        );
        assert_eq!(
            markdown_to_plain_text("- a\n  - b\n- c"),
            "\u{2022} a\n  \u{2022} b\n\u{2022} c"
        );
    }

    #[test]
    fn counts_skip_markdown_syntax() {
        assert_eq!(text_counts("# Hello **big** world"), (3, 15));
        assert_eq!(text_counts(""), (0, 0));
    }
}
