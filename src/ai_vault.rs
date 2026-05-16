#![allow(dead_code)]

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::Output;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub(crate) struct RootAiVaultSectionOptions {
    pub enabled: bool,
    pub max_results: usize,
    pub min_query_chars: usize,
    pub providers: Vec<crate::config::AiVaultProvider>,
    pub cache_ttl_ms: u64,
    pub search_content: bool,
}

impl Default for RootAiVaultSectionOptions {
    fn default() -> Self {
        Self {
            enabled: false,
            max_results: crate::config::defaults::DEFAULT_UNIFIED_SEARCH_AI_VAULT_MAX_RESULTS,
            min_query_chars:
                crate::config::defaults::DEFAULT_UNIFIED_SEARCH_AI_VAULT_MIN_QUERY_CHARS,
            providers: crate::config::AiVaultProvider::default_root_providers(),
            cache_ttl_ms: crate::config::defaults::DEFAULT_UNIFIED_SEARCH_AI_VAULT_CACHE_TTL_MS,
            search_content: crate::config::defaults::DEFAULT_UNIFIED_SEARCH_AI_VAULT_SEARCH_CONTENT,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum AiVaultMatchedField {
    Title,
    Transcript,
    Model,
    Workspace,
    Provider,
    Recent,
}

impl Default for AiVaultMatchedField {
    fn default() -> Self {
        Self::Recent
    }
}

impl AiVaultMatchedField {
    pub(crate) fn label(&self) -> &'static str {
        match self {
            Self::Title => "Matched title",
            Self::Transcript => "Matched transcript",
            Self::Model => "Matched model",
            Self::Workspace => "Matched workspace",
            Self::Provider => "Matched provider",
            Self::Recent => "Recent session",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AiVaultHit {
    pub provider: String,
    #[serde(default)]
    pub provider_display_name: String,
    pub session_id: String,
    pub source_kind: Option<String>,
    #[serde(default)]
    pub safe_title: String,
    pub workspace_path: Option<String>,
    pub model: Option<String>,
    pub modified_at: Option<String>,
    #[serde(default)]
    pub matched_field: AiVaultMatchedField,
    #[serde(default)]
    pub stable_key: String,
    #[serde(default)]
    pub score: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum AiVaultTerminalRouting {
    UserPreferred,
    NewTerminal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AiVaultResumeReceipt {
    pub status: String,
    pub provider: String,
    pub session_id: String,
    pub terminal_routing: String,
    pub terminal_target_id: Option<String>,
    pub error: Option<String>,
}

pub(crate) fn root_ai_vault_query_is_eligible(
    query: &str,
    options: &RootAiVaultSectionOptions,
) -> bool {
    let query = query.trim();
    options.enabled && (query.is_empty() || query.chars().count() >= options.min_query_chars)
}

pub(crate) fn search_root_ai_vault_direct(
    query: &str,
    options: RootAiVaultSectionOptions,
) -> Vec<AiVaultHit> {
    let (mut hits, fixture_backed) = match load_fixture_hits() {
        Ok(Some(hits)) => (hits, true),
        Ok(None) => {
            let hits = search_local_vault(query, options.clone()).unwrap_or_else(|_error| {
                tracing::warn!(
                    target: "script_kit::ai_vault",
                    event = "ai_vault_local_search_unavailable",
                    error_kind = "other",
                    "AI Vault search failed"
                );
                Vec::new()
            });
            (hits, false)
        }
        Err(_error) => {
            tracing::warn!(
                target: "script_kit::ai_vault",
                event = "ai_vault_fixture_unavailable",
                error_kind = "other",
                "AI Vault fixture search failed"
            );
            (Vec::new(), true)
        }
    };

    for hit in &mut hits {
        normalize_hit(hit);
    }

    let normalized_query = query.trim().to_ascii_lowercase();
    if fixture_backed && !normalized_query.is_empty() {
        hits.retain(|hit| hit_matches_query(hit, &normalized_query, options.search_content));
    }
    hits.truncate(options.max_results);
    hits
}

pub(crate) fn resume_vault_session(
    hit: &AiVaultHit,
    terminal_routing: AiVaultTerminalRouting,
) -> AiVaultResumeReceipt {
    let request = serde_json::json!({
        "type": "aiVault.resume.v1",
        "provider": hit.provider,
        "sessionId": hit.session_id,
        "sourceKind": hit.source_kind,
        "workspacePath": hit.workspace_path,
        "terminalRouting": terminal_routing_value(terminal_routing),
    });

    let Some(mut command) = cmux_command() else {
        return AiVaultResumeReceipt {
            status: "unavailable".to_string(),
            provider: hit.provider.clone(),
            session_id: hit.session_id.clone(),
            terminal_routing: terminal_routing_value(terminal_routing).to_string(),
            terminal_target_id: None,
            error: Some("cmux command not configured".to_string()),
        };
    };

    command
        .arg("ai-vault")
        .arg("resume")
        .arg("--json")
        .arg(request.to_string());
    match output_with_timeout(command, Duration::from_millis(5_000)) {
        Ok(output) if output.status.success() => {
            serde_json::from_slice::<AiVaultResumeReceipt>(&output.stdout)
                .unwrap_or_else(|_| launched_receipt(hit, terminal_routing, "launched"))
        }
        Ok(output) => AiVaultResumeReceipt {
            status: "error".to_string(),
            provider: hit.provider.clone(),
            session_id: hit.session_id.clone(),
            terminal_routing: terminal_routing_value(terminal_routing).to_string(),
            terminal_target_id: None,
            error: Some(cmux_failure_message("resume", output.status.code())),
        },
        Err(_error) => AiVaultResumeReceipt {
            status: "error".to_string(),
            provider: hit.provider.clone(),
            session_id: hit.session_id.clone(),
            terminal_routing: terminal_routing_value(terminal_routing).to_string(),
            terminal_target_id: None,
            error: Some(cmux_failure_message("resume", None)),
        },
    }
}

pub(crate) fn reveal_vault_session(hit: &AiVaultHit) -> AiVaultResumeReceipt {
    let Some(mut command) = cmux_command() else {
        return AiVaultResumeReceipt {
            status: "error".to_string(),
            provider: hit.provider.clone(),
            session_id: hit.session_id.clone(),
            terminal_routing: "reveal".to_string(),
            terminal_target_id: None,
            error: Some("cmux command unavailable".to_string()),
        };
    };

    let request = serde_json::json!({
        "type": "aiVault.reveal.v1",
        "provider": hit.provider,
        "sessionId": hit.session_id,
        "sourceKind": hit.source_kind,
        "workspacePath": hit.workspace_path,
    });
    command
        .arg("ai-vault")
        .arg("reveal")
        .arg("--json")
        .arg(request.to_string());
    let output = output_with_timeout(command, Duration::from_millis(5_000));
    match output {
        Ok(output) if output.status.success() => {
            serde_json::from_slice::<AiVaultResumeReceipt>(&output.stdout).unwrap_or_else(|_| {
                launched_receipt(hit, AiVaultTerminalRouting::UserPreferred, "opened")
            })
        }
        Ok(output) => AiVaultResumeReceipt {
            status: "error".to_string(),
            provider: hit.provider.clone(),
            session_id: hit.session_id.clone(),
            terminal_routing: "reveal".to_string(),
            terminal_target_id: None,
            error: Some(cmux_failure_message("reveal", output.status.code())),
        },
        Err(_error) => AiVaultResumeReceipt {
            status: "error".to_string(),
            provider: hit.provider.clone(),
            session_id: hit.session_id.clone(),
            terminal_routing: "reveal".to_string(),
            terminal_target_id: None,
            error: Some(cmux_failure_message("reveal", None)),
        },
    }
}

fn search_cmux_vault(query: &str, options: RootAiVaultSectionOptions) -> Result<Vec<AiVaultHit>> {
    let cache_key = ai_vault_cache_key(query, &options);
    if let Some(hits) = ai_vault_cache_get(&cache_key, options.cache_ttl_ms) {
        return Ok(hits);
    }

    let Some(mut command) = cmux_command() else {
        return Ok(Vec::new());
    };
    let providers = options
        .providers
        .iter()
        .map(crate::config::AiVaultProvider::cmux_id)
        .collect::<Vec<_>>();
    let request = serde_json::json!({
        "type": "aiVault.search.v1",
        "query": query,
        "limit": options.max_results,
        "offset": 0,
        "providers": providers,
        "cwdFilter": serde_json::Value::Null,
        "includeContent": false,
    });
    command
        .arg("ai-vault")
        .arg("search")
        .arg("--json")
        .arg(request.to_string());
    let output = output_with_timeout(
        command,
        Duration::from_millis(options.cache_ttl_ms.min(5_000)),
    )
    .context("run cmux ai-vault search")?;
    if !output.status.success() {
        return Ok(Vec::new());
    }
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct SearchResponse {
        hits: Vec<AiVaultHit>,
    }
    match serde_json::from_slice::<SearchResponse>(&output.stdout) {
        Ok(response) => {
            ai_vault_cache_put(cache_key, response.hits.clone());
            Ok(response.hits)
        }
        Err(_error) => {
            tracing::warn!(
                target: "script_kit::ai_vault",
                event = "ai_vault_cmux_response_parse_failed",
                error_kind = "json",
                "AI Vault cmux response parse failed"
            );
            Ok(Vec::new())
        }
    }
}

fn search_local_vault(query: &str, options: RootAiVaultSectionOptions) -> Result<Vec<AiVaultHit>> {
    let cache_key = ai_vault_cache_key(query, &options);
    if let Some(hits) = ai_vault_cache_get(&cache_key, options.cache_ttl_ms) {
        return Ok(hits);
    }

    let normalized_query = query.trim().to_ascii_lowercase();
    let mut hits = Vec::new();
    hits.extend(read_claude_vault_hits(&normalized_query)?);
    hits.extend(read_codex_vault_hits(&normalized_query)?);
    hits.sort_by(|a, b| {
        b.modified_at
            .cmp(&a.modified_at)
            .then_with(|| a.stable_key.cmp(&b.stable_key))
    });
    hits.truncate(options.max_results);
    ai_vault_cache_put(cache_key, hits.clone());
    Ok(hits)
}

fn read_claude_vault_hits(query: &str) -> Result<Vec<AiVaultHit>> {
    let root = home_dir().join(".claude").join("projects");
    let mut files = Vec::new();
    collect_jsonl_files(&root, &mut files);
    files.sort_by(|a, b| b.1.cmp(&a.1));
    files.truncate(300);

    let mut hits = Vec::new();
    for (path, mtime) in files {
        if let Some(hit) = read_claude_vault_hit(&path, mtime, query) {
            hits.push(hit);
        }
    }
    Ok(hits)
}

fn read_claude_vault_hit(path: &Path, mtime: SystemTime, query: &str) -> Option<AiVaultHit> {
    let file = File::open(path).ok()?;
    let reader = BufReader::new(file);
    let session_id = path.file_stem()?.to_string_lossy().to_string();
    let mut title = None;
    let mut cwd = None;
    let mut model = None;
    let mut newest = mtime;

    for line in reader.lines().map_while(Result::ok).take(1000) {
        let event: serde_json::Value = serde_json::from_str(&line).ok()?;
        if let Some(timestamp) = event.get("timestamp").and_then(|value| value.as_str()) {
            if let Some(parsed) = parse_timestamp(timestamp) {
                newest = newest.max(parsed);
            }
        }
        if cwd.is_none() {
            cwd = event
                .get("cwd")
                .and_then(|value| value.as_str())
                .filter(|value| !value.trim().is_empty())
                .map(ToString::to_string);
        }
        if model.is_none() {
            model = event
                .pointer("/message/model")
                .and_then(|value| value.as_str())
                .filter(|value| !value.trim().is_empty())
                .map(ToString::to_string);
        }
        if title.is_some() || event.get("isMeta").and_then(|value| value.as_bool()) == Some(true) {
            continue;
        }
        let role = event
            .pointer("/message/role")
            .or_else(|| event.get("type"))
            .and_then(|value| value.as_str());
        if role != Some("user") {
            continue;
        }
        let content = event
            .pointer("/message/content")
            .or_else(|| event.get("content"));
        title = content.and_then(text_from_content_for_title);
    }

    let safe_title =
        title.unwrap_or_else(|| format!("Claude Code session {}", short_id(&session_id)));
    let matched_field = matched_field(
        query,
        &safe_title,
        cwd.as_deref(),
        Some("Claude Code"),
        &session_id,
    );
    if !query.is_empty() && matched_field.is_none() {
        return None;
    }

    Some(AiVaultHit {
        provider: "claude".to_string(),
        provider_display_name: "Claude Code".to_string(),
        session_id: session_id.clone(),
        source_kind: Some("cli".to_string()),
        safe_title,
        workspace_path: cwd,
        model,
        modified_at: Some(system_time_to_rfc3339(newest)),
        matched_field: matched_field.unwrap_or(AiVaultMatchedField::Recent),
        stable_key: format!("ai-vault/claude/cli/{session_id}"),
        score: 0,
    })
}

fn read_codex_vault_hits(query: &str) -> Result<Vec<AiVaultHit>> {
    let index_path = home_dir().join(".codex").join("session_index.jsonl");
    let Ok(file) = File::open(index_path) else {
        return Ok(Vec::new());
    };
    let reader = BufReader::new(file);
    let mut hits = Vec::new();
    for line in reader.lines().map_while(Result::ok) {
        let Ok(row) = serde_json::from_str::<serde_json::Value>(&line) else {
            continue;
        };
        let Some(session_id) = row
            .get("id")
            .and_then(|value| value.as_str())
            .filter(|value| !value.trim().is_empty())
        else {
            continue;
        };
        let safe_title = row
            .get("thread_name")
            .and_then(|value| value.as_str())
            .and_then(normalize_title)
            .unwrap_or_else(|| format!("Codex session {}", short_id(session_id)));
        let cwd = row
            .get("cwd")
            .and_then(|value| value.as_str())
            .filter(|value| !value.trim().is_empty())
            .map(ToString::to_string);
        let matched_field = matched_field(
            query,
            &safe_title,
            cwd.as_deref(),
            Some("Codex"),
            session_id,
        );
        if !query.is_empty() && matched_field.is_none() {
            continue;
        }
        let modified = row
            .get("updated_at")
            .and_then(|value| value.as_str())
            .and_then(parse_timestamp)
            .unwrap_or(UNIX_EPOCH);
        hits.push(AiVaultHit {
            provider: "codex".to_string(),
            provider_display_name: "Codex".to_string(),
            session_id: session_id.to_string(),
            source_kind: Some("cli".to_string()),
            safe_title,
            workspace_path: cwd,
            model: row
                .get("model")
                .and_then(|value| value.as_str())
                .map(ToString::to_string),
            modified_at: Some(system_time_to_rfc3339(modified)),
            matched_field: matched_field.unwrap_or(AiVaultMatchedField::Recent),
            stable_key: format!("ai-vault/codex/cli/{session_id}"),
            score: 0,
        });
    }
    hits.truncate(300);
    Ok(hits)
}

fn collect_jsonl_files(root: &Path, files: &mut Vec<(PathBuf, SystemTime)>) {
    let Ok(entries) = std::fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let Ok(metadata) = entry.metadata() else {
            continue;
        };
        if metadata.is_dir() {
            collect_jsonl_files(&path, files);
        } else if metadata.is_file()
            && path.extension().and_then(|ext| ext.to_str()) == Some("jsonl")
        {
            files.push((path, metadata.modified().unwrap_or(UNIX_EPOCH)));
        }
    }
}

fn text_from_content_for_title(content: &serde_json::Value) -> Option<String> {
    let mut fragments = Vec::new();
    collect_text_fragments(content, &mut fragments);
    normalize_title(&fragments.join("\n\n"))
}

fn collect_text_fragments(value: &serde_json::Value, fragments: &mut Vec<String>) {
    match value {
        serde_json::Value::String(text) => fragments.push(text.clone()),
        serde_json::Value::Array(items) => {
            for item in items {
                collect_text_fragments(item, fragments);
            }
        }
        serde_json::Value::Object(object) => {
            let value_type = object.get("type").and_then(|value| value.as_str());
            if matches!(
                value_type,
                Some("tool_use" | "tool_result" | "function_call" | "function_call_output")
            ) {
                return;
            }
            if let Some(text) = object.get("text").and_then(|value| value.as_str()) {
                fragments.push(text.to_string());
                return;
            }
            for key in ["content", "message"] {
                if let Some(child) = object.get(key) {
                    collect_text_fragments(child, fragments);
                    if !fragments.is_empty() {
                        return;
                    }
                }
            }
        }
        _ => {}
    }
}

fn normalize_title(raw: &str) -> Option<String> {
    let mut value = raw.replace('\r', " ").replace('\n', " ").replace('\t', " ");
    for tag in [
        "system-reminder",
        "command-name",
        "command-message",
        "command-args",
    ] {
        value = strip_tag(&value, tag);
    }
    value = value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .replace("[Image #1]", "")
        .trim()
        .to_string();
    if value.is_empty() {
        None
    } else if value.chars().count() > 160 {
        Some(format!(
            "{}...",
            value.chars().take(157).collect::<String>().trim_end()
        ))
    } else {
        Some(value)
    }
}

fn strip_tag(input: &str, tag: &str) -> String {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    input.replace(&open, " ").replace(&close, " ")
}

fn matched_field(
    query: &str,
    title: &str,
    workspace: Option<&str>,
    provider: Option<&str>,
    session_id: &str,
) -> Option<AiVaultMatchedField> {
    if query.is_empty() {
        return Some(AiVaultMatchedField::Recent);
    }
    let query = query.to_ascii_lowercase();
    if title.to_ascii_lowercase().contains(&query) {
        return Some(AiVaultMatchedField::Title);
    }
    if workspace
        .unwrap_or("")
        .to_ascii_lowercase()
        .contains(&query)
    {
        return Some(AiVaultMatchedField::Workspace);
    }
    if provider.unwrap_or("").to_ascii_lowercase().contains(&query) {
        return Some(AiVaultMatchedField::Provider);
    }
    if session_id.to_ascii_lowercase().contains(&query) {
        return Some(AiVaultMatchedField::Recent);
    }
    None
}

fn parse_timestamp(timestamp: &str) -> Option<SystemTime> {
    chrono::DateTime::parse_from_rfc3339(timestamp)
        .ok()
        .map(|value| {
            UNIX_EPOCH
                + Duration::from_secs(value.timestamp().max(0) as u64)
                + Duration::from_nanos(value.timestamp_subsec_nanos() as u64)
        })
}

fn system_time_to_rfc3339(time: SystemTime) -> String {
    let datetime: chrono::DateTime<chrono::Utc> = time.into();
    datetime.to_rfc3339()
}

fn home_dir() -> PathBuf {
    dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
}

fn short_id(session_id: &str) -> String {
    session_id.chars().take(12).collect()
}

fn load_fixture_hits() -> Result<Option<Vec<AiVaultHit>>> {
    let Ok(value) = std::env::var("SCRIPT_KIT_AI_VAULT_TEST_PROVIDER") else {
        return Ok(None);
    };
    let raw = if Path::new(&value).exists() {
        std::fs::read_to_string(value).context("read SCRIPT_KIT_AI_VAULT_TEST_PROVIDER fixture")?
    } else {
        value
    };
    Ok(Some(
        serde_json::from_str::<Vec<AiVaultHit>>(&raw)
            .context("parse SCRIPT_KIT_AI_VAULT_TEST_PROVIDER fixture")?,
    ))
}

fn hit_matches_query(hit: &AiVaultHit, query: &str, search_content: bool) -> bool {
    let mut fields = vec![
        hit.safe_title.as_str(),
        hit.provider.as_str(),
        hit.provider_display_name.as_str(),
        hit.session_id.as_str(),
    ];
    if let Some(model) = hit.model.as_deref() {
        fields.push(model);
    }
    if let Some(workspace) = hit.workspace_path.as_deref() {
        fields.push(workspace);
    }
    if search_content && matches!(hit.matched_field, AiVaultMatchedField::Transcript) {
        fields.push(hit.matched_field.label());
    }
    fields
        .into_iter()
        .any(|field| field.to_ascii_lowercase().contains(query))
}

fn normalize_hit(hit: &mut AiVaultHit) {
    if hit.provider_display_name.trim().is_empty() {
        hit.provider_display_name = human_provider(&hit.provider).to_string();
    }
    if hit.safe_title.trim().is_empty() {
        hit.safe_title = generic_title(hit);
    }
    if hit.stable_key.trim().is_empty() {
        let source = hit.source_kind.as_deref().unwrap_or("session");
        hit.stable_key = format!("ai-vault/{}/{}/{}", hit.provider, source, hit.session_id);
    }
}

fn generic_title(hit: &AiVaultHit) -> String {
    let short = hit.session_id.chars().take(8).collect::<String>();
    format!("{} session {}", human_provider(&hit.provider), short)
}

fn human_provider(provider: &str) -> &str {
    match provider {
        "hermes-agent" | "hermesAgent" => "Hermes Agent",
        "rovodev" | "rovoDev" => "Rovo Dev",
        _ => "AI Vault",
    }
}

fn cmux_command() -> Option<std::process::Command> {
    let binary = std::env::var("SCRIPT_KIT_CMUX_COMMAND")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "cmux".to_string());
    Some(std::process::Command::new(binary))
}

fn output_with_timeout(
    mut command: std::process::Command,
    timeout: Duration,
) -> std::io::Result<Output> {
    let mut child = command.spawn()?;
    let started = Instant::now();
    loop {
        if child.try_wait()?.is_some() {
            return child.wait_with_output();
        }
        if started.elapsed() >= timeout {
            let _ = child.kill();
            let _ = child.wait();
            return Err(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "cmux ai-vault command timed out",
            ));
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

fn cmux_failure_message(action: &str, code: Option<i32>) -> String {
    match code {
        Some(code) => format!("cmux {action} failed with status {code}"),
        None => format!("cmux {action} failed"),
    }
}

fn ai_vault_cache_key(query: &str, options: &RootAiVaultSectionOptions) -> String {
    let providers = options
        .providers
        .iter()
        .map(crate::config::AiVaultProvider::cmux_id)
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{}\u{1f}{}\u{1f}{}\u{1f}{}",
        query.trim(),
        options.max_results,
        options.search_content,
        providers
    )
}

fn ai_vault_cache() -> &'static Mutex<HashMap<String, (Instant, Vec<AiVaultHit>)>> {
    static CACHE: OnceLock<Mutex<HashMap<String, (Instant, Vec<AiVaultHit>)>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn ai_vault_cache_get(key: &str, ttl_ms: u64) -> Option<Vec<AiVaultHit>> {
    let ttl = Duration::from_millis(ttl_ms);
    let cache = ai_vault_cache().lock().ok()?;
    let (created_at, hits) = cache.get(key)?;
    (created_at.elapsed() <= ttl).then(|| hits.clone())
}

fn ai_vault_cache_put(key: String, hits: Vec<AiVaultHit>) {
    if let Ok(mut cache) = ai_vault_cache().lock() {
        cache.insert(key, (Instant::now(), hits));
    }
}

fn terminal_routing_value(routing: AiVaultTerminalRouting) -> &'static str {
    match routing {
        AiVaultTerminalRouting::UserPreferred => "userPreferred",
        AiVaultTerminalRouting::NewTerminal => "newTerminal",
    }
}

fn launched_receipt(
    hit: &AiVaultHit,
    terminal_routing: AiVaultTerminalRouting,
    status: &str,
) -> AiVaultResumeReceipt {
    AiVaultResumeReceipt {
        status: status.to_string(),
        provider: hit.provider.clone(),
        session_id: hit.session_id.clone(),
        terminal_routing: terminal_routing_value(terminal_routing).to_string(),
        terminal_target_id: None,
        error: None,
    }
}
