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
    Todo,
    AgentChatHistory,
    AiVault,
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
            "todo" | "todos" => Some(Self::Todo),
            "agent_chathistory" | "agent_chat-history" | "ai-conversation" | "ai-conversations" => {
                Some(Self::AgentChatHistory)
            }
            "ai-vault" | "aivault" | "vault" | "vault-session" | "vault-sessions" => {
                Some(Self::AiVault)
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
    Brain,
    Notes,
    Todo,
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
            Self::Brain => "brain",
            Self::Notes => "notes",
            Self::Todo => "todo",
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
            Self::Brain => "From Your Brain",
            Self::Notes => "Notes",
            Self::Todo => "Todos",
            Self::ClipboardHistory => "Clipboard History",
            Self::BrowserTabs => "Browser Tabs",
            Self::BrowserHistory => "Browser History",
            Self::Apps => "Apps",
            Self::Scripts => "Scripts",
            Self::Commands => "Commands",
            Self::Conversations => "Agent Chat Conversations",
            Self::AiVault => "AI Vault",
            Self::Dictation => "Dictation History",
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
        source: RootUnifiedSourceFilter::Brain,
        canonical: "brain:",
        short: None,
        label: "From Your Brain",
        description: "Search your local brain memory",
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
        source: RootUnifiedSourceFilter::Todo,
        canonical: "todo:",
        short: None,
        label: "Todos",
        description: "Search unchecked tasks on recent day pages",
        planned: true,
    },
    SourceHeadSpec {
        source: RootUnifiedSourceFilter::ClipboardHistory,
        canonical: "clipboard:",
        short: Some("c:"),
        label: "Clipboard History",
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
        label: "Agent Chat Conversations",
        description: "Search saved Agent Chat conversation records",
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
        label: "Dictation History",
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
    SourceHeadSpec {
        source: RootUnifiedSourceFilter::Processes,
        canonical: "processes:",
        short: Some("p:"),
        label: "Processes",
        description: "Search running process records",
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

impl FromIterator<RootUnifiedSourceFilter> for RootUnifiedSourceFilterSet {
    fn from_iter<T: IntoIterator<Item = RootUnifiedSourceFilter>>(iter: T) -> Self {
        let mut set = Self::default();
        for source in iter {
            set.insert(source);
        }
        set
    }
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

    pub fn has_positive_includes(&self) -> bool {
        self.include
            .iter()
            .any(|source| !self.exclude.contains(source))
    }

    pub fn positive_includes(&self) -> impl Iterator<Item = RootUnifiedSourceFilter> + '_ {
        self.include
            .iter()
            .copied()
            .filter(|source| !self.exclude.contains(source))
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CaptureOperation {
    Create,
    Update,
    Delete,
    Open,
    Remind,
    Snooze,
    Defer,
    Append,
    Save,
}

impl CaptureOperation {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Create => "create",
            Self::Update => "update",
            Self::Delete => "delete",
            Self::Open => "open",
            Self::Remind => "remind",
            Self::Snooze => "snooze",
            Self::Defer => "defer",
            Self::Append => "append",
            Self::Save => "save",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CanonicalCaptureTarget {
    Todo,
    Note,
    Link,
    Snippet,
    Cal,
    Social,
    Mcal,
}

impl CanonicalCaptureTarget {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Todo => "todo",
            Self::Note => "note",
            Self::Link => "link",
            Self::Snippet => "snippet",
            Self::Cal => "cal",
            Self::Social => "social",
            Self::Mcal => "mcal",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptureTargetResolution {
    pub raw_target: String,
    pub canonical_target: CanonicalCaptureTarget,
    pub target_alias_of: Option<CanonicalCaptureTarget>,
    pub operation: CaptureOperation,
    pub product_owned: bool,
    pub picker_visible: bool,
    pub title: &'static str,
    pub detail: &'static str,
}

impl CaptureTargetResolution {
    pub fn canonical_target_str(&self) -> &'static str {
        self.canonical_target.as_str()
    }

    pub fn target_alias_of_str(&self) -> Option<&'static str> {
        self.target_alias_of.map(CanonicalCaptureTarget::as_str)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CaptureObjectKind {
    Todo,
    Note,
    Link,
    Snippet,
}

impl CaptureObjectKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Todo => "todo",
            Self::Note => "note",
            Self::Link => "link",
            Self::Snippet => "snippet",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActiveObjectSelector {
    pub kind: CaptureObjectKind,
    pub query: String,
    pub range: (usize, usize),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureObjectRef {
    pub role: String,
    pub kind: CaptureObjectKind,
    pub id: String,
    pub label: String,
    pub source: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deeplink: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub range: Option<(usize, usize)>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    pub resolved: bool,
}

pub fn object_refs_for_raw_capture(target: &str, raw: &str) -> Vec<CaptureObjectRef> {
    let Some(default_kind) = object_kind_for_capture_target(target) else {
        return Vec::new();
    };
    let Some(body_start) = raw_capture_body_start(raw) else {
        return Vec::new();
    };
    let body = &raw[body_start..];
    let mut out = Vec::new();
    let mut offset = 0;
    for segment in body.split_whitespace() {
        let Some(rel_start) = body[offset..].find(segment).map(|idx| idx + offset) else {
            continue;
        };
        let rel_end = rel_start + segment.len();
        offset = rel_end;
        if segment == "--" {
            break;
        }
        let Some(query) = segment.strip_prefix('@') else {
            continue;
        };
        if query.is_empty() || segment.starts_with("@@") || segment.starts_with("\\@") {
            continue;
        }
        let (kind, id, resolved) = object_ref_parts(default_kind, query);
        out.push(CaptureObjectRef {
            role: if out.is_empty() {
                "primary".to_string()
            } else {
                "related".to_string()
            },
            kind,
            id: id.to_string(),
            label: id.to_string(),
            source: "inline-token".to_string(),
            deeplink: resolved.then(|| object_ref_deeplink(kind, id)),
            query: Some(query.to_string()),
            range: Some((body_start + rel_start, body_start + rel_end)),
            token: Some(segment.to_string()),
            resolved,
        });
    }
    out
}

pub fn object_ref_deeplink(kind: CaptureObjectKind, id: &str) -> String {
    format!("@{}:{}", kind.as_str(), id)
}

fn object_ref_parts(
    default_kind: CaptureObjectKind,
    query: &str,
) -> (CaptureObjectKind, &str, bool) {
    let Some((prefix, id)) = query.split_once(':') else {
        return (default_kind, query, false);
    };
    let Some(kind) = object_kind_for_ref_prefix(prefix) else {
        return (default_kind, query, false);
    };
    if id.trim().is_empty() {
        (kind, id, false)
    } else {
        (kind, id, true)
    }
}

fn object_kind_for_ref_prefix(prefix: &str) -> Option<CaptureObjectKind> {
    match prefix.trim().to_ascii_lowercase().as_str() {
        "todo" | "todos" => Some(CaptureObjectKind::Todo),
        "note" | "notes" => Some(CaptureObjectKind::Note),
        "link" | "links" => Some(CaptureObjectKind::Link),
        "snippet" | "snippets" => Some(CaptureObjectKind::Snippet),
        _ => None,
    }
}

fn raw_capture_body_start(raw: &str) -> Option<usize> {
    if let Some(rest) = raw.strip_prefix(';').or_else(|| raw.strip_prefix('+')) {
        let end = rest.find(char::is_whitespace).unwrap_or(rest.len());
        return Some(1 + end);
    }
    let colon_idx = raw.find(':')?;
    let head = &raw[..colon_idx];
    if head.is_empty() || head.contains(char::is_whitespace) {
        return None;
    }
    Some(colon_idx + 1)
}

pub fn object_kind_for_capture_target(target: &str) -> Option<CaptureObjectKind> {
    match resolve_capture_target(target)?.canonical_target {
        CanonicalCaptureTarget::Todo => Some(CaptureObjectKind::Todo),
        CanonicalCaptureTarget::Note => Some(CaptureObjectKind::Note),
        CanonicalCaptureTarget::Link => Some(CaptureObjectKind::Link),
        CanonicalCaptureTarget::Snippet => Some(CaptureObjectKind::Snippet),
        CanonicalCaptureTarget::Cal
        | CanonicalCaptureTarget::Mcal
        | CanonicalCaptureTarget::Social => None,
    }
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

pub const PICKER_VISIBLE_CAPTURE_TARGETS: &[&str] =
    &["todo", "note", "link", "snippet", "cal", "social"];

pub const KNOWN_CAPTURE_TARGETS: &[&str] = &[
    "todo", "note", "link", "snippet", "cal", "social", "reminder", "snooze", "defer", "notes",
];

pub fn is_known_capture_target(target: &str) -> bool {
    resolve_capture_target(target).is_some()
}

pub fn picker_visible_capture_targets() -> &'static [&'static str] {
    PICKER_VISIBLE_CAPTURE_TARGETS
}

pub fn resolve_capture_target(target: &str) -> Option<CaptureTargetResolution> {
    let raw_target = target.trim().to_ascii_lowercase();
    let (canonical_target, target_alias_of, operation, picker_visible, title, detail) =
        match raw_target.as_str() {
            "todo" => (
                CanonicalCaptureTarget::Todo,
                None,
                CaptureOperation::Create,
                true,
                "Todo inbox",
                "Create or update a Todo task",
            ),
            "reminder" => (
                CanonicalCaptureTarget::Todo,
                Some(CanonicalCaptureTarget::Todo),
                CaptureOperation::Remind,
                false,
                "Todo reminder",
                "Todo alias: set a reminder time",
            ),
            "snooze" => (
                CanonicalCaptureTarget::Todo,
                Some(CanonicalCaptureTarget::Todo),
                CaptureOperation::Snooze,
                false,
                "Todo snooze",
                "Todo alias: hide until a wake time",
            ),
            "defer" => (
                CanonicalCaptureTarget::Todo,
                Some(CanonicalCaptureTarget::Todo),
                CaptureOperation::Defer,
                false,
                "Todo defer",
                "Todo alias: defer until a start date",
            ),
            "note" => (
                CanonicalCaptureTarget::Note,
                None,
                CaptureOperation::Create,
                true,
                "Note",
                "Create or update a Note",
            ),
            "notes" => (
                CanonicalCaptureTarget::Note,
                Some(CanonicalCaptureTarget::Note),
                CaptureOperation::Create,
                false,
                "Note compatibility alias",
                "Compatibility alias of ;note",
            ),
            "link" => (
                CanonicalCaptureTarget::Link,
                None,
                CaptureOperation::Save,
                true,
                "Saved link",
                "Save or update a tagged link",
            ),
            "snippet" => (
                CanonicalCaptureTarget::Snippet,
                None,
                CaptureOperation::Create,
                true,
                "Snippet",
                "Create, update, or remove a snippet",
            ),
            "cal" => (
                CanonicalCaptureTarget::Cal,
                None,
                CaptureOperation::Create,
                true,
                "Calendar event",
                "Create a calendar event",
            ),
            "mcal" => (
                CanonicalCaptureTarget::Mcal,
                None,
                CaptureOperation::Create,
                false,
                "macOS Calendar event",
                "Schema-known Calendar target",
            ),
            "social" => (
                CanonicalCaptureTarget::Social,
                None,
                CaptureOperation::Create,
                true,
                "Social draft",
                "Create a social draft",
            ),
            _ => return None,
        };

    Some(CaptureTargetResolution {
        raw_target,
        canonical_target,
        target_alias_of,
        operation,
        product_owned: true,
        picker_visible,
        title,
        detail,
    })
}

#[cfg(test)]
mod capture_target_taxonomy_tests {
    use super::*;

    #[test]
    fn core_capture_targets_are_stable_taxonomy() {
        assert_eq!(
            PICKER_VISIBLE_CAPTURE_TARGETS,
            &["todo", "note", "link", "snippet", "cal", "social"]
        );
    }

    #[test]
    fn known_capture_targets_are_case_insensitive_for_product_targets_aliases_and_mcal() {
        for target in [
            "todo", "TODO", "cal", "CAL", "note", "NOTE", "social", "SOCIAL", "link", "LINK",
            "snippet", "SNIPPET", "mcal", "MCAL", "reminder", "REMINDER", "snooze", "SNOOZE",
            "defer", "DEFER", "notes", "NOTES",
        ] {
            assert!(
                is_known_capture_target(target),
                "`{target}` should be parser-known"
            );
        }
    }

    #[test]
    fn shipped_dynamic_targets_are_not_parser_known_without_metadata() {
        for target in ["gcal", "github", "expense", "fixture"] {
            assert!(
                !is_known_capture_target(target),
                "`{target}` should stay metadata-driven until registered"
            );
        }
    }

    #[test]
    fn resolves_todo_alias_targets_to_canonical_todo_operations() {
        for (target, operation) in [
            ("reminder", CaptureOperation::Remind),
            ("snooze", CaptureOperation::Snooze),
            ("defer", CaptureOperation::Defer),
        ] {
            let resolved = resolve_capture_target(target).expect("target should resolve");
            assert_eq!(resolved.raw_target, target);
            assert_eq!(resolved.canonical_target, CanonicalCaptureTarget::Todo);
            assert_eq!(resolved.target_alias_of, Some(CanonicalCaptureTarget::Todo));
            assert_eq!(resolved.operation, operation);
            assert!(!resolved.picker_visible);
        }
    }

    #[test]
    fn resolves_notes_alias_to_note_but_preserves_raw_target() {
        let resolved = resolve_capture_target("notes").expect("notes alias should resolve");
        assert_eq!(resolved.raw_target, "notes");
        assert_eq!(resolved.canonical_target, CanonicalCaptureTarget::Note);
        assert_eq!(resolved.target_alias_of, Some(CanonicalCaptureTarget::Note));
        assert_eq!(resolved.operation, CaptureOperation::Create);
        assert!(!resolved.picker_visible);
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
    /// See [[removed-docs Syntax#Schema Overrides History]].
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inline_object_refs_resolve_typed_kind_prefixes() {
        let refs = object_refs_for_raw_capture(
            "snippet",
            ";snippet update @snippet:fetch-json -- const value = 1",
        );
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].kind, CaptureObjectKind::Snippet);
        assert_eq!(refs[0].id, "fetch-json");
        assert_eq!(refs[0].role, "primary");
        assert_eq!(refs[0].token.as_deref(), Some("@snippet:fetch-json"));
        assert_eq!(refs[0].source, "inline-token");
        assert_eq!(refs[0].deeplink.as_deref(), Some("@snippet:fetch-json"));
        assert!(refs[0].resolved);
    }

    #[test]
    fn inline_object_refs_resolve_link_url_prefix() {
        let refs =
            object_refs_for_raw_capture("link", ";link delete @link:https://example.test/docs");
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].kind, CaptureObjectKind::Link);
        assert_eq!(refs[0].id, "https://example.test/docs");
        assert_eq!(refs[0].role, "primary");
        assert_eq!(
            refs[0].token.as_deref(),
            Some("@link:https://example.test/docs")
        );
        assert_eq!(
            refs[0].deeplink.as_deref(),
            Some("@link:https://example.test/docs")
        );
        assert!(refs[0].resolved);
    }

    #[test]
    fn inline_object_refs_keep_bare_query_unresolved_for_picker_handoff() {
        let refs = object_refs_for_raw_capture("note", ";note @Project due:tomorrow");
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].kind, CaptureObjectKind::Note);
        assert_eq!(refs[0].id, "Project");
        assert_eq!(refs[0].query.as_deref(), Some("Project"));
        assert!(!refs[0].resolved);
    }

    #[test]
    fn inline_object_refs_ignore_snippet_body_after_delimiter() {
        let refs = object_refs_for_raw_capture(
            "snippet",
            ";snippet update @snippet:fetch-json -- @decorator class Example {}",
        );
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].id, "fetch-json");
    }
}
