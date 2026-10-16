//! What type-specific clipboard actions do with an entry: open a link,
//! reveal a path, pretty-print JSON, or convert a color.

/// What a type-specific clipboard action does with an entry's content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntryTransform {
    /// Open this URL or path (reveal it in Finder for paths).
    Open(String),
    /// Copy this text and say `message`.
    Copy { text: String, message: String },
}

pub fn entry_transform(action_id: &str, content: &str) -> Result<EntryTransform, String> {
    let color = |format: fn(super::Rgba) -> String| {
        let color =
            super::parse_color(content).ok_or_else(|| "This entry isn't a color".to_string())?;
        let text = format(color);
        Ok(EntryTransform::Copy {
            message: format!("Copied {text}"),
            text,
        })
    };
    match action_id {
        "clipboard_open_link" => Ok(EntryTransform::Open(content.to_string())),
        "clipboard_reveal_file" => {
            let path = match content.strip_prefix("~/") {
                Some(rest) => dirs::home_dir()
                    .map(|home| home.join(rest).to_string_lossy().to_string())
                    .unwrap_or_else(|| content.to_string()),
                None => content.to_string(),
            };
            if std::path::Path::new(&path).exists() {
                Ok(EntryTransform::Open(path))
            } else {
                Err(format!("{path} doesn't exist any more"))
            }
        }
        "clipboard_copy_json" => {
            let value: serde_json::Value = serde_json::from_str(content)
                .map_err(|error| format!("This isn't valid JSON: {error}"))?;
            let text = serde_json::to_string_pretty(&value).map_err(|error| error.to_string())?;
            Ok(EntryTransform::Copy {
                text,
                message: "Copied formatted JSON".to_string(),
            })
        }
        "clipboard_copy_color_hex" => color(super::Rgba::hex),
        "clipboard_copy_color_rgb" => color(super::Rgba::rgb),
        "clipboard_copy_color_hsl" => color(super::Rgba::hsl),
        other => Err(format!("Unknown clipboard action {other}")),
    }
}

/// Open a copied link, or reveal a copied path in Finder.
pub fn open_entry_target(target: &str, reveal: bool) -> Result<(), String> {
    if reveal {
        return crate::platform::reveal_in_finder(std::path::Path::new(target));
    }
    crate::runtime_policy::check(crate::runtime_policy::ExternalEffect::OpenExternal)
        .map_err(|error| error.to_string())?;
    open::that_detached(target).map_err(|error| format!("Couldn't open {target}: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn type_specific_clipboard_actions_transform_the_content() {
        let copy = |id: &str, content: &str| match entry_transform(id, content) {
            Ok(EntryTransform::Copy { text, .. }) => text,
            other => panic!("{other:?}"),
        };
        assert_eq!(
            copy("clipboard_copy_json", r#"{"a":[1,2]}"#),
            "{\n  \"a\": [\n    1,\n    2\n  ]\n}"
        );
        assert!(entry_transform("clipboard_copy_json", "{nope").is_err());
        assert_eq!(
            copy("clipboard_copy_color_hsl", "#FF8800"),
            "hsl(32, 100%, 50%)"
        );
        assert_eq!(
            copy("clipboard_copy_color_hex", "rgb(255, 136, 0)"),
            "#FF8800"
        );
        assert!(entry_transform("clipboard_copy_color_rgb", "hello").is_err());
        assert_eq!(
            entry_transform("clipboard_open_link", "https://example.com"),
            Ok(EntryTransform::Open("https://example.com".to_string()))
        );
        let dir = tempfile::tempdir().expect("temp dir");
        let path = dir.path().to_string_lossy().to_string();
        assert_eq!(
            entry_transform("clipboard_reveal_file", &path),
            Ok(EntryTransform::Open(path))
        );
        assert!(entry_transform("clipboard_reveal_file", "/no/such/path/here").is_err());
    }
}
