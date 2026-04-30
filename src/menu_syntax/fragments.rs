use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MenuSyntaxFragmentRole {
    Prefix,
    Subject,
    Date,
    DateRange,
    Duration,
    Recurrence,
    Kv,
    Tag,
    Url,
    Priority,
    Unresolved,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MenuSyntaxFragmentStatus {
    Resolved,
    Unresolved,
    Ignored,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MenuSyntaxFragment {
    pub role: MenuSyntaxFragmentRole,
    pub source: String,
    pub source_span: (usize, usize),
    pub status: MenuSyntaxFragmentStatus,
}
