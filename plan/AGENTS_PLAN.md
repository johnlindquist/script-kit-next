# Agents System Plan

> Integrate mdflow as runnable markdown agents in `~/.sk/kit/main/agents/*.md`

## Overview

This plan covers integrating [mdflow](https://github.com/johnlindquist/mdflow) as first-class agents in Script Kit, following the same patterns established for scripts and scriptlets:

1. **File watching**: Monitor `~/.sk/kit/*/agents/` directories for changes
2. **Parsing**: Extract agent metadata from YAML frontmatter and filename patterns
3. **Main menu integration**: Display agents alongside scripts/scriptlets in unified search
4. **Execution**: Run agents via `mdflow` CLI with proper stdin/stdout handling
5. **Backend selection**: Support `.claude.md`, `.gemini.md`, `.codex.md`, `.copilot.md` filename patterns

## mdflow Key Concepts

| Concept | Description | Example |
|---------|-------------|---------|
| **Filename → Command** | `task.claude.md` runs `claude` | `review.gemini.md` → `gemini` |
| **Frontmatter → CLI Flags** | YAML keys become CLI flags | `model: opus` → `--model opus` |
| **Body → Prompt** | Markdown body is the final argument | Full prompt text |
| **`@file` imports** | Inline file contents | `@./src/**/*.ts` |
| **`!command` inlines** | Execute and inline output | `` !`git log -5` `` |
| **`{{_stdin}}`** | Piped input variable | Template substitution |
| **`{{_1}}`, `{{_2}}`** | Positional arguments | CLI arg placeholders |
| **`.i.` marker** | Interactive mode | `task.i.claude.md` |

## Current State Analysis

### Existing Patterns to Follow

| Feature | Scripts | Scriptlets | Agents (Planned) |
|---------|---------|------------|------------------|
| **Location** | `~/.sk/kit/*/scripts/*.ts` | `~/.sk/kit/*/scriptlets/*.md` | `~/.sk/kit/*/agents/*.md` |
| **Struct** | `Script` | `Scriptlet` | `Agent` |
| **Match type** | `ScriptMatch` | `ScriptletMatch` | `AgentMatch` |
| **Search result** | `SearchResult::Script` | `SearchResult::Scriptlet` | `SearchResult::Agent` |
| **File watcher** | `ScriptWatcher` | (same watcher) | Extend `ScriptWatcher` |
| **Loader fn** | `read_scripts()` | `load_scriptlets()` | `load_agents()` |
| **Fuzzy search** | `fuzzy_search_scripts()` | `fuzzy_search_scriptlets()` | `fuzzy_search_agents()` |

### Key Files to Modify

| File | Purpose | Changes Needed |
|------|---------|----------------|
| `src/scripts.rs` | Core types and search | Add `Agent`, `AgentMatch`, extend `SearchResult` |
| `src/watcher.rs` | File watching | Watch `agents/` directory, add `AgentReloadEvent` |
| `src/agents.rs` | **NEW** | Agent parsing, frontmatter extraction, backend detection |
| `src/executor.rs` | Script execution | Add `execute_agent()` function |
| `src/app_impl.rs` | App state | Add `agents: Vec<Arc<Agent>>` field |
| `src/render_script_list.rs` | UI rendering | Handle `SearchResult::Agent` |
| `src/frecency.rs` | Recency tracking | Include agents in frecency store |

---

## Phase 1: Data Model

### Agent Struct

```rust
// src/agents.rs

/// Represents an mdflow agent parsed from a .md file
#[derive(Clone, Debug)]
pub struct Agent {
    /// Display name (from frontmatter or filename)
    pub name: String,
    /// File path to the .md file
    pub path: PathBuf,
    /// Backend inferred from filename (claude, gemini, codex, copilot)
    pub backend: AgentBackend,
    /// Whether this is interactive mode (has .i. in filename)
    pub interactive: bool,
    /// Description from frontmatter
    pub description: Option<String>,
    /// Icon name
    pub icon: Option<String>,
    /// Keyboard shortcut
    pub shortcut: Option<String>,
    /// Alias for quick triggering
    pub alias: Option<String>,
    /// Model override from frontmatter
    pub model: Option<String>,
    /// Raw frontmatter for CLI flag generation
    pub frontmatter: AgentFrontmatter,
    /// The kit this agent belongs to (e.g., "main", "custom-kit")
    pub kit: Option<String>,
}

/// Supported AI backends
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AgentBackend {
    Claude,
    Gemini,
    Codex,
    Copilot,
    /// Generic - requires --_command flag
    Generic,
}

impl AgentBackend {
    /// Parse backend from filename pattern
    /// e.g., "review.claude.md" → Claude
    pub fn from_filename(filename: &str) -> Self {
        let name = filename.to_lowercase();
        if name.contains(".claude.") { Self::Claude }
        else if name.contains(".gemini.") { Self::Gemini }
        else if name.contains(".codex.") { Self::Codex }
        else if name.contains(".copilot.") { Self::Copilot }
        else { Self::Generic }
    }
    
    /// Get the CLI command for this backend
    pub fn command(&self) -> Option<&'static str> {
        match self {
            Self::Claude => Some("claude"),
            Self::Gemini => Some("gemini"),
            Self::Codex => Some("codex"),
            Self::Copilot => Some("copilot"),
            Self::Generic => None,
        }
    }
    
    /// Icon for this backend
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Claude => "anthropic",
            Self::Gemini => "google",
            Self::Codex => "openai",
            Self::Copilot => "github",
            Self::Generic => "bot",
        }
    }
}

/// Parsed frontmatter from agent .md file
#[derive(Clone, Debug, Default)]
pub struct AgentFrontmatter {
    /// Model to use (e.g., "opus", "sonnet", "gemini-2.0-flash")
    pub model: Option<String>,
    /// Print mode (non-interactive)
    pub print: Option<bool>,
    /// Interactive mode override
    pub interactive: Option<bool>,
    /// MCP config file path
    pub mcp_config: Option<String>,
    /// Skip permission prompts
    pub dangerously_skip_permissions: Option<bool>,
    /// Template variables (keys starting with _)
    pub variables: HashMap<String, String>,
    /// All other keys → CLI flags
    pub extra: HashMap<String, serde_yaml::Value>,
}
```

### Search Integration

```rust
// In src/scripts.rs

/// Represents a scored match result for fuzzy search on agents
#[derive(Clone, Debug)]
pub struct AgentMatch {
    pub agent: Arc<Agent>,
    pub score: i32,
    /// The display name for matching
    pub display_name: String,
    /// Indices of matched characters for UI highlighting
    pub match_indices: MatchIndices,
}

// Extend SearchResult enum
pub enum SearchResult {
    Script(ScriptMatch),
    Scriptlet(ScriptletMatch),
    BuiltIn(BuiltInMatch),
    App(AppMatch),
    Window(WindowMatch),
    Agent(AgentMatch),  // NEW
}

impl SearchResult {
    pub fn name(&self) -> &str {
        match self {
            // ... existing matches ...
            SearchResult::Agent(am) => &am.agent.name,
        }
    }
    
    pub fn description(&self) -> Option<&str> {
        match self {
            // ... existing matches ...
            SearchResult::Agent(am) => am.agent.description.as_deref(),
        }
    }
    
    pub fn type_label(&self) -> &'static str {
        match self {
            // ... existing matches ...
            SearchResult::Agent(_) => "Agent",
        }
    }
}
```

---

## Phase 2: Parsing

### Frontmatter Parser

```rust
// src/agents.rs

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Parse agent frontmatter from markdown content
/// 
/// Frontmatter format:
/// ```yaml
/// ---
/// model: opus
/// _feature: "default value"
/// dangerously-skip-permissions: true
/// ---
/// ```
pub fn parse_agent_frontmatter(content: &str) -> Option<AgentFrontmatter> {
    let trimmed = content.trim_start();
    
    // Must start with ---
    if !trimmed.starts_with("---") {
        return None;
    }
    
    // Find closing ---
    let after_first = &trimmed[3..];
    let end_pos = after_first.find("\n---")?;
    let yaml_content = &after_first[..end_pos].trim();
    
    // Parse as generic YAML first
    let raw: HashMap<String, serde_yaml::Value> = 
        serde_yaml::from_str(yaml_content).ok()?;
    
    let mut frontmatter = AgentFrontmatter::default();
    
    for (key, value) in raw {
        match key.as_str() {
            "model" => {
                frontmatter.model = value.as_str().map(|s| s.to_string());
            }
            "print" | "_print" => {
                frontmatter.print = value.as_bool();
            }
            "_interactive" | "_i" => {
                frontmatter.interactive = value.as_bool().or(Some(true));
            }
            "mcp-config" | "mcp_config" => {
                frontmatter.mcp_config = value.as_str().map(|s| s.to_string());
            }
            "dangerously-skip-permissions" => {
                frontmatter.dangerously_skip_permissions = value.as_bool();
            }
            _ if key.starts_with('_') => {
                // Template variable
                if let Some(s) = value.as_str() {
                    frontmatter.variables.insert(key, s.to_string());
                }
            }
            _ => {
                // Other key → CLI flag
                frontmatter.extra.insert(key, value);
            }
        }
    }
    
    Some(frontmatter)
}

/// Parse agent metadata from file path and content
pub fn parse_agent(path: &Path, content: &str) -> Option<Agent> {
    let filename = path.file_name()?.to_str()?;
    
    // Skip hidden files
    if filename.starts_with('.') {
        return None;
    }
    
    // Must be .md file
    if !filename.ends_with(".md") {
        return None;
    }
    
    // Parse backend from filename
    let backend = AgentBackend::from_filename(filename);
    
    // Check for interactive marker (.i.)
    let interactive = filename.contains(".i.");
    
    // Parse frontmatter
    let frontmatter = parse_agent_frontmatter(content).unwrap_or_default();
    
    // Extract name: prefer frontmatter name, fall back to filename
    let name = frontmatter.extra
        .get("name")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| {
            // Remove .md and backend suffix for display name
            filename
                .trim_end_matches(".md")
                .split('.')
                .next()
                .unwrap_or(filename)
                .replace('-', " ")
                .replace('_', " ")
        });
    
    // Extract optional metadata
    let description = frontmatter.extra
        .get("description")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    
    let icon = frontmatter.extra
        .get("icon")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    
    let shortcut = frontmatter.extra
        .get("shortcut")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    
    let alias = frontmatter.extra
        .get("alias")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    
    Some(Agent {
        name,
        path: path.to_path_buf(),
        backend,
        interactive,
        description,
        icon,
        shortcut,
        alias,
        model: frontmatter.model.clone(),
        frontmatter,
        kit: None, // Set by loader
    })
}
```

### Loader Function

```rust
// src/agents.rs

