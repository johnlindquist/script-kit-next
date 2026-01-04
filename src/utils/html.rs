//! HTML parsing and element types for Script Kit GPUI

/// Strip HTML tags from a string, returning plain text.
///
/// This function removes all HTML tags (content between < and >) and normalizes
/// whitespace. Consecutive whitespace is collapsed to a single space, and the
/// result is trimmed.
///
/// # Examples
///
/// ```
/// use script_kit_gpui::utils::strip_html_tags;
///
/// assert_eq!(strip_html_tags("<p>Hello</p>"), "Hello");
/// assert_eq!(strip_html_tags("<div><span>A</span><span>B</span></div>"), "A B");
/// ```
#[allow(dead_code)]
pub fn strip_html_tags(html: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;
    let mut pending_space = false;

    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
                pending_space = true; // Add space between tags
            }
            _ if !in_tag => {
                if ch.is_whitespace() {
                    if !result.is_empty() && !result.ends_with(' ') {
                        pending_space = true;
                    }
                } else {
                    if pending_space && !result.is_empty() {
                        result.push(' ');
                    }
                    pending_space = false;
                    result.push(ch);
                }
            }
            _ => {} // Skip characters inside tags
        }
    }

    result.trim().to_string()
}

/// Represents a parsed HTML element with its type and content
#[derive(Debug, Clone, PartialEq)]
pub enum HtmlElement {
    /// Plain text content
    Text(String),
    /// Header (h1-h6) with level (1-6) and content
    Header {
        level: u8,
        children: Vec<HtmlElement>,
    },
    /// Paragraph
    Paragraph(Vec<HtmlElement>),
    /// Bold/Strong text
    Bold(Vec<HtmlElement>),
    /// Italic/Emphasis text
    Italic(Vec<HtmlElement>),
    /// Inline code with monospace styling
    InlineCode(String),
    /// Code block with optional language
    CodeBlock {
        language: Option<String>,
        code: String,
    },
    /// Unordered list
    UnorderedList(Vec<HtmlElement>),
    /// Ordered list
    OrderedList(Vec<HtmlElement>),
    /// List item
    ListItem(Vec<HtmlElement>),
    /// Blockquote
    Blockquote(Vec<HtmlElement>),
    /// Horizontal rule
    HorizontalRule,
    /// Link with href and text
    Link {
        href: String,
        children: Vec<HtmlElement>,
    },
    /// Line break
    LineBreak,
    /// Div container with optional class attribute
    Div {
        classes: Option<String>,
        children: Vec<HtmlElement>,
    },
    /// Span inline container with optional class attribute
    Span {
        classes: Option<String>,
        children: Vec<HtmlElement>,
    },
}

