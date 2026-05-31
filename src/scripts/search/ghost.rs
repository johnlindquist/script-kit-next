use crate::scripts::types::SearchResult;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Confidence carried by an accepted, sanitized LLM completion. The LLM is the
/// real predictor now; deterministic logic only seeds the request and provides
/// a low-confidence starter when the model has nothing yet.
const AGENT_LLM_PROMPT_CONFIDENCE: f32 = 0.82;
/// Deterministic fallback "starter" confidence. Intentionally low and NOT
/// Tab-acceptable: it is a faint, action-oriented hint shown while the LLM is
/// pending or unavailable, never a claim about what the user wants.
const AGENT_STARTER_CONFIDENCE: f32 = 0.35;
const MAX_CONTEXT_DOC_CHARS: usize = 24_000;
const MAX_CONTEXT_TASK_PHRASES: usize = 12;
const MAX_CONTEXT_TOPIC_KEYWORDS: usize = 12;
const MAX_COMPLETION_CHARS: usize = 96;
const MAX_CONTEXT_CACHE_ENTRIES: usize = 16;

/// Max chars of sanitized README/AGENTS excerpt sent to the LLM as a bias.
/// Kept small: this is only a steering bias, and on the on-device (CPU until
/// Metal lands) ghost engine every excerpt char is prompt-eval latency. A 4k
/// excerpt added ~1k prompt tokens and pushed warm completions to multiple
/// seconds; ~700 chars keeps the prompt short while still biasing suggestions.
const MAX_LLM_CONTEXT_EXCERPT_CHARS: usize = 700;
/// Hard client-side cap on the LLM completion suffix length.
const MAX_LLM_COMPLETION_CHARS: usize = 120;
/// LRU bound on the per-app LLM ghost cache.
pub const GHOST_LLM_CACHE_LIMIT: usize = 64;
/// How long a cached LLM ghost prediction stays usable.
pub const GHOST_LLM_CACHE_TTL: std::time::Duration = std::time::Duration::from_secs(10 * 60);

/// Low-risk, high-signal facts extracted from the cwd's README.md / AGENTS.md
/// used to drive context-aware ghost-text completions. Source filenames and
/// raw paths/secrets are deliberately never stored here.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ContextDigest {
    pub project_name: Option<String>,
    pub task_phrases: Vec<String>,
    pub topic_keywords: Vec<String>,
}