use glob::glob;
use std::sync::Arc;
use tracing::{debug, warn};
use crate::setup::get_kit_path;

/// Load agents from all kits
/// 
/// Globs: ~/.sk/kit/*/agents/*.md
/// 
/// Returns Arc-wrapped agents sorted by name.
pub fn load_agents() -> Vec<Arc<Agent>> {
    let kit_path = get_kit_path();
    let mut agents = Vec::new();
    
    let pattern = kit_path.join("*/agents/*.md");
    let pattern_str = pattern.to_string_lossy().to_string();
    
    debug!(pattern = %pattern_str, "Globbing for agent files");
    
    match glob(&pattern_str) {
        Ok(paths) => {
            for entry in paths {
                match entry {
                    Ok(path) => {
                        debug!(path = %path.display(), "Parsing agent file");
                        
                        match std::fs::read_to_string(&path) {
                            Ok(content) => {
                                if let Some(mut agent) = parse_agent(&path, &content) {
                                    // Extract kit from path
                                    agent.kit = extract_kit_from_path(&path, &kit_path);
                                    agents.push(Arc::new(agent));
                                }
                            }
                            Err(e) => {
                                warn!(
                                    error = %e,
                                    path = %path.display(),
                                    "Failed to read agent file"
                                );
                            }
                        }
                    }
                    Err(e) => {
                        warn!(error = %e, "Failed to process glob entry");
                    }
                }
            }
        }
        Err(e) => {
            warn!(
                error = %e,
                pattern = %pattern_str,
                "Failed to glob agent files"
            );
        }
    }
    
    // Sort by name
    agents.sort_by(|a, b| a.name.cmp(&b.name));
    
    debug!(count = agents.len(), "Loaded agents");
    agents
}

