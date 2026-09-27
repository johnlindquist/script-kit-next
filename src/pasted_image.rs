const PASTED_IMAGE_LABEL_PREFIX: &str = "Pasted image #";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PastedImagePreparation {
    Pending,
    Failed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PastedImageToken {
    pub(crate) token: String,
    pub(crate) label: String,
    pub(crate) path: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PreparedPastedImage {
    pub(crate) insertion_text: String,
    pub(crate) token: PastedImageToken,
}

#[allow(dead_code)] // The binary's launcher paste stages a clipboard image.
pub(crate) fn prepare_pasted_image(
    path: &str,
    existing_tokens: &[PastedImageToken],
    input_text: &str,
) -> PreparedPastedImage {
    let index = next_token_index(existing_tokens, input_text);
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

#[allow(dead_code)] // The binary's launcher paste stages a clipboard image.
pub(crate) fn reserve_png_temp_file() -> anyhow::Result<tempfile::NamedTempFile> {
    use anyhow::Context as _;
    tempfile::Builder::new()
        .prefix("script-kit-pasted-image-")
        .suffix(".png")
        .tempfile()
        .context("Failed to create temp file for pasted image")
}

#[allow(dead_code)] // The binary's launcher paste stages a clipboard image.
pub(crate) fn write_png_bytes_to_temp_file(
    mut temp_file: tempfile::NamedTempFile,
    png_bytes: &[u8],
) -> anyhow::Result<String> {
    use anyhow::Context as _;
    use std::io::Write as _;

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

fn build_pasted_image_label(index: usize) -> String {
    format!("{PASTED_IMAGE_LABEL_PREFIX}{index}")
}

fn build_pasted_image_token(index: usize) -> String {
    format!("@img:paste{index}")
}

fn next_token_index(existing_tokens: &[PastedImageToken], input_text: &str) -> usize {
    // A switched/restored draft can contain tokens absent from the view cache.
    // Never let a new paste overwrite one of those attachment aliases.
    let mut used = std::collections::HashSet::new();
    for source in
        std::iter::once(input_text).chain(existing_tokens.iter().map(|token| token.token.as_str()))
    {
        let lower = source.to_ascii_lowercase();
        for (start, _) in lower.match_indices("@img:paste") {
            let digits = lower[start + "@img:paste".len()..]
                .split(|ch: char| !ch.is_ascii_digit())
                .next()
                .unwrap_or_default();
            if let Ok(index) = digits.parse::<usize>() {
                used.insert(index);
            }
        }
    }
    used.iter()
        .copied()
        .max()
        .unwrap_or(0)
        .checked_add(1)
        .unwrap_or_else(|| {
            // If every index through len is occupied, len + 1 must be free.
            (1..=used.len())
                .find(|index| !used.contains(index))
                .unwrap_or_else(|| used.len() + 1)
        })
}

#[cfg(test)]
mod tests {
    use super::{
        label_looks_like_pasted_image, prepare_pasted_image, token_for_label,
        token_looks_like_pasted_image,
    };

    #[test]
    fn prepare_pasted_image_uses_stable_img_alias_tokens() {
        let prepared = prepare_pasted_image("/tmp/pasted-image.png", &[], "");

        assert_eq!(prepared.insertion_text, "@img:paste1 ");
        assert_eq!(prepared.token.token, "@img:paste1");
        assert_eq!(prepared.token.label, "Pasted image #1");
        assert_eq!(prepared.token.path, "/tmp/pasted-image.png");
        let restored = prepare_pasted_image("/tmp/restored.png", &[], "@img:paste1 @img:paste2 ");
        assert_eq!(restored.token.token, "@img:paste3");
        let after_removal =
            prepare_pasted_image("/tmp/next.png", &[restored.token], "@img:paste3 ");
        assert_eq!(after_removal.token.token, "@img:paste4");
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
}