impl ContextDigest {
    pub fn is_empty(&self) -> bool {
        self.project_name.is_none()
            && self.task_phrases.is_empty()
            && self.topic_keywords.is_empty()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PredictionRevision {
    pub query_rev: u64,
    pub catalog_rev: u64,
    pub context_rev: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GhostPredictionKind {
    CommandCompletion,
    /// An accepted, sanitized LLM continuation of the user's prompt. Tab-acceptable.
    AgentPromptCompletion,
    /// Deterministic, action-oriented fallback shown while the LLM is pending or
    /// unavailable. Faint and NOT Tab-acceptable so weak text never gets committed.
    AgentPromptStarter,
}

#[derive(Clone, Debug)]
pub struct GhostPrediction {
    pub query: String,
    pub full_label: String,
    pub ghost_suffix: String,
    pub confidence: f32,
    pub revision: PredictionRevision,
    pub ghost_id: u64,
    pub kind: GhostPredictionKind,
}

impl GhostPrediction {
    pub fn accepts_tab(&self) -> bool {
        matches!(
            self.kind,
            GhostPredictionKind::CommandCompletion | GhostPredictionKind::AgentPromptCompletion
        )
    }

    pub fn kind_label(&self) -> &'static str {
        match self.kind {
            GhostPredictionKind::CommandCompletion => "command_completion",
            GhostPredictionKind::AgentPromptCompletion => "agent_prompt_completion",
            GhostPredictionKind::AgentPromptStarter => "agent_prompt_starter",
        }
    }
}

pub fn first_word_acceptance_suffix(suffix: &str) -> &str {
    let mut saw_non_whitespace = false;
    for (idx, ch) in suffix.char_indices() {
        if ch.is_whitespace() {
            if saw_non_whitespace {
                return &suffix[..idx];
            }
        } else {
            saw_non_whitespace = true;
        }
    }
    suffix
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GhostContext {
    pub cwd_label: Option<String>,
    pub has_agents_md: bool,
    pub has_readme_md: bool,
    pub project_hint: GhostProjectHint,
    pub digest: ContextDigest,
    /// Sanitized, multi-line excerpt of the cwd README/AGENTS used to BIAS the
    /// LLM ghost prediction. Never raw docs: code fences, URLs, secrets, and
    /// source filenames are stripped. `None` when there is nothing safe to send.
    pub llm_excerpt: Option<String>,
}

/// Cache key for an LLM ghost prediction. Distinct query / cwd / context
/// revision / model id all produce distinct predictions.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GhostLlmCacheKey {
    pub query: String,
    pub cwd: Option<PathBuf>,
    pub context_rev: u64,
    pub model_id: String,
}

#[derive(Clone, Debug)]
pub struct GhostLlmCacheEntry {
    pub prediction: GhostPrediction,
    pub inserted_at: std::time::Instant,
}

impl GhostLlmCacheEntry {
    pub fn is_fresh(&self) -> bool {
        self.inserted_at.elapsed() <= GHOST_LLM_CACHE_TTL
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GhostProjectHint {
    #[default]
    General,
    Rust,
    TypeScript,
    JavaScript,
}

impl GhostContext {
    /// Reads the cwd docs and builds the full context (including the parsed
    /// digest) every time. Kept public for tests/back-compat; the hot keystroke
    /// path goes through [`GhostContextCache::context_for_cwd`] instead so it
    /// only stats files when their metadata is unchanged.
    pub fn from_cwd(cwd: &Path) -> Self {
        Self::from_cwd_uncached(cwd)
    }

    fn from_cwd_uncached(cwd: &Path) -> Self {
        let cwd_label = cwd
            .file_name()
            .and_then(|name| name.to_str())
            .map(|name| name.to_string())
            .or_else(|| Some(cwd.display().to_string()));
        let agents_content = read_context_doc(&cwd.join("AGENTS.md"));
        let readme_content = read_context_doc(&cwd.join("README.md"));
        let agents = agents_content.as_deref().unwrap_or_default();
        let readme = readme_content.as_deref().unwrap_or_default();
        let project_hint = infer_project_hint(agents, readme);
        let digest = parse_context_digest(cwd_label.as_deref(), agents, readme);
        let llm_excerpt = build_llm_context_excerpt(agents, readme);
        Self {
            cwd_label,
            has_agents_md: agents_content.is_some(),
            has_readme_md: readme_content.is_some(),
            project_hint,
            digest,
            llm_excerpt,
        }
    }
}

/// Metadata fingerprint of a context doc; lets the cache decide whether to
/// re-read + re-parse without touching the file contents on each keystroke.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ContextDocMeta {
    len: u64,
    modified: Option<SystemTime>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct GhostContextCacheKey {
    cwd: PathBuf,
    agents_md: Option<ContextDocMeta>,
    readme_md: Option<ContextDocMeta>,
}

#[derive(Clone, Debug)]
struct GhostContextCacheEntry {
    key: GhostContextCacheKey,
    context: GhostContext,
    revision: u64,
}

/// Per-cwd cache of parsed [`GhostContext`] keyed by AGENTS.md/README.md
/// existence + length + mtime. Lives on the app struct (cwd is session state),
/// not as a global, to avoid cross-window/cross-project leakage.
#[derive(Debug, Default)]
pub struct GhostContextCache {
    entries: HashMap<PathBuf, GhostContextCacheEntry>,
    next_revision: u64,
}

impl GhostContextCache {
    /// Returns the cached context for `cwd`, re-reading + re-parsing only when
    /// the context docs' metadata changed. The returned revision changes
    /// whenever the parsed context is rebuilt, so callers can invalidate
    /// downstream predictions.
    pub fn context_for_cwd(&mut self, cwd: &Path) -> (GhostContext, u64) {
        let key = GhostContextCacheKey::from_cwd(cwd);
        let cwd_key = key.cwd.clone();
        if let Some(entry) = self.entries.get(&cwd_key) {
            if entry.key == key {
                return (entry.context.clone(), entry.revision);
            }
        }
        let context = GhostContext::from_cwd_uncached(cwd);
        self.next_revision = self.next_revision.wrapping_add(1).max(1);
        let revision = self.next_revision;
        self.entries.insert(
            cwd_key.clone(),
            GhostContextCacheEntry {
                key,
                context: context.clone(),
                revision,
            },
        );
        if self.entries.len() > MAX_CONTEXT_CACHE_ENTRIES {
            if let Some(remove_key) = self
                .entries
                .keys()
                .find(|candidate| *candidate != &cwd_key)
                .cloned()
            {
                self.entries.remove(&remove_key);
            }
        }
        (context, revision)
    }

    #[cfg(test)]
    fn len(&self) -> usize {
        self.entries.len()
    }
}

impl GhostContextCacheKey {
    fn from_cwd(cwd: &Path) -> Self {
        let cwd = cwd.to_path_buf();
        Self {
            agents_md: context_doc_meta(&cwd.join("AGENTS.md")),
            readme_md: context_doc_meta(&cwd.join("README.md")),
            cwd,
        }
    }
}

fn context_doc_meta(path: &Path) -> Option<ContextDocMeta> {
    let metadata = std::fs::metadata(path).ok()?;
    if !metadata.is_file() {
        return None;
    }
    Some(ContextDocMeta {
        len: metadata.len(),
        modified: metadata.modified().ok(),
    })
}

fn read_context_doc(path: &Path) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    Some(content.chars().take(MAX_CONTEXT_DOC_CHARS).collect())
}

fn infer_project_hint(agents_content: &str, readme_content: &str) -> GhostProjectHint {
    let combined = format!("{agents_content}\n{readme_content}").to_ascii_lowercase();
    if combined.contains("cargo")
        || combined.contains("rust")
        || combined.contains("gpui")
        || combined.contains(".rs")
    {
        return GhostProjectHint::Rust;
    }
    if combined.contains("typescript")
        || combined.contains("tsconfig")
        || combined.contains(".tsx")
        || combined.contains(".ts")
    {
        return GhostProjectHint::TypeScript;
    }
    if combined.contains("javascript")
        || combined.contains("package.json")
        || combined.contains("node")
        || combined.contains("bun")
    {
        return GhostProjectHint::JavaScript;
    }
    GhostProjectHint::General
}

static GHOST_ID_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

fn next_ghost_id() -> u64 {
    GHOST_ID_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

pub fn compute_ghost_prediction(
    query: &str,
    flat_results: &[SearchResult],
) -> Option<GhostPrediction> {
    compute_ghost_prediction_with_revision(query, flat_results, PredictionRevision::default())
}

pub fn compute_ghost_prediction_with_revision(
    query: &str,
    flat_results: &[SearchResult],
    revision: PredictionRevision,
) -> Option<GhostPrediction> {
    compute_ghost_prediction_with_context(query, flat_results, revision, &GhostContext::default())
}

pub fn compute_ghost_prediction_with_context(
    query: &str,
    flat_results: &[SearchResult],
    revision: PredictionRevision,
    context: &GhostContext,
) -> Option<GhostPrediction> {
    // Pure/synchronous path: a dominant command always wins (instant,
    // Tab-acceptable); otherwise show the deterministic faint starter. The real
    // prompt prediction is the debounced LLM side-channel in the app layer,
    // which only replaces the starter when a sanitized, current result returns.
    // We deliberately never surface a decorative "Ask Agent Chat" hint.
    compute_command_ghost_prediction(query, flat_results, revision)
        .or_else(|| fallback_prompt_starter_prediction(query, revision, context))
}

/// Command-completion only. Highest priority and the hard gate the LLM
/// side-channel must yield to: if this returns `Some`, no LLM call should fire.
pub fn compute_command_ghost_prediction(
    query: &str,
    flat_results: &[SearchResult],
    revision: PredictionRevision,
) -> Option<GhostPrediction> {
    if query.chars().count() < 2 || query.ends_with(' ') {
        return None;
    }
    command_completion_prediction(query, flat_results, revision)
}

fn command_completion_prediction(
    query: &str,
    flat_results: &[SearchResult],
    revision: PredictionRevision,
) -> Option<GhostPrediction> {
    let eligible: Vec<&SearchResult> = flat_results
        .iter()
        .filter(|r| is_eligible_for_ghost(r))
        .take(10)
        .collect();

    if eligible.is_empty() {
        return None;
    }

    let top = eligible[0];
    let label = top.name();

    let suffix = suffix_for_prefix(query, label)?;

    if suffix.is_empty() {
        return None;
    }

    let top_score = top.score();
    let second_score = eligible.get(1).map(|r| r.score()).unwrap_or(0);
    let top_tier = top.match_tier();

    if !dominant_enough(top_score, second_score, top_tier) {
        return None;
    }

    let gap = if second_score > 0 {
        (top_score - second_score) as f32 / top_score.max(1) as f32
    } else {
        1.0
    };

    Some(GhostPrediction {
        query: query.to_string(),
        full_label: label.to_string(),
        ghost_suffix: suffix,
        confidence: gap.clamp(0.0, 1.0),
        revision,
        ghost_id: next_ghost_id(),
        kind: GhostPredictionKind::CommandCompletion,
    })
}

// ===========================================================================
// Deterministic fallback "starter"
//
// Shown instantly while the debounced LLM prediction is pending or unavailable.
// It must predict an ACTION the agent could take next (a real prompt starter),
// never a decorative location tail. The rejected "in {project}" / "in this
// project" tail-append path is gone: a starter is allowed only when it is a
// genuine actionable continuation. Faint and NOT Tab-acceptable.
// ===========================================================================

/// The deterministic, low-confidence fallback prediction. Returns `None` rather
/// than fabricating a location-only tail.
pub fn fallback_prompt_starter_prediction(
    query: &str,
    revision: PredictionRevision,
    context: &GhostContext,
) -> Option<GhostPrediction> {
    let trimmed = query.trim();
    if !is_safe_agent_prompt_seed(trimmed) {
        return None;
    }
    let label = fallback_prompt_starter_label(trimmed, context)?;
    let ghost_suffix = suffix_for_prefix(trimmed, &label)?;
    if ghost_suffix.is_empty() {
        return None;
    }
    if contains_case_insensitive(&ghost_suffix, "agent chat")
        || contains_case_insensitive(&ghost_suffix, "in this project")
        || context
            .digest
            .project_name
            .as_deref()
            .is_some_and(|name| contains_case_insensitive(&ghost_suffix, name))
    {
        return None;
    }
    Some(GhostPrediction {
        query: query.to_string(),
        full_label: label,
        ghost_suffix,
        confidence: AGENT_STARTER_CONFIDENCE,
        revision,
        ghost_id: next_ghost_id(),
        kind: GhostPredictionKind::AgentPromptStarter,
    })
}

fn fallback_prompt_starter_label(query: &str, context: &GhostContext) -> Option<String> {
    let lower = query.to_ascii_lowercase();
    let topic = fallback_topic(context).unwrap_or_else(|| "the relevant code".to_string());
    let candidates = [
        format!("when is {topic} updated"),
        format!("when is {topic} initialized"),
        format!("what's the best place to change {topic}"),
        format!("what is the best place to change {topic}"),
        format!("where is {topic} implemented"),
        format!("how does {topic} work"),
        format!("why is {topic} failing"),
        format!("fix {topic} by tracing the relevant code path"),
        format!("debug {topic} by checking the state flow"),
        format!("test {topic} with the smallest focused check"),
        format!("review {topic} for regressions"),
        format!("find the code that updates {topic}"),
    ];
    for candidate in candidates {
        let Some(candidate) = sanitize_completion_candidate(&candidate) else {
            continue;
        };
        if candidate.to_ascii_lowercase().starts_with(&lower) {
            return Some(candidate);
        }
    }
    // Arbitrary safe multi-word text: preserve the user's exact words and add an
    // action-oriented continuation — never a project-name tail.
    if query.split_whitespace().count() >= 2 {
        return Some(format!("{query} by tracing the relevant code path"));
    }
    None
}

fn fallback_topic(context: &GhostContext) -> Option<String> {
    completion_topics(context).into_iter().find(|topic| {
        !is_low_signal_topic(topic)
            && !contains_case_insensitive(topic, "agent chat")
            && context
                .digest
                .project_name
                .as_deref()
                .is_none_or(|project| !contains_case_insensitive(topic, project))
    })
}

fn suffix_for_prefix(query: &str, label: &str) -> Option<String> {
    let query_char_count = query.chars().count();
    let split = byte_index_after_n_chars(label, query_char_count)?;
    let label_prefix = &label[..split];
    if label_prefix.to_lowercase() == query.to_lowercase() {
        Some(label[split..].to_string())
    } else {
        None
    }
}

fn byte_index_after_n_chars(s: &str, n: usize) -> Option<usize> {
    if n == 0 {
        return Some(0);
    }
    s.char_indices()
        .nth(n)
        .map(|(idx, _)| idx)
        .or_else(|| (s.chars().count() == n).then_some(s.len()))
}

/// Whether a query is eligible to receive an agent-prompt ghost (deterministic
/// starter OR debounced LLM). Broadened to accept natural language like
/// "What's the" / "When is" (apostrophes, periods, commas, colons) while still
/// rejecting sigil/url/math/path inputs that belong to other surfaces.
pub fn is_safe_agent_prompt_seed(query: &str) -> bool {
    let char_count = query.chars().count();
    if !(3..=160).contains(&char_count) {
        return false;
    }
    if query.starts_with([':', '+', '@', '/', '~', '!', '#', '>', '|']) {
        return false;
    }
    if crate::scripts::input_detection::is_url(query)
        || crate::scripts::input_detection::is_math_expression(query)
        || crate::scripts::input_detection::is_file_path(query)
    {
        return false;
    }
    query.chars().all(|ch| {
        ch.is_alphanumeric()
            || ch.is_whitespace()
            || matches!(
                ch,
                '-' | '_' | '?' | '\'' | '\u{2019}' | '"' | '.' | ',' | ':' | ';'
            )
    })
}

/// All usable topics for candidate generation: high-signal topics first, then
/// low-signal tech tags, excluding the project name (already used elsewhere).
fn completion_topics(context: &GhostContext) -> Vec<String> {
    let project_lower = context
        .digest
        .project_name
        .as_deref()
        .map(|name| name.to_ascii_lowercase());
    let mut topics = Vec::new();
    for include_low_signal in [false, true] {
        for topic in &context.digest.topic_keywords {
            let Some(topic) = sanitize_completion_candidate(topic) else {
                continue;
            };
            let lower = topic.to_ascii_lowercase();
            if project_lower.as_deref() == Some(lower.as_str()) {
                continue;
            }
            if is_low_signal_topic(&lower) != include_low_signal {
                continue;
            }
            push_unique(&mut topics, lower);
        }
    }
    if topics.is_empty() {
        if let Some(topic) = best_topic(context) {
            push_unique(&mut topics, topic);
        }
    }
    topics
}

fn contains_case_insensitive(haystack: &str, needle: &str) -> bool {
    haystack
        .to_ascii_lowercase()
        .contains(&needle.to_ascii_lowercase())
}

fn has_alphanumeric(text: &str) -> bool {
    text.chars().any(char::is_alphanumeric)
}

fn sanitize_completion_candidate(candidate: &str) -> Option<String> {
    let cleaned = sanitize_fragment(candidate)?;
    if cleaned.contains("README.md") || cleaned.contains("AGENTS.md") {
        return None;
    }
    // Ghost text must never suggest "Agent Chat" (as a topic or inside any
    // generated candidate). This repo legitimately documents an "Agent Chat"
    // feature, so the topic would otherwise resurface (e.g. "how do" ->
    // "how does agent chat work"). Drop any candidate that mentions it.
    if contains_case_insensitive(&cleaned, "agent chat") {
        return None;
    }
    if cleaned.chars().count() > MAX_COMPLETION_CHARS {
        return None;
    }
    Some(cleaned)
}

fn best_topic(context: &GhostContext) -> Option<String> {
    // The H1 project heading also lands in topic_keywords; don't reuse the
    // project name as the "topic" since it already appears as the project.
    let project_lower = context
        .digest
        .project_name
        .as_deref()
        .map(|name| name.to_ascii_lowercase());
    let is_project = |topic: &str| project_lower.as_deref() == Some(topic);
    let topics = &context.digest.topic_keywords;
    topics
        .iter()
        .find(|topic| !is_low_signal_topic(topic) && !is_project(topic))
        .or_else(|| topics.iter().find(|topic| !is_project(topic)))
        .or_else(|| topics.first())
        .cloned()
}

fn is_low_signal_topic(topic: &str) -> bool {
    matches!(
        topic,
        "rust" | "typescript" | "javascript" | "bun" | "cargo" | "gpui"
    )
}

// ===========================================================================
// LLM ghost prediction (debounced side-channel)
//
// The real prompt predictor. ghost.rs only owns the PURE pieces: sanitized
// context excerpt, request construction, output sanitization, and the cache
// key/value types. The async orchestration (debounce, cancel, cache, write-back)
// lives in the app layer (ScriptListApp::maybe_start_ghost_llm_prediction).
// ===========================================================================

/// System prompt for the LLM ghost predictor. Forces a single short
/// continuation of the user's partial query and bans Agent Chat / doc filenames
/// / secrets / URLs.
pub const GHOST_LLM_SYSTEM_PROMPT: &str = r#"You predict inline ghost text for a macOS launcher.
Predict the single most likely full prompt the user is composing for a coding/agent assistant working in this project.
Return ONLY the continuation: the text that comes AFTER the user's exact partial query.
Rules:
- No preamble.
- No labels.
- No quotes.
- No markdown.
- Do not repeat the partial query.
- One line only.
- Keep it short: usually 4 to 14 words.
- Never include "Agent Chat".
- Do not mention README.md or AGENTS.md.
- Do not output secrets, paths, URLs, or API keys.
- If there is no useful prediction, return an empty string."#;

/// Builds a sanitized, multi-line excerpt of the cwd docs to BIAS the LLM. Skips
/// code fences, markdown noise, secrets, URLs, and source filenames. `None` when
/// nothing safe remains.
fn build_llm_context_excerpt(agents: &str, readme: &str) -> Option<String> {
    let mut out = String::new();
    for source in [agents, readme] {
        let mut in_code_fence = false;
        for raw_line in source.lines() {
            let line = raw_line.trim();
            if line.starts_with("```") || line.starts_with("~~~") {
                in_code_fence = !in_code_fence;
                continue;
            }
            if in_code_fence || line.is_empty() || looks_like_markdown_noise(line) {
                continue;
            }
            if looks_sensitive(line) || line.contains("://") {
                continue;
            }
            let stripped = strip_list_marker(line);
            let Some(fragment) = sanitize_fragment(stripped) else {
                continue;
            };
            if out.len() + fragment.len() + 1 > MAX_LLM_CONTEXT_EXCERPT_CHARS {
                break;
            }
            if !out.is_empty() {
                out.push('\n');
            }
            out.push_str(&fragment);
        }
    }
    (!out.trim().is_empty()).then_some(out)
}

/// Builds the provider messages for one ghost prediction request.
pub fn build_ghost_llm_messages(
    partial_query: &str,
    context: &GhostContext,
) -> Vec<crate::ai::providers::ProviderMessage> {
    let digest = &context.digest;
    let project_name = digest
        .project_name
        .as_deref()
        .or(context.cwd_label.as_deref())
        .unwrap_or("unknown project");
    let task_phrases = if digest.task_phrases.is_empty() {
        "- none".to_string()
    } else {
        digest
            .task_phrases
            .iter()
            .take(8)
            .map(|s| format!("- {s}"))
            .collect::<Vec<_>>()
            .join("\n")
    };
    let topic_keywords = if digest.topic_keywords.is_empty() {
        "- none".to_string()
    } else {
        digest
            .topic_keywords
            .iter()
            .take(10)
            .map(|s| format!("- {s}"))
            .collect::<Vec<_>>()
            .join("\n")
    };
    let excerpt = context.llm_excerpt.as_deref().unwrap_or("none");
    vec![
        crate::ai::providers::ProviderMessage::system(GHOST_LLM_SYSTEM_PROMPT),
        crate::ai::providers::ProviderMessage::user(format!(
            r#"Project context:
Project name: {project_name}
Task phrases:
{task_phrases}
Topic keywords:
{topic_keywords}
Sanitized project notes:
---BEGIN PROJECT NOTES---
{excerpt}
---END PROJECT NOTES---
Partial query:
---BEGIN PARTIAL QUERY---
{partial_query}
---END PARTIAL QUERY---
Return only the continuation after the partial query."#
        )),
    ]
}

/// Builds the SINGLE composed prompt string for the on-device (llama.cpp /
/// GGUF) ghost predictor. Local models have no separate system channel, so all
/// instruction + project bias + the caret prefix must live in one string (the
/// Cotabby `LlamaPromptRenderer` approach). The caret prefix is intentionally
/// the last payload so small instruct models continue it directly.
pub fn build_local_ghost_prompt(partial_query: &str, context: &GhostContext) -> String {
    let partial_query = partial_query.trim_end();
    let digest = &context.digest;
    let project_name = digest
        .project_name
        .as_deref()
        .or(context.cwd_label.as_deref())
        .unwrap_or("unknown project");
    let task_phrases = if digest.task_phrases.is_empty() {
        "- none".to_string()
    } else {
        digest
            .task_phrases
            .iter()
            .take(8)
            .map(|s| format!("- {s}"))
            .collect::<Vec<_>>()
            .join("\n")
    };
    let topic_keywords = if digest.topic_keywords.is_empty() {
        "- none".to_string()
    } else {
        digest
            .topic_keywords
            .iter()
            .take(10)
            .map(|s| format!("- {s}"))
            .collect::<Vec<_>>()
            .join("\n")
    };
    let excerpt = context.llm_excerpt.as_deref().unwrap_or("none");
    // The caret prefix is intentionally the LAST payload (Cotabby's choice): small
    // instruct models continue the trailing text directly instead of re-answering.
    // There is no trailing "Continuation:" cue — that made instruct models emit an
    // empty turn. The model literally continues `{partial_query}`.
    format!(
        r#"Task:
- Continue the user's existing launcher input exactly where it stops.
- This is inline autocomplete, not chat. Do not answer the user.
- The input may be a command, a question, or a natural-language prompt for a coding agent.
- Use the project context only as a bias when it helps.
- Continue on the same line; output only the text that comes next.
- Return plain text only: no labels, bullets, markdown, quotes, or explanation.
- Keep it short: usually 4 to 14 words.
- Never include "Agent Chat".
- Do not mention README.md or AGENTS.md.
- Do not output secrets, paths, URLs, or API keys.
Project context:
Project name: {project_name}
Task phrases:
{task_phrases}
Topic keywords:
{topic_keywords}
Sanitized project notes:
---BEGIN PROJECT NOTES---
{excerpt}
---END PROJECT NOTES---
Continue the partial input below. Output only the words that follow it.

{partial_query}"#
    )
}

/// Sanitizes a raw model response into a safe ghost suffix, or `None`.
pub fn sanitize_llm_completion_suffix(raw_response: &str, query: &str) -> Option<String> {
    let query = query.trim_end();
    if query.is_empty() {
        return None;
    }
    let mut text = raw_response
        .trim()
        .trim_matches('`')
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
        .trim()
        .to_string();
    for prefix in [
        "completion:",
        "continuation:",
        "suffix:",
        "answer:",
        "ghost text:",
    ] {
        if text.to_ascii_lowercase().starts_with(prefix) {
            text = text[prefix.len()..].trim_start().to_string();
        }
    }
    // Re-trim quotes/backticks a label may have hidden (e.g. `completion: "x"`).
    text = text
        .trim()
        .trim_matches('`')
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
        .trim()
        .to_string();
    // One line only.
    text = text.lines().next().unwrap_or("").trim().to_string();
    // Strip echoed full prompt or echoed prefix.
    if let Some(suffix) = suffix_for_prefix(query, &text) {
        text = suffix.trim_start().to_string();
    }
    // Collapse whitespace.
    text = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if text.is_empty() {
        return None;
    }
    if contains_case_insensitive(&text, "agent chat")
        || text.contains("README.md")
        || text.contains("AGENTS.md")
        || text.contains("://")
        || looks_sensitive(&text)
    {
        return None;
    }
    // Make it a real suffix when the model forgot the leading space.
    let query_last = query.chars().last();
    let suffix_first = text.chars().next();
    if query_last.is_some_and(|ch| ch.is_alphanumeric())
        && suffix_first.is_some_and(|ch| ch.is_alphanumeric())
    {
        text.insert(0, ' ');
    }
    let capped = cap_completion_chars(&text, MAX_LLM_COMPLETION_CHARS);
    if capped.trim().is_empty() || !has_alphanumeric(&capped) {
        return None;
    }
    Some(capped)
}

fn cap_completion_chars(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_string();
    }
    let mut out: String = text.chars().take(max_chars).collect();
    // Avoid ending mid-word when possible.
    if let Some(last_space) = out.rfind(' ') {
        out.truncate(last_space);
    }
    out.trim_end_matches(['-', ',', ';', ':']).to_string()
}

/// Builds an accepted, Tab-acceptable [`GhostPrediction`] from a raw model
/// response, or `None` if the sanitized suffix is empty/unsafe.
pub fn llm_prediction_from_response(
    query: &str,
    raw_response: &str,
    revision: PredictionRevision,
) -> Option<GhostPrediction> {
    let ghost_suffix = sanitize_llm_completion_suffix(raw_response, query)?;
    let full_label = format!("{}{}", query.trim_end(), ghost_suffix);
    Some(GhostPrediction {
        query: query.to_string(),
        full_label,
        ghost_suffix,
        confidence: AGENT_LLM_PROMPT_CONFIDENCE,
        revision,
        ghost_id: next_ghost_id(),
        kind: GhostPredictionKind::AgentPromptCompletion,
    })
}

/// Parses README.md / AGENTS.md markdown into a conservative [`ContextDigest`].
/// Deliberately extracts only headings, imperative bullets, and summarized
/// known commands; never raw paths, URLs, secrets, or the source filenames.
pub fn parse_context_digest(
    cwd_label: Option<&str>,
    agents_content: &str,
    readme_content: &str,
) -> ContextDigest {
    let mut digest = ContextDigest {
        project_name: extract_project_name(readme_content)
            .or_else(|| extract_project_name(agents_content))
            .or_else(|| cwd_label.and_then(sanitize_project_name)),
        ..ContextDigest::default()
    };
    collect_task_phrases(agents_content, &mut digest.task_phrases);
    collect_task_phrases(readme_content, &mut digest.task_phrases);
    dedupe_truncate(&mut digest.task_phrases, MAX_CONTEXT_TASK_PHRASES);
    collect_topic_keywords(readme_content, &mut digest.topic_keywords);
    collect_topic_keywords(agents_content, &mut digest.topic_keywords);
    if let Some(project_name) = digest.project_name.as_deref() {
        collect_project_name_topics(project_name, &mut digest.topic_keywords);
    }
    dedupe_truncate(&mut digest.topic_keywords, MAX_CONTEXT_TOPIC_KEYWORDS);
    digest
}

fn extract_project_name(markdown: &str) -> Option<String> {
    for line in markdown.lines() {
        let trimmed = line.trim();
        let heading = trimmed
            .strip_prefix("# ")
            .or_else(|| trimmed.strip_prefix("#\t"));
        let Some(raw_name) = heading else {
            continue;
        };
        if let Some(name) = sanitize_project_name(raw_name) {
            return Some(name);
        }
    }
    None
}

fn sanitize_project_name(raw: &str) -> Option<String> {
    let cleaned = sanitize_fragment(raw)?;
    let lower = cleaned.to_ascii_lowercase();
    if lower == "readme"
        || lower == "agents"
        || lower.contains("readme.md")
        || lower.contains("agents.md")
    {
        return None;
    }
    let char_count = cleaned.chars().count();
    if !(2..=48).contains(&char_count) {
        return None;
    }
    Some(cleaned)
}

fn collect_task_phrases(markdown: &str, out: &mut Vec<String>) {
    let mut in_code_fence = false;
    for raw_line in markdown.lines() {
        let line = raw_line.trim();
        if line.starts_with("```") || line.starts_with("~~~") {
            in_code_fence = !in_code_fence;
            continue;
        }
        if in_code_fence {
            if let Some(command) = command_phrase_from_line(line) {
                push_unique(out, command);
            }
            continue;
        }
        if line.is_empty() || looks_like_markdown_noise(line) {
            continue;
        }
        if let Some(command) = command_phrase_from_line(line) {
            push_unique(out, command);
            continue;
        }
        let stripped = strip_list_marker(line);
        let Some(fragment) = sanitize_fragment(stripped) else {
            continue;
        };
        if starts_with_task_verb(&fragment) {
            push_unique(out, lower_first(fragment));
        }
    }
}

fn command_phrase_from_line(line: &str) -> Option<String> {
    let lower = line.to_ascii_lowercase();
    if looks_sensitive(&lower) || lower.contains("://") {
        return None;
    }
    if lower.contains("agent-cargo.sh") || lower.contains("agent-cargo") {
        if lower.contains("test") {
            return Some("run agent-cargo test".to_string());
        }
        if lower.contains("check") {
            return Some("run agent-cargo check".to_string());
        }
        if lower.contains("clippy") {
            return Some("run agent-cargo clippy".to_string());
        }
        return Some("use agent-cargo for cargo checks".to_string());
    }
    if lower.contains("cargo ") {
        if lower.contains(" test") {
            return Some("run cargo test".to_string());
        }
        if lower.contains(" check") {
            return Some("run cargo check".to_string());
        }
        if lower.contains(" clippy") {
            return Some("run cargo clippy".to_string());
        }
        if lower.contains(" build") {
            return Some("run cargo build".to_string());
        }
    }
    if lower.contains("bun ") {
        if lower.contains(" test") {
            return Some("run bun test".to_string());
        }
        if lower.contains(" install") {
            return Some("run bun install".to_string());
        }
        if lower.contains(" run") {
            return Some("run bun script".to_string());
        }
    }
    if lower.contains("npm ") {
        if lower.contains(" test") {
            return Some("run npm test".to_string());
        }
        if lower.contains(" install") {
            return Some("run npm install".to_string());
        }
    }
    if lower.contains("pnpm ") {
        if lower.contains(" test") {
            return Some("run pnpm test".to_string());
        }
        if lower.contains(" install") {
            return Some("run pnpm install".to_string());
        }
    }
    if lower.contains("yarn ") {
        if lower.contains(" test") {
            return Some("run yarn test".to_string());
        }
        if lower.contains(" install") {
            return Some("run yarn install".to_string());
        }
    }
    if lower.contains("make ") || lower == "make" {
        return Some("run make".to_string());
    }
    None
}

fn collect_topic_keywords(markdown: &str, out: &mut Vec<String>) {
    for line in markdown.lines() {
        let trimmed = line.trim();
        if looks_like_markdown_noise(trimmed) || looks_sensitive(trimmed) {
            continue;
        }
        if let Some(heading) = markdown_heading_text(trimmed) {
            if let Some(topic) = sanitize_topic_phrase(heading) {
                push_unique(out, topic);
            }
        }
    }
    let lower = markdown.to_ascii_lowercase();
    for (needle, topic) in [
        ("ghost text", "ghost text"),
        // Intentionally NOT seeding "agent chat" as a topic: ghost text must
        // never recommend Agent Chat. (sanitize_completion_candidate also
        // defends against it surfacing via README/AGENTS headings.)
        ("fuzzy search", "fuzzy search"),
        ("command palette", "command palette"),
        ("launcher", "launcher"),
        ("gpui", "gpui"),
        ("rust", "rust"),
        ("typescript", "typescript"),
        ("javascript", "javascript"),
        ("bun", "bun"),
        ("cargo", "cargo"),
    ] {
        if lower.contains(needle) {
            push_unique(out, topic.to_string());
        }
    }
}

fn collect_project_name_topics(project_name: &str, out: &mut Vec<String>) {
    let lower = project_name.to_ascii_lowercase();
    for token in lower
        .split(|ch: char| !ch.is_alphanumeric())
        .filter(|token| token.chars().count() >= 4)
    {
        if !is_stopword(token) {
            push_unique(out, token.to_string());
        }
    }
}

fn markdown_heading_text(line: &str) -> Option<&str> {
    let hashes = line.chars().take_while(|ch| *ch == '#').count();
    if !(1..=4).contains(&hashes) {
        return None;
    }
    line.get(hashes..)?.trim().strip_prefix(' ').map(str::trim)
}

fn sanitize_topic_phrase(raw: &str) -> Option<String> {
    let cleaned = sanitize_fragment(raw)?;
    let lower = cleaned.to_ascii_lowercase();
    if matches!(
        lower.as_str(),
        "overview"
            | "introduction"
            | "getting started"
            | "installation"
            | "usage"
            | "development"
            | "contributing"
            | "license"
            | "table of contents"
    ) {
        return None;
    }
    if cleaned.split_whitespace().count() > 4 {
        return None;
    }
    Some(lower)
}

/// Strips markdown decorations, link URLs, and risky punctuation from a line,
/// rejecting fragments that contain secrets, URLs, or source filenames.
fn sanitize_fragment(raw: &str) -> Option<String> {
    if raw.contains("README.md") || raw.contains("AGENTS.md") {
        return None;
    }
    let lower = raw.to_ascii_lowercase();
    if looks_sensitive(&lower) || lower.contains("://") {
        return None;
    }
    let mut out = String::with_capacity(raw.len());
    let mut in_link_url = false;
    let mut previous_was_space = false;
    for ch in raw.chars() {
        match ch {
            '(' if out.ends_with(']') => {
                in_link_url = true;
            }
            ')' if in_link_url => {
                in_link_url = false;
            }
            _ if in_link_url => {}
            '`' | '*' | '_' | '[' | ']' | '<' | '>' | '"' => {
                if !previous_was_space {
                    out.push(' ');
                    previous_was_space = true;
                }
            }
            '/' | '\\' | '=' | '$' => {
                if !previous_was_space {
                    out.push(' ');
                    previous_was_space = true;
                }
            }
            ch if ch.is_alphanumeric()
                || ch.is_whitespace()
                || matches!(ch, '-' | '\'' | '?' | ':') =>
            {
                if ch.is_whitespace() {
                    if !previous_was_space {
                        out.push(' ');
                        previous_was_space = true;
                    }
                } else {
                    out.push(ch);
                    previous_was_space = false;
                }
            }
            _ => {
                if !previous_was_space {
                    out.push(' ');
                    previous_was_space = true;
                }
            }
        }
    }
    let cleaned = out
        .trim()
        .trim_matches(|ch: char| matches!(ch, '-' | ':' | '.'))
        .trim()
        .to_string();
    let char_count = cleaned.chars().count();
    if !(2..=MAX_COMPLETION_CHARS).contains(&char_count) {
        return None;
    }
    if cleaned.contains("README.md") || cleaned.contains("AGENTS.md") {
        return None;
    }
    Some(cleaned)
}

fn strip_list_marker(line: &str) -> &str {
    let trimmed = line.trim_start();
    for marker in ["- ", "* ", "+ "] {
        if let Some(rest) = trimmed.strip_prefix(marker) {
            return rest.trim_start();
        }
    }
    let Some(dot_index) = trimmed.find('.') else {
        return trimmed;
    };
    let (prefix, rest) = trimmed.split_at(dot_index);
    if !prefix.is_empty() && prefix.chars().all(|ch| ch.is_ascii_digit()) {
        return rest.trim_start_matches('.').trim_start();
    }
    trimmed
}

fn starts_with_task_verb(fragment: &str) -> bool {
    let first = fragment
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .trim_matches(|ch: char| !ch.is_alphanumeric())
        .to_ascii_lowercase();
    matches!(
        first.as_str(),
        "use"
            | "run"
            | "execute"
            | "open"
            | "create"
            | "update"
            | "review"
            | "test"
            | "fix"
            | "debug"
            | "check"
            | "build"
            | "install"
            | "start"
            | "reload"
            | "verify"
            | "prefer"
            | "keep"
            | "avoid"
    )
}

fn looks_like_markdown_noise(line: &str) -> bool {
    let lower = line.to_ascii_lowercase();
    line.starts_with("[!")
        || lower.contains("shields.io")
        || lower.contains("<img")
        || lower.contains("<svg")
        || lower.contains("</")
        || lower.starts_with("---")
}

fn looks_sensitive(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    lower.contains("password")
        || lower.contains("passwd")
        || lower.contains("secret")
        || lower.contains("api_key")
        || lower.contains("api key")
        || lower.contains("access token")
        || lower.contains("private key")
        || lower.contains("bearer ")
        || has_long_hex_token(&lower)
}

fn has_long_hex_token(text: &str) -> bool {
    let mut run = 0;
    for ch in text.chars() {
        if ch.is_ascii_hexdigit() {
            run += 1;
            if run >= 24 {
                return true;
            }
        } else {
            run = 0;
        }
    }
    false
}

fn lower_first(s: String) -> String {
    let mut chars = s.chars();
    let Some(first) = chars.next() else {
        return s;
    };
    format!("{}{}", first.to_lowercase(), chars.collect::<String>())
}

fn push_unique(out: &mut Vec<String>, value: String) {
    let value = value.trim().to_string();
    if value.is_empty() {
        return;
    }
    let lower = value.to_ascii_lowercase();
    if out
        .iter()
        .any(|existing| existing.to_ascii_lowercase() == lower)
    {
        return;
    }
    out.push(value);
}

fn dedupe_truncate(values: &mut Vec<String>, max: usize) {
    let mut seen = HashSet::new();
    values.retain(|value| {
        let lower = value.to_ascii_lowercase();
        seen.insert(lower)
    });
    values.truncate(max);
}

fn is_stopword(token: &str) -> bool {
    matches!(
        token,
        "this"
            | "that"
            | "with"
            | "from"
            | "into"
            | "your"
            | "their"
            | "project"
            | "readme"
            | "agents"
            | "using"
            | "built"
    )
}

fn dominant_enough(top_score: i32, second_score: i32, top_tier: i32) -> bool {
    if top_tier < 850 {
        return false;
    }
    let gap = top_score - second_score;
    gap > 200 || second_score == 0
}

fn is_eligible_for_ghost(result: &SearchResult) -> bool {
    !matches!(
        result,
        SearchResult::Fallback(_)
            | SearchResult::File(_)
            | SearchResult::Note(_)
            | SearchResult::Todo(_)
            | SearchResult::ClipboardHistory(_)
            | SearchResult::DictationHistory(_)
            | SearchResult::BrowserHistory(_)
            | SearchResult::BrowserTab(_)
            | SearchResult::ScriptIssue(_)
            | SearchResult::SpineProjection(_)
            | SearchResult::Agent(_)
    )
}

pub fn reconcile_typed_through(
    old_query: &str,
    new_query: &str,
    prediction: &GhostPrediction,
) -> Option<GhostPrediction> {
    if !prediction.accepts_tab() {
        return None;
    }
    if !new_query.starts_with(old_query) {
        return None;
    }
    let added = &new_query[old_query.len()..];
    let suffix_lower = prediction.ghost_suffix.to_lowercase();
    let added_lower = added.to_lowercase();
    if suffix_lower.starts_with(&added_lower) && added_lower.len() < suffix_lower.len() {
        let new_suffix = &prediction.ghost_suffix[added.len()..];
        Some(GhostPrediction {
            query: new_query.to_string(),
            full_label: prediction.full_label.clone(),
            ghost_suffix: new_suffix.to_string(),
            confidence: prediction.confidence,
            revision: prediction.revision,
            ghost_id: next_ghost_id(),
            kind: prediction.kind,
        })
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- helpers ----------------------------------------------------------

    fn make_builtin_result(name: &str, score: i32) -> SearchResult {
        SearchResult::BuiltIn(crate::scripts::types::BuiltInMatch {
            entry: crate::builtins::BuiltInEntry {
                id: name.to_lowercase().replace(' ', "-"),
                name: name.to_string(),
                description: format!("Open {name}"),
                keywords: vec![],
                feature: crate::builtins::BuiltInFeature::ClipboardHistory,
                icon: None,
                group: crate::builtins::BuiltInGroup::Core,
            },
            score,
            match_evidence: None,
        })
    }

    fn make_send_to_ai_fallback_result() -> SearchResult {
        let fallback = crate::fallbacks::builtins::get_builtin_fallbacks()
            .into_iter()
            .find(|fallback| fallback.id == crate::fallbacks::builtins::SEND_TO_AI_FALLBACK_ID)
            .expect("send-to-ai fallback should exist");
        SearchResult::Fallback(crate::scripts::types::FallbackMatch::new(
            crate::fallbacks::FallbackItem::Builtin(fallback),
            0,
        ))
    }

    fn test_prediction(query: &str, label: &str, suffix: &str) -> GhostPrediction {
        GhostPrediction {
            query: query.to_string(),
            full_label: label.to_string(),
            ghost_suffix: suffix.to_string(),
            confidence: 0.8,
            revision: PredictionRevision::default(),
            ghost_id: 0,
            kind: GhostPredictionKind::CommandCompletion,
        }
    }

    /// Context with a real digest, used to drive the deterministic starter
    /// topic selection. "agent chat" is intentionally present to prove it is
    /// never surfaced.
    fn always_infer_context() -> GhostContext {
        GhostContext {
            cwd_label: Some("script-kit-gpui".to_string()),
            has_agents_md: true,
            has_readme_md: true,
            project_hint: GhostProjectHint::Rust,
            digest: ContextDigest {
                project_name: Some("Script Kit GPUI".to_string()),
                task_phrases: vec!["run agent-cargo test".to_string()],
                topic_keywords: vec![
                    "ghost text".to_string(),
                    "agent chat".to_string(),
                    "fuzzy search".to_string(),
                    "launcher".to_string(),
                    "gpui".to_string(),
                    "rust".to_string(),
                ],
            },
            llm_excerpt: Some("Use the agent cargo wrapper for Rust checks.".to_string()),
        }
    }

    // -- prefix / command completion (unchanged behavior) -----------------

    #[test]
    fn suffix_extraction() {
        assert_eq!(
            suffix_for_prefix("clip", "Clipboard History"),
            Some("board History".to_string())
        );
        assert_eq!(
            suffix_for_prefix("Clip", "Clipboard History"),
            Some("board History".to_string())
        );
        assert_eq!(suffix_for_prefix("xyz", "Clipboard History"), None);
    }

    #[test]
    fn suffix_extraction_handles_utf8_case_insensitive_prefix() {
        assert_eq!(suffix_for_prefix("é", "Éclair"), Some("clair".to_string()));
        assert_eq!(
            suffix_for_prefix("🙂", "🙂 Smile"),
            Some(" Smile".to_string())
        );
    }

    #[test]
    fn no_ghost_for_short_query() {
        assert!(compute_ghost_prediction("", &[]).is_none());
        assert!(compute_ghost_prediction("a", &[]).is_none());
    }

    #[test]
    fn no_ghost_for_trailing_space_command() {
        assert!(
            compute_command_ghost_prediction("clip ", &[], PredictionRevision::default()).is_none()
        );
    }

    #[test]
    fn ghost_prediction_with_dominant_prefix_match() {
        let results = vec![
            make_builtin_result("Clipboard History", 950_200),
            make_builtin_result("Clear Cache", 850_100),
        ];
        let pred = compute_ghost_prediction("cli", &results).expect("dominant prefix match");
        assert_eq!(pred.ghost_suffix, "pboard History");
        assert_eq!(pred.full_label, "Clipboard History");
        assert_eq!(pred.kind, GhostPredictionKind::CommandCompletion);
        assert!(pred.accepts_tab());
    }

    #[test]
    fn ghost_first_word_acceptance_suffix_matches_launcher_tab_contract() {
        assert_eq!(first_word_acceptance_suffix("board history"), "board");
        assert_eq!(first_word_acceptance_suffix(" history"), " history");
        assert_eq!(first_word_acceptance_suffix("done"), "done");
    }

    #[test]
    fn no_ghost_when_no_prefix_match() {
        let results = vec![
            make_builtin_result("Process Manager", 950_200),
            make_builtin_result("Settings", 850_100),
        ];
        assert!(
            compute_command_ghost_prediction("cli", &results, PredictionRevision::default())
                .is_none()
        );
    }

    #[test]
    fn no_ghost_for_close_scores() {
        let results = vec![
            make_builtin_result("Clipboard History", 950_200),
            make_builtin_result("CLI Tools", 950_100),
        ];
        assert!(
            compute_command_ghost_prediction("cli", &results, PredictionRevision::default())
                .is_none()
        );
    }

    #[test]
    fn command_completion_for_single_dominant_result() {
        let eligible = make_builtin_result("Settings", 950_500);
        assert!(is_eligible_for_ghost(&eligible));
        let results = vec![make_builtin_result("Settings", 950_500)];
        let pred = compute_ghost_prediction("se", &results).expect("single dominant result");
        assert_eq!(pred.ghost_suffix, "ttings");
        assert_eq!(pred.kind, GhostPredictionKind::CommandCompletion);
    }

    #[test]
    fn command_completion_wins_over_starter() {
        let results = vec![
            make_builtin_result("Clipboard History", 950_200),
            make_send_to_ai_fallback_result(),
        ];
        let context = always_infer_context();
        let pred = compute_ghost_prediction_with_context(
            "cli",
            &results,
            PredictionRevision::default(),
            &context,
        )
        .expect("command completion should win");
        assert_eq!(pred.full_label, "Clipboard History");
        assert_eq!(pred.kind, GhostPredictionKind::CommandCompletion);
        assert!(pred.accepts_tab());
    }

    #[test]
    fn typed_through_advances_command() {
        let pred = test_prediction("cli", "Clipboard History", "pboard History");
        let result = reconcile_typed_through("cli", "clip", &pred).expect("advance");
        assert_eq!(result.ghost_suffix, "board History");
        assert_eq!(result.query, "clip");
    }

    #[test]
    fn typed_through_rejects_mismatch() {
        let pred = test_prediction("cli", "Clipboard History", "pboard History");
        assert!(reconcile_typed_through("cli", "clx", &pred).is_none());
    }

    #[test]
    fn ghost_serializes_in_state() {
        let pred = test_prediction("cli", "Clipboard History", "pboard History");
        let json = serde_json::json!({
            "query": pred.query,
            "fullLabel": pred.full_label,
            "ghostSuffix": pred.ghost_suffix,
            "confidence": pred.confidence,
        });
        assert_eq!(json["ghostSuffix"], "pboard History");
        assert_eq!(json["fullLabel"], "Clipboard History");
    }

    #[test]
    fn revision_stale_detection() {
        let rev1 = PredictionRevision {
            query_rev: 1,
            catalog_rev: 1,
            context_rev: 1,
        };
        let rev2 = PredictionRevision {
            query_rev: 2,
            catalog_rev: 1,
            context_rev: 1,
        };
        assert_ne!(rev1, rev2);
        assert_eq!(rev1, rev1);
    }

    #[test]
    fn ghost_ids_are_unique() {
        let results = vec![make_builtin_result("Settings", 950_500)];
        let p1 = compute_ghost_prediction("se", &results).unwrap();
        let p2 = compute_ghost_prediction("se", &results).unwrap();
        assert_ne!(p1.ghost_id, p2.ghost_id);
    }

    // -- safe seed eligibility -------------------------------------------

    #[test]
    fn safe_agent_prompt_seed_allows_natural_language() {
        assert!(is_safe_agent_prompt_seed("What's the"));
        assert!(is_safe_agent_prompt_seed("When is"));
        assert!(is_safe_agent_prompt_seed("fix the login"));
        assert!(is_safe_agent_prompt_seed("Are snails tacos?"));
    }

    #[test]
    fn safe_agent_prompt_seed_rejects_url_math_path_and_sigil() {
        assert!(!is_safe_agent_prompt_seed(":home"));
        assert!(!is_safe_agent_prompt_seed("@agent"));
        assert!(!is_safe_agent_prompt_seed("/tmp/file"));
        assert!(!is_safe_agent_prompt_seed("ab")); // too short
    }

    // -- deterministic fallback starter ----------------------------------

    #[test]
    fn fallback_starter_handles_when_is_without_project_boilerplate() {
        let context = always_infer_context();
        let pred =
            fallback_prompt_starter_prediction("When is", PredictionRevision::default(), &context)
                .expect("when is should get an action-oriented starter");
        assert_eq!(pred.kind, GhostPredictionKind::AgentPromptStarter);
        assert!(!pred.accepts_tab(), "starter must not be Tab-accepted");
        assert!(pred.full_label.to_ascii_lowercase().starts_with("when is"));
        let suffix = pred.ghost_suffix.to_ascii_lowercase();
        assert!(!suffix.contains("in this project"));
        assert!(!suffix.contains("script kit gpui"));
    }

    #[test]
    fn fallback_starter_handles_whats_the_without_project_boilerplate() {
        let context = always_infer_context();
        let pred = fallback_prompt_starter_prediction(
            "What's the",
            PredictionRevision::default(),
            &context,
        )
        .expect("apostrophe input should still be eligible");
        assert!(pred
            .full_label
            .to_ascii_lowercase()
            .starts_with("what's the"));
        let suffix = pred.ghost_suffix.to_ascii_lowercase();
        assert!(!suffix.contains("in this project"));
        assert!(!suffix.contains("agent chat"));
    }

    #[test]
    fn fallback_starter_preserves_arbitrary_query_with_action_tail() {
        let context = always_infer_context();
        let pred = fallback_prompt_starter_prediction(
            "refactor the sidebar state",
            PredictionRevision::default(),
            &context,
        )
        .expect("arbitrary multi-word query should get an action tail");
        assert!(pred.full_label.starts_with("refactor the sidebar state "));
        assert!(pred.ghost_suffix.starts_with(" by tracing"));
    }

    #[test]
    fn fallback_starter_never_appends_in_project_or_project_name() {
        let context = always_infer_context();
        for phrase in ["fix the login", "debug the crash", "update the routing"] {
            let pred =
                fallback_prompt_starter_prediction(phrase, PredictionRevision::default(), &context)
                    .unwrap_or_else(|| panic!("expected starter for {phrase:?}"));
            let suffix = pred.ghost_suffix.to_ascii_lowercase();
            assert!(
                !suffix.contains("in this project"),
                "{phrase:?} -> {suffix:?}"
            );
            assert!(
                !suffix.contains("script kit gpui"),
                "{phrase:?} -> {suffix:?}"
            );
        }
    }

    #[test]
    fn fallback_starter_never_emits_agent_chat() {
        let context = always_infer_context();
        for phrase in [
            "how does",
            "open agent",
            "what is agent",
            "tell me about the agent",
            "explain the agent panel",
        ] {
            if let Some(pred) =
                fallback_prompt_starter_prediction(phrase, PredictionRevision::default(), &context)
            {
                assert!(
                    !pred
                        .ghost_suffix
                        .to_ascii_lowercase()
                        .contains("agent chat"),
                    "{phrase:?} injected agent chat: {:?}",
                    pred.ghost_suffix
                );
            }
        }
    }

    #[test]
    fn fallback_starter_stays_quiet_for_single_command_word_without_topic() {
        let pred = fallback_prompt_starter_prediction(
            "quit",
            PredictionRevision::default(),
            &GhostContext::default(),
        );
        assert!(
            pred.is_none(),
            "single non-verb word with no topic should stay quiet"
        );
    }

    /// Multi-word actionable phrases should always infer a non-empty,
    /// leak-free, non-Agent-Chat starter while the LLM is pending.
    #[test]
    fn fallback_starter_covers_multi_word_action_phrases() {
        let context = always_infer_context();
        let phrases = [
            "fix the login",
            "debug the websocket reconnect",
            "test the cache layer",
            "review my pull request",
            "refactor spine cwd handling",
            "update the error messages",
            "optimize the fuzzy search",
            "add retry logic",
            "document the public api",
            "search the codebase",
            "write unit tests",
            "explain the architecture",
            "investigate the crash",
            "clean up imports",
            "trace the keypress routing",
            "when is ghost",
            "what's the best place to change",
            "how does ghost text",
        ];
        let mut failures = Vec::new();
        for phrase in phrases {
            match fallback_prompt_starter_prediction(
                phrase,
                PredictionRevision::default(),
                &context,
            ) {
                Some(pred) if !pred.ghost_suffix.is_empty() => {
                    let label = pred.full_label.to_ascii_lowercase();
                    assert!(
                        !label.contains("agents.md")
                            && !label.contains("readme.md")
                            && !label.contains("agent chat"),
                        "{phrase:?} leaked banned text: {}",
                        pred.full_label
                    );
                }
                _ => failures.push(phrase),
            }
        }
        assert!(failures.is_empty(), "no starter for: {failures:?}");
    }

    // -- never Agent Chat across the full path ---------------------------

    #[test]
    fn ghost_text_never_emits_agent_chat() {
        let context = always_infer_context();
        let results = vec![make_send_to_ai_fallback_result()];
        let phrases = [
            "how do you",
            "how does the",
            "how does agent",
            "tell me about agent",
            "open the agent",
            "what is the agent",
            "ask the agent chat",
            "How the weather today",
            "Are snails tacos?",
            "who is the fastest man in the world?",
            "explain this thing",
            "search the agent code",
        ];
        for phrase in phrases {
            if let Some(pred) = compute_ghost_prediction_with_context(
                phrase,
                &results,
                PredictionRevision::default(),
                &context,
            ) {
                let suffix = pred.ghost_suffix.to_ascii_lowercase();
                assert!(
                    !suffix.contains("agent chat"),
                    "{phrase:?} injected agent chat into suffix: {:?}",
                    pred.ghost_suffix
                );
                assert!(!suffix.contains("⌘↵"), "{phrase:?} surfaced removed hint");
            }
        }
    }

    // -- LLM request building --------------------------------------------

    #[test]
    fn ghost_llm_messages_include_partial_query_digest_and_excerpt() {
        let context = always_infer_context();
        let messages = build_ghost_llm_messages("fix the auth", &context);
        assert_eq!(messages.len(), 2);
        let user = &messages[1].content;
        assert!(
            user.contains("fix the auth"),
            "missing partial query: {user}"
        );
        assert!(user.contains("Script Kit GPUI"), "missing project name");
        assert!(user.contains("run agent-cargo test"), "missing task phrase");
        assert!(user.contains("ghost text"), "missing topic keyword");
        assert!(user.contains("Rust checks"), "missing excerpt");
    }

    #[test]
    fn ghost_llm_excerpt_excludes_filenames_paths_secrets_and_fences() {
        let agents = "# Title\n\n- Use README.md when stuck.\n```\nexport API_KEY=deadbeef\n```\nSee https://example.com for docs.\npassword: hunter2\nReview the parser changes.";
        let readme = "Run the launcher and verify ghost text.";
        let excerpt = build_llm_context_excerpt(agents, readme).expect("some safe lines remain");
        assert!(!excerpt.contains("README.md"));
        assert!(!excerpt.contains("://"));
        assert!(!excerpt.to_ascii_lowercase().contains("password"));
        assert!(!excerpt.contains("API_KEY"));
        assert!(excerpt.contains("verify ghost text") || excerpt.contains("parser changes"));
    }

    // -- LLM output sanitization -----------------------------------------

    #[test]
    fn sanitize_llm_completion_strips_echoed_prefix() {
        let suffix = sanitize_llm_completion_suffix("fix the bug in auth", "fix the")
            .expect("echoed full prompt should reduce to the suffix");
        assert_eq!(suffix, " bug in auth");
    }

    #[test]
    fn sanitize_llm_completion_strips_labels_quotes_and_markdown() {
        let suffix = sanitize_llm_completion_suffix("completion: \"the rest of it\"", "do x")
            .expect("labels/quotes should be stripped");
        assert_eq!(suffix, " the rest of it");
    }

    #[test]
    fn sanitize_llm_completion_rejects_agent_chat() {
        assert!(sanitize_llm_completion_suffix("open Agent Chat now", "open the").is_none());
        assert!(sanitize_llm_completion_suffix("see README.md", "open the").is_none());
        assert!(sanitize_llm_completion_suffix("visit https://x.dev", "go to").is_none());
    }

    #[test]
    fn sanitize_llm_completion_caps_to_single_line_and_max_chars() {
        let multi = "first line stays\nsecond line dropped";
        let suffix = sanitize_llm_completion_suffix(multi, "show").expect("single line");
        assert!(!suffix.contains('\n'));
        assert!(suffix.contains("first line stays"));
        assert!(!suffix.contains("second line"));

        let long = "word ".repeat(80);
        let capped = sanitize_llm_completion_suffix(&long, "list").expect("capped");
        assert!(capped.chars().count() <= MAX_LLM_COMPLETION_CHARS + 1);
    }

    #[test]
    fn sanitize_llm_completion_adds_space_after_alphanumeric_prefix() {
        let suffix = sanitize_llm_completion_suffix("the bug", "fix").expect("space inserted");
        assert_eq!(suffix, " the bug");
    }

    #[test]
    fn sanitize_llm_completion_rejects_punctuation_only_suffixes() {
        for raw in [".", "?", "!", "…", " - ", "..."] {
            assert!(
                sanitize_llm_completion_suffix(raw, "fix the login").is_none(),
                "raw response {raw:?} should not become a punctuation-only ghost suffix"
            );
            assert!(
                llm_prediction_from_response("fix the login", raw, PredictionRevision::default())
                    .is_none(),
                "raw response {raw:?} should not become a Tab-acceptable prediction"
            );
        }
    }

    #[test]
    fn llm_prediction_from_response_sets_tab_acceptable_kind() {
        let pred =
            llm_prediction_from_response("fix the", "bug now", PredictionRevision::default())
                .expect("valid prediction");
        assert_eq!(pred.kind, GhostPredictionKind::AgentPromptCompletion);
        assert!(pred.accepts_tab());
        assert_eq!(pred.full_label, "fix the bug now");
        assert_eq!(pred.ghost_suffix, " bug now");
    }

    // -- LLM cache key ----------------------------------------------------

    #[test]
    fn ghost_llm_cache_key_includes_query_cwd_context_rev_and_model() {
        let base = GhostLlmCacheKey {
            query: "fix the".to_string(),
            cwd: Some(std::path::PathBuf::from("/a")),
            context_rev: 1,
            model_id: "m1".to_string(),
        };
        let diff_query = GhostLlmCacheKey {
            query: "fix th".to_string(),
            ..base.clone()
        };
        let diff_cwd = GhostLlmCacheKey {
            cwd: Some(std::path::PathBuf::from("/b")),
            ..base.clone()
        };
        let diff_rev = GhostLlmCacheKey {
            context_rev: 2,
            ..base.clone()
        };
        let diff_model = GhostLlmCacheKey {
            model_id: "m2".to_string(),
            ..base.clone()
        };
        assert_ne!(base, diff_query);
        assert_ne!(base, diff_cwd);
        assert_ne!(base, diff_rev);
        assert_ne!(base, diff_model);
        assert_eq!(base, base.clone());
    }

    // -- context digest parsing (unchanged) ------------------------------

    #[test]
    fn context_digest_extracts_project_tasks_and_topics_without_source_filenames() {
        let digest = parse_context_digest(
            Some("fallback-project"),
            "- Use ./scripts/agentic/agent-cargo.sh for cargo checks.\n- Run ./scripts/agentic/agent-cargo.sh test --lib ghost.\n- Keep edits narrowly scoped.",
            "# Script Kit GPUI\n\nA macOS launcher built in Rust on GPUI.\n\n## Ghost Text\nInline gray completion for the launcher.\n\n## Agent Chat",
        );
        assert_eq!(digest.project_name.as_deref(), Some("Script Kit GPUI"));
        assert!(digest
            .task_phrases
            .iter()
            .any(|p| p.contains("agent-cargo")));
        assert!(digest.topic_keywords.iter().any(|t| t == "ghost text"));
        let debug = format!("{digest:?}");
        assert!(!debug.contains("AGENTS.md"));
        assert!(!debug.contains("README.md"));
        assert!(!debug.contains("./scripts/agentic"));
    }

    #[test]
    fn ghost_context_cache_reuses_context_until_doc_metadata_changes() {
        let mut dir = std::env::temp_dir();
        dir.push(format!(
            "script-kit-ghost-cache-test-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system time should be after epoch")
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).expect("create temp ghost cache dir");
        std::fs::write(dir.join("README.md"), "# Alpha\n\n## Ghost Text")
            .expect("write initial README.md");
        let mut cache = GhostContextCache::default();
        let (first, first_rev) = cache.context_for_cwd(&dir);
        let (second, second_rev) = cache.context_for_cwd(&dir);
        assert_eq!(first, second);
        assert_eq!(first_rev, second_rev);
        assert_eq!(cache.len(), 1);
        std::fs::write(
            dir.join("README.md"),
            "# Beta Project\n\n## Ghost Text\n\nExtra content to change len.",
        )
        .expect("update README.md");
        let (third, third_rev) = cache.context_for_cwd(&dir);
        assert_ne!(first_rev, third_rev);
        assert_eq!(third.digest.project_name.as_deref(), Some("Beta Project"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn from_cwd_builds_sanitized_llm_excerpt() {
        let mut dir = std::env::temp_dir();
        dir.push(format!(
            "script-kit-ghost-excerpt-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system time should be after epoch")
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        std::fs::write(
            dir.join("AGENTS.md"),
            "Use the agent cargo wrapper.\nKeep edits scoped.",
        )
        .expect("write AGENTS.md");
        std::fs::write(
            dir.join("README.md"),
            "# Demo\n\nVerify ghost text rendering.",
        )
        .expect("write README.md");
        let context = GhostContext::from_cwd(&dir);
        let excerpt = context.llm_excerpt.expect("excerpt built");
        assert!(!excerpt.contains("AGENTS.md"));
        assert!(!excerpt.contains("README.md"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    // -- local (on-device) single-string prompt --------------------------

    #[test]
    fn local_ghost_prompt_includes_partial_query_digest_and_excerpt() {
        let context = always_infer_context();
        let prompt = build_local_ghost_prompt("fix the auth", &context);
        assert!(prompt.contains("fix the auth"), "missing partial query");
        assert!(prompt.contains("Script Kit GPUI"), "missing project name");
        assert!(
            prompt.contains("run agent-cargo test"),
            "missing task phrase"
        );
        assert!(prompt.contains("ghost text"), "missing topic keyword");
        assert!(prompt.contains("Rust checks"), "missing excerpt");
        assert!(prompt.contains("output only the text that comes next"));
        // The caret prefix is the last payload so the model continues it.
        assert!(prompt.trim_end().ends_with("fix the auth"));
    }

    #[test]
    fn local_ghost_prompt_keeps_single_string_contract() {
        let context = GhostContext::default();
        let prompt = build_local_ghost_prompt("When is", &context);
        assert!(prompt.starts_with("Task:"));
        assert!(prompt.contains("Continue the partial input below"));
        // The caret prefix must be the very last payload so the model continues it.
        assert!(prompt.trim_end().ends_with("When is"));
    }
}