/// Extract kit name from path
fn extract_kit_from_path(path: &Path, kit_root: &Path) -> Option<String> {
    let kit_prefix = format!("{}/", kit_root.display());
    let path_str = path.to_string_lossy();
    
    if path_str.starts_with(&kit_prefix) {
        let relative = &path_str[kit_prefix.len()..];
        relative.split('/').next().map(|s| s.to_string())
    } else {
        None
    }
}
```

---

## Phase 3: File Watching

### Extend ScriptWatcher

```rust
// src/watcher.rs

/// Event types for agent file changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentReloadEvent {
    /// A specific agent file was modified
    FileChanged(PathBuf),
    /// A new agent file was created
    FileCreated(PathBuf),
    /// An agent file was deleted
    FileDeleted(PathBuf),
    /// Full reload needed
    FullReload,
}

/// Check if path is a relevant agent file
fn is_agent_file(path: &Path) -> bool {
    // Skip hidden files
    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
        if name.starts_with('.') {
            return false;
        }
    }
    
    // Must be .md file
    if path.extension().and_then(|e| e.to_str()) != Some("md") {
        return false;
    }
    
    // Must be in an agents/ directory
    path.parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .map(|n| n == "agents")
        .unwrap_or(false)
}

// In ScriptWatcher::watch_loop(), add:
// 1. Watch ~/.sk/kit/*/agents/ directories
// 2. Filter for .md files in agents/ subdirs
// 3. Emit AgentReloadEvent for changes
```

---

## Phase 4: Fuzzy Search

```rust
// src/scripts.rs

