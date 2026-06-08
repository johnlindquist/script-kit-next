//! Canonical Agent Chat content-block boundary.
//!
//! Agent Chat owns these payload types locally so the Pi-backed runtime does
//! not depend on the deprecated backend client library.

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ContentBlock {
    Text(TextContent),
    Image(ImageContent),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TextContent {
    pub text: String,
}

impl TextContent {
    pub(crate) fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ImageContent {
    pub data: String,
    pub mime_type: String,
}

impl ImageContent {
    pub(crate) fn new(data: impl Into<String>, mime_type: impl Into<String>) -> Self {
        Self {
            data: data.into(),
            mime_type: mime_type.into(),
        }
    }
}
