use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ArtifactKind {
    Script,
    Scriptlet,
    Skill,
    Agent,
    Builtin,
    App,
    Window,
    File,
    Note,
    AcpHistory,
    ClipboardHistory,
    DictationHistory,
    BrowserTab,
    BrowserHistory,
    Fallback,
    Issue,
}

impl ArtifactKind {
    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "script" | "scripts" => Some(Self::Script),
            "scriptlet" | "scriptlets" => Some(Self::Scriptlet),
            "skill" | "skills" => Some(Self::Skill),
            "agent" | "agents" => Some(Self::Agent),
            "builtin" | "built-in" | "builtins" => Some(Self::Builtin),
            "app" | "apps" => Some(Self::App),
            "window" | "windows" => Some(Self::Window),
            "file" | "files" => Some(Self::File),
            "note" | "notes" => Some(Self::Note),
            "acphistory" | "acp-history" | "ai-conversation" | "ai-conversations" => {
                Some(Self::AcpHistory)
            }
            "clipboard" | "clipboard-history" | "clipboardhistory" => Some(Self::ClipboardHistory),
            "dictation" | "dictation-history" | "dictationhistory" | "transcript"
            | "transcripts" => Some(Self::DictationHistory),
            "browser-tab" | "browser-tabs" | "browsertab" | "browsertabs" | "tab" | "tabs" => {
                Some(Self::BrowserTab)
            }
            "browser" | "browser-history" | "browserhistory" | "web" => Some(Self::BrowserHistory),
            "fallback" | "fallbacks" => Some(Self::Fallback),
            "issue" | "issues" | "scriptissue" | "script-issue" => Some(Self::Issue),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Predicate {
    Type(ArtifactKind),
    Tag(String),
    HasShortcut(ShortcutPredicate),
    Source(String),
    Plugin(String),
    Name(String),
    Desc(String),
    Alias(String),
    Has(String),
    MetaPath { path: String, value: String },
    Negate(Box<Predicate>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShortcutPredicate {
    Any,
    None,
    Literal(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdvancedQuery {
    pub free_text: String,
    pub predicates: Vec<Predicate>,
    pub source_filters: RootUnifiedSourceFilterSet,
    pub raw: String,
}

impl AdvancedQuery {
    pub fn has_predicates(&self) -> bool {
        !self.predicates.is_empty()
    }

    pub fn has_source_filters(&self) -> bool {
        self.source_filters.active()
    }

    pub fn is_source_filter_only(&self) -> bool {
        self.has_source_filters() && !self.has_predicates()
    }

    pub fn filter_indicators(&self) -> Vec<FilterIndicator> {
        self.source_filters
            .entries()
            .into_iter()
            .map(|entry| FilterIndicator {
                id: entry.id,
                label: entry.label,
                head: entry.head,
                value: None,
                negated: entry.negated,
                tone: if entry.negated {
                    FilterIndicatorTone::Excluded
                } else {
                    FilterIndicatorTone::Normal
                },
            })
            .collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RootUnifiedSourceFilter {
    Files,
    Notes,
    ClipboardHistory,
    BrowserTabs,
    BrowserHistory,
    Apps,
    Scripts,
    Commands,
    Conversations,
    AiVault,
    Dictation,
    Windows,
    Processes,
}

impl RootUnifiedSourceFilter {
    pub fn receipt_label(self) -> &'static str {
        match self {
            Self::Files => "files",
            Self::Notes => "notes",
            Self::ClipboardHistory => "clipboard",
            Self::BrowserTabs => "tabs",
            Self::BrowserHistory => "history",
            Self::Apps => "apps",
            Self::Scripts => "scripts",
            Self::Commands => "commands",
            Self::Conversations => "conversations",
            Self::AiVault => "vault",
            Self::Dictation => "dictation",
            Self::Windows => "windows",
            Self::Processes => "processes",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Files => "Files",
            Self::Notes => "Notes",
            Self::ClipboardHistory => "Clipboard",
            Self::BrowserTabs => "Browser Tabs",
            Self::BrowserHistory => "Browser History",
            Self::Apps => "Apps",
            Self::Scripts => "Scripts",
            Self::Commands => "Commands",
            Self::Conversations => "AI Conversations",
            Self::AiVault => "AI Vault",
            Self::Dictation => "Dictation",
            Self::Windows => "Windows",
            Self::Processes => "Processes",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceHeadSpec {
    pub source: RootUnifiedSourceFilter,
    pub canonical: &'static str,
    pub short: Option<&'static str>,
    pub label: &'static str,
    pub description: &'static str,
    pub planned: bool,
}

pub const SOURCE_HEAD_SPECS: &[SourceHeadSpec] = &[
    SourceHeadSpec {
        source: RootUnifiedSourceFilter::Files,
        canonical: "files:",
        short: Some("f:"),
        label: "Files",
        description: "Search local file results",
        planned: true,
    },
    SourceHeadSpec {
        source: RootUnifiedSourceFilter::Notes,
        canonical: "notes:",
        short: Some("n:"),
        label: "Notes",
        description: "Search note records",
        planned: true,
    },
    SourceHeadSpec {
        source: RootUnifiedSourceFilter::ClipboardHistory,
        canonical: "clipboard:",
        short: Some("c:"),
        label: "Clipboard",
        description: "Search clipboard history",
        planned: true,
    },
    SourceHeadSpec {
        source: RootUnifiedSourceFilter::BrowserTabs,
        canonical: "tabs:",
        short: Some("t:"),
        label: "Browser Tabs",
        description: "Search current browser tab metadata",
        planned: true,
    },
    SourceHeadSpec {
        source: RootUnifiedSourceFilter::BrowserHistory,
        canonical: "history:",
        short: Some("h:"),
        label: "Browser History",
        description: "Search browser history metadata",
        planned: true,
    },
    SourceHeadSpec {
        source: RootUnifiedSourceFilter::Apps,
        canonical: "apps:",
        short: Some("a:"),
        label: "Apps",
        description: "Search installed apps",
        planned: true,
    },
    SourceHeadSpec {
        source: RootUnifiedSourceFilter::Scripts,
        canonical: "scripts:",
        short: Some("s:"),
        label: "Scripts",
        description: "Search user-authored Kit scripts and scriptlets",
        planned: true,
    },
    SourceHeadSpec {
        source: RootUnifiedSourceFilter::Commands,
        canonical: "commands:",
        short: Some("cmd:"),
        label: "Commands",
        description: "Search executable launcher commands",
        planned: true,
    },
    SourceHeadSpec {
        source: RootUnifiedSourceFilter::Conversations,
        canonical: "conversations:",
        short: Some("ai:"),
        label: "AI Conversations",
        description: "Search saved AI conversation records",
        planned: true,
    },
    SourceHeadSpec {
        source: RootUnifiedSourceFilter::AiVault,
        canonical: "vault:",
        short: Some("v:"),
        label: "AI Vault",
        description: "Search cmux AI conversation vault sessions",
        planned: true,
    },
    SourceHeadSpec {
        source: RootUnifiedSourceFilter::Dictation,
        canonical: "dictation:",
        short: Some("d:"),
        label: "Dictation",
        description: "Search saved dictation records",
        planned: true,
    },
    SourceHeadSpec {
        source: RootUnifiedSourceFilter::Windows,
        canonical: "windows:",
        short: Some("w:"),
        label: "Windows",
        description: "Search window records",
        planned: true,
    },
];

pub fn source_for_head(head_with_colon: &str) -> Option<RootUnifiedSourceFilter> {
    let normalized = head_with_colon.trim().to_ascii_lowercase();
    SOURCE_HEAD_SPECS.iter().find_map(|spec| {
        (spec.canonical == normalized || spec.short == Some(normalized.as_str()))
            .then_some(spec.source)
    })
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RootUnifiedSourceFilterSet {
    include: BTreeSet<RootUnifiedSourceFilter>,
    exclude: BTreeSet<RootUnifiedSourceFilter>,
}

impl RootUnifiedSourceFilterSet {
    pub fn is_empty(&self) -> bool {
        self.include.is_empty() && self.exclude.is_empty()
    }

    pub fn active(&self) -> bool {
        !self.is_empty()
    }

    pub fn insert(&mut self, source: RootUnifiedSourceFilter) {
        self.include.insert(source);
    }

    pub fn exclude(&mut self, source: RootUnifiedSourceFilter) {
        self.exclude.insert(source);
    }

    pub fn allows(&self, source: RootUnifiedSourceFilter) -> bool {
        if self.exclude.contains(&source) {
            return false;
        }
        self.include.is_empty() || self.include.contains(&source)
    }

    pub fn excludes(&self, source: RootUnifiedSourceFilter) -> bool {
        self.exclude.contains(&source)
    }

    pub fn includes(&self, source: RootUnifiedSourceFilter) -> bool {
        self.include.contains(&source)
    }

    pub fn effective_is_empty(&self) -> bool {
        self.active()
            && self
                .include
                .iter()
                .all(|source| self.exclude.contains(source))
    }

    pub fn entries(&self) -> Vec<SourceFilterEntry> {
        let mut entries = Vec::new();
        entries.extend(self.include.iter().copied().map(|source| {
            SourceFilterEntry {
                id: source.receipt_label().to_string(),
                label: source.label().to_string(),
                head: SOURCE_HEAD_SPECS
                    .iter()
                    .find(|spec| spec.source == source)
                    .map(|spec| spec.canonical.trim_end_matches(':').to_string())
                    .unwrap_or_else(|| source.receipt_label().to_string()),
                negated: false,
            }
        }));
        entries.extend(self.exclude.iter().copied().map(|source| {
            SourceFilterEntry {
                id: format!("-{}", source.receipt_label()),
                label: format!("Not {}", source.label()),
                head: SOURCE_HEAD_SPECS
                    .iter()
                    .find(|spec| spec.source == source)
                    .map(|spec| format!("-{}", spec.canonical.trim_end_matches(':')))
                    .unwrap_or_else(|| format!("-{}", source.receipt_label())),
                negated: true,
            }
        }));
        entries
    }

    pub fn labels(&self) -> Vec<String> {
        self.entries().into_iter().map(|entry| entry.id).collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceFilterEntry {
    pub id: String,
    pub label: String,
    pub head: String,
    pub negated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FilterIndicatorTone {
    Normal,
    Excluded,
    Incomplete,
    Invalid,
    Unavailable,
    Contradiction,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FilterIndicator {
    pub id: String,
    pub label: String,
    pub head: String,
    pub value: Option<String>,
    pub negated: bool,
    pub tone: FilterIndicatorTone,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CaptureAlias {
    /// A sigil-prefixed capture such as `;todo body` or legacy `+todo body`.
    CapturePrefix,
    Keyword,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptureInvocation {
    pub target: String,
    pub alias_form: CaptureAlias,
    pub body: String,
    pub tags: Vec<String>,
    pub priority: Option<u8>,
    pub url: Option<String>,
    pub duration: Option<String>,
    pub kv: Vec<(String, String)>,
    pub date_phrases: Vec<DatePhrase>,
    pub raw: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DateRole {
    Due,
    At,
    Start,
    End,
    Inferred,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatePhrase {
    pub role: DateRole,
    pub source: String,
    pub source_span: (usize, usize),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArgvInvocation {
    pub head: String,
    pub fields: Vec<(String, String)>,
    pub tags: Vec<String>,
    pub argv: Vec<String>,
    pub raw: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IncompleteKind {
    BareQueryPrefix,
    BareCapturePrefix,
    UnknownCaptureTarget(String),
    MissingCaptureBody(String),
    BareArgvPrefix,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IncompleteSyntax {
    pub kind: IncompleteKind,
    pub hint: String,
}

pub const KNOWN_CAPTURE_TARGETS: &[&str] = &["todo", "cal", "note", "social", "link"];

pub fn is_known_capture_target(target: &str) -> bool {
    if target.eq_ignore_ascii_case("mcal") {
        return true;
    }
    KNOWN_CAPTURE_TARGETS
        .iter()
        .any(|k| k.eq_ignore_ascii_case(target))
}

#[cfg(test)]
mod capture_target_taxonomy_tests {
    use super::*;

    #[test]
    fn core_capture_targets_are_stable_taxonomy() {
        assert_eq!(
            KNOWN_CAPTURE_TARGETS,
            &["todo", "cal", "note", "social", "link"]
        );
    }

    #[test]
    fn known_capture_targets_are_case_insensitive_for_core_and_mcal() {
        for target in [
            "todo", "TODO", "cal", "CAL", "note", "NOTE", "social", "SOCIAL", "link", "LINK",
            "mcal", "MCAL",
        ] {
            assert!(
                is_known_capture_target(target),
                "`{target}` should be parser-known"
            );
        }
    }

    #[test]
    fn shipped_dynamic_targets_are_not_parser_known_without_metadata() {
        for target in [
            "gcal", "github", "expense", "snippet", "fixture", "reminder", "snooze", "defer",
        ] {
            assert!(
                !is_known_capture_target(target),
                "`{target}` should stay metadata-driven until registered"
            );
        }
    }
}

/// Positional argument descriptor for a `command.v1` handler. Mirrors
/// `CommandArgSpec` in `kit-init/types/menu-syntax.d.ts` so authors get the
/// same shape on both sides. Empty `values` means any string is accepted.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CommandArgSpec {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub required: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub example: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub values: Vec<String>,
}

/// Flag descriptor for a `command.v1` handler. Mirrors `CommandFlagSpec` in
/// `kit-init/types/menu-syntax.d.ts`. `name` is the long form (`--dry-run`);
/// `alias` is the short form (`-n`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CommandFlagSpec {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub alias: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub required: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub example: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub values: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MenuSyntaxHandlerSpec {
    pub family: String,
    #[serde(default)]
    pub targets: Vec<String>,
    #[serde(default)]
    pub accepts: Vec<String>,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub payload_schema: Option<String>,
    #[serde(default)]
    pub default_handler: bool,
    /// `command.v1` only: the bare slug after `!`. Optional so capture/skill
    /// handlers don't need to set it; ignored unless `family == "command.v1"`.
    #[serde(default)]
    pub head: Option<String>,
    /// Optional human description of the command — surfaced as the hint card
    /// subtitle when the user types `!head`.
    #[serde(default)]
    pub description: Option<String>,
    /// Positional args expected after `--`. Surfaced as hint rows in the
    /// command composer.
    #[serde(default)]
    pub args: Vec<CommandArgSpec>,
    /// Long/short flags accepted after `--`. Surfaced as hint rows in the
    /// command composer.
    #[serde(default)]
    pub flags: Vec<CommandFlagSpec>,
    /// Free-form usage string shown verbatim in the hint card when present.
    #[serde(default)]
    pub usage: Option<String>,
    /// `capture.v1` only: per-key enum overrides for the future autocomplete
    /// popup. When the user parks the cursor on `key:` inside an active
    /// capture body for one of `targets`, the popup ranks
    /// `kv_enums[key]` first (in declared order, dimming any non-matching
    /// history values). Empty map → fall through to pure-history ranking.
    /// See [[lat.md/menu-syntax#Menu Syntax#Schema Overrides History]].
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub kv_enums: std::collections::BTreeMap<String, Vec<String>>,
    /// `capture.v1` only: required field tokens (e.g. `"body"`, `"url"`,
    /// `"kv:amount"`, `"date:start"`). Parsed into [[crate::menu_syntax::capture_schema::FieldRequirement]]
    /// by `dynamic_capture_schema_from_spec` to build a per-target schema.
    #[serde(default)]
    pub required: Vec<String>,
    /// `capture.v1` only: optional field tokens. Same vocabulary as `required`.
    #[serde(default)]
    pub optional: Vec<String>,
    /// `capture.v1` only: forbidden field tokens. Same vocabulary as `required`.
    #[serde(default)]
    pub forbidden: Vec<String>,
}

impl MenuSyntaxHandlerSpec {
    pub fn handles_capture_target(&self, target: &str) -> bool {
        if self.family != "capture.v1" {
            return false;
        }
        self.targets
            .iter()
            .any(|t| t == "*" || t.eq_ignore_ascii_case(target))
    }

    /// True iff this is a `command.v1` handler whose `head` matches the
    /// invocation slug case-insensitively.
    pub fn handles_command_head(&self, head: &str) -> bool {
        if self.family != "command.v1" {
            return false;
        }
        self.head
            .as_deref()
            .map(|h| h.eq_ignore_ascii_case(head))
            .unwrap_or(false)
    }
}
