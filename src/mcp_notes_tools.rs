use crate::mcp_kit_tools::{ToolContent, ToolDefinition, ToolResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

pub const NOTES_CREATE_TOOL: &str = "kit/notes_create";
pub const NOTES_UPDATE_TOOL: &str = "kit/notes_update";
pub const NOTES_DELETE_TOOL: &str = "kit/notes_delete";
pub const NOTE_BODY_MAX_BYTES: usize = 1024 * 1024;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NotesMutationEnvelope {
    pub ok: bool,
    pub action: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<NotesMutationResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<NotesMutationError>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NotesMutationErrorCode {
    InvalidParams,
    MissingRuntime,
    NotFound,
    Conflict,
    ConfirmRequired,
    Internal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotesMutationError {
    pub code: NotesMutationErrorCode,
    pub message: String,
}

impl NotesMutationError {
    pub fn new(code: NotesMutationErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    pub fn invalid_params(message: impl Into<String>) -> Self {
        Self::new(NotesMutationErrorCode::InvalidParams, message)
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NotesCreateArgs {
    pub id: Option<String>,
    pub title: Option<String>,
    #[serde(alias = "content")]
    pub body: String,
    #[serde(default)]
    pub is_pinned: bool,
    #[serde(default)]
    pub sort_order: Option<i32>,
    #[serde(default)]
    pub open: bool,
    #[serde(default)]
    pub select: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NotesUpdateArgs {
    pub id: String,
    pub title: Option<String>,
    #[serde(default, alias = "content")]
    pub body: Option<String>,
    #[serde(default)]
    pub is_pinned: Option<bool>,
    #[serde(default)]
    pub sort_order: Option<i32>,
    #[serde(default)]
    pub open: bool,
    #[serde(default)]
    pub select: bool,
    #[serde(default)]
    pub force: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NotesDeleteArgs {
    pub id: String,
    #[serde(default)]
    pub permanent: bool,
    #[serde(default)]
    pub confirm: bool,
}

#[derive(Debug, Clone)]
pub enum NotesMutationRequest {
    Create(NotesCreateArgs),
    Update(NotesUpdateArgs),
    Delete(NotesDeleteArgs),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotesMutationResult {
    pub id: String,
    pub uri: String,
    pub title: Option<String>,
    pub deleted: bool,
    pub permanent: bool,
}

#[async_trait::async_trait]
pub trait McpNotesMutationBridge: Send + Sync + 'static {
    async fn mutate_notes(
        &self,
        request: NotesMutationRequest,
    ) -> Result<NotesMutationResult, NotesMutationError>;
}

pub type SharedNotesMutationBridge = Arc<dyn McpNotesMutationBridge>;

pub fn is_notes_tool(name: &str) -> bool {
    matches!(
        name,
        NOTES_CREATE_TOOL | NOTES_UPDATE_TOOL | NOTES_DELETE_TOOL
    )
}

pub fn get_notes_tool_definitions() -> Vec<ToolDefinition> {
    vec![
        ToolDefinition {
            name: NOTES_CREATE_TOOL.to_string(),
            description: "Create a Script Kit note and optionally open/select it in the Notes window.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "additionalProperties": false,
                "properties": {
                    "id": { "type": "string", "description": "Optional UUID. Omit to generate one." },
                    "title": { "type": "string" },
                    "body": { "type": "string", "maxLength": NOTE_BODY_MAX_BYTES },
                    "content": { "type": "string", "maxLength": NOTE_BODY_MAX_BYTES },
                    "isPinned": { "type": "boolean", "default": false },
                    "sortOrder": { "type": "integer" },
                    "open": { "type": "boolean", "default": false },
                    "select": { "type": "boolean", "default": false }
                },
                "anyOf": [
                    { "required": ["body"] },
                    { "required": ["content"] }
                ]
            }),
        },
        ToolDefinition {
            name: NOTES_UPDATE_TOOL.to_string(),
            description: "Update a Script Kit note by id and optionally open/select it in the Notes window.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "additionalProperties": false,
                "properties": {
                    "id": { "type": "string" },
                    "title": { "type": "string" },
                    "body": { "type": "string", "maxLength": NOTE_BODY_MAX_BYTES },
                    "content": { "type": "string", "maxLength": NOTE_BODY_MAX_BYTES },
                    "isPinned": { "type": "boolean" },
                    "sortOrder": { "type": "integer" },
                    "open": { "type": "boolean", "default": false },
                    "select": { "type": "boolean", "default": false },
                    "force": { "type": "boolean", "default": false }
                },
                "required": ["id"]
            }),
        },
        ToolDefinition {
            name: NOTES_DELETE_TOOL.to_string(),
            description: "Delete a Script Kit note. Soft delete by default; permanent deletion requires permanent=true and confirm=true.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "additionalProperties": false,
                "properties": {
                    "id": { "type": "string" },
                    "permanent": { "type": "boolean", "default": false },
                    "confirm": { "type": "boolean", "default": false }
                },
                "required": ["id"]
            }),
        },
    ]
}

pub fn parse_notes_mutation_request(
    name: &str,
    arguments: Value,
) -> Result<NotesMutationRequest, NotesMutationError> {
    match name {
        NOTES_CREATE_TOOL => {
            let args: NotesCreateArgs = serde_json::from_value(arguments).map_err(|error| {
                NotesMutationError::invalid_params(format!(
                    "Invalid notes_create arguments: {error}"
                ))
            })?;
            validate_body_len(&args.body)?;
            Ok(NotesMutationRequest::Create(args))
        }
        NOTES_UPDATE_TOOL => {
            let args: NotesUpdateArgs = serde_json::from_value(arguments).map_err(|error| {
                NotesMutationError::invalid_params(format!(
                    "Invalid notes_update arguments: {error}"
                ))
            })?;
            if let Some(body) = &args.body {
                validate_body_len(body)?;
            }
            Ok(NotesMutationRequest::Update(args))
        }
        NOTES_DELETE_TOOL => {
            let args: NotesDeleteArgs = serde_json::from_value(arguments).map_err(|error| {
                NotesMutationError::invalid_params(format!(
                    "Invalid notes_delete arguments: {error}"
                ))
            })?;
            if args.permanent && !args.confirm {
                return Err(NotesMutationError::new(
                    NotesMutationErrorCode::ConfirmRequired,
                    "kit/notes_delete with permanent:true requires confirm:true",
                ));
            }
            Ok(NotesMutationRequest::Delete(args))
        }
        _ => Err(NotesMutationError::invalid_params(format!(
            "Unknown notes tool: {name}"
        ))),
    }
}

pub async fn handle_notes_tool_call(
    bridge: Option<&dyn McpNotesMutationBridge>,
    name: &str,
    arguments: Value,
) -> ToolResult {
    let request = match parse_notes_mutation_request(name, arguments) {
        Ok(request) => request,
        Err(error) => return error_tool_result(name, error),
    };

    let Some(bridge) = bridge else {
        return error_tool_result(
            name,
            NotesMutationError::new(
                NotesMutationErrorCode::MissingRuntime,
                "Notes mutation runtime is not installed",
            ),
        );
    };

    match bridge.mutate_notes(request).await {
        Ok(result) => success_tool_result(name, result),
        Err(error) => error_tool_result(name, error),
    }
}

pub fn success_tool_result(action: &str, result: NotesMutationResult) -> ToolResult {
    let resource_uri = Some(result.uri.clone());
    envelope_tool_result(NotesMutationEnvelope {
        ok: true,
        action: action_label(action),
        resource_uri,
        result: Some(result),
        error: None,
    })
}

pub fn error_tool_result(action: &str, error: NotesMutationError) -> ToolResult {
    let mut result = envelope_tool_result(NotesMutationEnvelope {
        ok: false,
        action: action_label(action),
        resource_uri: None,
        result: None,
        error: Some(error),
    });
    result.is_error = Some(true);
    result
}

fn validate_body_len(body: &str) -> Result<(), NotesMutationError> {
    if body.len() > NOTE_BODY_MAX_BYTES {
        return Err(NotesMutationError::invalid_params(format!(
            "Note body exceeds {NOTE_BODY_MAX_BYTES} byte limit"
        )));
    }
    Ok(())
}

fn action_label(action: &str) -> &'static str {
    match action {
        NOTES_CREATE_TOOL => "notes_create",
        NOTES_UPDATE_TOOL => "notes_update",
        NOTES_DELETE_TOOL => "notes_delete",
        _ => "notes_unknown",
    }
}

fn envelope_tool_result(envelope: NotesMutationEnvelope) -> ToolResult {
    ToolResult {
        content: vec![ToolContent {
            content_type: "text".to_string(),
            text: serde_json::to_string(&envelope).unwrap_or_else(|error| {
                format!(
                    r#"{{"ok":false,"action":"notes_internal","error":{{"code":"internal","message":"Failed to serialize notes result: {error}"}}}}"#
                )
            }),
        }],
        is_error: None,
    }
}
