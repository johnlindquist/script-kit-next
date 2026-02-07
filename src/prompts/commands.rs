//! Slash command system for AI chat
//!
//! Provides /command support for common AI operations:
//! - `/explain` - Explain selected code or topic
//! - `/fix` - Fix errors in code
//! - `/test` - Generate unit tests
//! - `/improve` - Improve writing or code
//! - `/summarize` - Summarize text
//!
//! Based on patterns from GitHub Copilot and Raycast.

/// Types of slash commands available
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SlashCommandType {
    /// Explain code or concepts clearly
    Explain,
    /// Fix errors or bugs in code
    Fix,
    /// Generate unit tests for code
    Test,
    /// Improve code quality or writing
    Improve,
    /// Summarize text or content
    Summarize,
}

impl SlashCommandType {
    pub const fn all() -> &'static [SlashCommandType] {
        &[
            SlashCommandType::Explain,
            SlashCommandType::Fix,
            SlashCommandType::Test,
            SlashCommandType::Improve,
            SlashCommandType::Summarize,
        ]
    }

    /// Parse a user-provided command keyword (without slash) to a typed command.
    pub fn from_keyword(keyword: &str) -> Option<Self> {
        match keyword {
            "explain" => Some(SlashCommandType::Explain),
            "fix" => Some(SlashCommandType::Fix),
            "test" | "tests" => Some(SlashCommandType::Test),
            "improve" => Some(SlashCommandType::Improve),
            "summarize" | "summary" => Some(SlashCommandType::Summarize),
            _ => None,
        }
    }

    /// All accepted command keywords (canonical name first, then aliases).
    pub fn keywords(&self) -> &'static [&'static str] {
        match self {
            SlashCommandType::Explain => &["explain"],
            SlashCommandType::Fix => &["fix"],
            SlashCommandType::Test => &["test", "tests"],
            SlashCommandType::Improve => &["improve"],
            SlashCommandType::Summarize => &["summarize", "summary"],
        }
    }

    pub fn matches_keyword_prefix(&self, prefix: &str) -> bool {
        self.keywords()
            .iter()
            .any(|keyword| keyword.starts_with(prefix))
    }

    /// Get the system prompt prefix for this command type
    pub fn system_context(&self) -> &'static str {
        match self {
            SlashCommandType::Explain => {
                "You are a helpful coding assistant. Your task is to explain the following clearly and concisely. \
                 Break down complex concepts into understandable parts. Use examples where helpful. \
                 If explaining code, describe what it does, how it works, and any important patterns or techniques used."
            }
            SlashCommandType::Fix => {
                "You are an expert debugger and code fixer. Your task is to identify and fix errors in the following code. \
                 First, identify the problem(s). Then provide the corrected code with clear explanations of what was wrong \
                 and how you fixed it. If there are multiple issues, address them all."
            }
            SlashCommandType::Test => {
                "You are an expert at writing comprehensive unit tests. Your task is to generate thorough test cases \
                 for the following code. Include edge cases, error conditions, and typical use cases. \
                 Use appropriate testing patterns and frameworks for the language. Provide clear test names that describe what is being tested."
            }
            SlashCommandType::Improve => {
                "You are an expert code reviewer and technical writer. Your task is to improve the following. \
                 For code: suggest better patterns, cleaner structure, improved performance, and enhanced readability. \
                 For writing: improve clarity, grammar, flow, and impact. Provide the improved version with explanations of changes."
            }
            SlashCommandType::Summarize => {
                "You are a skilled summarizer. Your task is to provide a clear, concise summary of the following content. \
                 Capture the key points and main ideas. For code, describe its purpose and functionality. \
                 For text, extract the essential information while preserving meaning."
            }
        }
    }

    /// Get the command name (without slash)
    pub fn name(&self) -> &'static str {
        match self {
            SlashCommandType::Explain => "explain",
            SlashCommandType::Fix => "fix",
            SlashCommandType::Test => "test",
            SlashCommandType::Improve => "improve",
            SlashCommandType::Summarize => "summarize",
        }
    }

    /// Get a user-friendly description
    pub fn description(&self) -> &'static str {
        match self {
            SlashCommandType::Explain => "Explain code or concepts",
            SlashCommandType::Fix => "Fix errors in code",
            SlashCommandType::Test => "Generate unit tests",
            SlashCommandType::Improve => "Improve code or writing",
            SlashCommandType::Summarize => "Summarize content",
        }
    }

    /// Get an icon name for the command
    pub fn icon(&self) -> &'static str {
        match self {
            SlashCommandType::Explain => "book-open",
            SlashCommandType::Fix => "wrench",
            SlashCommandType::Test => "beaker",
            SlashCommandType::Improve => "sparkles",
            SlashCommandType::Summarize => "document-text",
        }
    }
}

