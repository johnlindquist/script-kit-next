use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NamingTarget {
    Script,
    Extension,
}

impl NamingTarget {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Script => "script",
            Self::Extension => "extension",
        }
    }

    pub const fn display_name(self) -> &'static str {
        match self {
            Self::Script => "Script",
            Self::Extension => "Extension",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NamingValidationError {
    EmptyName,
    InvalidCharacters {
        characters: String,
    },
    DuplicateFilename {
        filename: String,
        target_directory: PathBuf,
    },
    SubmissionEncodingFailed,
}

impl NamingValidationError {
    pub fn message(&self) -> String {
        match self {
            Self::EmptyName => "Name cannot be empty".to_string(),
            Self::InvalidCharacters { characters } => {
                format!("Name contains invalid characters: {characters}")
            }
            Self::DuplicateFilename {
                filename,
                target_directory,
            } => {
                format!(
                    "{filename} already exists in {}",
                    target_directory.display()
                )
            }
            Self::SubmissionEncodingFailed => {
                "Could not encode naming payload. Try again.".to_string()
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamingDerivedState {
    pub friendly_name_trimmed: String,
    pub filename_stem: String,
    pub filename: String,
    pub validation_error: Option<NamingValidationError>,
}

#[derive(Serialize)]
struct NamingSubmitPayload<'a> {
    friendly_name: &'a str,
    filename: &'a str,
    target: NamingTarget,
}

/// Deserialized naming submit payload for use in completion handlers.
#[derive(Debug, Deserialize)]
pub struct NamingSubmitResult {
    pub friendly_name: String,
    pub filename: String,
    pub target: NamingTarget,
}

pub fn normalize_extension(extension: &str) -> String {
    extension
        .trim()
        .trim_start_matches('.')
        .to_ascii_lowercase()
}

pub fn build_filename(stem: &str, extension: &str) -> String {
    if extension.is_empty() {
        stem.to_string()
    } else {
        format!("{stem}.{extension}")
    }
}

pub fn kebab_case_filename_stem(friendly_name: &str) -> String {
    let mut slug = String::new();
    let mut last_was_dash = false;

    for ch in friendly_name.trim().chars() {
        if ch.is_alphanumeric() {
            for lower in ch.to_lowercase() {
                slug.push(lower);
            }
            last_was_dash = false;
            continue;
        }

        if (ch.is_whitespace() || ch == '_' || ch == '-') && !last_was_dash && !slug.is_empty() {
            slug.push('-');
            last_was_dash = true;
        }
    }

    while slug.ends_with('-') {
        slug.pop();
    }

    slug
}

fn invalid_character_summary(friendly_name: &str) -> Option<String> {
    let mut rendered = Vec::new();

    for ch in friendly_name.chars() {
        if ch == '/' || ch == '\\' || ch.is_control() {
            let label = if ch == '/' {
                "'/'".to_string()
            } else if ch == '\\' {
                "'\\\\'".to_string()
            } else {
                format!("control(U+{:04X})", ch as u32)
            };

            if !rendered.contains(&label) {
                rendered.push(label);
            }
        }
    }

    if rendered.is_empty() {
        None
    } else {
        Some(rendered.join(", "))
    }
}

pub fn derive_naming_state(
    friendly_name: &str,
    extension: &str,
    target_directory: &Path,
) -> NamingDerivedState {
    let friendly_name_trimmed = friendly_name.trim().to_string();
    let filename_stem = kebab_case_filename_stem(&friendly_name_trimmed);
    let extension = normalize_extension(extension);
    let filename = build_filename(&filename_stem, &extension);

    let validation_error = if friendly_name_trimmed.is_empty() || filename_stem.is_empty() {
        Some(NamingValidationError::EmptyName)
    } else if let Some(characters) = invalid_character_summary(&friendly_name_trimmed) {
        Some(NamingValidationError::InvalidCharacters { characters })
    } else {
        let candidate_path = target_directory.join(&filename);
        if candidate_path.exists() {
            Some(NamingValidationError::DuplicateFilename {
                filename: filename.clone(),
                target_directory: target_directory.to_path_buf(),
            })
        } else {
            None
        }
    };

    NamingDerivedState {
        friendly_name_trimmed,
        filename_stem,
        filename,
        validation_error,
    }
}

pub fn build_submit_payload(
    friendly_name: &str,
    filename: &str,
    target: NamingTarget,
) -> Result<String, serde_json::Error> {
    let payload = NamingSubmitPayload {
        friendly_name,
        filename,
        target,
    };

    serde_json::to_string(&payload)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use tempfile::tempdir;

    #[test]
    fn test_kebab_case_filename_stem_generates_hyphenated_slug_when_name_contains_spaces() {
        assert_eq!(
            kebab_case_filename_stem("  My Script_Name  "),
            "my-script-name"
        );
    }

    #[test]
    fn test_derive_naming_state_sets_empty_name_error_when_input_is_blank() {
        let temp = tempdir().expect("temp dir should be created");
        let state = derive_naming_state("   ", "ts", temp.path());

        assert_eq!(state.filename_stem, "");
        assert!(matches!(
            state.validation_error,
            Some(NamingValidationError::EmptyName)
        ));
    }

    #[test]
    fn test_derive_naming_state_sets_invalid_character_error_when_name_has_separator() {
        let temp = tempdir().expect("temp dir should be created");
        let state = derive_naming_state("bad/name", "ts", temp.path());

        assert!(matches!(
            state.validation_error,
            Some(NamingValidationError::InvalidCharacters { .. })
        ));
    }

    #[test]
    fn test_derive_naming_state_sets_duplicate_error_when_filename_already_exists() {
        let temp = tempdir().expect("temp dir should be created");
        let existing_path = temp.path().join("my-script.ts");
        std::fs::write(&existing_path, "existing").expect("existing file should be written");

        let state = derive_naming_state("My Script", "ts", temp.path());

        assert!(matches!(
            state.validation_error,
            Some(NamingValidationError::DuplicateFilename { .. })
        ));
    }

    #[test]
    fn test_build_submit_payload_serializes_target_filename_and_friendly_name() {
        let payload = build_submit_payload("My Script", "my-script.ts", NamingTarget::Script)
            .expect("payload should serialize");
        let json: Value = serde_json::from_str(&payload).expect("payload should parse as json");

        assert_eq!(json["friendly_name"], "My Script");
        assert_eq!(json["filename"], "my-script.ts");
        assert_eq!(json["target"], "script");
    }
}
