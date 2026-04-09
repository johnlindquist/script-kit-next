use gpui::SharedString;

/// Whether the picker was triggered by `@` (mention) or `/` (slash command).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ContextPickerTrigger {
    Mention,
    Slash,
}

/// Which full built-in view a portal item opens for rich browsing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortalKind {
    /// Open the Spotlight-powered file search view.
    FileSearch,
    /// Open the visual clipboard history browser.
    ClipboardHistory,
    /// Open the ACP conversation history popup (prefiltered by typed query).
    AcpHistory,
}

/// Source-aware identity for a slash command in the ACP picker.
///
/// Carries enough information to distinguish duplicate slash slugs from
/// different plugins and to stage local skill content on acceptance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SlashCommandPayload {
    /// A built-in Claude Code slash command (e.g. `/compact`, `/clear`).
    Default { name: String },
    /// A skill from a discovered plugin.
    PluginSkill(crate::plugins::PluginSkill),
    /// A user-level Claude Code skill from `~/.scriptkit/.claude/skills/`.
    ClaudeCodeSkill {
        skill_id: String,
        skill_path: std::path::PathBuf,
    },
}

impl SlashCommandPayload {
    /// The bare slash name shown in the picker (e.g. `"compact"`, `"review"`).
    pub fn slash_name(&self) -> &str {
        match self {
            Self::Default { name } => name.as_str(),
            Self::PluginSkill(skill) => skill.skill_id.as_str(),
            Self::ClaudeCodeSkill { skill_id, .. } => skill_id.as_str(),
        }
    }

    /// A stable identifier that distinguishes duplicate slash slugs from
    /// different sources (e.g. `"default:compact"`, `"plugin:alpha:review"`).
    pub fn stable_id(&self) -> String {
        match self {
            Self::Default { name } => format!("default:{name}"),
            Self::PluginSkill(skill) => {
                format!("plugin:{}:{}", skill.plugin_id, skill.skill_id)
            }
            Self::ClaudeCodeSkill { skill_id, .. } => format!("claude:{skill_id}"),
        }
    }

    /// Formatted meta string for the picker row, showing the slash name
    /// and owner context. Default commands show just the slash name;
    /// plugin and Claude skills include the owner label.
    pub fn picker_owner_meta(&self) -> String {
        match self {
            Self::Default { name } => format!("/{name}"),
            Self::PluginSkill(skill) => {
                let owner = if skill.plugin_title.is_empty() {
                    skill.plugin_id.as_str()
                } else {
                    skill.plugin_title.as_str()
                };
                format!("/{} \u{b7} {} skill", skill.skill_id, owner)
            }
            Self::ClaudeCodeSkill { skill_id, .. } => {
                format!("/{skill_id} \u{b7} Claude Code skill")
            }
        }
    }

    /// Human-readable owner label for display in the picker meta column.
    pub fn owner_label(&self) -> String {
        match self {
            Self::Default { .. } => "Built-in".to_string(),
            Self::PluginSkill(skill) => {
                if skill.plugin_title.is_empty() {
                    skill.plugin_id.clone()
                } else {
                    skill.plugin_title.clone()
                }
            }
            Self::ClaudeCodeSkill { .. } => "Claude Code".to_string(),
        }
    }
}

/// The kind of item in the context picker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContextPickerItemKind {
    /// A built-in context attachment (seeded from `context_attachment_specs()`).
    BuiltIn(crate::ai::context_contract::ContextAttachmentKind),
    /// A local file attachment.
    File(std::path::PathBuf),
    /// A local folder attachment.
    Folder(std::path::PathBuf),
    /// A slash command with source-aware identity. Default commands insert
    /// literal `/command` text; plugin and Claude skills stage local content.
    SlashCommand(SlashCommandPayload),
    /// Opens a full built-in view as a portal for rich browsing.
    /// Selection in the portal attaches the result back to the ACP chat.
    Portal(PortalKind),
    /// A non-actionable placeholder row (loading spinner, empty state).
    /// Acceptance is a no-op; the row exists only for visual feedback.
    Inert,
}

/// A single row in the context picker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextPickerItem {
    /// Unique identifier for this row (e.g. `"builtin:selection"`, `"file:/path"`).
    pub id: SharedString,
    /// Display label (e.g. `"Selection"`, `"chat.rs"`).
    pub label: SharedString,
    /// Compact synopsis shown for the focused item.
    pub description: SharedString,
    /// Right-side metadata (slash command, mention, or path).
    pub meta: SharedString,
    /// The kind of item — determines how acceptance creates a context part.
    pub kind: ContextPickerItemKind,
    /// Relevance score used for deterministic ranking (higher = better match).
    /// Ties are broken by insertion order.
    pub score: u32,
    /// Indices into `label` that matched the query (for gold highlighting).
    pub label_highlight_indices: Vec<usize>,
    /// Indices into `meta` that matched the query (for gold highlighting).
    pub meta_highlight_indices: Vec<usize>,
}

/// Mutable state for the inline context picker overlay.
///
/// Created when the user types `@` or `/` in the composer; dropped on Escape,
/// Enter (accept), or when the composer loses focus.
#[derive(Debug, Clone)]
pub struct ContextPickerState {
    /// Which trigger character opened this picker.
    pub trigger: ContextPickerTrigger,
    /// The raw query string after the trigger (e.g. `"sel"` from `@sel`).
    pub query: String,
    /// Ranked items matching the current query.
    pub items: Vec<ContextPickerItem>,
    /// Currently highlighted row index (keyboard navigation).
    pub selected_index: usize,
}

impl ContextPickerState {
    pub fn new(
        trigger: ContextPickerTrigger,
        query: String,
        items: Vec<ContextPickerItem>,
    ) -> Self {
        Self {
            trigger,
            query,
            items,
            selected_index: 0,
        }
    }

    /// Machine-readable snapshot of picker entries and selection state.
    /// Used by agents to verify UI state without brittle string scraping.
    pub fn snapshot(&self) -> ContextPickerSnapshot {
        ContextPickerSnapshot {
            trigger: self.trigger,
            query: self.query.clone(),
            selected_index: self.selected_index,
            items: self
                .items
                .iter()
                .map(|item| ContextPickerItemSnapshot {
                    id: item.id.to_string(),
                    label: item.label.to_string(),
                    section: match &item.kind {
                        ContextPickerItemKind::BuiltIn(_) => "builtin",
                        ContextPickerItemKind::File(_) => "file",
                        ContextPickerItemKind::Folder(_) => "folder",
                        ContextPickerItemKind::SlashCommand(_) => "slash_command",
                        ContextPickerItemKind::Portal(_) => "portal",
                        ContextPickerItemKind::Inert => "inert",
                    },
                    score: item.score,
                })
                .collect(),
        }
    }
}

/// Serializable snapshot of picker state for agent verification.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ContextPickerSnapshot {
    pub trigger: ContextPickerTrigger,
    pub query: String,
    pub selected_index: usize,
    pub items: Vec<ContextPickerItemSnapshot>,
}

/// Serializable snapshot of a single picker item.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ContextPickerItemSnapshot {
    pub id: String,
    pub label: String,
    pub section: &'static str,
    pub score: u32,
}

/// Section header for grouped picker results.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextPickerSection {
    BuiltIn,
    Files,
    Folders,
}

impl ContextPickerSection {
    pub fn label(self) -> &'static str {
        match self {
            Self::BuiltIn => "Context",
            Self::Files => "Files",
            Self::Folders => "Folders",
        }
    }
}
