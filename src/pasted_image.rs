use std::ops::Range;

const PASTED_IMAGE_LABEL_PREFIX: &str = "Pasted image #";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PastedImageToken {
    pub(crate) token: String,
    pub(crate) label: String,
    pub(crate) path: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PastedImageTokenRange {
    pub(crate) range: Range<usize>,
    pub(crate) token: String,
    pub(crate) label: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PreparedPastedImage {
    pub(crate) insertion_text: String,
    pub(crate) token: PastedImageToken,
}

pub(crate) fn prepare_pasted_image(
    path: &str,
    existing_tokens: &[PastedImageToken],
) -> PreparedPastedImage {
    let index = next_token_index(existing_tokens);
    let label = build_pasted_image_label(index);
    let token = build_pasted_image_token(index);

    PreparedPastedImage {
        insertion_text: format!("{token} "),
        token: PastedImageToken {
            token,
            label,
            path: path.to_string(),
        },
    }
}

pub(crate) fn write_png_bytes_to_temp_file(png_bytes: &[u8]) -> anyhow::Result<String> {
    use anyhow::Context as _;
    use std::io::Write as _;

    let mut temp_file = tempfile::Builder::new()
        .prefix("script-kit-pasted-image-")
        .suffix(".png")
        .tempfile()
        .context("Failed to create temp file for pasted image")?;

    temp_file
        .write_all(png_bytes)
        .context("Failed to write pasted image to temp file")?;
    temp_file
        .flush()
        .context("Failed to flush pasted image temp file")?;

    let (_file, path) = temp_file
        .keep()
        .context("Failed to persist pasted image temp file")?;

    Ok(path.to_string_lossy().into_owned())
}

pub(crate) fn token_for_label(label: &str) -> Option<String> {
    let index = label
        .strip_prefix(PASTED_IMAGE_LABEL_PREFIX)?
        .trim()
        .parse::<usize>()
        .ok()?;
    Some(build_pasted_image_token(index))
}

pub(crate) fn label_looks_like_pasted_image(label: &str) -> bool {
    token_for_label(label).is_some()
}

pub(crate) fn token_looks_like_pasted_image(token: &str) -> bool {
    let Some((prefix, value)) = crate::ai::context_mentions::typed_mention_token_parts(token)
    else {
        return false;
    };
    if prefix != "img" {
        return false;
    }
    let Some(rest) = value.trim().strip_prefix("paste") else {
        return false;
    };
    !rest.is_empty() && rest.chars().all(|ch| ch.is_ascii_digit())
}

pub(crate) fn token_ranges(text: &str, tokens: &[PastedImageToken]) -> Vec<PastedImageTokenRange> {
    let mut ranges = Vec::new();

    for token in tokens {
        for (byte_start, _) in text.match_indices(&token.token) {
            let start = text[..byte_start].chars().count();
            let end = start + token.token.chars().count();
            ranges.push(PastedImageTokenRange {
                range: start..end,
                token: token.token.clone(),
                label: token.label.clone(),
            });
        }
    }

    ranges.sort_by_key(|entry| entry.range.start);
    ranges
}

pub(crate) fn sync_pasted_image_tokens(tokens: &mut Vec<PastedImageToken>, text: &str) {
    tokens.retain(|token| text.contains(&token.token));
}

pub(crate) fn remove_pasted_image_token_at_cursor(
    text: &str,
    cursor: usize,
    delete_forward: bool,
    tokens: &mut Vec<PastedImageToken>,
) -> Option<(String, usize)> {
    let range = token_range_for_atomic_delete(text, cursor, delete_forward, tokens)?;
    let start = char_to_byte_offset(text, range.start);
    let mut end = char_to_byte_offset(text, range.end);

    if text[end..].starts_with(' ') {
        end += ' '.len_utf8();
    }

    let mut next = String::with_capacity(text.len().saturating_sub(end.saturating_sub(start)));
    next.push_str(&text[..start]);
    next.push_str(&text[end..]);
    sync_pasted_image_tokens(tokens, &next);
    Some((next, range.start))
}

fn token_range_for_atomic_delete(
    text: &str,
    cursor: usize,
    delete_forward: bool,
    tokens: &[PastedImageToken],
) -> Option<Range<usize>> {
    let ranges = token_ranges(text, tokens);

    if delete_forward {
        ranges
            .iter()
            .find(|entry| cursor >= entry.range.start && cursor < entry.range.end)
            .map(|entry| entry.range.clone())
            .or_else(|| {
                ranges
                    .iter()
                    .find(|entry| cursor == entry.range.start)
                    .map(|entry| entry.range.clone())
            })
    } else {
        ranges
            .iter()
            .find(|entry| cursor > entry.range.start && cursor <= entry.range.end)
            .map(|entry| entry.range.clone())
    }
}

fn build_pasted_image_label(index: usize) -> String {
    format!("{PASTED_IMAGE_LABEL_PREFIX}{index}")
}

fn build_pasted_image_token(index: usize) -> String {
    format!("@img:paste{index}")
}

fn next_token_index(existing_tokens: &[PastedImageToken]) -> usize {
    existing_tokens.len() + 1
}

fn char_to_byte_offset(text: &str, char_idx: usize) -> usize {
    text.char_indices()
        .nth(char_idx)
        .map(|(byte_idx, _)| byte_idx)
        .unwrap_or(text.len())
}

#[cfg(test)]
mod tests {
    use super::{
        label_looks_like_pasted_image, prepare_pasted_image, remove_pasted_image_token_at_cursor,
        token_for_label, token_looks_like_pasted_image,
    };

    #[test]
    fn prepare_pasted_image_uses_stable_img_alias_tokens() {
        let prepared = prepare_pasted_image("/tmp/pasted-image.png", &[]);

        assert_eq!(prepared.insertion_text, "@img:paste1 ");
        assert_eq!(prepared.token.token, "@img:paste1");
        assert_eq!(prepared.token.label, "Pasted image #1");
        assert_eq!(prepared.token.path, "/tmp/pasted-image.png");
    }

    #[test]
    fn pasted_image_label_roundtrips_to_alias_token() {
        assert_eq!(
            token_for_label("Pasted image #3"),
            Some("@img:paste3".to_string())
        );
        assert!(label_looks_like_pasted_image("Pasted image #3"));
        assert!(!label_looks_like_pasted_image("Clipboard Image"));
        assert!(token_looks_like_pasted_image("@img:paste3"));
        assert!(token_looks_like_pasted_image("@IMG:paste3"));
        assert!(token_looks_like_pasted_image("@img: paste3"));
        assert!(!token_looks_like_pasted_image("@img:path/to/file.png"));
    }

    #[test]
    fn remove_pasted_image_token_at_cursor_deletes_whole_token_and_space() {
        let prepared = prepare_pasted_image("/tmp/pasted-image.png", &[]);
        let mut tokens = vec![prepared.token];
        let input = format!("before {}after", prepared.insertion_text);
        let cursor = "before @img:paste1".chars().count();

        let (next, next_cursor) =
            remove_pasted_image_token_at_cursor(&input, cursor, false, &mut tokens)
                .expect("token should delete atomically");

        assert_eq!(next, "before after");
        assert_eq!(next_cursor, "before ".chars().count());
        assert!(tokens.is_empty());
    }
}
