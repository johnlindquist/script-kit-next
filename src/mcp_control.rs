use crate::mcp_kit_tools::{ToolDefinition, ToolResult};
use crate::mcp_notes_tools::{
    self, McpNotesMutationBridge, NotesMutationError, NotesMutationErrorCode, NotesMutationRequest,
    NotesMutationResult, SharedNotesMutationBridge,
};
use crate::mcp_scripts_tools;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

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
    pub requires_confirm: bool,
}

#[derive(Clone)]
pub struct MutationContext {
    pub trace_id: String,
    pub token_scopes: Vec<String>,
    pub notes_bridge: Option<SharedNotesMutationBridge>,
}

pub trait DynMutationTool: Send + Sync {
    fn meta(&self) -> MutationToolMeta;
    fn definition(&self) -> ToolDefinition;
    fn call(&self, args: Value, ctx: &MutationContext) -> ToolResult;
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

    pub fn call(&self, name: &str, args: Value, ctx: &MutationContext) -> Option<ToolResult> {
        let tool = self.tools.get(name)?;
        let meta = tool.meta();
        if !scope_allows(&ctx.token_scopes, meta.required_scope) {
            let result = mcp_notes_tools::error_tool_result(
                meta.name,
                NotesMutationError::new(
                    NotesMutationErrorCode::InvalidParams,
                    format!("Missing required MCP scope: {}", meta.required_scope),
                ),
            );
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

        let result = tool.call(args, ctx);
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
    pub response_tx: std::sync::mpsc::SyncSender<Result<NotesMutationResult, NotesMutationError>>,
}

impl McpNotesMutationBridge for GpuiNotesMcpBridge {
    fn mutate_notes(
        &self,
        request: NotesMutationRequest,
    ) -> Result<NotesMutationResult, NotesMutationError> {
        let (response_tx, response_rx) = std::sync::mpsc::sync_channel(1);
        self.tx
            .send_blocking(NotesMcpCommand {
                request,
                response_tx,
            })
            .map_err(|_| {
                NotesMutationError::new(
                    NotesMutationErrorCode::MissingRuntime,
                    "Notes mutation runtime is disconnected",
                )
            })?;

        response_rx.recv_timeout(self.timeout).map_err(|_| {
            NotesMutationError::new(
                NotesMutationErrorCode::Internal,
                "Timed out waiting for GPUI notes mutation bridge",
            )
        })?
    }
}

struct NotesCreateTool;
struct NotesUpdateTool;
struct NotesDeleteTool;
struct ScriptsCreateTool;
struct ScriptsUpdateTool;
struct ScriptsDeleteTool;
struct ScriptsRunTool;

impl DynMutationTool for NotesCreateTool {
    fn meta(&self) -> MutationToolMeta {
        MutationToolMeta {
            name: mcp_notes_tools::NOTES_CREATE_TOOL,
            description: "Create a Script Kit note",
            risk: RiskClass::StateMutating,
            required_scope: "notes:write",
            requires_confirm: false,
        }
    }

    fn definition(&self) -> ToolDefinition {
        mcp_notes_tools::get_notes_tool_definitions()
            .into_iter()
            .find(|tool| tool.name == mcp_notes_tools::NOTES_CREATE_TOOL)
            .expect("notes create tool definition")
    }

    fn call(&self, args: Value, ctx: &MutationContext) -> ToolResult {
        mcp_notes_tools::handle_notes_tool_call(
            ctx.notes_bridge.as_deref(),
            mcp_notes_tools::NOTES_CREATE_TOOL,
            args,
        )
    }
}

impl DynMutationTool for NotesUpdateTool {
    fn meta(&self) -> MutationToolMeta {
        MutationToolMeta {
            name: mcp_notes_tools::NOTES_UPDATE_TOOL,
            description: "Update a Script Kit note",
            risk: RiskClass::StateMutating,
            required_scope: "notes:write",
            requires_confirm: false,
        }
    }

    fn definition(&self) -> ToolDefinition {
        mcp_notes_tools::get_notes_tool_definitions()
            .into_iter()
            .find(|tool| tool.name == mcp_notes_tools::NOTES_UPDATE_TOOL)
            .expect("notes update tool definition")
    }

    fn call(&self, args: Value, ctx: &MutationContext) -> ToolResult {
        mcp_notes_tools::handle_notes_tool_call(
            ctx.notes_bridge.as_deref(),
            mcp_notes_tools::NOTES_UPDATE_TOOL,
            args,
        )
    }
}

impl DynMutationTool for NotesDeleteTool {
    fn meta(&self) -> MutationToolMeta {
        MutationToolMeta {
            name: mcp_notes_tools::NOTES_DELETE_TOOL,
            description: "Delete a Script Kit note",
            risk: RiskClass::Destructive,
            required_scope: "notes:write",
            requires_confirm: true,
        }
    }

    fn definition(&self) -> ToolDefinition {
        mcp_notes_tools::get_notes_tool_definitions()
            .into_iter()
            .find(|tool| tool.name == mcp_notes_tools::NOTES_DELETE_TOOL)
            .expect("notes delete tool definition")
    }

    fn call(&self, args: Value, ctx: &MutationContext) -> ToolResult {
        mcp_notes_tools::handle_notes_tool_call(
            ctx.notes_bridge.as_deref(),
            mcp_notes_tools::NOTES_DELETE_TOOL,
            args,
        )
    }
}

impl DynMutationTool for ScriptsCreateTool {
    fn meta(&self) -> MutationToolMeta {
        MutationToolMeta {
            name: mcp_scripts_tools::SCRIPTS_CREATE_TOOL,
            description: "Create a Script Kit script",
            risk: RiskClass::StateMutating,
            required_scope: "scripts:write",
            requires_confirm: false,
        }
    }

    fn definition(&self) -> ToolDefinition {
        mcp_scripts_tools::get_scripts_tool_definitions()
            .into_iter()
            .find(|tool| tool.name == mcp_scripts_tools::SCRIPTS_CREATE_TOOL)
            .expect("scripts create tool definition")
    }

    fn call(&self, args: Value, _ctx: &MutationContext) -> ToolResult {
        mcp_scripts_tools::handle_scripts_tool_call(mcp_scripts_tools::SCRIPTS_CREATE_TOOL, args)
    }
}

impl DynMutationTool for ScriptsUpdateTool {
    fn meta(&self) -> MutationToolMeta {
        MutationToolMeta {
            name: mcp_scripts_tools::SCRIPTS_UPDATE_TOOL,
            description: "Update a Script Kit script",
            risk: RiskClass::StateMutating,
            required_scope: "scripts:write",
            requires_confirm: false,
        }
    }

    fn definition(&self) -> ToolDefinition {
        mcp_scripts_tools::get_scripts_tool_definitions()
            .into_iter()
            .find(|tool| tool.name == mcp_scripts_tools::SCRIPTS_UPDATE_TOOL)
            .expect("scripts update tool definition")
    }

    fn call(&self, args: Value, _ctx: &MutationContext) -> ToolResult {
        mcp_scripts_tools::handle_scripts_tool_call(mcp_scripts_tools::SCRIPTS_UPDATE_TOOL, args)
    }
}

impl DynMutationTool for ScriptsDeleteTool {
    fn meta(&self) -> MutationToolMeta {
        MutationToolMeta {
            name: mcp_scripts_tools::SCRIPTS_DELETE_TOOL,
            description: "Delete a Script Kit script",
            risk: RiskClass::Destructive,
            required_scope: "scripts:write",
            requires_confirm: true,
        }
    }

    fn definition(&self) -> ToolDefinition {
        mcp_scripts_tools::get_scripts_tool_definitions()
            .into_iter()
            .find(|tool| tool.name == mcp_scripts_tools::SCRIPTS_DELETE_TOOL)
            .expect("scripts delete tool definition")
    }

    fn call(&self, args: Value, _ctx: &MutationContext) -> ToolResult {
        mcp_scripts_tools::handle_scripts_tool_call(mcp_scripts_tools::SCRIPTS_DELETE_TOOL, args)
    }
}

impl DynMutationTool for ScriptsRunTool {
    fn meta(&self) -> MutationToolMeta {
        MutationToolMeta {
            name: mcp_scripts_tools::SCRIPTS_RUN_TOOL,
            description: "Run a Script Kit script",
            risk: RiskClass::ExternalProcess,
            required_scope: "scripts:run",
            requires_confirm: false,
        }
    }

    fn definition(&self) -> ToolDefinition {
        mcp_scripts_tools::get_scripts_tool_definitions()
            .into_iter()
            .find(|tool| tool.name == mcp_scripts_tools::SCRIPTS_RUN_TOOL)
            .expect("scripts run tool definition")
    }

    fn call(&self, args: Value, _ctx: &MutationContext) -> ToolResult {
        mcp_scripts_tools::handle_scripts_tool_call(mcp_scripts_tools::SCRIPTS_RUN_TOOL, args)
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

fn scope_allows(scopes: &[String], required: &str) -> bool {
    scopes.iter().any(|scope| {
        scope == required
            || scope == "*"
            || scope == "mcp:*"
            || scope == "dev:*"
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
