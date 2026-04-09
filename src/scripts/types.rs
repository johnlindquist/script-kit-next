//! Script and scriptlet type definitions
//!
//! This module contains the core data types for scripts, scriptlets,
//! and search results used throughout the script system.

use std::path::PathBuf;
use std::sync::Arc;

use crate::agents::Agent;
use crate::fallbacks::collector::FallbackItem;
use crate::metadata_parser::TypedMetadata;
use crate::schema_parser::Schema;

/// Represents a script file with its metadata
#[derive(Clone, Debug, Default)]
pub struct Script {
    pub name: String,
    pub path: PathBuf,
    pub extension: String,
    pub description: Option<String>,
    /// Icon name from // Icon: metadata (e.g., "File", "Terminal", "Star")
    /// Defaults to "Code" if not specified
    pub icon: Option<String>,
    /// Alias for quick triggering (e.g., "gc" for "git-commit")
    pub alias: Option<String>,
    /// Keyboard shortcut for direct invocation (e.g., "opt i", "cmd shift k")
    pub shortcut: Option<String>,
    /// Typed metadata from `metadata = { ... }` declaration in script
    pub typed_metadata: Option<TypedMetadata>,
    /// Schema definition from `schema = { ... }` declaration in script
    pub schema: Option<Schema>,
    /// Plugin that owns this script (e.g., "main", "cleanshot", "tools")
    pub plugin_id: String,
    /// Human-readable plugin title for display (e.g., "Main", "CleanShot X")
    pub plugin_title: Option<String>,
    /// Kit name extracted from path (e.g., "main", "cleanshot")
    /// Used for grouping scripts by their source kit in the main menu
    pub kit_name: Option<String>,
    /// Full file body text, read once at load time for content search
    pub body: Option<String>,
}

/// Represents a scriptlet parsed from a markdown file
/// Scriptlets are code snippets extracted from .md files with metadata
#[derive(Clone, Debug)]
pub struct Scriptlet {
    pub name: String,
    pub description: Option<String>,
    pub code: String,
    pub tool: String, // "ts", "bash", "paste", etc.
    pub shortcut: Option<String>,
    pub keyword: Option<String>,
    /// Group name from H1 header (e.g., "Productivity", "Development")
    pub group: Option<String>,
    /// Plugin that owns this scriptlet (e.g., "main", "cleanshot", "tools")
    pub plugin_id: String,
    /// Human-readable plugin title for display (e.g., "Main", "CleanShot X")
    pub plugin_title: Option<String>,
    /// Source file path with anchor for execution (e.g., "/path/to/file.md#slug")
    pub file_path: Option<String>,
    /// Command slug for execution
    pub command: Option<String>,
    /// Alias for quick triggering
    pub alias: Option<String>,
}

impl Scriptlet {
    /// Get a human-readable display name for the tool type
    pub fn tool_display_name(&self) -> &str {
        match self.tool.as_str() {
            "ts" => "TypeScript",
            "js" => "JavaScript",
            "bash" | "sh" => "Shell",
            "zsh" => "Zsh",
            "python" => "Python",
            "ruby" => "Ruby",
            "node" => "Node.js",
            "bun" => "Bun",
            "open" => "Open URL",
            "paste" => "Paste",
            "applescript" => "AppleScript",
            other => other,
        }
    }
}

/// Represents match indices for highlighting matched characters
#[derive(Clone, Debug, Default)]
pub struct MatchIndices {
    /// Indices of matched characters in the name
    pub name_indices: Vec<usize>,
    /// Indices of matched characters in the filename/path
    pub filename_indices: Vec<usize>,
    /// Indices of matched characters in the description
    pub description_indices: Vec<usize>,
}

/// Describes which field produced the winning match for a script
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum ScriptMatchKind {
    /// Matched on name (default)
    #[default]
    Name,
    /// Matched on description
    Description,
    /// Matched on filename
    Filename,
    /// Matched on file body content
    Content,
}

/// Content-hit metadata for a script that matched on body text
#[derive(Clone, Debug)]
pub struct ScriptContentMatch {
    /// 1-based line number of the matching line
    pub line_number: usize,
    /// The full text of the matching line (trimmed)
    pub line_text: String,
    /// Character indices within `line_text` that matched the query
    pub line_match_indices: Vec<usize>,
    /// Byte range of the match within the original body text
    pub byte_range: std::ops::Range<usize>,
}

/// Compute a cache-key signature from a content match: (line_number, byte_start, byte_end).
/// Returns None when there is no content match, matching the "no match" cache state.
pub fn preview_match_signature(
    content_match: Option<&ScriptContentMatch>,
) -> Option<(usize, usize, usize)> {
    content_match.map(|cm| (cm.line_number, cm.byte_range.start, cm.byte_range.end))
}