/// Fuzzy search agents by query string
/// Searches across name, description, backend, and alias
pub fn fuzzy_search_agents(agents: &[Arc<Agent>], query: &str) -> Vec<AgentMatch> {
    if query.is_empty() {
        return agents
            .iter()
            .map(|a| AgentMatch {
                agent: Arc::clone(a),
                score: 0,
                display_name: a.name.clone(),
                match_indices: MatchIndices::default(),
            })
            .collect();
    }
    
    let query_lower = query.to_lowercase();
    let mut matches = Vec::new();
    
    let pattern = Pattern::parse(
        &query_lower,
        nucleo_matcher::pattern::CaseMatching::Ignore,
        nucleo_matcher::pattern::Normalization::Smart,
    );
    let mut matcher = Matcher::new(nucleo_matcher::Config::DEFAULT);
    
    for agent in agents {
        let mut score = 0i32;
        
        // Score by name match - highest priority
        if let Some(pos) = find_ignore_ascii_case(&agent.name, &query_lower) {
            score += if pos == 0 { 100 } else { 75 };
        }
        
        // Nucleo fuzzy matching on name
        if let Some(nucleo_s) = nucleo_score(&agent.name, &pattern, &mut matcher) {
            score += 50 + (nucleo_s / 20) as i32;
        }
        
        // Score by backend name
        if let Some(cmd) = agent.backend.command() {
            if contains_ignore_ascii_case(cmd, &query_lower) {
                score += 40;
            }
        }
        
        // Score by description
        if let Some(ref desc) = agent.description {
            if contains_ignore_ascii_case(desc, &query_lower) {
                score += 25;
            }
        }
        
        // Score by alias
        if let Some(ref alias) = agent.alias {
            if contains_ignore_ascii_case(alias, &query_lower) {
                score += 60;
            }
        }
        
        if score > 0 {
            matches.push(AgentMatch {
                agent: Arc::clone(agent),
                score,
                display_name: agent.name.clone(),
                match_indices: MatchIndices::default(),
            });
        }
    }
    
    matches.sort_by(|a, b| match b.score.cmp(&a.score) {
        Ordering::Equal => a.agent.name.cmp(&b.agent.name),
        other => other,
    });
    
    matches
}
```

---

## Phase 5: Execution

### Agent Executor

```rust
// src/agents.rs or src/executor.rs

use std::process::{Command, Stdio};

/// Execute an agent using mdflow
/// 
/// Flow:
/// 1. Build CLI args from frontmatter
/// 2. Spawn `mdflow <agent.md> [args...]`
/// 3. Pipe stdin if provided
/// 4. Return stdout/stderr
pub fn execute_agent(
    agent: &Agent,
    positional_args: &[String],
    stdin_input: Option<&str>,
) -> anyhow::Result<std::process::Child> {
    let mut cmd = Command::new("mdflow");
    
    // Add the agent file path
    cmd.arg(&agent.path);
    
    // Add positional arguments
    for arg in positional_args {
        cmd.arg(arg);
    }
    
    // Convert frontmatter to CLI flags
    for (key, value) in &agent.frontmatter.extra {
        add_frontmatter_flag(&mut cmd, key, value);
    }
    
    // Add template variable overrides
    for (key, value) in &agent.frontmatter.variables {
        cmd.arg(format!("--{}", key));
        cmd.arg(value);
    }
    
    // Set up I/O
    cmd.stdin(if stdin_input.is_some() { Stdio::piped() } else { Stdio::null() });
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    
    let mut child = cmd.spawn()?;
    
    // Write stdin if provided
    if let Some(input) = stdin_input {
        if let Some(mut stdin) = child.stdin.take() {
            use std::io::Write;
            stdin.write_all(input.as_bytes())?;
        }
    }
    
    Ok(child)
}

