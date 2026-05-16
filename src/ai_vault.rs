#![allow(dead_code)]

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::process::Output;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

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
    options.enabled && query.trim().chars().count() >= options.min_query_chars
}

pub(crate) fn search_root_ai_vault_direct(
    query: &str,
    options: RootAiVaultSectionOptions,
) -> Vec<AiVaultHit> {
    let (mut hits, fixture_backed) = match load_fixture_hits() {
        Ok(Some(hits)) => (hits, true),
        Ok(None) => {
            let hits = search_cmux_vault(query, options.clone()).unwrap_or_else(|_error| {
                tracing::warn!(
                    target: "script_kit::ai_vault",
                    event = "ai_vault_cmux_search_unavailable",
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
