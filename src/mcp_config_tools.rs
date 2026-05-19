use crate::mcp_kit_tools::{ToolContent, ToolDefinition, ToolResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;
use std::process::Command;

pub const CONFIG_GET_TOOL: &str = "kit/config_get";
pub const CONFIG_LIST_TOOL: &str = "kit/config_list";
pub const CONFIG_VALIDATE_TOOL: &str = "kit/config_validate";
pub const CONFIG_VALIDATE_CHANGE_TOOL: &str = "kit/config_validate_change";
pub const CONFIG_SET_TOOL: &str = "kit/config_set";
pub const CONFIG_RESET_TOOL: &str = "kit/config_reset";
pub const CONFIG_SET_COMMAND_SHORTCUT_TOOL: &str = "kit/config_set_command_shortcut";
pub const CONFIG_REMOVE_COMMAND_SHORTCUT_TOOL: &str = "kit/config_remove_command_shortcut";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigMutationEnvelope {
    pub ok: bool,
    pub action: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ConfigToolError>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConfigToolErrorCode {
    InvalidParams,
    ConfirmRequired,
    CliUnavailable,
    CliFailed,
    Internal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigToolError {
    pub code: ConfigToolErrorCode,
    pub message: String,
}

impl ConfigToolError {
    fn new(code: ConfigToolErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    fn invalid_params(message: impl Into<String>) -> Self {
        Self::new(ConfigToolErrorCode::InvalidParams, message)
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ConfigGetArgs {
    key: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ConfigSetArgs {
    key: String,
    value: Value,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ConfigResetArgs {
    key: Option<String>,
    #[serde(default)]
    confirm: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ConfigValidateChangeArgs {
    key: String,
    value: Value,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ConfigSetCommandShortcutArgs {
    command_id: String,
    key: String,
    #[serde(default)]
    cmd: bool,
    #[serde(default)]
    ctrl: bool,
    #[serde(default)]
    alt: bool,
    #[serde(default)]
    shift: bool,
    #[serde(default)]
    skip_existing: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ConfigRemoveCommandShortcutArgs {
    command_id: String,
    #[serde(default)]
    confirm: bool,
}

pub fn is_config_tool(name: &str) -> bool {
    matches!(
        name,
        CONFIG_GET_TOOL
            | CONFIG_LIST_TOOL
            | CONFIG_VALIDATE_TOOL
            | CONFIG_VALIDATE_CHANGE_TOOL
            | CONFIG_SET_TOOL
            | CONFIG_RESET_TOOL
            | CONFIG_SET_COMMAND_SHORTCUT_TOOL
            | CONFIG_REMOVE_COMMAND_SHORTCUT_TOOL
    )
}

pub fn get_config_tool_definitions() -> Vec<ToolDefinition> {
    vec![
        ToolDefinition {
            name: CONFIG_GET_TOOL.to_string(),
            description: "Read Script Kit config.ts, optionally by dot-path key.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "additionalProperties": false,
                "properties": { "key": { "type": "string" } }
            }),
        },
        ToolDefinition {
            name: CONFIG_LIST_TOOL.to_string(),
            description: "List supported Script Kit config keys with current/default values."
                .to_string(),
            input_schema: empty_schema(),
        },
        ToolDefinition {
            name: CONFIG_VALIDATE_TOOL.to_string(),
            description: "Validate the current Script Kit config.ts file.".to_string(),
            input_schema: empty_schema(),
        },
        ToolDefinition {
            name: CONFIG_VALIDATE_CHANGE_TOOL.to_string(),
            description: "Validate a proposed Script Kit config key/value change without writing."
                .to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "additionalProperties": false,
                "properties": {
                    "key": { "type": "string" },
                    "value": {}
                },
                "required": ["key", "value"]
            }),
        },
        ToolDefinition {
            name: CONFIG_SET_TOOL.to_string(),
            description: "Set a Script Kit config.ts key to a JSON-compatible value.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "additionalProperties": false,
                "properties": {
                    "key": { "type": "string" },
                    "value": {}
                },
                "required": ["key", "value"]
            }),
        },
        ToolDefinition {
            name: CONFIG_RESET_TOOL.to_string(),
            description:
                "Reset one Script Kit config key, or the entire config, with confirm:true."
                    .to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "additionalProperties": false,
                "properties": {
                    "key": { "type": "string" },
                    "confirm": { "type": "boolean", "default": false }
                },
                "required": ["confirm"]
            }),
        },
        ToolDefinition {
            name: CONFIG_SET_COMMAND_SHORTCUT_TOOL.to_string(),
            description: "Set a launcher command shortcut in Script Kit config.ts.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "additionalProperties": false,
                "properties": {
                    "commandId": { "type": "string" },
                    "key": { "type": "string" },
                    "cmd": { "type": "boolean", "default": false },
                    "ctrl": { "type": "boolean", "default": false },
                    "alt": { "type": "boolean", "default": false },
                    "shift": { "type": "boolean", "default": false },
                    "skipExisting": { "type": "boolean", "default": false }
                },
                "required": ["commandId", "key"]
            }),
        },
        ToolDefinition {
            name: CONFIG_REMOVE_COMMAND_SHORTCUT_TOOL.to_string(),
            description:
                "Remove a launcher command shortcut from Script Kit config.ts with confirm:true."
                    .to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "additionalProperties": false,
                "properties": {
                    "commandId": { "type": "string" },
                    "confirm": { "type": "boolean", "default": false }
                },
                "required": ["commandId", "confirm"]
            }),
        },
    ]
}