/// Convert YAML value to CLI flag(s)
fn add_frontmatter_flag(cmd: &mut Command, key: &str, value: &serde_yaml::Value) {
    let flag = if key.len() == 1 {
        format!("-{}", key)
    } else {
        format!("--{}", key)
    };
    
    match value {
        serde_yaml::Value::Bool(true) => {
            cmd.arg(&flag);
        }
        serde_yaml::Value::Bool(false) => {
            // Omit false booleans
        }
        serde_yaml::Value::String(s) => {
            cmd.arg(&flag);
            cmd.arg(s);
        }
        serde_yaml::Value::Number(n) => {
            cmd.arg(&flag);
            cmd.arg(n.to_string());
        }
        serde_yaml::Value::Sequence(arr) => {
            // Repeated flags for arrays
            for item in arr {
                if let Some(s) = item.as_str() {
                    cmd.arg(&flag);
                    cmd.arg(s);
                }
            }
        }
        _ => {}
    }
}
```

### Interactive Agent Session

For interactive agents (`.i.` marker or `_interactive: true`), we need special handling:

```rust
/// Start an interactive agent session
/// 
/// This creates a pseudo-terminal for the AI CLI to interact with the user.
pub fn execute_agent_interactive(
    agent: &Agent,
    positional_args: &[String],
) -> anyhow::Result<AgentSession> {
    // Interactive agents need a PTY
    // Similar to how we handle terminal prompts in term_prompt.rs
    
    // For MVP, we can shell out to mdflow in a terminal window
    // Later: embed using portable-pty like we do for term()
    
    todo!("Implement interactive agent session")
}
```

---

## Phase 6: UI Integration

### Main Menu Rendering

```rust
// src/render_script_list.rs

// In the list item rendering closure, add handling for Agent:

GroupedListItem::Item(result_idx) => {
    if let Some(result) = flat_results_clone.get(*result_idx) {
        match result {
            // ... existing Script, Scriptlet, BuiltIn handlers ...
            
            SearchResult::Agent(am) => {
                // Render agent list item
                render_agent_item(
                    &am.agent,
                    ix,
                    is_selected,
                    is_hovered,
                    theme_colors,
                )
            }
        }
    }
}

fn render_agent_item(
    agent: &Agent,
    ix: usize,
    is_selected: bool,
    is_hovered: bool,
    colors: ListItemColors,
) -> AnyElement {
    let bg_color = if is_selected {
        colors.background_selected
    } else if is_hovered {
        colors.background_hovered
    } else {
        colors.background
    };
    
    let backend_badge = match agent.backend {
        AgentBackend::Claude => "Claude",
        AgentBackend::Gemini => "Gemini",
        AgentBackend::Codex => "Codex",
        AgentBackend::Copilot => "Copilot",
        AgentBackend::Generic => "Agent",
    };
    
    div()
        .id(ElementId::NamedInteger("agent-item".into(), ix as u64))
        .h(px(LIST_ITEM_HEIGHT))
        .w_full()
        .flex()
        .flex_row()
        .items_center()
        .px(px(16.))
        .gap(px(12.))
        .bg(rgb(bg_color))
        // Icon
        .child(
            div()
                .flex()
                .items_center()
                .justify_center()
                .size(px(24.))
                .child(
                    svg()
                        .path(format!("icons/{}.svg", agent.icon.as_deref()
                            .unwrap_or(agent.backend.icon())))
                        .size(px(16.))
                        .text_color(rgb(colors.text_secondary))
                )
        )
        // Name and description
        .child(
            div()
                .flex_1()
                .flex()
                .flex_col()
                .overflow_hidden()
                .child(
                    div()
                        .text_color(rgb(colors.text_primary))
                        .text_ellipsis()
                        .child(agent.name.clone())
                )
                .when_some(agent.description.clone(), |d, desc| {
                    d.child(
                        div()
                            .text_color(rgb(colors.text_secondary))
                            .text_xs()
                            .text_ellipsis()
                            .child(desc)
                    )
                })
        )
        // Backend badge
        .child(
            div()
                .px(px(8.))
                .py(px(2.))
                .rounded(px(4.))
                .bg(rgba((colors.accent << 8) | 0x30))
                .text_xs()
                .text_color(rgb(colors.text_secondary))
                .child(backend_badge)
        )
        .into_any_element()
}
```

### Grouped Results Integration

```rust
// src/scripts.rs - extend get_grouped_results()

