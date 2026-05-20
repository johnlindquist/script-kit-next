use crate::mcp_clipboard_tools;
use crate::mcp_config_tools;
use crate::mcp_kit_tools::{ToolContent, ToolDefinition, ToolResult};
use crate::mcp_notes_tools::{
    self, McpNotesMutationBridge, NotesMutationError, NotesMutationErrorCode, NotesMutationRequest,
    NotesMutationResult, SharedNotesMutationBridge,
};
use crate::mcp_scripts_tools;
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::oneshot;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskClass {
    Safe,
    StateMutating,
    Destructive,
    ExternalProcess,
}

#[derive(Debug, Clone)]
pub struct MutationToolMeta {
    pub name: &'static str,
    pub description: &'static str,
    pub risk: RiskClass,
    pub required_scope: &'static str,
}

#[derive(Clone)]
pub struct MutationContext {
    pub trace_id: String,
    pub token_scopes: Vec<String>,
    pub notes_bridge: Option<SharedNotesMutationBridge>,
}

#[async_trait]
pub trait DynMutationTool: Send + Sync {
    fn meta(&self) -> MutationToolMeta;
    fn definition(&self) -> ToolDefinition;
    async fn call(&self, args: Value, ctx: &MutationContext) -> ToolResult;
}

#[derive(Default)]
pub struct MutationRegistry {
    tools: HashMap<&'static str, Arc<dyn DynMutationTool>>,
}

impl MutationRegistry {
    pub fn register<T>(&mut self, tool: T)
    where
        T: DynMutationTool + 'static,
    {
        self.tools.insert(tool.meta().name, Arc::new(tool));
    }

    pub fn definitions(&self) -> Vec<ToolDefinition> {
        let mut tools: Vec<_> = self.tools.values().map(|tool| tool.definition()).collect();
        tools.sort_by(|a, b| a.name.cmp(&b.name));
        tools
    }

    pub fn contains(&self, name: &str) -> bool {
        self.tools.contains_key(name)
    }

    pub async fn call(&self, name: &str, args: Value, ctx: &MutationContext) -> Option<ToolResult> {
        let tool = self.tools.get(name)?;
        let meta = tool.meta();
        if !scope_allows(&ctx.token_scopes, meta.required_scope) {
            let result = scope_denied_tool_result(meta.name, meta.required_scope);
            append_audit_event(&McpAuditEvent {
                ts: chrono::Utc::now(),
                trace_id: ctx.trace_id.clone(),
                method: "tools/call".to_string(),
                tool: Some(meta.name.to_string()),
                resource_uri: None,
                action: meta.name.to_string(),
                risk: format!("{:?}", meta.risk),
                success: false,
                error_code: Some("scope_denied".to_string()),
                target_id: None,
            });
            return Some(result);
        }

        let result = tool.call(args, ctx).await;
        let success = result.is_error != Some(true);
        let error_code = if success {
            None
        } else {
            tool_result_error_code(&result).or_else(|| Some("tool_error".to_string()))
        };
        append_audit_event(&McpAuditEvent {
            ts: chrono::Utc::now(),
            trace_id: ctx.trace_id.clone(),
            method: "tools/call".to_string(),
            tool: Some(meta.name.to_string()),
            resource_uri: None,
            action: meta.name.to_string(),
            risk: format!("{:?}", meta.risk),
            success,
            error_code,
            target_id: None,
        });
        Some(result)
    }
}

pub fn build_default_mutation_registry() -> MutationRegistry {
    let mut registry = MutationRegistry::default();
    registry.register(NotesCreateTool);
    registry.register(NotesUpdateTool);
    registry.register(NotesDeleteTool);
    registry.register(ScriptsCreateTool);
    registry.register(ScriptsUpdateTool);
    registry.register(ScriptsDeleteTool);
    registry.register(ScriptsRunTool);
    registry.register(ClipboardCopyTool);
    registry.register(ClipboardPinTool);
    registry.register(ClipboardUnpinTool);
    registry.register(ClipboardDeleteTool);
    registry.register(ClipboardClearUnpinnedTool);
    registry.register(ConfigGetTool);
    registry.register(ConfigListTool);
    registry.register(ConfigValidateTool);
    registry.register(ConfigValidateChangeTool);
    registry.register(ConfigSetTool);
    registry.register(ConfigResetTool);
    registry.register(ConfigSetCommandShortcutTool);
    registry.register(ConfigRemoveCommandShortcutTool);
    registry
}