fn empty_schema() -> Value {
    serde_json::json!({
        "type": "object",
        "additionalProperties": false,
        "properties": {}
    })
}

pub fn handle_config_tool_call(name: &str, arguments: Value) -> ToolResult {
    match config_cli_args(name, arguments).and_then(|(action, args)| run_config_cli(action, args)) {
        Ok((action, result)) => success_tool_result(action, result),
        Err((action, error)) => error_tool_result(action, error),
    }
}

fn config_cli_args(
    name: &str,
    arguments: Value,
) -> Result<(&'static str, Vec<String>), (&'static str, ConfigToolError)> {
    match name {
        CONFIG_GET_TOOL => {
            let args: ConfigGetArgs = parse_args(name, arguments)?;
            let mut cli_args = vec!["get".to_string()];
            if let Some(key) = args.key {
                validate_non_empty("key", &key).map_err(|error| ("config_get", error))?;
                cli_args.push(key);
            }
            Ok(("config_get", cli_args))
        }
        CONFIG_LIST_TOOL => {
            parse_empty_args(name, arguments)?;
            Ok(("config_list", vec!["list".to_string()]))
        }
        CONFIG_VALIDATE_TOOL => {
            parse_empty_args(name, arguments)?;
            Ok(("config_validate", vec!["validate".to_string()]))
        }
        CONFIG_VALIDATE_CHANGE_TOOL => {
            let args: ConfigValidateChangeArgs = parse_args(name, arguments)?;
            validate_non_empty("key", &args.key)
                .map_err(|error| ("config_validate_change", error))?;
            let payload = serde_json::json!({
                "key": args.key,
                "value": args.value
            });
            Ok((
                "config_validate_change",
                vec!["validate-change".to_string(), payload.to_string()],
            ))
        }
        CONFIG_SET_TOOL => {
            let args: ConfigSetArgs = parse_args(name, arguments)?;
            validate_non_empty("key", &args.key).map_err(|error| ("config_set", error))?;
            Ok((
                "config_set",
                vec!["set".to_string(), args.key, value_to_cli_arg(&args.value)],
            ))
        }
        CONFIG_RESET_TOOL => {
            let args: ConfigResetArgs = parse_args(name, arguments)?;
            if !args.confirm {
                return Err((
                    "config_reset",
                    ConfigToolError::new(
                        ConfigToolErrorCode::ConfirmRequired,
                        "kit/config_reset requires confirm:true",
                    ),
                ));
            }
            let mut cli_args = vec!["reset".to_string()];
            if let Some(key) = args.key {
                validate_non_empty("key", &key).map_err(|error| ("config_reset", error))?;
                cli_args.push(key);
            }
            Ok(("config_reset", cli_args))
        }
        CONFIG_SET_COMMAND_SHORTCUT_TOOL => {
            let args: ConfigSetCommandShortcutArgs = parse_args(name, arguments)?;
            validate_non_empty("commandId", &args.command_id)
                .map_err(|error| ("config_set_command_shortcut", error))?;
            validate_non_empty("key", &args.key)
                .map_err(|error| ("config_set_command_shortcut", error))?;
            let mut cli_args = vec![
                "set-command-shortcut".to_string(),
                args.command_id,
                args.key,
                args.cmd.to_string(),
                args.ctrl.to_string(),
                args.alt.to_string(),
                args.shift.to_string(),
            ];
            if args.skip_existing {
                cli_args.push("--skip-existing".to_string());
            }
            Ok(("config_set_command_shortcut", cli_args))
        }
        CONFIG_REMOVE_COMMAND_SHORTCUT_TOOL => {
            let args: ConfigRemoveCommandShortcutArgs = parse_args(name, arguments)?;
            validate_non_empty("commandId", &args.command_id)
                .map_err(|error| ("config_remove_command_shortcut", error))?;
            if !args.confirm {
                return Err((
                    "config_remove_command_shortcut",
                    ConfigToolError::new(
                        ConfigToolErrorCode::ConfirmRequired,
                        "kit/config_remove_command_shortcut requires confirm:true",
                    ),
                ));
            }
            Ok((
                "config_remove_command_shortcut",
                vec!["remove-command-shortcut".to_string(), args.command_id],
            ))
        }
        _ => Err((
            "config_unknown",
            ConfigToolError::invalid_params(format!("Unknown config tool: {name}")),
        )),
    }
}