// Add agents to the unified search
pub fn fuzzy_search_unified_with_agents(
    scripts: &[Arc<Script>],
    scriptlets: &[Arc<Scriptlet>],
    builtins: &[BuiltInEntry],
    apps: &[AppInfo],
    agents: &[Arc<Agent>],  // NEW
    query: &str,
) -> Vec<SearchResult> {
    let mut results = Vec::new();
    
    // ... existing builtin, app, script, scriptlet searches ...
    
    // Search agents
    let agent_matches = fuzzy_search_agents(agents, query);
    for am in agent_matches {
        results.push(SearchResult::Agent(am));
    }
    
    // Sort with agents after apps but before scripts
    results.sort_by(|a, b| {
        match b.score().cmp(&a.score()) {
            Ordering::Equal => {
                let type_order = |r: &SearchResult| -> i32 {
                    match r {
                        SearchResult::BuiltIn(_) => 0,
                        SearchResult::App(_) => 1,
                        SearchResult::Agent(_) => 2,  // NEW
                        SearchResult::Window(_) => 3,
                        SearchResult::Script(_) => 4,
                        SearchResult::Scriptlet(_) => 5,
                    }
                };
                type_order(a).cmp(&type_order(b))
            }
            other => other,
        }
    });
    
    results
}
```

---

## Phase 7: App State Integration

```rust
// src/app_impl.rs

impl ScriptListApp {
    // Add agents field
    agents: Vec<Arc<Agent>>,
    
    // In new() or initialize:
    pub fn load_all_content(&mut self) {
        self.scripts = scripts::read_scripts();
        self.scriptlets = scripts::load_scriptlets();
        self.agents = agents::load_agents();  // NEW
        
        // Invalidate caches
        self.results_cache.dirty = true;
        self.grouped_cache.dirty = true;
    }
    
    // In execute_selected():
    pub fn execute_selected(&mut self, cx: &mut Context<Self>) {
        let (_, results) = self.get_grouped_results_cached();
        
        if let Some(grouped_item) = self.grouped_items.get(self.selected_index) {
            if let GroupedListItem::Item(result_idx) = grouped_item {
                if let Some(result) = results.get(*result_idx) {
                    match result {
                        SearchResult::Script(sm) => self.execute_script(&sm.script, cx),
                        SearchResult::Scriptlet(sm) => self.execute_scriptlet(&sm.scriptlet, cx),
                        SearchResult::Agent(am) => self.execute_agent(&am.agent, cx),  // NEW
                        // ... other handlers ...
                    }
                }
            }
        }
    }
    
    // New agent execution handler
    fn execute_agent(&mut self, agent: &Agent, cx: &mut Context<Self>) {
        logging::log("EXEC", &format!("Executing agent: {}", agent.name));
        
        // Record frecency
        let agent_key = format!("agent:{}", agent.path.display());
        self.frecency_store.record_access(&agent_key);
        
        // Execute via mdflow
        match agents::execute_agent(agent, &[], None) {
            Ok(child) => {
                // Handle output similar to script execution
                // For interactive agents, may need terminal integration
                self.handle_agent_process(child, agent, cx);
            }
            Err(e) => {
                logging::log("EXEC", &format!("Agent execution failed: {}", e));
                // Show error to user
            }
        }
    }
}
```

---

## Phase 8: Testing

### Test Cases

```rust
#[cfg(test)]
mod agent_tests {
    use super::*;
    
