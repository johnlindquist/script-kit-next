fn markdown_reference_for_day_page_context_part(
    token: &str,
    part: Option<&crate::ai::message_parts::AiContextPart>,
) -> Option<String> {
    let part = part.cloned().or_else(|| {
        crate::ai::context_contract::ContextAttachmentKind::from_mention_line(token)
            .map(|kind| kind.part())
    })?;
    let (label, href) = match part {
        crate::ai::message_parts::AiContextPart::FilePath { path, label }
        | crate::ai::message_parts::AiContextPart::SkillFile { path, label, .. } => {
            (label, file_href_for_day_page_markdown(&path))
        }
        crate::ai::message_parts::AiContextPart::ResourceUri { uri, label } => (label, uri),
        crate::ai::message_parts::AiContextPart::TextBlock { label, source, .. } => {
            if !source.contains(':') || source.contains(char::is_whitespace) {
                return None;
            }
            (label, source)
        }
        crate::ai::message_parts::AiContextPart::FocusedTarget { target, label } => {
            if let Some(path) = target
                .metadata
                .as_ref()
                .and_then(|metadata| metadata.get("path"))
                .and_then(|value| value.as_str())
            {
                (label, file_href_for_day_page_markdown(path))
            } else {
                (label, format!("kit://focused-target/{}", target.semantic_id))
            }
        }
        crate::ai::message_parts::AiContextPart::AmbientContext { label } => (
            label.clone(),
            format!(
                "kit://context?label={}",
                encode_day_page_markdown_url_component(&label)
            ),
        ),
    };
    let label = label
        .trim()
        .replace('[', "\\[")
        .replace(']', "\\]");
    if label.is_empty() || href.trim().is_empty() {
        return None;
    }
    Some(format!(
        "[{label}]({})",
        href.replace(')', "%29")
    ))
}

fn day_page_context_parts_from_markdown_links(
    markdown: &str,
) -> Vec<crate::ai::message_parts::AiContextPart> {
    let mut parts = Vec::new();
    for (label, href) in day_page_markdown_links(markdown) {
        let Some(part) = day_page_context_part_from_markdown_link(&label, &href) else {
            continue;
        };
        if !parts.contains(&part) {
            parts.push(part);
        }
    }
    parts
}

fn day_page_context_part_from_markdown_link(
    label: &str,
    href: &str,
) -> Option<crate::ai::message_parts::AiContextPart> {
    let label = label.trim().to_string();
    let href = href.trim();
    if label.is_empty() || href.is_empty() {
        return None;
    }
    if let Some(path) = href.strip_prefix("file://") {
        return Some(crate::ai::message_parts::AiContextPart::FilePath {
            path: decode_day_page_markdown_url_component(path),
            label,
        });
    }
    if href.starts_with("kit://") {
        return Some(crate::ai::message_parts::AiContextPart::ResourceUri {
            uri: href.to_string(),
            label,
        });
    }
    if href.starts_with("http://") || href.starts_with("https://") {
        return Some(crate::ai::message_parts::AiContextPart::TextBlock {
            label,
            source: href.to_string(),
            text: href.to_string(),
            mime_type: Some("text/uri-list".to_string()),
        });
    }
    None
}

fn day_page_markdown_links(markdown: &str) -> Vec<(String, String)> {
    let mut links = Vec::new();
    let bytes = markdown.as_bytes();
    let mut index = 0usize;
    while index < bytes.len() {
        if bytes[index] != b'[' {
            index += 1;
            continue;
        }
        let Some(label_end) = find_unescaped_day_page_markdown_byte(markdown, index + 1, b']')
        else {
            break;
        };
        if !markdown[label_end..].starts_with("](") {
            index = label_end + 1;
            continue;
        }
        let href_start = label_end + 2;
        let Some(href_end) = find_unescaped_day_page_markdown_byte(markdown, href_start, b')')
        else {
            break;
        };
        links.push((
            markdown[index + 1..label_end]
                .replace("\\[", "[")
                .replace("\\]", "]"),
            markdown[href_start..href_end].to_string(),
        ));
        index = href_end + 1;
    }
    links
}

fn find_unescaped_day_page_markdown_byte(text: &str, start: usize, needle: u8) -> Option<usize> {
    let bytes = text.as_bytes();
    let mut index = start;
    while index < bytes.len() {
        if bytes[index] == needle {
            let mut slash_count = 0usize;
            let mut cursor = index;
            while cursor > 0 && bytes[cursor - 1] == b'\\' {
                slash_count += 1;
                cursor -= 1;
            }
            if slash_count % 2 == 0 {
                return Some(index);
            }
        }
        index += 1;
    }
    None
}

fn file_href_for_day_page_markdown(path: &str) -> String {
    format!("file://{}", encode_day_page_markdown_url_path(path))
}

fn encode_day_page_markdown_url_path(path: &str) -> String {
    path.chars()
        .map(|ch| match ch {
            ' ' => "%20".to_string(),
            ')' => "%29".to_string(),
            '(' => "%28".to_string(),
            '%' => "%25".to_string(),
            _ => ch.to_string(),
        })
        .collect()
}

fn encode_day_page_markdown_url_component(value: &str) -> String {
    value
        .chars()
        .map(|ch| match ch {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => ch.to_string(),
            ' ' => "%20".to_string(),
            _ => format!("%{:02X}", ch as u32),
        })
        .collect()
}

fn decode_day_page_markdown_url_component(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut index = 0usize;
    while index < bytes.len() {
        if bytes[index] == b'%' && index + 2 < bytes.len() {
            if let Ok(hex) = std::str::from_utf8(&bytes[index + 1..index + 3]) {
                if let Ok(byte) = u8::from_str_radix(hex, 16) {
                    out.push(byte);
                    index += 3;
                    continue;
                }
            }
        }
        out.push(bytes[index]);
        index += 1;
    }
String::from_utf8(out)
        .unwrap_or_else(|error| String::from_utf8_lossy(&error.into_bytes()).to_string())
}