/// A parsed slash command from user input
#[derive(Clone, Debug)]
pub struct SlashCommand {
    /// The type of command
    pub kind: SlashCommandType,
    /// The raw command text (e.g., "/explain")
    pub raw: String,
    /// The argument/content after the command (trimmed)
    pub argument: String,
}

impl SlashCommand {
    /// Create a new slash command
    pub fn new(kind: SlashCommandType, raw: String, argument: String) -> Self {
        Self {
            kind,
            raw,
            argument,
        }
    }

    /// Transform the command into a full prompt with system context
    /// Returns (system_context, user_message)
    pub fn to_prompt(&self) -> (String, String) {
        let system = self.kind.system_context().to_string();
        let user_msg = if self.argument.is_empty() {
            // If no argument, prompt user for input
            format!(
                "Please provide the content you'd like me to {} for.",
                self.kind.name()
            )
        } else {
            self.argument.clone()
        };
        (system, user_msg)
    }
}

/// Available slash command options for autocomplete
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CommandOption {
    /// The command type this option creates
    pub kind: SlashCommandType,
}

impl CommandOption {
    pub fn label(&self) -> &'static str {
        self.kind.name()
    }

    pub fn description(&self) -> &'static str {
        self.kind.description()
    }

    pub fn icon(&self) -> &'static str {
        self.kind.icon()
    }

    /// Get all available command options
    pub fn all() -> Vec<Self> {
        SlashCommandType::all()
            .iter()
            .copied()
            .map(|kind| CommandOption { kind })
            .collect()
    }

    /// Filter options by prefix
    pub fn filter_by_prefix(prefix: &str) -> Vec<Self> {
        let prefix_lower = prefix.to_ascii_lowercase();
        Self::all()
            .into_iter()
            .filter(|opt| opt.kind.matches_keyword_prefix(&prefix_lower))
            .collect()
    }
}

/// Parse a slash command from the beginning of input text
/// Returns None if input doesn't start with a valid slash command
pub fn parse_command(input: &str) -> Option<SlashCommand> {
    let trimmed = input.trim_start();

    // Must start with /
    if !trimmed.starts_with('/') {
        return None;
    }

    // Extract the command name (alphanumeric characters after /)
    let after_slash = &trimmed[1..];
    let command_end = after_slash
        .find(|c: char| !c.is_alphanumeric() && c != '_' && c != '-')
        .unwrap_or(after_slash.len());

    if command_end == 0 {
        return None;
    }

    let command_keyword = after_slash[..command_end].to_ascii_lowercase();
    let argument = after_slash[command_end..].trim().to_string();

    let kind = SlashCommandType::from_keyword(&command_keyword)?;

    // Always store canonical raw command token (e.g. "/test" for "/tests").
    let raw = format!("/{}", kind.name());
    Some(SlashCommand::new(kind, raw, argument))
}

/// Check if input starts with an incomplete slash command (for showing autocomplete)
/// Returns Some((slash_position, prefix_after_slash)) if autocomplete should show
pub fn get_incomplete_command(input: &str, cursor_pos: usize) -> Option<(usize, String)> {
    let before_cursor = &input[..cursor_pos.min(input.len())];
    let trimmed = before_cursor.trim_start();

    // Must start with /
    if !trimmed.starts_with('/') {
        return None;
    }

    // Find the / position in the original string
    let slash_pos = before_cursor.find('/').unwrap_or(0);
    let after_slash = &before_cursor[slash_pos + 1..];

    // Check if it's a valid partial command (alphanumeric only, no spaces)
    if after_slash
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    {
        // Don't show autocomplete for already-complete commands.
        let keyword = after_slash.to_ascii_lowercase();
        if SlashCommandType::from_keyword(&keyword).is_none() {
            return Some((slash_pos, after_slash.to_string()));
        }
    }

    None
}