/// Returns true when the preview cache already holds valid highlighted lines for the
/// requested script path and content-match signature. A miss forces a re-read + re-highlight.
pub fn preview_cache_is_valid(
    cached_path: Option<&str>,
    cached_match_signature: Option<(usize, usize, usize)>,
    cached_lines_empty: bool,
    requested_path: &str,
    content_match: Option<&ScriptContentMatch>,
) -> bool {
    cached_path == Some(requested_path)
        && cached_match_signature == preview_match_signature(content_match)
        && !cached_lines_empty
}

/// Represents a scored match result for fuzzy search
/// Uses Arc<Script> for cheap cloning during filter operations (H1 optimization)
#[derive(Clone, Debug)]
pub struct ScriptMatch {
    pub script: Arc<Script>,
    pub score: i32,
    /// The filename used for matching (e.g., "my-script.ts")
    pub filename: String,
    /// Indices of matched characters for UI highlighting
    pub match_indices: MatchIndices,
    /// Which field produced the dominant match
    pub match_kind: ScriptMatchKind,
    /// Content-hit metadata when match_kind == Content
    pub content_match: Option<ScriptContentMatch>,
}

/// Represents a scored match result for fuzzy search on scriptlets
/// Uses Arc<Scriptlet> for cheap cloning during filter operations (H1 optimization)
#[derive(Clone, Debug)]
pub struct ScriptletMatch {
    pub scriptlet: Arc<Scriptlet>,
    pub score: i32,
    /// The display file path with anchor for matching (e.g., "url.md#open-github")
    pub display_file_path: Option<String>,
    /// Indices of matched characters for UI highlighting
    pub match_indices: MatchIndices,
}

/// Represents a scored match result for fuzzy search on built-in entries
#[derive(Clone, Debug)]
pub struct BuiltInMatch {
    pub entry: crate::builtins::BuiltInEntry,
    pub score: i32,
}

/// Represents a scored match result for fuzzy search on applications
#[derive(Clone, Debug)]
pub struct AppMatch {
    pub app: crate::app_launcher::AppInfo,
    pub score: i32,
}

/// Represents a scored match result for fuzzy search on windows
#[derive(Clone, Debug)]
pub struct WindowMatch {
    pub window: crate::window_control::WindowInfo,
    pub score: i32,
}

/// Represents a scored match result for a plugin-owned skill.
/// Skills always open ACP Chat when selected from the main menu.
#[derive(Clone, Debug)]
pub struct SkillMatch {
    pub skill: Arc<crate::plugins::PluginSkill>,
    pub score: i32,
    /// Indices of matched characters for UI highlighting
    pub match_indices: MatchIndices,
}

/// Represents a scored match result for fuzzy search on agents
/// Uses Arc<Agent> for cheap cloning during filter operations
#[derive(Clone, Debug)]
pub struct AgentMatch {
    pub agent: Arc<Agent>,
    pub score: i32,
    /// The display name for matching
    pub display_name: String,
    /// Indices of matched characters for UI highlighting
    pub match_indices: MatchIndices,
}

/// Represents a fallback command match for the "Use with..." section
///
/// Fallbacks are always shown at the bottom of search results when there's a filter query.
/// They provide Raycast-style actions like "Search Google", "Copy to Clipboard", etc.
#[derive(Clone, Debug)]
pub struct FallbackMatch {
    /// The fallback item (either built-in or script fallback)
    pub fallback: FallbackItem,
    /// Score is always 0 for fallbacks (they sort by priority, not score)
    pub score: i32,
}

/// Unified search result that can be a Script, Scriptlet, Skill, BuiltIn, App, Window, Agent, or Fallback
#[derive(Clone, Debug)]
#[allow(clippy::large_enum_variant)]
pub enum SearchResult {
    Script(ScriptMatch),
    Scriptlet(ScriptletMatch),
    /// Plugin-owned skill that always opens ACP Chat when selected
    Skill(SkillMatch),
    BuiltIn(BuiltInMatch),
    App(AppMatch),
    Window(WindowMatch),
    /// Legacy agent artifact — suppressed from the launcher pipeline.
    /// Agent results are actively filtered out of search/grouping/selection.
    /// ACP agent catalog and provider selection remain intact in `src/ai/acp/`.
    /// Kept as a variant for compilation compatibility; never yielded to the UI.
    Agent(AgentMatch),
    /// Fallback command from "Use with..." section (shown at bottom of search results)
    Fallback(FallbackMatch),
}