/// Parser state for HTML parsing
struct HtmlParser<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> HtmlParser<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    fn remaining(&self) -> &str {
        &self.input[self.pos..]
    }

    fn is_eof(&self) -> bool {
        self.pos >= self.input.len()
    }

    fn peek_char(&self) -> Option<char> {
        self.remaining().chars().next()
    }

    /// Advance by n bytes, ensuring we land on a valid UTF-8 char boundary
    fn advance(&mut self, n: usize) {
        let new_pos = (self.pos + n).min(self.input.len());
        // Ensure we're at a valid char boundary
        self.pos = self.find_char_boundary(new_pos);
    }

    /// Advance by one character (handles multi-byte UTF-8)
    fn advance_char(&mut self) {
        if let Some(c) = self.peek_char() {
            self.pos = (self.pos + c.len_utf8()).min(self.input.len());
        }
    }

    /// Find the nearest valid char boundary at or after the given position
    fn find_char_boundary(&self, pos: usize) -> usize {
        let mut p = pos.min(self.input.len());
        // Move forward until we hit a char boundary
        while p < self.input.len() && !self.input.is_char_boundary(p) {
            p += 1;
        }
        p.min(self.input.len())
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek_char() {
            if c.is_whitespace() {
                self.advance_char();
            } else {
                break;
            }
        }
    }

    /// Check if the remaining input starts with a string (case-insensitive for tags)
    fn starts_with_ci(&self, s: &str) -> bool {
        self.remaining()
            .get(..s.len())
            .map(|prefix| prefix.eq_ignore_ascii_case(s))
            .unwrap_or(false)
    }

    /// Parse an opening tag, returns (tag_name, attributes) or None
    fn parse_opening_tag(&mut self) -> Option<(String, Vec<(String, String)>)> {
        if !self.remaining().starts_with('<') {
            return None;
        }

        // Check for closing tag, comment, or doctype
        if self.remaining().starts_with("</")
            || self.remaining().starts_with("<!--")
            || self.remaining().starts_with("<!")
        {
            return None;
        }

        self.advance_char(); // skip '<'

        // Parse tag name
        let mut tag_name = String::new();
        while let Some(c) = self.peek_char() {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                tag_name.push(c.to_ascii_lowercase());
                self.advance_char();
            } else {
                break;
            }
        }

        if tag_name.is_empty() {
            return None;
        }

        // Parse attributes
        let mut attributes = Vec::new();
        loop {
            self.skip_whitespace();

            if let Some(c) = self.peek_char() {
                if c == '>' {
                    self.advance_char();
                    break;
                }
                if c == '/' {
                    // Self-closing tag
                    self.advance_char();
                    self.skip_whitespace();
                    if self.peek_char() == Some('>') {
                        self.advance_char();
                    }
                    break;
                }

                // Parse attribute name
                let mut attr_name = String::new();
                while let Some(c) = self.peek_char() {
                    if c.is_alphanumeric() || c == '-' || c == '_' {
                        attr_name.push(c.to_ascii_lowercase());
                        self.advance_char();
                    } else {
                        break;
                    }
                }

                if attr_name.is_empty() {
                    self.advance_char(); // Skip unknown character
                    continue;
                }

                self.skip_whitespace();

                let attr_value = if self.peek_char() == Some('=') {
                    self.advance_char(); // skip '='
                    self.skip_whitespace();
                    self.parse_attribute_value()
                } else {
                    String::new()
                };

                attributes.push((attr_name, attr_value));
            } else {
                break;
            }
        }

        Some((tag_name, attributes))
    }

    fn parse_attribute_value(&mut self) -> String {
        let quote_char = self.peek_char();
        if quote_char == Some('"') || quote_char == Some('\'') {
            let quote = quote_char.unwrap();
            self.advance_char(); // skip opening quote

            let mut value = String::new();
            while let Some(c) = self.peek_char() {
                if c == quote {
                    self.advance_char();
                    break;
                }
                value.push(c);
                self.advance_char();
            }
            value
        } else {
            // Unquoted value
            let mut value = String::new();
            while let Some(c) = self.peek_char() {
                if c.is_whitespace() || c == '>' || c == '/' {
                    break;
                }
                value.push(c);
                self.advance_char();
            }
            value
        }
    }

    /// Parse a closing tag, returns tag name or None
    fn parse_closing_tag(&mut self, expected: &str) -> bool {
        let close_tag = format!("</{}>", expected);
        if self.starts_with_ci(&close_tag) {
            self.advance(close_tag.len());
            true
        } else {
            false
        }
    }

    /// Parse text content until a tag is encountered
    fn parse_text(&mut self) -> String {
        let mut text = String::new();
        while let Some(c) = self.peek_char() {
            if c == '<' {
                break;
            }
            text.push(c);
            self.advance_char();
        }

        // Decode common HTML entities
        decode_html_entities(&text)
    }

    /// Parse children until a closing tag is found
    fn parse_children(&mut self, end_tag: &str) -> Vec<HtmlElement> {
        let mut children = Vec::new();

        while !self.is_eof() {
            // Check for closing tag
            let close_tag = format!("</{}>", end_tag);
            if self.starts_with_ci(&close_tag) {
                self.advance(close_tag.len());
                break;
            }

            if let Some(element) = self.parse_element() {
                children.push(element);
            } else if !self.is_eof() {
                // If we can't parse an element, try to parse text
                let text = self.parse_text();
                if !text.is_empty() {
                    children.push(HtmlElement::Text(text));
                }
            }
        }

        children
    }

    /// Parse a single element (tag or text)
    fn parse_element(&mut self) -> Option<HtmlElement> {
        if self.is_eof() {
            return None;
        }

        // Check for text first
        if self.peek_char() != Some('<') {
            let text = self.parse_text();
            if !text.is_empty() {
                return Some(HtmlElement::Text(text));
            }
            return None;
        }

        // Check for closing tag (shouldn't happen at this level)
        if self.remaining().starts_with("</") {
            return None;
        }

        // Check for comment
        if self.remaining().starts_with("<!--") {
            // Skip comment
            if let Some(end) = self.remaining().find("-->") {
                self.advance(end + 3);
            }
            return self.parse_element();
        }

        // Parse opening tag
        let (tag_name, attributes) = self.parse_opening_tag()?;

        // Handle self-closing tags
        match tag_name.as_str() {
            "br" => return Some(HtmlElement::LineBreak),
            "hr" => return Some(HtmlElement::HorizontalRule),
            _ => {}
        }

        // Parse children and closing tag
        match tag_name.as_str() {
            "h1" => {
                let children = self.parse_children("h1");
                Some(HtmlElement::Header { level: 1, children })
            }
            "h2" => {
                let children = self.parse_children("h2");
                Some(HtmlElement::Header { level: 2, children })
            }
            "h3" => {
                let children = self.parse_children("h3");
                Some(HtmlElement::Header { level: 3, children })
            }
            "h4" => {
                let children = self.parse_children("h4");
                Some(HtmlElement::Header { level: 4, children })
            }
            "h5" => {
                let children = self.parse_children("h5");
                Some(HtmlElement::Header { level: 5, children })
            }
            "h6" => {
                let children = self.parse_children("h6");
                Some(HtmlElement::Header { level: 6, children })
            }
            "p" => {
                let children = self.parse_children("p");
                Some(HtmlElement::Paragraph(children))
            }
            "strong" | "b" => {
                let children = self.parse_children(&tag_name);
                Some(HtmlElement::Bold(children))
            }
            "em" | "i" => {
                let children = self.parse_children(&tag_name);
                Some(HtmlElement::Italic(children))
            }
            "code" => {
                // Check if parent is a pre (handled at pre level)
                let text = self.parse_text();
                self.parse_closing_tag("code");
                Some(HtmlElement::InlineCode(text))
            }
            "pre" => {
                // Look for code block inside
                let saved_pos = self.pos;
                if let Some((inner_tag, inner_attrs)) = self.parse_opening_tag() {
                    if inner_tag == "code" {
                        let language =
                            inner_attrs
                                .iter()
                                .find(|(k, _)| k == "class")
                                .and_then(|(_, v)| {
                                    // Extract language from class like "language-typescript"
                                    v.strip_prefix("language-").map(|s| s.to_string())
                                });
                        let code = self.parse_text();
                        self.parse_closing_tag("code");
                        self.parse_closing_tag("pre");
                        return Some(HtmlElement::CodeBlock { language, code });
                    }
                }
                // No code tag, just pre content
                self.pos = saved_pos;
                let code = self.parse_text();
                self.parse_closing_tag("pre");
                Some(HtmlElement::CodeBlock {
                    language: None,
                    code,
                })
            }
            "ul" => {
                let children = self.parse_children("ul");
                Some(HtmlElement::UnorderedList(children))
            }
            "ol" => {
                let children = self.parse_children("ol");
                Some(HtmlElement::OrderedList(children))
            }
            "li" => {
                let children = self.parse_children("li");
                Some(HtmlElement::ListItem(children))
            }
            "blockquote" => {
                let children = self.parse_children("blockquote");
                Some(HtmlElement::Blockquote(children))
            }
            "a" => {
                let href = attributes
                    .iter()
                    .find(|(k, _)| k == "href")
                    .map(|(_, v)| v.clone())
                    .unwrap_or_default();
                let children = self.parse_children("a");
                Some(HtmlElement::Link { href, children })
            }
            "div" => {
                let classes = attributes
                    .iter()
                    .find(|(k, _)| k == "class")
                    .map(|(_, v)| v.clone());
                let children = self.parse_children("div");
                Some(HtmlElement::Div { classes, children })
            }
            "span" => {
                let classes = attributes
                    .iter()
                    .find(|(k, _)| k == "class")
                    .map(|(_, v)| v.clone());
                let children = self.parse_children("span");
                Some(HtmlElement::Span { classes, children })
            }
            _ => {
                // Unknown tag - try to parse as container
                let children = self.parse_children(&tag_name);
                if children.is_empty() {
                    None
                } else if children.len() == 1 {
                    Some(children.into_iter().next().unwrap())
                } else {
                    Some(HtmlElement::Div {
                        classes: None,
                        children,
                    })
                }
            }
        }
    }

    /// Parse the entire HTML document
    fn parse(&mut self) -> Vec<HtmlElement> {
        let mut elements = Vec::new();

        while !self.is_eof() {
            self.skip_whitespace();
            if let Some(element) = self.parse_element() {
                elements.push(element);
            } else if !self.is_eof() {
                // Skip any remaining content we can't parse
                let text = self.parse_text();
                if !text.trim().is_empty() {
                    elements.push(HtmlElement::Text(text));
                }
            }
        }

        elements
    }
}