/// Transform input with slash command into system context + user message
/// Returns (system_context, transformed_user_message) or (None, original) if no command
pub fn transform_with_command(input: &str) -> (Option<String>, String) {
    if let Some(cmd) = parse_command(input) {
        let (system, user_msg) = cmd.to_prompt();
        (Some(system), user_msg)
    } else {
        (None, input.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_explain_command() {
        let cmd = parse_command("/explain How does async work?");
        assert!(cmd.is_some());
        let cmd = cmd.unwrap();
        assert_eq!(cmd.kind, SlashCommandType::Explain);
        assert_eq!(cmd.raw, "/explain");
        assert_eq!(cmd.argument, "How does async work?");
    }

    #[test]
    fn test_parse_fix_command() {
        let cmd = parse_command("/fix @clipboard");
        assert!(cmd.is_some());
        let cmd = cmd.unwrap();
        assert_eq!(cmd.kind, SlashCommandType::Fix);
        assert_eq!(cmd.argument, "@clipboard");
    }

    #[test]
    fn test_parse_command_no_argument() {
        let cmd = parse_command("/summarize");
        assert!(cmd.is_some());
        let cmd = cmd.unwrap();
        assert_eq!(cmd.kind, SlashCommandType::Summarize);
        assert_eq!(cmd.argument, "");
    }

    #[test]
    fn test_parse_command_with_whitespace() {
        let cmd = parse_command("  /improve   some code here  ");
        assert!(cmd.is_some());
        let cmd = cmd.unwrap();
        assert_eq!(cmd.kind, SlashCommandType::Improve);
        assert_eq!(cmd.argument, "some code here");
    }

    #[test]
    fn test_parse_invalid_command() {
        assert!(parse_command("explain something").is_none());
        assert!(parse_command("/ command").is_none());
        assert!(parse_command("/unknown command").is_none());
    }

    #[test]
    fn test_incomplete_command() {
        let result = get_incomplete_command("/exp", 4);
        assert!(result.is_some());
        let (pos, prefix) = result.unwrap();
        assert_eq!(pos, 0);
        assert_eq!(prefix, "exp");
    }

    #[test]
    fn test_incomplete_command_with_leading_space() {
        let result = get_incomplete_command("  /fi", 5);
        assert!(result.is_some());
        let (pos, prefix) = result.unwrap();
        assert_eq!(pos, 2);
        assert_eq!(prefix, "fi");
    }

    #[test]
    fn test_filter_by_prefix() {
        let options = CommandOption::filter_by_prefix("ex");
        assert_eq!(options.len(), 1);
        assert_eq!(options[0].label(), "explain");

        let options = CommandOption::filter_by_prefix("");
        assert_eq!(options.len(), 5); // All commands
    }

    #[test]
    fn test_filter_by_prefix_matches_aliases() {
        let options = CommandOption::filter_by_prefix("tests");
        assert_eq!(options.len(), 1);
        assert_eq!(options[0].kind, SlashCommandType::Test);
    }

    #[test]
    fn test_transform_with_command() {
        let (system, msg) = transform_with_command("/explain What is Rust?");
        assert!(system.is_some());
        assert!(system.unwrap().contains("explain"));
        assert_eq!(msg, "What is Rust?");
    }

    #[test]
    fn test_transform_without_command() {
        let (system, msg) = transform_with_command("Just a regular question");
        assert!(system.is_none());
        assert_eq!(msg, "Just a regular question");
    }

    #[test]
    fn test_command_aliases() {
        // Test "tests" alias for "test"
        let cmd = parse_command("/tests write tests for this");
        assert!(cmd.is_some());
        assert_eq!(cmd.unwrap().kind, SlashCommandType::Test);

        // Test "summary" alias for "summarize"
        let cmd = parse_command("/summary of the article");
        assert!(cmd.is_some());
        assert_eq!(cmd.unwrap().kind, SlashCommandType::Summarize);
    }

    #[test]
    fn test_parse_command_aliases_use_canonical_raw_name() {
        let tests_alias = parse_command("/tests write tests for this").expect("tests alias");
        assert_eq!(tests_alias.raw, "/test");

        let summary_alias = parse_command("/summary of the article").expect("summary alias");
        assert_eq!(summary_alias.raw, "/summarize");
    }
}