impl SearchResult {
    /// Returns true if this result is a legacy Agent variant that should be
    /// suppressed from the launcher pipeline.
    pub fn is_suppressed_agent(&self) -> bool {
        matches!(self, SearchResult::Agent(_))
    }
}

impl SearchResult {
    /// Get the display name for this result
    pub fn name(&self) -> &str {
        match self {
            SearchResult::Script(sm) => &sm.script.name,
            SearchResult::Scriptlet(sm) => &sm.scriptlet.name,
            SearchResult::Skill(sm) => &sm.skill.title,
            SearchResult::BuiltIn(bm) => &bm.entry.name,
            SearchResult::App(am) => &am.app.name,
            SearchResult::Window(wm) => &wm.window.title,
            SearchResult::Agent(am) => &am.agent.name,
            SearchResult::Fallback(fm) => fm.fallback.name(),
        }
    }

    /// Get the description for this result
    pub fn description(&self) -> Option<&str> {
        match self {
            SearchResult::Script(sm) => sm.script.description.as_deref(),
            SearchResult::Scriptlet(sm) => sm.scriptlet.description.as_deref(),
            SearchResult::Skill(sm) => {
                if sm.skill.description.is_empty() {
                    None
                } else {
                    Some(&sm.skill.description)
                }
            }
            SearchResult::BuiltIn(bm) => Some(&bm.entry.description),
            SearchResult::App(am) => am.app.path.to_str(),
            SearchResult::Window(wm) => Some(&wm.window.app),
            SearchResult::Agent(am) => am.agent.description.as_deref(),
            SearchResult::Fallback(fm) => Some(fm.fallback.description()),
        }
    }

    /// Get the score for this result
    pub fn score(&self) -> i32 {
        match self {
            SearchResult::Script(sm) => sm.score,
            SearchResult::Scriptlet(sm) => sm.score,
            SearchResult::Skill(sm) => sm.score,
            SearchResult::BuiltIn(bm) => bm.score,
            SearchResult::App(am) => am.score,
            SearchResult::Window(wm) => wm.score,
            SearchResult::Agent(am) => am.score,
            SearchResult::Fallback(fm) => fm.score,
        }
    }

    /// Get the type label for UI display
    pub fn type_label(&self) -> &'static str {
        match self {
            SearchResult::Script(_) => "Script",
            SearchResult::Scriptlet(_) => "Snippet",
            SearchResult::Skill(_) => "Skill",
            SearchResult::BuiltIn(_) => "Built-in",
            SearchResult::App(_) => "App",
            SearchResult::Window(_) => "Window",
            SearchResult::Agent(_) => "Agent",
            SearchResult::Fallback(_) => "Fallback",
        }
    }

    /// Get a colored type tag for display as a pill badge during search mode.
    /// Returns (label, color) where color is a u32 hex RGB value.
    /// Each type gets a distinct, muted color for visual scanning.
    pub fn type_tag_info(&self) -> (&'static str, u32) {
        match self {
            SearchResult::Script(_) => ("Script", 0x3B82F6), // Blue-500 (saturated for vibrancy)
            SearchResult::Scriptlet(_) => ("Snippet", 0x8B5CF6), // Violet-500
            SearchResult::Skill(_) => ("Skill", 0xFBBF24),   // Gold-400 (matches brand accent)
            SearchResult::BuiltIn(_) => ("Command", 0x34D399), // Emerald-400
            SearchResult::App(_) => ("App", 0xF59E0B),       // Amber-500
            SearchResult::Window(_) => ("Window", 0xEC4899), // Pink-500
            SearchResult::Agent(_) => ("Agent", 0x0EA5E9),   // Sky-500
            SearchResult::Fallback(_) => ("Fallback", 0x6B7280), // Gray-500
        }
    }

    /// Get the plugin/source name for this result (used during search to show origin).
    ///
    /// Resolves to the owning plugin title (or id) for scripts, scriptlets, and skills.
    /// Returns None for items without a meaningful source (built-ins, apps, etc.)
    pub fn source_name(&self) -> Option<&str> {
        match self {
            SearchResult::Script(sm) => {
                sm.script
                    .plugin_title
                    .as_deref()
                    .or(if sm.script.plugin_id.is_empty() {
                        sm.script.kit_name.as_deref()
                    } else {
                        Some(sm.script.plugin_id.as_str())
                    })
            }
            SearchResult::Scriptlet(sm) => {
                sm.scriptlet
                    .plugin_title
                    .as_deref()
                    .or(if sm.scriptlet.plugin_id.is_empty() {
                        sm.scriptlet.group.as_deref()
                    } else {
                        Some(sm.scriptlet.plugin_id.as_str())
                    })
            }
            SearchResult::Skill(sm) => {
                if sm.skill.plugin_title.is_empty() {
                    Some(&sm.skill.plugin_id)
                } else {
                    Some(&sm.skill.plugin_title)
                }
            }
            SearchResult::Agent(am) => am.agent.kit.as_deref(),
            _ => None,
        }
    }

    /// Get the default action text for the primary button.
    ///
    /// Priority:
    /// 1. If the item has a custom `enter` text in typed metadata, use that
    /// 2. Otherwise, return type-based fallback text:
    ///    - Scripts → "Run Script"
    ///    - Commands/Built-ins → "Run Command"
    ///    - Scriptlets/Snippets → "Paste Snippet"
    ///    - Skills → "Open in ACP Chat"
    ///    - Apps → "Launch App"
    ///    - Windows → "Switch to Window"
    ///    - Agents → "Run Agent"
    ///    - Fallbacks → "Run"
    ///
    /// This method is used by both the footer button text and execute_selected().
    pub fn get_default_action_text(&self) -> &str {
        match self {
            SearchResult::Script(sm) => {
                // Check for custom enter text in typed metadata
                if let Some(ref typed_meta) = sm.script.typed_metadata {
                    if let Some(ref enter) = typed_meta.enter {
                        return enter.as_str();
                    }
                }
                "Run Script"
            }
            SearchResult::Scriptlet(sm) => {
                // Scriptlets can also have typed metadata with custom enter text
                // For now, use tool-based fallback
                match sm.scriptlet.tool.as_str() {
                    "paste" | "snippet" => "Paste Snippet",
                    "bash" | "sh" | "zsh" => "Run Command",
                    _ => "Run Snippet",
                }
            }
            SearchResult::Skill(_) => "Open in ACP Chat",
            SearchResult::BuiltIn(_) => "Run Command",
            SearchResult::App(_) => "Launch App",
            SearchResult::Window(_) => "Switch to Window",
            SearchResult::Agent(_) => {
                // Suppressed: agents are not launchable from the main menu
                "Agent (suppressed)"
            }
            SearchResult::Fallback(fm) => {
                // Fallbacks use their action type
                if fm.fallback.is_builtin() {
                    "Run"
                } else {
                    "Run Script"
                }
            }
        }
    }
}