fn parse_args<T: for<'de> Deserialize<'de>>(
    name: &str,
    arguments: Value,
) -> Result<T, (&'static str, ConfigToolError)> {
    serde_json::from_value(arguments).map_err(|error| {
        (
            action_label(name),
            ConfigToolError::invalid_params(format!("Invalid {name} arguments: {error}")),
        )
    })
}

fn parse_empty_args(name: &str, arguments: Value) -> Result<(), (&'static str, ConfigToolError)> {
    let object = arguments.as_object().ok_or_else(|| {
        (
            action_label(name),
            ConfigToolError::invalid_params(format!("Invalid {name} arguments: expected object")),
        )
    })?;
    if object.is_empty() {
        Ok(())
    } else {
        Err((
            action_label(name),
            ConfigToolError::invalid_params(format!("{name} does not accept arguments")),
        ))
    }
}

fn validate_non_empty(field: &str, value: &str) -> Result<(), ConfigToolError> {
    if value.trim().is_empty() {
        Err(ConfigToolError::invalid_params(format!(
            "{field} cannot be empty"
        )))
    } else {
        Ok(())
    }
}

fn value_to_cli_arg(value: &Value) -> String {
    match value {
        Value::String(value) => value.clone(),
        _ => value.to_string(),
    }
}

fn run_config_cli(
    action: &'static str,
    args: Vec<String>,
) -> Result<(&'static str, Value), (&'static str, ConfigToolError)> {
    let cli_path = config_cli_path().map_err(|error| (action, error))?;
    let output = Command::new("bun")
        .arg(cli_path)
        .args(args)
        .output()
        .map_err(|error| {
            (
                action,
                ConfigToolError::new(
                    ConfigToolErrorCode::CliUnavailable,
                    format!("Failed to run config-cli.ts with bun: {error}"),
                ),
            )
        })?;

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let parsed = if stdout.is_empty() {
        Value::Null
    } else {
        serde_json::from_str(&stdout).map_err(|error| {
            (
                action,
                ConfigToolError::new(
                    ConfigToolErrorCode::Internal,
                    format!("config-cli.ts returned invalid JSON: {error}; stdout={stdout}"),
                ),
            )
        })?
    };

    if output.status.success()
        && parsed
            .get("success")
            .and_then(Value::as_bool)
            .unwrap_or(false)
    {
        return Ok((action, parsed));
    }

    let message = parsed
        .get("error")
        .and_then(Value::as_str)
        .map(ToString::to_string)
        .or_else(|| {
            parsed
                .get("errors")
                .and_then(Value::as_array)
                .map(|errors| {
                    errors
                        .iter()
                        .map(Value::to_string)
                        .collect::<Vec<_>>()
                        .join("; ")
                })
        })
        .unwrap_or_else(|| {
            if stderr.is_empty() {
                format!("config-cli.ts failed with status {}", output.status)
            } else {
                format!(
                    "config-cli.ts failed with status {}: {stderr}",
                    output.status
                )
            }
        });
    Err((
        action,
        ConfigToolError::new(ConfigToolErrorCode::CliFailed, message),
    ))
}