#[derive(Clone)]
pub struct GpuiNotesMcpBridge {
    tx: async_channel::Sender<NotesMcpCommand>,
    timeout: Duration,
}

impl GpuiNotesMcpBridge {
    pub fn new(tx: async_channel::Sender<NotesMcpCommand>, timeout: Duration) -> Self {
        Self { tx, timeout }
    }

    pub fn with_default_timeout(tx: async_channel::Sender<NotesMcpCommand>) -> Self {
        Self::new(tx, Duration::from_secs(10))
    }
}

pub struct NotesMcpCommand {
    pub request: NotesMutationRequest,
    pub response_tx: oneshot::Sender<Result<NotesMutationResult, NotesMutationError>>,
}

#[async_trait]
impl McpNotesMutationBridge for GpuiNotesMcpBridge {
    async fn mutate_notes(
        &self,
        request: NotesMutationRequest,
    ) -> Result<NotesMutationResult, NotesMutationError> {
        let (response_tx, response_rx) = oneshot::channel();
        let deadline = tokio::time::Instant::now() + self.timeout;
        let command = NotesMcpCommand {
            request,
            response_tx,
        };

        match tokio::time::timeout_at(deadline, self.tx.send(command)).await {
            Ok(Ok(())) => {}
            Ok(Err(_)) => {
                return Err(NotesMutationError::new(
                    NotesMutationErrorCode::MissingRuntime,
                    "Notes mutation runtime is disconnected",
                ));
            }
            Err(_) => {
                return Err(NotesMutationError::new(
                    NotesMutationErrorCode::Internal,
                    "Timed out enqueueing GPUI notes mutation",
                ));
            }
        }

        match tokio::time::timeout_at(deadline, response_rx).await {
            Ok(Ok(result)) => result,
            Ok(Err(_)) => Err(NotesMutationError::new(
                NotesMutationErrorCode::Internal,
                "GPUI notes mutation bridge response channel closed",
            )),
            Err(_) => Err(NotesMutationError::new(
                NotesMutationErrorCode::Internal,
                "Timed out waiting for GPUI notes mutation bridge",
            )),
        }
    }
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct MutationToolError {
    code: &'static str,
    message: String,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct MutationToolEnvelope {
    ok: bool,
    action: String,
    error: MutationToolError,
}

fn scope_denied_tool_result(action: &str, required_scope: &str) -> ToolResult {
    let envelope = MutationToolEnvelope {
        ok: false,
        action: action.to_string(),
        error: MutationToolError {
            code: "scope_denied",
            message: format!("Missing required MCP scope: {required_scope}"),
        },
    };
    ToolResult {
        content: vec![ToolContent {
            content_type: "text".to_string(),
            text: serde_json::to_string(&envelope).unwrap_or_else(|error| {
                format!(
                    r#"{{"ok":false,"action":"{action}","error":{{"code":"internal","message":"Failed to serialize mutation result: {error}"}}}}"#
                )
            }),
        }],
        is_error: Some(true),
    }
}

struct NotesCreateTool;
struct NotesUpdateTool;
struct NotesDeleteTool;
struct ScriptsCreateTool;
struct ScriptsUpdateTool;
struct ScriptsDeleteTool;
struct ScriptsRunTool;
struct ClipboardCopyTool;
struct ClipboardPinTool;
struct ClipboardUnpinTool;
struct ClipboardDeleteTool;
struct ClipboardClearUnpinnedTool;
struct ConfigGetTool;
struct ConfigListTool;
struct ConfigValidateTool;
struct ConfigValidateChangeTool;
struct ConfigSetTool;
struct ConfigResetTool;
struct ConfigSetCommandShortcutTool;
struct ConfigRemoveCommandShortcutTool;

fn required_tool_definition(definitions: Vec<ToolDefinition>, name: &str) -> ToolDefinition {
    match definitions.into_iter().find(|tool| tool.name == name) {
        Some(tool) => tool,
        None => panic!("missing required tool definition: {name}"),
    }
}

#[async_trait]
impl DynMutationTool for NotesCreateTool {
    fn meta(&self) -> MutationToolMeta {
        MutationToolMeta {
            name: mcp_notes_tools::NOTES_CREATE_TOOL,
            description: "Create a Script Kit note",
            risk: RiskClass::StateMutating,
            required_scope: "notes:write",
        }
    }

    fn definition(&self) -> ToolDefinition {
        required_tool_definition(
            mcp_notes_tools::get_notes_tool_definitions(),
            mcp_notes_tools::NOTES_CREATE_TOOL,
        )
    }

    async fn call(&self, args: Value, ctx: &MutationContext) -> ToolResult {
        mcp_notes_tools::handle_notes_tool_call(
            ctx.notes_bridge.as_deref(),
            mcp_notes_tools::NOTES_CREATE_TOOL,
            args,
        )
        .await
    }
}

#[async_trait]
impl DynMutationTool for NotesUpdateTool {
    fn meta(&self) -> MutationToolMeta {
        MutationToolMeta {
            name: mcp_notes_tools::NOTES_UPDATE_TOOL,
            description: "Update a Script Kit note",
            risk: RiskClass::StateMutating,
            required_scope: "notes:write",
        }
    }

    fn definition(&self) -> ToolDefinition {
        required_tool_definition(
            mcp_notes_tools::get_notes_tool_definitions(),
            mcp_notes_tools::NOTES_UPDATE_TOOL,
        )
    }

    async fn call(&self, args: Value, ctx: &MutationContext) -> ToolResult {
        mcp_notes_tools::handle_notes_tool_call(
            ctx.notes_bridge.as_deref(),
            mcp_notes_tools::NOTES_UPDATE_TOOL,
            args,
        )
        .await
    }
}

#[async_trait]
impl DynMutationTool for NotesDeleteTool {
    fn meta(&self) -> MutationToolMeta {
        MutationToolMeta {
            name: mcp_notes_tools::NOTES_DELETE_TOOL,
            description: "Delete a Script Kit note",
            risk: RiskClass::Destructive,
            required_scope: "notes:write",
        }
    }

    fn definition(&self) -> ToolDefinition {
        required_tool_definition(
            mcp_notes_tools::get_notes_tool_definitions(),
            mcp_notes_tools::NOTES_DELETE_TOOL,
        )
    }

    async fn call(&self, args: Value, ctx: &MutationContext) -> ToolResult {
        mcp_notes_tools::handle_notes_tool_call(
            ctx.notes_bridge.as_deref(),
            mcp_notes_tools::NOTES_DELETE_TOOL,
            args,
        )
        .await
    }
}

#[async_trait]
impl DynMutationTool for ScriptsCreateTool {
    fn meta(&self) -> MutationToolMeta {
        MutationToolMeta {
            name: mcp_scripts_tools::SCRIPTS_CREATE_TOOL,
            description: "Create a Script Kit script",
            risk: RiskClass::StateMutating,
            required_scope: "scripts:write",
        }
    }

    fn definition(&self) -> ToolDefinition {
        required_tool_definition(
            mcp_scripts_tools::get_scripts_tool_definitions(),
            mcp_scripts_tools::SCRIPTS_CREATE_TOOL,
        )
    }

    async fn call(&self, args: Value, _ctx: &MutationContext) -> ToolResult {
        mcp_scripts_tools::handle_scripts_tool_call(mcp_scripts_tools::SCRIPTS_CREATE_TOOL, args)
    }
}

#[async_trait]
impl DynMutationTool for ScriptsUpdateTool {
    fn meta(&self) -> MutationToolMeta {
        MutationToolMeta {
            name: mcp_scripts_tools::SCRIPTS_UPDATE_TOOL,
            description: "Update a Script Kit script",
            risk: RiskClass::StateMutating,
            required_scope: "scripts:write",
        }
    }

    fn definition(&self) -> ToolDefinition {
        required_tool_definition(
            mcp_scripts_tools::get_scripts_tool_definitions(),
            mcp_scripts_tools::SCRIPTS_UPDATE_TOOL,
        )
    }

    async fn call(&self, args: Value, _ctx: &MutationContext) -> ToolResult {
        mcp_scripts_tools::handle_scripts_tool_call(mcp_scripts_tools::SCRIPTS_UPDATE_TOOL, args)
    }
}

#[async_trait]
impl DynMutationTool for ScriptsDeleteTool {
    fn meta(&self) -> MutationToolMeta {
        MutationToolMeta {
            name: mcp_scripts_tools::SCRIPTS_DELETE_TOOL,
            description: "Delete a Script Kit script",
            risk: RiskClass::Destructive,
            required_scope: "scripts:write",
        }
    }

    fn definition(&self) -> ToolDefinition {
        required_tool_definition(
            mcp_scripts_tools::get_scripts_tool_definitions(),
            mcp_scripts_tools::SCRIPTS_DELETE_TOOL,
        )
    }

    async fn call(&self, args: Value, _ctx: &MutationContext) -> ToolResult {
        mcp_scripts_tools::handle_scripts_tool_call(mcp_scripts_tools::SCRIPTS_DELETE_TOOL, args)
    }
}

#[async_trait]
impl DynMutationTool for ScriptsRunTool {
    fn meta(&self) -> MutationToolMeta {
        MutationToolMeta {
            name: mcp_scripts_tools::SCRIPTS_RUN_TOOL,
            description: "Run a Script Kit script",
            risk: RiskClass::ExternalProcess,
            required_scope: "scripts:run",
        }
    }

    fn definition(&self) -> ToolDefinition {
        required_tool_definition(
            mcp_scripts_tools::get_scripts_tool_definitions(),
            mcp_scripts_tools::SCRIPTS_RUN_TOOL,
        )
    }

    async fn call(&self, args: Value, _ctx: &MutationContext) -> ToolResult {
        mcp_scripts_tools::handle_scripts_tool_call(mcp_scripts_tools::SCRIPTS_RUN_TOOL, args)
    }
}

fn clipboard_tool_definition(name: &str) -> ToolDefinition {
    mcp_clipboard_tools::get_clipboard_tool_definitions()
        .into_iter()
        .find(|tool| tool.name == name)
        .unwrap_or_else(|| panic!("clipboard tool definition missing: {name}"))
}

#[async_trait]
impl DynMutationTool for ClipboardCopyTool {
    fn meta(&self) -> MutationToolMeta {
        MutationToolMeta {
            name: mcp_clipboard_tools::CLIPBOARD_COPY_TOOL,
            description: "Copy a clipboard-history entry to the system clipboard",
            risk: RiskClass::StateMutating,
            required_scope: "clipboard:write",
        }
    }

    fn definition(&self) -> ToolDefinition {
        clipboard_tool_definition(mcp_clipboard_tools::CLIPBOARD_COPY_TOOL)
    }

    async fn call(&self, args: Value, _ctx: &MutationContext) -> ToolResult {
        mcp_clipboard_tools::handle_clipboard_tool_call(
            mcp_clipboard_tools::CLIPBOARD_COPY_TOOL,
            args,
        )
    }
}

#[async_trait]
impl DynMutationTool for ClipboardPinTool {
    fn meta(&self) -> MutationToolMeta {
        MutationToolMeta {
            name: mcp_clipboard_tools::CLIPBOARD_PIN_TOOL,
            description: "Pin a clipboard-history entry",
            risk: RiskClass::StateMutating,
            required_scope: "clipboard:write",
        }
    }

    fn definition(&self) -> ToolDefinition {
        clipboard_tool_definition(mcp_clipboard_tools::CLIPBOARD_PIN_TOOL)
    }

    async fn call(&self, args: Value, _ctx: &MutationContext) -> ToolResult {
        mcp_clipboard_tools::handle_clipboard_tool_call(
            mcp_clipboard_tools::CLIPBOARD_PIN_TOOL,
            args,
        )
    }
}

#[async_trait]
impl DynMutationTool for ClipboardUnpinTool {
    fn meta(&self) -> MutationToolMeta {
        MutationToolMeta {
            name: mcp_clipboard_tools::CLIPBOARD_UNPIN_TOOL,
            description: "Unpin a clipboard-history entry",
            risk: RiskClass::StateMutating,
            required_scope: "clipboard:write",
        }
    }

    fn definition(&self) -> ToolDefinition {
        clipboard_tool_definition(mcp_clipboard_tools::CLIPBOARD_UNPIN_TOOL)
    }

    async fn call(&self, args: Value, _ctx: &MutationContext) -> ToolResult {
        mcp_clipboard_tools::handle_clipboard_tool_call(
            mcp_clipboard_tools::CLIPBOARD_UNPIN_TOOL,
            args,
        )
    }
}

#[async_trait]
impl DynMutationTool for ClipboardDeleteTool {
    fn meta(&self) -> MutationToolMeta {
        MutationToolMeta {
            name: mcp_clipboard_tools::CLIPBOARD_DELETE_TOOL,
            description: "Delete a clipboard-history entry",
            risk: RiskClass::Destructive,
            required_scope: "clipboard:write",
        }
    }

    fn definition(&self) -> ToolDefinition {
        clipboard_tool_definition(mcp_clipboard_tools::CLIPBOARD_DELETE_TOOL)
    }

    async fn call(&self, args: Value, _ctx: &MutationContext) -> ToolResult {
        mcp_clipboard_tools::handle_clipboard_tool_call(
            mcp_clipboard_tools::CLIPBOARD_DELETE_TOOL,
            args,
        )
    }
}

#[async_trait]
impl DynMutationTool for ClipboardClearUnpinnedTool {
    fn meta(&self) -> MutationToolMeta {
        MutationToolMeta {
            name: mcp_clipboard_tools::CLIPBOARD_CLEAR_UNPINNED_TOOL,
            description: "Clear unpinned clipboard-history entries",
            risk: RiskClass::Destructive,
            required_scope: "clipboard:write",
        }
    }

    fn definition(&self) -> ToolDefinition {
        clipboard_tool_definition(mcp_clipboard_tools::CLIPBOARD_CLEAR_UNPINNED_TOOL)
    }

    async fn call(&self, args: Value, _ctx: &MutationContext) -> ToolResult {
        mcp_clipboard_tools::handle_clipboard_tool_call(
            mcp_clipboard_tools::CLIPBOARD_CLEAR_UNPINNED_TOOL,
            args,
        )
    }
}

fn config_tool_definition(name: &str) -> ToolDefinition {
    mcp_config_tools::get_config_tool_definitions()
        .into_iter()
        .find(|tool| tool.name == name)
        .unwrap_or_else(|| panic!("config tool definition missing: {name}"))
}

#[async_trait]
impl DynMutationTool for ConfigGetTool {
    fn meta(&self) -> MutationToolMeta {
        MutationToolMeta {
            name: mcp_config_tools::CONFIG_GET_TOOL,
            description: "Read Script Kit config",
            risk: RiskClass::Safe,
            required_scope: "config:read",
        }
    }

    fn definition(&self) -> ToolDefinition {
        config_tool_definition(mcp_config_tools::CONFIG_GET_TOOL)
    }

    async fn call(&self, args: Value, _ctx: &MutationContext) -> ToolResult {
        mcp_config_tools::handle_config_tool_call(mcp_config_tools::CONFIG_GET_TOOL, args)
    }
}

#[async_trait]
impl DynMutationTool for ConfigListTool {
    fn meta(&self) -> MutationToolMeta {
        MutationToolMeta {
            name: mcp_config_tools::CONFIG_LIST_TOOL,
            description: "List Script Kit config keys",
            risk: RiskClass::Safe,
            required_scope: "config:read",
        }
    }

    fn definition(&self) -> ToolDefinition {
        config_tool_definition(mcp_config_tools::CONFIG_LIST_TOOL)
    }

    async fn call(&self, args: Value, _ctx: &MutationContext) -> ToolResult {
        mcp_config_tools::handle_config_tool_call(mcp_config_tools::CONFIG_LIST_TOOL, args)
    }
}

#[async_trait]
impl DynMutationTool for ConfigValidateTool {
    fn meta(&self) -> MutationToolMeta {
        MutationToolMeta {
            name: mcp_config_tools::CONFIG_VALIDATE_TOOL,
            description: "Validate Script Kit config",
            risk: RiskClass::Safe,
            required_scope: "config:read",
        }
    }

    fn definition(&self) -> ToolDefinition {
        config_tool_definition(mcp_config_tools::CONFIG_VALIDATE_TOOL)
    }

    async fn call(&self, args: Value, _ctx: &MutationContext) -> ToolResult {
        mcp_config_tools::handle_config_tool_call(mcp_config_tools::CONFIG_VALIDATE_TOOL, args)
    }
}

#[async_trait]
impl DynMutationTool for ConfigValidateChangeTool {
    fn meta(&self) -> MutationToolMeta {
        MutationToolMeta {
            name: mcp_config_tools::CONFIG_VALIDATE_CHANGE_TOOL,
            description: "Validate a proposed Script Kit config change",
            risk: RiskClass::Safe,
            required_scope: "config:read",
        }
    }

    fn definition(&self) -> ToolDefinition {
        config_tool_definition(mcp_config_tools::CONFIG_VALIDATE_CHANGE_TOOL)
    }

    async fn call(&self, args: Value, _ctx: &MutationContext) -> ToolResult {
        mcp_config_tools::handle_config_tool_call(
            mcp_config_tools::CONFIG_VALIDATE_CHANGE_TOOL,
            args,
        )
    }
}

#[async_trait]
impl DynMutationTool for ConfigSetTool {
    fn meta(&self) -> MutationToolMeta {
        MutationToolMeta {
            name: mcp_config_tools::CONFIG_SET_TOOL,
            description: "Set a Script Kit config value",
            risk: RiskClass::StateMutating,
            required_scope: "config:write",
        }
    }

    fn definition(&self) -> ToolDefinition {
        config_tool_definition(mcp_config_tools::CONFIG_SET_TOOL)
    }

    async fn call(&self, args: Value, _ctx: &MutationContext) -> ToolResult {
        mcp_config_tools::handle_config_tool_call(mcp_config_tools::CONFIG_SET_TOOL, args)
    }
}

#[async_trait]
impl DynMutationTool for ConfigResetTool {
    fn meta(&self) -> MutationToolMeta {
        MutationToolMeta {
            name: mcp_config_tools::CONFIG_RESET_TOOL,
            description: "Reset Script Kit config",
            risk: RiskClass::Destructive,
            required_scope: "config:write",
        }
    }

    fn definition(&self) -> ToolDefinition {
        config_tool_definition(mcp_config_tools::CONFIG_RESET_TOOL)
    }

    async fn call(&self, args: Value, _ctx: &MutationContext) -> ToolResult {
        mcp_config_tools::handle_config_tool_call(mcp_config_tools::CONFIG_RESET_TOOL, args)
    }
}

#[async_trait]
impl DynMutationTool for ConfigSetCommandShortcutTool {
    fn meta(&self) -> MutationToolMeta {
        MutationToolMeta {
            name: mcp_config_tools::CONFIG_SET_COMMAND_SHORTCUT_TOOL,
            description: "Set a Script Kit command shortcut",
            risk: RiskClass::StateMutating,
            required_scope: "config:write",
        }
    }

    fn definition(&self) -> ToolDefinition {
        config_tool_definition(mcp_config_tools::CONFIG_SET_COMMAND_SHORTCUT_TOOL)
    }

    async fn call(&self, args: Value, _ctx: &MutationContext) -> ToolResult {
        mcp_config_tools::handle_config_tool_call(
            mcp_config_tools::CONFIG_SET_COMMAND_SHORTCUT_TOOL,
            args,
        )
    }
}

#[async_trait]
impl DynMutationTool for ConfigRemoveCommandShortcutTool {
    fn meta(&self) -> MutationToolMeta {
        MutationToolMeta {
            name: mcp_config_tools::CONFIG_REMOVE_COMMAND_SHORTCUT_TOOL,
            description: "Remove a Script Kit command shortcut",
            risk: RiskClass::Destructive,
            required_scope: "config:write",
        }
    }

    fn definition(&self) -> ToolDefinition {
        config_tool_definition(mcp_config_tools::CONFIG_REMOVE_COMMAND_SHORTCUT_TOOL)
    }

    async fn call(&self, args: Value, _ctx: &MutationContext) -> ToolResult {
        mcp_config_tools::handle_config_tool_call(
            mcp_config_tools::CONFIG_REMOVE_COMMAND_SHORTCUT_TOOL,
            args,
        )
    }
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct McpAuditEvent {
    #[serde(default = "chrono::Utc::now")]
    ts: chrono::DateTime<chrono::Utc>,
    trace_id: String,
    method: String,
    tool: Option<String>,
    resource_uri: Option<String>,
    action: String,
    risk: String,
    success: bool,
    error_code: Option<String>,
    target_id: Option<String>,
}

pub(crate) fn scope_allows(scopes: &[String], required: &str) -> bool {
    scopes.iter().any(|scope| {
        scope == required
            || scope == "*"
            || scope == "mcp:*"
            || scope == "dev:*"
            || (required.starts_with("clipboard:") && scope == "clipboard:*")
            || (required.starts_with("config:") && scope == "config:*")
            || (required.starts_with("notes:") && scope == "notes:*")
            || (required.starts_with("scripts:") && scope == "scripts:*")
    })
}

fn append_audit_event(event: &McpAuditEvent) {
    let Some(home) = dirs::home_dir() else {
        return;
    };
    let audit_path = home.join(".scriptkit").join("mcp-audit.jsonl");
    if let Some(parent) = audit_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let Ok(line) = serde_json::to_string(event) else {
        return;
    };
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(audit_path)
    {
        use std::io::Write;
        let _ = writeln!(file, "{line}");
    }
}

fn tool_result_error_code(result: &ToolResult) -> Option<String> {
    let text = result.content.first()?.text.as_str();
    let value: serde_json::Value = serde_json::from_str(text).ok()?;
    value
        .get("error")
        .and_then(|error| error.get("code"))
        .and_then(|code| code.as_str())
        .map(ToString::to_string)
}
