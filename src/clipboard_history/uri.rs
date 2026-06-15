const CLIPBOARD_HISTORY_RESOURCE_PREFIX: &str = "kit://clipboard-history?id=";
const LEGACY_CLIPBOARD_PROVENANCE_PREFIX: &str = "scriptkit://clipboard/";

pub fn entry_resource_uri(entry_id: &str) -> String {
    format!(
        "{CLIPBOARD_HISTORY_RESOURCE_PREFIX}{}",
        encode_url_component(entry_id)
    )
}

pub fn parse_entry_resource_uri(uri: &str) -> Option<String> {
    if let Some(id) = uri.strip_prefix(CLIPBOARD_HISTORY_RESOURCE_PREFIX) {
        let id = id.split('&').next().unwrap_or(id);
        return decode_url_component(id).filter(|id| !id.is_empty());
    }
    uri.strip_prefix(LEGACY_CLIPBOARD_PROVENANCE_PREFIX)
        .filter(|id| !id.is_empty())
        .map(ToString::to_string)
}

fn encode_url_component(value: &str) -> String {
    value
        .chars()
        .map(|ch| match ch {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => ch.to_string(),
            _ => format!("%{:02X}", ch as u32),
        })
        .collect()
}

fn decode_url_component(value: &str) -> Option<String> {
    let bytes = value.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut index = 0usize;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            if index + 2 >= bytes.len() {
                return None;
            }
            let hex = std::str::from_utf8(&bytes[index + 1..index + 3]).ok()?;
            let byte = u8::from_str_radix(hex, 16).ok()?;
            out.push(byte);
            index += 3;
            continue;
        }
        out.push(bytes[index]);
        index += 1;
    }
    String::from_utf8(out).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entry_resource_uri_round_trips_ids() {
        let uri = entry_resource_uri("entry with spaces");
        assert_eq!(uri, "kit://clipboard-history?id=entry%20with%20spaces");
        assert_eq!(
            parse_entry_resource_uri(&uri).as_deref(),
            Some("entry with spaces")
        );
    }

    #[test]
    fn parser_accepts_legacy_clipboard_provenance() {
        assert_eq!(
            parse_entry_resource_uri("scriptkit://clipboard/entry-1").as_deref(),
            Some("entry-1")
        );
    }
}