/// Metadata extracted from script file comments
#[derive(Debug, Default, Clone)]
pub struct ScriptMetadata {
    pub name: Option<String>,
    pub description: Option<String>,
    /// Icon name (e.g., "File", "Terminal", "Star", "Folder")
    pub icon: Option<String>,
    /// Alias for quick invocation (e.g., "gpt" triggers on "gpt ")
    pub alias: Option<String>,
    /// Keyboard shortcut for direct invocation (e.g., "opt i", "cmd shift k")
    pub shortcut: Option<String>,
}

/// Schedule metadata extracted from script file comments
/// Used for cron-based script scheduling
#[derive(Debug, Default, Clone)]
pub struct ScheduleMetadata {
    /// Raw cron expression from `// Cron: */5 * * * *`
    pub cron: Option<String>,
    /// Natural language schedule from `// Schedule: every tuesday at 2pm`
    pub schedule: Option<String>,
}

/// Runtime configuration for fallback commands
/// Fallback commands are shown when no search results match,
/// allowing the typed text to be used as input.
#[derive(Debug, Clone)]
pub struct FallbackConfig {
    /// The script that handles this fallback
    pub script: std::sync::Arc<Script>,
    /// Display label with {input} placeholder replaced with actual input
    /// (e.g., "Search docs for {input}" -> "Search docs for my query")
    pub label: String,
    /// The original label template with {input} placeholder
    pub label_template: String,
}

impl FallbackConfig {
    /// Create a new FallbackConfig from a script with fallback metadata
    ///
    /// Returns None if the script doesn't have fallback enabled
    pub fn from_script(script: std::sync::Arc<Script>) -> Option<Self> {
        let typed_meta = script.typed_metadata.as_ref()?;

        if !typed_meta.fallback {
            return None;
        }

        // Use fallback_label if provided, otherwise use script name with {input}
        let label_template = typed_meta
            .fallback_label
            .clone()
            .unwrap_or_else(|| format!("{} {{input}}", script.name));

        Some(Self {
            script,
            label: label_template.clone(), // Will be replaced with actual input at runtime
            label_template,
        })
    }

    /// Update the label by replacing {input} placeholder with actual user input
    pub fn with_input(&self, input: &str) -> Self {
        Self {
            script: self.script.clone(),
            label: self.label_template.replace("{input}", input),
            label_template: self.label_template.clone(),
        }
    }
}