fn config_cli_path() -> Result<PathBuf, ConfigToolError> {
    if let Ok(path) = std::env::var("SCRIPT_KIT_CONFIG_CLI_PATH") {
        let path = PathBuf::from(path);
        if path.exists() {
            return Ok(path);
        }
        return Err(ConfigToolError::new(
            ConfigToolErrorCode::CliUnavailable,
            format!(
                "SCRIPT_KIT_CONFIG_CLI_PATH does not exist: {}",
                path.display()
            ),
        ));
    }

    if let Ok(current_dir) = std::env::current_dir() {
        let candidate = current_dir.join("scripts").join("config-cli.ts");
        if candidate.exists() {
            return Ok(candidate);
        }
    }

    if let Some(home) = dirs::home_dir() {
        let candidate = home.join(".scriptkit").join("sdk").join("config-cli.ts");
        if candidate.exists() {
            return Ok(candidate);
        }
    }

    Err(ConfigToolError::new(
        ConfigToolErrorCode::CliUnavailable,
        "Could not locate scripts/config-cli.ts or ~/.scriptkit/sdk/config-cli.ts",
    ))
}

fn success_tool_result(action: &'static str, result: Value) -> ToolResult {
    envelope_tool_result(ConfigMutationEnvelope {
        ok: true,
        action,
        result: Some(result),
        error: None,
    })
}

pub fn error_tool_result(action: &str, error: ConfigToolError) -> ToolResult {
    let mut result = envelope_tool_result(ConfigMutationEnvelope {
        ok: false,
        action: action_label(action),
        result: None,
        error: Some(error),
    });
    result.is_error = Some(true);
    result
}

fn action_label(action: &str) -> &'static str {
    match action {
        CONFIG_GET_TOOL | "config_get" => "config_get",
        CONFIG_LIST_TOOL | "config_list" => "config_list",
        CONFIG_VALIDATE_TOOL | "config_validate" => "config_validate",
        CONFIG_VALIDATE_CHANGE_TOOL | "config_validate_change" => "config_validate_change",
        CONFIG_SET_TOOL | "config_set" => "config_set",
        CONFIG_RESET_TOOL | "config_reset" => "config_reset",
        CONFIG_SET_COMMAND_SHORTCUT_TOOL | "config_set_command_shortcut" => {
            "config_set_command_shortcut"
        }
        CONFIG_REMOVE_COMMAND_SHORTCUT_TOOL | "config_remove_command_shortcut" => {
            "config_remove_command_shortcut"
        }
        _ => "config_unknown",
    }
}

fn envelope_tool_result(envelope: ConfigMutationEnvelope) -> ToolResult {
    ToolResult {
        content: vec![ToolContent {
            content_type: "text".to_string(),
            text: serde_json::to_string(&envelope).unwrap_or_else(|error| {
                format!(
                    r#"{{"ok":false,"action":"config_internal","error":{{"code":"internal","message":"Failed to serialize config result: {error}"}}}}"#
                )
            }),
        }],
        is_error: None,
    }
}
