//! Agents module - mdflow agent integration
//!
//! This module provides functionality for:
//! - Loading agents from `~/.scriptkit/plugins/*/agents/*.md`
//! - Parsing agent metadata from YAML frontmatter
//! - Executing agents via the mdflow CLI
//! - Fuzzy search across agents
//!
//! # Architecture

// These are public API exports for future integration - allow them to be unused for now
#![allow(unused_imports)]
//!
//! Agents are markdown files that can be executed via mdflow, a multi-backend
//! CLI for AI prompts. The filename determines which backend to use:
//!
//! | Pattern | Backend |
//! |---------|---------|
//! | `*.claude.md` | Claude |
//! | `*.gemini.md` | Gemini |
//! | `*.codex.md` | Codex |
//! | `*.copilot.md` | Copilot |
//! | `*.md` | Generic |
//!
//! # Metadata Convention
//!
//! Script Kit metadata uses `_sk_*` prefixed keys in frontmatter:
//!
//! ```yaml
//! ---
//! _sk_name: "Review PR"
//! _sk_description: "Reviews staged changes"
//! _sk_icon: "git-pull-request"
//! _sk_alias: "review"
//! _sk_shortcut: "cmd shift r"
//! model: sonnet
//! ---
//! ```
//!
//! This convention ensures Script Kit metadata doesn't leak to CLI flags,
//! as mdflow treats underscore-prefixed keys as template variables.
//!
//! # Execution Model
//!
//! We spawn `mdflow <file>` directly and let mdflow handle:
//! - Frontmatter → CLI flag conversion
//! - Config merging
//! - Template variable substitution
//! - `@file` imports and `!command` inlines
//!
//! Script Kit only adds runtime overrides:
//! - `--_quiet --raw` for UI capture mode
//! - `--_varname value` for user-provided variables
//! - stdin piping for `{{ _stdin }}` support
//!
//! # Module Structure
//!
//! - `types` - Core data types (Agent, AgentBackend, AgentFrontmatter, etc.)
//! - `parser` - Frontmatter and agent file parsing
//! - `loader` - File system loading and globbing
//! - `executor` - Agent execution via mdflow CLI

mod executor;
mod loader;
mod parser;
mod types;

// Re-export core types
pub use types::{
    Agent, AgentAvailability, AgentBackend, AgentExecutionMode, AgentFrontmatter, MdflowInput,
    MdflowInputType, MdflowInputs,
};

// AgentMatch is defined in scripts::types to avoid circular dependencies
// Re-export it from there for convenience
pub use crate::scripts::AgentMatch;

// Re-export loader functions
pub use loader::{load_agent_from_path, load_agents, load_agents_from_path};

// Re-export parser functions
pub use parser::{
    has_remote_imports, has_shell_inlines, is_interactive_filename, name_from_filename,
    parse_agent, parse_frontmatter,
};

// Re-export executor functions
pub use executor::{
    build_terminal_command, check_availability, dry_run_agent, execute_agent, explain_agent,
    get_install_instructions, get_mdflow_command, is_mdflow_available,
};

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_module_exports() {
        // Verify types are exported
        let _backend = AgentBackend::Claude;
        let _mode = AgentExecutionMode::UiCapture;
        let _input_type = MdflowInputType::Text;

        // Verify functions are exported
        let _ = is_mdflow_available();
        let _ = AgentBackend::from_filename("test.claude.md");
        let _ = name_from_filename("test.claude.md");
        let _ = is_interactive_filename("test.i.claude.md");
    }

    #[test]
    fn test_parse_agent_integration() {
        let content = r#"---
_sk_name: "Test Agent"
_sk_description: "Test description"
model: sonnet
---
Test prompt
"#;

        let agent = parse_agent(Path::new("/test/agent.claude.md"), content).unwrap();

        assert_eq!(agent.name, "Test Agent");
        assert_eq!(agent.description, Some("Test description".to_string()));
        assert_eq!(agent.backend, AgentBackend::Claude);
    }
}
