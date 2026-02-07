use super::*;

/// Input definition for a template placeholder
#[derive(Clone, Debug)]
pub struct TemplateInput {
    /// Name of the placeholder (e.g., "name", "email")
    pub name: String,
    /// Human-readable label shown in form UI
    pub label: String,
    /// Placeholder text to show when empty
    pub placeholder: String,
    /// Group header for visual organization
    pub group: String,
    /// Whether this field must be provided
    pub required: bool,
}
