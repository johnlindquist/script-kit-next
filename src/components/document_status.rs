//! Shared facts for document-save feedback in Notes and Today.

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DocumentFeedbackKind {
    Information,
    Error,
    SaveFailure,
    SaveRecovered,
}
impl DocumentFeedbackKind {
    pub fn is_error(self) -> bool {
        matches!(self, Self::Error | Self::SaveFailure)
    }
}
pub struct DocumentSaveFailureNotification;

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentSaveFailure {
    pub code: &'static str,
    pub diagnostic_fingerprint: String,
}

impl DocumentSaveFailure {
    pub fn capture(error: &impl std::fmt::Display) -> Self {
        let safe = crate::logging::log_private_user_value(&error.to_string());
        Self {
            code: "document_save_failed",
            diagnostic_fingerprint: safe.sha256,
        }
    }

    pub fn details(&self) -> String {
        format!(
            "{}\nYour changes are still open.\nDiagnostic: {}",
            self.code, self.diagnostic_fingerprint
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum DocumentSaveStatus {
    Saved,
    Unsaved,
    Failed,
}

impl DocumentSaveStatus {
    pub fn from_facts(dirty: bool, failure: Option<&DocumentSaveFailure>) -> Self {
        if failure.is_some() {
            Self::Failed
        } else if dirty {
            Self::Unsaved
        } else {
            Self::Saved
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Saved => "Saved",
            Self::Unsaved => "Unsaved changes",
            Self::Failed => "Save failed · Retry Save in Actions",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn failed_write_is_distinct_from_unsaved_even_if_dirty_projection_is_clean() {
        let failure = DocumentSaveFailure {
            code: "document_save_failed",
            diagnostic_fingerprint: "fixture".to_string(),
        };
        assert_eq!(
            DocumentSaveStatus::from_facts(true, None),
            DocumentSaveStatus::Unsaved
        );
        assert_eq!(
            DocumentSaveStatus::from_facts(true, Some(&failure)),
            DocumentSaveStatus::Failed
        );
        assert_eq!(
            DocumentSaveStatus::from_facts(false, Some(&failure)),
            DocumentSaveStatus::Failed
        );
        assert_eq!(
            DocumentSaveStatus::from_facts(false, None),
            DocumentSaveStatus::Saved
        );
    }
}
