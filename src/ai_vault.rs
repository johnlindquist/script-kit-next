use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

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
    pub provider_display_name: String,
    pub session_id: String,
    pub source_kind: Option<String>,
    pub safe_title: String,
    pub workspace_path: Option<String>,
    pub model: Option<String>,
    pub modified_at: Option<String>,
    pub matched_field: AiVaultMatchedField,
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
    options: RootAiVaultSectionOptions,
) -> bool {
    options.enabled && query.trim().chars().count() >= options.min_query_chars
}

pub(crate) fn search_root_ai_vault_direct(
    query: &str,
    options: RootAiVaultSectionOptions,
) -> Vec<AiVaultHit> {
    let mut hits = match load_fixture_hits() {
        Ok(Some(hits)) => hits,
        Ok(None) => search_cmux_vault(query, options.clone()).unwrap_or_else(|error| {
            tracing::warn!(
                target: "script_kit::ai_vault",
                event = "ai_vault_cmux_search_failed",
                error = %error,
                "AI Vault search failed"
            );
            Vec::new()
        }),
        Err(error) => {
            tracing::warn!(
                target: "script_kit::ai_vault",
                event = "ai_vault_fixture_failed",
                error = %error,
                "AI Vault fixture search failed"
            );
            Vec::new()
        }
    };

    let query = query.trim().to_ascii_lowercase();
    if !query.is_empty() {
        hits.retain(|hit| hit_matches_query(hit, &query, options.search_content));
    }
    for hit in &mut hits {
        normalize_hit(hit);
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
    match command.output() {
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
            error: Some(String::from_utf8_lossy(&output.stderr).trim().to_string()),
        },
        Err(error) => AiVaultResumeReceipt {
            status: "error".to_string(),
            provider: hit.provider.clone(),
            session_id: hit.session_id.clone(),
            terminal_routing: terminal_routing_value(terminal_routing).to_string(),
            terminal_target_id: None,
            error: Some(error.to_string()),
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
    let output = command
        .arg("ai-vault")
        .arg("reveal")
        .arg("--json")
        .arg(request.to_string())
        .output();
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
            error: Some(String::from_utf8_lossy(&output.stderr).trim().to_string()),
        },
        Err(error) => AiVaultResumeReceipt {
            status: "error".to_string(),
            provider: hit.provider.clone(),
            session_id: hit.session_id.clone(),
            terminal_routing: "reveal".to_string(),
            terminal_target_id: None,
            error: Some(error.to_string()),
        },
    }
}

fn search_cmux_vault(query: &str, options: RootAiVaultSectionOptions) -> Result<Vec<AiVaultHit>> {
    let Some(mut command) = cmux_command() else {
        return Ok(Vec::new());
    };
    let providers = options
        .providers
        .iter()
        .map(|provider| provider.cmux_id())
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
    let output = command.output().context("run cmux ai-vault search")?;
    if !output.status.success() {
        return Ok(Vec::new());
    }
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct SearchResponse {
        hits: Vec<AiVaultHit>,
    }
    Ok(serde_json::from_slice::<SearchResponse>(&output.stdout)
        .map(|response| response.hits)
        .unwrap_or_default())
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