/// Decode common HTML entities
fn decode_html_entities(text: &str) -> String {
    text.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ")
        .replace("&#160;", " ")
        .replace("&ndash;", "\u{2013}")
        .replace("&mdash;", "\u{2014}")
        .replace("&copy;", "\u{00A9}")
        .replace("&reg;", "\u{00AE}")
        .replace("&trade;", "\u{2122}")
        .replace("&hellip;", "\u{2026}")
        .replace("&lsquo;", "\u{2018}")
        .replace("&rsquo;", "\u{2019}")
        .replace("&ldquo;", "\u{201C}")
        .replace("&rdquo;", "\u{201D}")
}

/// Parse HTML string into a vector of HtmlElement
///
/// # Examples
///
/// ```
/// use script_kit_gpui::utils::parse_html;
///
/// let elements = parse_html("<h1>Title</h1><p>Content</p>");
/// assert_eq!(elements.len(), 2);
/// ```
pub fn parse_html(html: &str) -> Vec<HtmlElement> {
    let mut parser = HtmlParser::new(html);
    parser.parse()
}

/// Extract plain text from parsed HTML elements
#[allow(dead_code)]
pub fn elements_to_text(elements: &[HtmlElement]) -> String {
    let mut result = String::new();

    for element in elements {
        match element {
            HtmlElement::Text(text) => {
                result.push_str(text);
            }
            HtmlElement::Header { children, .. }
            | HtmlElement::Paragraph(children)
            | HtmlElement::Bold(children)
            | HtmlElement::Italic(children)
            | HtmlElement::ListItem(children)
            | HtmlElement::Blockquote(children)
            | HtmlElement::Div { children, .. }
            | HtmlElement::Span { children, .. }
            | HtmlElement::Link { children, .. } => {
                result.push_str(&elements_to_text(children));
            }
            HtmlElement::UnorderedList(children) | HtmlElement::OrderedList(children) => {
                for child in children {
                    if let HtmlElement::ListItem(li_children) = child {
                        result.push_str(&elements_to_text(li_children));
                        result.push('\n');
                    }
                }
            }
            HtmlElement::InlineCode(code) | HtmlElement::CodeBlock { code, .. } => {
                result.push_str(code);
            }
            HtmlElement::HorizontalRule => {
                result.push_str("\n---\n");
            }
            HtmlElement::LineBreak => {
                result.push('\n');
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // strip_html_tags tests
    // ========================================================================

    #[test]
    fn test_basic_tag_removal() {
        assert_eq!(strip_html_tags("<p>Hello</p>"), "Hello");
    }

    #[test]
    fn test_nested_tags() {
        assert_eq!(
            strip_html_tags("<div><span>A</span><span>B</span></div>"),
            "A B"
        );
    }

    #[test]
    fn test_empty_input() {
        assert_eq!(strip_html_tags(""), "");
    }

    #[test]
    fn test_no_tags() {
        assert_eq!(strip_html_tags("plain text"), "plain text");
    }

    #[test]
    fn test_whitespace_normalization() {
        assert_eq!(strip_html_tags("<p>Hello   World</p>"), "Hello World");
    }

    #[test]
    fn test_multiple_tags_with_content() {
        assert_eq!(
            strip_html_tags("<h1>Title</h1><p>Paragraph</p>"),
            "Title Paragraph"
        );
    }

    #[test]
    fn test_self_closing_tags() {
        assert_eq!(strip_html_tags("Hello<br/>World"), "Hello World");
    }

    #[test]
    fn test_tags_with_attributes() {
        assert_eq!(
            strip_html_tags("<a href=\"https://example.com\">Link</a>"),
            "Link"
        );
    }

    #[test]
    fn test_deeply_nested() {
        assert_eq!(
            strip_html_tags("<div><div><div>Deep</div></div></div>"),
            "Deep"
        );
    }

    #[test]
    fn test_only_tags() {
        assert_eq!(strip_html_tags("<div><span></span></div>"), "");
    }

    #[test]
    fn test_leading_trailing_whitespace() {
        assert_eq!(strip_html_tags("  <p>  Hello  </p>  "), "Hello");
    }

    #[test]
    fn test_newlines_in_html() {
        assert_eq!(
            strip_html_tags("<p>\n  Hello\n  World\n</p>"),
            "Hello World"
        );
    }

    // ========================================================================
    // HTML parsing tests
    // ========================================================================

    #[test]
    fn test_parse_simple_text() {
        let elements = parse_html("Hello World");
        assert_eq!(elements.len(), 1);
        assert!(matches!(&elements[0], HtmlElement::Text(t) if t == "Hello World"));
    }

    #[test]
    fn test_parse_paragraph() {
        let elements = parse_html("<p>Hello</p>");
        assert_eq!(elements.len(), 1);
        match &elements[0] {
            HtmlElement::Paragraph(children) => {
                assert_eq!(children.len(), 1);
                assert!(matches!(&children[0], HtmlElement::Text(t) if t == "Hello"));
            }
            _ => panic!("Expected Paragraph"),
        }
    }

    #[test]
    fn test_parse_headers() {
        for level in 1..=6u8 {
            let html = format!("<h{}>Header {}</h{}>", level, level, level);
            let elements = parse_html(&html);
            assert_eq!(elements.len(), 1);
            match &elements[0] {
                HtmlElement::Header { level: l, children } => {
                    assert_eq!(*l, level);
                    assert_eq!(children.len(), 1);
                }
                _ => panic!("Expected Header level {}", level),
            }
        }
    }

    #[test]
    fn test_parse_bold() {
        let elements = parse_html("<strong>Bold</strong>");
        assert_eq!(elements.len(), 1);
        assert!(matches!(&elements[0], HtmlElement::Bold(_)));

        let elements = parse_html("<b>Also Bold</b>");
        assert_eq!(elements.len(), 1);
        assert!(matches!(&elements[0], HtmlElement::Bold(_)));
    }

    #[test]
    fn test_parse_italic() {
        let elements = parse_html("<em>Italic</em>");
        assert_eq!(elements.len(), 1);
        assert!(matches!(&elements[0], HtmlElement::Italic(_)));

        let elements = parse_html("<i>Also Italic</i>");
        assert_eq!(elements.len(), 1);
        assert!(matches!(&elements[0], HtmlElement::Italic(_)));
    }

    #[test]
    fn test_parse_inline_code() {
        let elements = parse_html("<code>let x = 42;</code>");
        assert_eq!(elements.len(), 1);
        assert!(matches!(&elements[0], HtmlElement::InlineCode(c) if c == "let x = 42;"));
    }

    #[test]
    fn test_parse_code_block() {
        let elements =
            parse_html("<pre><code class=\"language-typescript\">const x = 1;</code></pre>");
        assert_eq!(elements.len(), 1);
        match &elements[0] {
            HtmlElement::CodeBlock { language, code } => {
                assert_eq!(language.as_deref(), Some("typescript"));
                assert_eq!(code, "const x = 1;");
            }
            _ => panic!("Expected CodeBlock"),
        }
    }

    #[test]
    fn test_parse_unordered_list() {
        let elements = parse_html("<ul><li>Item 1</li><li>Item 2</li></ul>");
        assert_eq!(elements.len(), 1);
        match &elements[0] {
            HtmlElement::UnorderedList(items) => {
                assert_eq!(items.len(), 2);
            }
            _ => panic!("Expected UnorderedList"),
        }
    }

    #[test]
    fn test_parse_ordered_list() {
        let elements = parse_html("<ol><li>First</li><li>Second</li></ol>");
        assert_eq!(elements.len(), 1);
        match &elements[0] {
            HtmlElement::OrderedList(items) => {
                assert_eq!(items.len(), 2);
            }
            _ => panic!("Expected OrderedList"),
        }
    }

    #[test]
    fn test_parse_blockquote() {
        let elements = parse_html("<blockquote>Quote</blockquote>");
        assert_eq!(elements.len(), 1);
        assert!(matches!(&elements[0], HtmlElement::Blockquote(_)));
    }

    #[test]
    fn test_parse_link() {
        let elements = parse_html("<a href=\"https://example.com\">Link</a>");
        assert_eq!(elements.len(), 1);
        match &elements[0] {
            HtmlElement::Link { href, children } => {
                assert_eq!(href, "https://example.com");
                assert_eq!(children.len(), 1);
            }
            _ => panic!("Expected Link"),
        }
    }

    #[test]
    fn test_parse_hr() {
        let elements = parse_html("<hr>");
        assert_eq!(elements.len(), 1);
        assert!(matches!(&elements[0], HtmlElement::HorizontalRule));

        let elements = parse_html("<hr/>");
        assert_eq!(elements.len(), 1);
        assert!(matches!(&elements[0], HtmlElement::HorizontalRule));
    }

    #[test]
    fn test_parse_br() {
        let elements = parse_html("Line1<br>Line2");
        assert_eq!(elements.len(), 3);
        assert!(matches!(&elements[0], HtmlElement::Text(t) if t == "Line1"));
        assert!(matches!(&elements[1], HtmlElement::LineBreak));
        assert!(matches!(&elements[2], HtmlElement::Text(t) if t == "Line2"));
    }

    #[test]
    fn test_parse_nested_elements() {
        let elements = parse_html("<p><strong>Bold</strong> and <em>italic</em></p>");
        assert_eq!(elements.len(), 1);
        match &elements[0] {
            HtmlElement::Paragraph(children) => {
                // Parser produces: Bold, " and ", Italic (3 children)
                assert_eq!(children.len(), 3);
            }
            _ => panic!("Expected Paragraph"),
        }
    }

    #[test]
    fn test_parse_html_entities() {
        let elements = parse_html("<p>&amp; &lt; &gt; &quot;</p>");
        match &elements[0] {
            HtmlElement::Paragraph(children) => match &children[0] {
                HtmlElement::Text(t) => {
                    assert!(t.contains('&'));
                    assert!(t.contains('<'));
                    assert!(t.contains('>'));
                    assert!(t.contains('"'));
                }
                _ => panic!("Expected Text"),
            },
            _ => panic!("Expected Paragraph"),
        }
    }

    #[test]
    fn test_elements_to_text() {
        let elements = parse_html("<h1>Title</h1><p>Paragraph with <strong>bold</strong></p>");
        let text = elements_to_text(&elements);
        assert!(text.contains("Title"));
        assert!(text.contains("Paragraph"));
        assert!(text.contains("bold"));
    }

    #[test]
    fn test_parse_complex_html() {
        let html = r#"
            <h1>Welcome</h1>
            <p>This is a <strong>test</strong> with <em>formatting</em>.</p>
            <ul>
                <li>Item one</li>
                <li>Item two</li>
            </ul>
            <blockquote>A quote</blockquote>
            <pre><code>let x = 1;</code></pre>
        "#;
        let elements = parse_html(html);
        // Should parse without panicking and produce multiple elements
        assert!(elements.len() >= 4);
    }

    #[test]
    fn test_case_insensitive_tags() {
        let elements = parse_html("<P>Paragraph</P>");
        assert_eq!(elements.len(), 1);
        assert!(matches!(&elements[0], HtmlElement::Paragraph(_)));

        let elements = parse_html("<STRONG>Bold</STRONG>");
        assert_eq!(elements.len(), 1);
        assert!(matches!(&elements[0], HtmlElement::Bold(_)));
    }

    #[test]
    fn test_parse_div_with_classes() {
        let elements = parse_html("<div class=\"p-4 bg-blue-500 text-white\">Content</div>");
        assert_eq!(elements.len(), 1);
        match &elements[0] {
            HtmlElement::Div { classes, children } => {
                assert_eq!(classes.as_deref(), Some("p-4 bg-blue-500 text-white"));
                assert_eq!(children.len(), 1);
            }
            _ => panic!("Expected Div with classes"),
        }
    }

    #[test]
    fn test_parse_span_with_classes() {
        let elements = parse_html("<span class=\"font-bold text-red-500\">Bold red</span>");
        assert_eq!(elements.len(), 1);
        match &elements[0] {
            HtmlElement::Span { classes, children } => {
                assert_eq!(classes.as_deref(), Some("font-bold text-red-500"));
                assert_eq!(children.len(), 1);
            }
            _ => panic!("Expected Span with classes"),
        }
    }

    #[test]
    fn test_parse_multibyte_utf8_bullet_points() {
        // This is the exact HTML that caused the crash - contains multi-byte '.' character
        let html = r#"
  <div class="p-6">
    <h1 class="text-2xl font-bold mb-4">Div with Actions</h1>
    <p class="text-gray-400 mb-2">This div prompt has an actions panel.</p>
    <p class="text-gray-500">Press <kbd class="bg-gray-700 px-2 py-1 rounded">Cmd+K</kbd> to open</p>
    <ul class="mt-4 space-y-2">
      <li>. First bullet point</li>
      <li>. Second bullet point</li>
      <li>. Third bullet point</li>
    </ul>
  </div>"#;

        // Should not panic
        let elements = parse_html(html);
        assert!(!elements.is_empty());
    }

    #[test]
    fn test_parse_multibyte_utf8_emojis() {
        // Test with emojis (4-byte UTF-8 characters)
        let html = "<p>Hello World </p>";
        let elements = parse_html(html);
        assert_eq!(elements.len(), 1);
        match &elements[0] {
            HtmlElement::Paragraph(children) => {
                assert_eq!(children.len(), 1);
            }
            _ => panic!("Expected Paragraph"),
        }
    }

    #[test]
    fn test_parse_multibyte_utf8_japanese() {
        // Test with Japanese characters (3-byte UTF-8)
        let html = "<div class=\"test\">Hello</div>";
        let elements = parse_html(html);
        assert_eq!(elements.len(), 1);
        match &elements[0] {
            HtmlElement::Div { children, .. } => {
                assert_eq!(children.len(), 1);
                if let HtmlElement::Text(text) = &children[0] {
                    assert_eq!(text, "Hello");
                }
            }
            _ => panic!("Expected Div"),
        }
    }

    #[test]
    fn test_parse_multibyte_utf8_mixed() {
        // Mix of ASCII, 2-byte, 3-byte, and 4-byte UTF-8
        let html = "<p>ASCII cafe text </p>";
        let elements = parse_html(html);
        assert_eq!(elements.len(), 1);
        match &elements[0] {
            HtmlElement::Paragraph(children) => {
                if let HtmlElement::Text(text) = &children[0] {
                    assert!(text.contains("ASCII"));
                    assert!(text.contains("cafe"));
                    assert!(text.contains("text"));
                }
            }
            _ => panic!("Expected Paragraph"),
        }
    }

    #[test]
    fn test_parse_multibyte_in_attribute() {
        // Multi-byte characters in attribute values
        let html = r#"<a href="https://example.com/test">Link</a>"#;
        let elements = parse_html(html);
        assert_eq!(elements.len(), 1);
        match &elements[0] {
            HtmlElement::Link { href, children } => {
                assert!(href.contains("test"));
                if let HtmlElement::Text(text) = &children[0] {
                    assert_eq!(text, "Link");
                }
            }
            _ => panic!("Expected Link"),
        }
    }
}