    #[test]
    fn test_backend_from_filename() {
        assert_eq!(
            AgentBackend::from_filename("review.claude.md"),
            AgentBackend::Claude
        );
        assert_eq!(
            AgentBackend::from_filename("task.gemini.md"),
            AgentBackend::Gemini
        );
        assert_eq!(
            AgentBackend::from_filename("analyze.i.codex.md"),
            AgentBackend::Codex
        );
        assert_eq!(
            AgentBackend::from_filename("generic.md"),
            AgentBackend::Generic
        );
    }
    
    #[test]
    fn test_interactive_detection() {
        let agent = parse_agent(
            Path::new("task.i.claude.md"),
            "---\nmodel: opus\n---\nPrompt here"
        ).unwrap();
        assert!(agent.interactive);
        
        let agent2 = parse_agent(
            Path::new("task.claude.md"),
            "---\nmodel: opus\n---\nPrompt here"
        ).unwrap();
        assert!(!agent2.interactive);
    }
    
    #[test]
    fn test_frontmatter_parsing() {
        let content = r#"---
model: opus
_feature_name: Authentication
dangerously-skip-permissions: true
add-dir:
  - ./src
  - ./tests
---
Build {{ _feature_name }}.
"#;
        
        let fm = parse_agent_frontmatter(content).unwrap();
        assert_eq!(fm.model, Some("opus".to_string()));
        assert_eq!(fm.dangerously_skip_permissions, Some(true));
        assert!(fm.variables.contains_key("_feature_name"));
        assert!(fm.extra.contains_key("add-dir"));
    }
}
```

### Smoke Test

```typescript
// tests/smoke/test-agents.ts
import '../../scripts/kit-sdk';

export const metadata = {
  name: "Agent Integration Test",
  description: "Tests agent loading and display",
};

console.error('[SMOKE] Testing agent integration...');

// Create a test agent file
const agentPath = join(process.env.HOME!, '.sk/kit/main/agents/test.claude.md');
await writeFile(agentPath, `---
model: sonnet
description: Test agent for smoke testing
---
Hello from test agent!
`);

// Verify it appears in search (would need SDK function)
// For now, just verify file was created
console.error('[SMOKE] Agent file created at:', agentPath);

process.exit(0);
```

---

## Implementation Order

1. **Phase 1**: Data model (`src/agents.rs` - structs only)
2. **Phase 2**: Parsing (frontmatter + file parsing)
3. **Phase 3**: Loader function (`load_agents()`)
4. **Phase 4**: Search integration (extend `SearchResult`, add `fuzzy_search_agents`)
5. **Phase 5**: File watcher (extend `ScriptWatcher`)
6. **Phase 6**: App state integration (add `agents` field, handle reload events)
7. **Phase 7**: UI rendering (`render_agent_item`)
8. **Phase 8**: Execution (spawn mdflow process)
9. **Phase 9**: Testing (unit tests + smoke tests)

---

## Open Questions

1. **Interactive mode handling**: Should interactive agents open in a terminal window or embed in the Script Kit UI?
   - Option A: Open system terminal (simple, reliable)
   - Option B: Embed with PTY (consistent UX, more complex)
   
2. **mdflow installation**: Should Script Kit bundle mdflow or require user installation?
   - Option A: Bundle as npm dependency
   - Option B: Auto-install on first use
   - Option C: Require manual installation, show helpful error
   
3. **Template variable UI**: When an agent has `_inputs`, should we show a form like scriptlets do?
   - Could leverage existing `fields()` prompt type
   
4. **Streaming output**: Should agent output stream to a div in real-time or wait for completion?
   - For print mode: stream to panel
   - For interactive mode: full terminal needed

5. **Backend availability**: How to handle missing backends (e.g., user doesn't have Claude CLI)?
   - Check on startup and show warning
   - Check on execution and show helpful install instructions

---

## Dependencies

- **mdflow**: `npm install -g mdflow` or bundled
- **AI CLIs**: claude, gemini, codex, copilot (user-installed)
- **Existing**: All current Script Kit infrastructure

## Success Criteria

- [ ] Agents in `~/.sk/kit/*/agents/*.md` appear in main menu
- [ ] Fuzzy search works across agent name, description, backend
- [ ] File watcher detects new/modified/deleted agents
- [ ] Executing an agent spawns mdflow with correct arguments
- [ ] Backend badge shows correct AI provider
- [ ] Frecency tracking includes agents
- [ ] Interactive agents work (basic terminal integration)
