use crate::dictation::download::format_bytes;
use crate::dictation::transcription::{resolve_default_model_path, resolve_whisper_model_path};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DictationModelId {
    ParakeetTdt06bV3,
    WhisperMedium,
}

impl Default for DictationModelId {
    fn default() -> Self {
        Self::ParakeetTdt06bV3
    }
}

impl DictationModelId {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ParakeetTdt06bV3 => "parakeet-tdt-0.6b-v3",
            Self::WhisperMedium => "whisper-medium",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "parakeet-tdt-0.6b-v3" => Some(Self::ParakeetTdt06bV3),
            "whisper-medium" => Some(Self::WhisperMedium),
            _ => None,
        }
    }

    pub fn from_preference(value: Option<&str>) -> Self {
        match value {
            Some(raw) => match Self::from_str(raw) {
                Some(model) => model,
                None => {
                    tracing::warn!(
                        category = "DICTATION",
                        model_id = raw,
                        fallback_model_id = Self::default().as_str(),
                        "Unknown dictation model preference; falling back to default"
                    );
                    Self::default()
                }
            },
            None => Self::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DictationEngineKind {
    Parakeet,
    Whisper,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DictationModelCatalogEntry {
    pub id: DictationModelId,
    pub stable_id: &'static str,
    pub display_name: &'static str,
    pub description: &'static str,
    pub recommended: bool,
    pub engine_kind: DictationEngineKind,
}

impl DictationModelCatalogEntry {
    pub fn path(&self) -> PathBuf {
        match self.id {
            DictationModelId::ParakeetTdt06bV3 => resolve_default_model_path(),
            DictationModelId::WhisperMedium => resolve_whisper_model_path(),
        }
    }

    /// Expected size of the remote download for this model, in bytes.
    pub fn download_size_bytes(&self) -> u64 {
        match self.id {
            DictationModelId::ParakeetTdt06bV3 => {
                crate::dictation::transcription::PARAKEET_MODEL_ARCHIVE_SIZE
            }
            DictationModelId::WhisperMedium => crate::dictation::transcription::WHISPER_MODEL_SIZE,
        }
    }

    /// Size of a resumable partial download for this model, when present.
    pub fn partial_download_size_bytes(&self) -> Option<u64> {
        match self.id {
            DictationModelId::ParakeetTdt06bV3 => {
                crate::dictation::download::parakeet_partial_archive_size()
            }
            DictationModelId::WhisperMedium => {
                crate::dictation::download::whisper_partial_model_size()
            }
        }
    }

    pub fn is_available(&self) -> bool {
        match self.id {
            DictationModelId::ParakeetTdt06bV3 => {
                let path = self.path();
                path.is_dir()
                    && std::fs::read_dir(&path)
                        .map(|mut entries| entries.next().is_some())
                        .unwrap_or(false)
            }
            DictationModelId::WhisperMedium => self.path().is_file(),
        }
    }

    pub fn downloaded_size_bytes(&self) -> Option<u64> {
        match self.id {
            DictationModelId::ParakeetTdt06bV3 => directory_size(self.path()).ok(),
            DictationModelId::WhisperMedium => self.path().metadata().ok().map(|meta| meta.len()),
        }
    }

    pub fn downloaded_size_label(&self) -> Option<String> {
        self.downloaded_size_bytes()
            .map(format_dictation_model_size)
    }
}

pub fn dictation_model_catalog() -> [DictationModelCatalogEntry; 2] {
    [
        DictationModelCatalogEntry {
            id: DictationModelId::ParakeetTdt06bV3,
            stable_id: DictationModelId::ParakeetTdt06bV3.as_str(),
            display_name: "Parakeet TDT 0.6B v3",
            description:
                "Fast and accurate. Auto-detects 25 European languages (ignores the language setting).",
            recommended: true,
            engine_kind: DictationEngineKind::Parakeet,
        },
        DictationModelCatalogEntry {
            id: DictationModelId::WhisperMedium,
            stable_id: DictationModelId::WhisperMedium.as_str(),
            display_name: "Whisper Medium",
            description:
                "Broadest language coverage and honors the language setting, but may run a bit slow.",
            recommended: false,
            engine_kind: DictationEngineKind::Whisper,
        },
    ]
}

pub fn dictation_model_entry(id: DictationModelId) -> DictationModelCatalogEntry {
    // Destructure the fixed catalog array and match on `id` so the mapping is
    // total by construction: adding a `DictationModelId` variant (or growing
    // the catalog) becomes a compile error here instead of a runtime panic.
    let [parakeet, whisper] = dictation_model_catalog();
    match id {
        DictationModelId::ParakeetTdt06bV3 => parakeet,
        DictationModelId::WhisperMedium => whisper,
    }
}

pub fn format_dictation_model_size(bytes: u64) -> String {
    format_bytes(bytes)
}

fn directory_size(path: PathBuf) -> std::io::Result<u64> {
    let metadata = path.metadata()?;
    if metadata.is_file() {
        return Ok(metadata.len());
    }

    let mut total = 0;
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let metadata = entry.metadata()?;
        if metadata.is_dir() {
            total += directory_size(entry.path())?;
        } else if metadata.is_file() {
            total += metadata.len();
        }
    }
    Ok(total)
}
