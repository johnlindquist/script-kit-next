#!/usr/bin/env bun
/**
 * Script Kit Config CLI
 * 
 * A CLI tool for AI agents to read and modify ~/.scriptkit/config.ts
 * 
 * Usage:
 *   bun scripts/config-cli.ts get [key]        - Read value(s)
 *   bun scripts/config-cli.ts set <key> <value> - Modify a value
 *   bun scripts/config-cli.ts set-command-shortcut <command_id> <key> <cmd> <ctrl> <alt> <shift>
 *   bun scripts/config-cli.ts remove-command-shortcut <command_id>
 *   bun scripts/config-cli.ts list             - Show all options with values
 *   bun scripts/config-cli.ts validate         - Check if config is valid
 *   bun scripts/config-cli.ts reset [key]      - Restore default(s)
 *   bun scripts/config-cli.ts validate-change <json> - Validate a proposed change
 *   bun scripts/config-cli.ts --help           - Show this help
 * 
 * Output is JSON by default for AI parsing.
 */

import * as fs from 'node:fs';
import * as path from 'node:path';
import * as os from 'node:os';

import {
  analyzeCommandConfigPath,
  validateCommandConfigFieldValue,
  validateCommandConfigValue,
  validateCommandIdList,
  validateCommandsConfig,
} from './config-schema';
import type {
  CommandId,
  ConfigChange,
  ValidateConfigChangeResult,
} from './config-schema';

// NOTE: This CLI manages the full ~/.scriptkit/config.ts surface,
// including runtime preference groups such as theme, dictation, AI, and
// windowManagement.

// =============================================================================
// Types (matching kit-sdk.ts and src/config.rs)
// =============================================================================

type KeyModifier = "meta" | "ctrl" | "alt" | "shift";
type KeyCode =
  | "KeyA" | "KeyB" | "KeyC" | "KeyD" | "KeyE" | "KeyF" | "KeyG"
  | "KeyH" | "KeyI" | "KeyJ" | "KeyK" | "KeyL" | "KeyM" | "KeyN"
  | "KeyO" | "KeyP" | "KeyQ" | "KeyR" | "KeyS" | "KeyT" | "KeyU"
  | "KeyV" | "KeyW" | "KeyX" | "KeyY" | "KeyZ"
  | "Digit0" | "Digit1" | "Digit2" | "Digit3" | "Digit4"
  | "Digit5" | "Digit6" | "Digit7" | "Digit8" | "Digit9"
  | "Space" | "Enter" | "Semicolon" | "Comma" | "Period" | "Slash"
  | "Tab" | "Escape" | "Backspace" | "Delete"
  | "ArrowUp" | "ArrowDown" | "ArrowLeft" | "ArrowRight"
  | "Home" | "End" | "PageUp" | "PageDown" | "Insert"
  | "Quote" | "Backslash" | "BracketLeft" | "BracketRight"
  | "Minus" | "Equal" | "Backquote"
  | "F1" | "F2" | "F3" | "F4" | "F5" | "F6"
  | "F7" | "F8" | "F9" | "F10" | "F11" | "F12";

interface HotkeyConfig {
  modifiers: KeyModifier[];
  key: KeyCode;
}

interface ContentPadding {
  top?: number;
  left?: number;
  right?: number;
}

interface BuiltInConfig {
  clipboardHistory?: boolean;
  appLauncher?: boolean;
  windowSwitcher?: boolean;
}

interface ProcessLimits {
  maxMemoryMb?: number;
  maxRuntimeSeconds?: number;
  healthCheckIntervalMs?: number;
}

interface SuggestedConfig {
  enabled?: boolean;
  maxItems?: number;
  minScore?: number;
  halfLifeDays?: number;
  trackUsage?: boolean;
  excludedCommands?: CommandId[];
}

interface WatcherConfig {
  debounceMs?: number;
  stormThreshold?: number;
  initialBackoffMs?: number;
  maxBackoffMs?: number;
  maxNotifyErrors?: number;
}

interface LayoutConfig {
  standardHeight?: number;
  maxHeight?: number;
}

interface ThemeSelectionPreferences {
  presetId?: string;
}

interface DictationPreferences {
  selectedDeviceId?: string;
}

type AgentChatBackend = "agent_chat" | "pi";

interface AiProfile {
  id?: string;
  name: string;
  backend?: AgentChatBackend;
  agent?: string;
  provider?: string;
  model?: string;
  systemPrompt?: string;
  appendSystemPrompt?: string;
  cwd?: string;
  tools?: string[];
  disableExtensions?: boolean;
  disableSkills?: boolean;
  disablePromptTemplates?: boolean;
  hideCwdInPrompt?: boolean;
  thinking?: string;
  extensionPolicy?: string;
  sessionDir?: string;
  noSession?: boolean;
  sessionDurability?: string;
}

interface AiPreferences {
  selectedModelId?: string;
  selectedAgentChatAgentId?: string;
  selectedProfileId?: string;
  selectedBackend?: AgentChatBackend;
  profiles?: AiProfile[];
  selectedProfileName?: string;
}

type SnapMode = "off" | "simple" | "expanded" | "precision";

interface WindowManagementPreferences {
  snapMode?: SnapMode;
}

type WindowVibrancyMaterial =
  | "default"
  | "none"
  | "hud"
  | "popover"
  | "sidebar";

type WindowAnimationMode = "system" | "reduced" | "off";

interface WindowAppearanceConfig {
  vibrancy?: WindowVibrancyMaterial;
  animations?: WindowAnimationMode;
}

interface CommandConfig {
  shortcut?: HotkeyConfig;
  hidden?: boolean;
  confirmationRequired?: boolean;
}

interface PromptTargetConfig {
  title?: string;
  description?: string;
  command: string;
  args?: string[];
  cwd?: string;
  env?: Record<string, string>;
}

type AiVaultProvider = "claude" | "codex" | "hermesAgent" | "rovoDev";
type AiVaultResumeTerminal = "cmux" | "quickTerminal";

interface UnifiedSearchAiVaultConfig {
  enabled?: boolean;
  maxResults?: number;
  minQueryChars?: number;
  providers?: AiVaultProvider[];
  cacheTtlMs?: number;
  searchContent?: boolean;
  resumeTerminal?: AiVaultResumeTerminal;
  excludePatterns?: string[];
}

interface UnifiedSearchConfig {
  aiVault?: UnifiedSearchAiVaultConfig;
}

type ClaudeCodePermissionMode = "plan" | "dontAsk";

interface ClaudeCodeConfig {
  enabled?: boolean;
  path?: string;
  permissionMode?: ClaudeCodePermissionMode;
  allowedTools?: string;
  addDirs?: string[];
}

type McpTransport = "stdio" | "http";

interface McpBaseServerConfig {
  name?: string;
  description?: string;
  enabled?: boolean;
}

interface McpStdioServerConfig extends McpBaseServerConfig {
  transport: "stdio";
  command: string;
  args?: string[];
  env?: Record<string, string>;
  cwd?: string;
}

interface McpHttpServerConfig extends McpBaseServerConfig {
  transport: "http";
  endpoint: string;
  headers?: Record<string, string>;
}

type McpServerConfig = McpStdioServerConfig | McpHttpServerConfig;

interface McpConfig {
  enabled?: boolean;
  servers?: Record<string, McpServerConfig>;
}

type PowerSyntaxCaptureSigil = ";" | "+" | "both";
type PowerSyntaxCommandSigil = ">" | "disabled";

interface PowerSyntaxCmdEnterAiConfig {
  enabled?: boolean;
  modelId?: string;
  systemPrompt?: string;
}

interface PowerSyntaxConfig {
  enabled?: boolean;
  captureSigil?: PowerSyntaxCaptureSigil;
  commandSigil?: PowerSyntaxCommandSigil;
  cmdEnterAi?: PowerSyntaxCmdEnterAiConfig;
}

interface TrayConfig {
  showCurrentAppCommands?: boolean;
  showNotes?: boolean;
  showAgentChat?: boolean;
  showReloadScripts?: boolean;
  showHelp?: boolean;
  showSocialLinks?: boolean;
  showUpdateCheck?: boolean;
}

interface UpdatesConfig {
  autoCheck?: boolean;
}

interface ClipboardHistorySecretRejectionConfig {
  extraBlockedSourceApps?: string[];
  extraSecretPatterns?: string[];
}

interface ClipboardHistoryPostCopyMenuConfig {
  enabled?: boolean;
  tapWindowMs?: number;
  triggerModifiers?: KeyModifier[];
}

interface Config {
  hotkey: HotkeyConfig;
  bun_path?: string;
  editor?: string;
  padding?: ContentPadding;
  editorFontSize?: number;
  terminalFontSize?: number;
  uiScale?: number;
  builtIns?: BuiltInConfig;
  clipboardHistoryMaxTextLength?: number;
  clipboardHistorySecretRejection?: ClipboardHistorySecretRejectionConfig;
  clipboardHistoryPostCopyMenu?: ClipboardHistoryPostCopyMenuConfig;
  processLimits?: ProcessLimits;
  suggested?: SuggestedConfig;
  notesHotkey?: HotkeyConfig;
  aiHotkey?: HotkeyConfig;
  aiHotkeyEnabled?: boolean;
  logsHotkey?: HotkeyConfig;
  logsHotkeyEnabled?: boolean;
  dictationHotkey?: HotkeyConfig;
  dictationHotkeyEnabled?: boolean;
  watcher?: WatcherConfig;
  layout?: LayoutConfig;
  theme?: ThemeSelectionPreferences;
  designs?: DesignsConfig;
  dictation?: DictationPreferences;
  ai?: AiPreferences;
  windowManagement?: WindowManagementPreferences;
  windowAppearance?: WindowAppearanceConfig;
  commands?: Record<string, CommandConfig>;
  promptTargets?: Record<string, PromptTargetConfig>;
  unifiedSearch?: UnifiedSearchConfig;
  claudeCode?: ClaudeCodeConfig;
  mcp?: McpConfig;
  powerSyntax?: PowerSyntaxConfig;
  tray?: TrayConfig;
  updates?: UpdatesConfig;
}

type Cmd1Behavior = "picker" | "cycle";
type DesignDensityChoice = "compact" | "comfortable" | "spacious";
type FontFamilyChoice = "system" | "monospace" | "serif";
type VibrancyChoice = "none" | "light" | "medium" | "heavy";
type ChromeOpacityChoice = "low" | "med" | "high";
type IconStyleChoice = "mono" | "color" | "hidden";
type SeparatorStyleChoice = "none" | "hairline" | "rule" | "grid";

interface DesignOverrides {
  accent?: string;
  density?: DesignDensityChoice;
  fontFamily?: FontFamilyChoice;
  fontScale?: number;
  vibrancy?: VibrancyChoice;
  chromeOpacity?: ChromeOpacityChoice;
  iconStyle?: IconStyleChoice;
  separatorStyle?: SeparatorStyleChoice;
  rowHeightNudge?: number;
}

interface DesignsConfig {
  activeId?: string;
  cmd1Behavior?: Cmd1Behavior;
  overrides?: Record<string, DesignOverrides>;
}

// =============================================================================
// Default Values (matching src/config.rs)
// =============================================================================

const DEFAULTS: Config & Record<string, unknown> = {
  hotkey: {
    modifiers: ["meta"],
    key: "Semicolon"
  },
  bun_path: "",  // Empty means auto-detect
  editor: "code",
  padding: {
    top: 8,
    left: 12,
    right: 12
  },
  editorFontSize: 16,
  terminalFontSize: 14,
  uiScale: 1.0,
  builtIns: {
    clipboardHistory: true,
    appLauncher: true,
    windowSwitcher: true
  },
  clipboardHistoryMaxTextLength: 100000,
  processLimits: {
    maxMemoryMb: undefined,
    maxRuntimeSeconds: undefined,
    healthCheckIntervalMs: 5000
  },
  suggested: {
    enabled: true,
    maxItems: 10,
    minScore: 0.1,
    halfLifeDays: 7,
    trackUsage: true,
    excludedCommands: ["builtin/quit-script-kit"],
  },
  aiHotkeyEnabled: true,
  logsHotkeyEnabled: true,
  dictationHotkey: { modifiers: ["meta", "shift"], key: "Semicolon" },
  dictationHotkeyEnabled: true,
  watcher: {
    debounceMs: 500,
    stormThreshold: 200,
    initialBackoffMs: 100,
    maxBackoffMs: 30000,
    maxNotifyErrors: 10
  },
  layout: {
    standardHeight: 500,
    maxHeight: 700
  },
  theme: undefined,
  dictation: undefined,
  ai: undefined,
  windowManagement: undefined,
  claudeCode: {
    enabled: false,
    path: undefined,
    permissionMode: "plan",
    allowedTools: undefined,
    addDirs: []
  },
  mcp: {
    enabled: true,
    servers: {}
  }
};

// =============================================================================
// Config Schema for Documentation
// =============================================================================

interface ConfigOption {
  key: string;
  type: string;
  default: unknown;
  description: string;
  example?: string;
}

const CONFIG_SCHEMA: ConfigOption[] = [
  {
    key: "hotkey.modifiers",
    type: "KeyModifier[]",
    default: ["meta"],
    description: "Modifier keys for global hotkey (meta, ctrl, alt, shift)",
    example: '["meta", "shift"]'
  },
  {
    key: "hotkey.key",
    type: "KeyCode",
    default: "Semicolon",
    description: "Main key for global hotkey (KeyA-KeyZ, Digit0-Digit9, Space, Enter, Semicolon, Comma, Period, Slash, F1-F12)",
    example: "KeyK"
  },
  {
    key: "bun_path",
    type: "string",
    default: "",
    description: "Custom path to bun executable (empty = auto-detect)",
    example: "/opt/homebrew/bin/bun"
  },
  {
    key: "editor",
    type: "string",
    default: "code",
    description: "Editor command for 'Open in Editor' actions",
    example: "vim"
  },
  {
    key: "padding.top",
    type: "number",
    default: 8,
    description: "Top padding in pixels for content areas"
  },
  {
    key: "padding.left",
    type: "number",
    default: 12,
    description: "Left padding in pixels for content areas"
  },
  {
    key: "padding.right",
    type: "number",
    default: 12,
    description: "Right padding in pixels for content areas"
  },
  {
    key: "editorFontSize",
    type: "number",
    default: 16,
    description: "Font size for editor prompt in pixels"
  },
  {
    key: "terminalFontSize",
    type: "number",
    default: 14,
    description: "Font size for terminal prompt in pixels"
  },
  {
    key: "uiScale",
    type: "number",
    default: 1.0,
    description: "UI scale factor (1.0 = 100%)"
  },
  {
    key: "builtIns.clipboardHistory",
    type: "boolean",
    default: true,
    description: "Enable clipboard history built-in feature"
  },
  {
    key: "builtIns.appLauncher",
    type: "boolean",
    default: true,
    description: "Enable app launcher built-in feature"
  },
  {
    key: "builtIns.windowSwitcher",
    type: "boolean",
    default: true,
    description: "Enable window switcher built-in feature"
  },
  {
    key: "clipboardHistoryMaxTextLength",
    type: "number",
    default: 100000,
    description: "Maximum text length (bytes) to store for clipboard history entries (0 = no limit)"
  },
  {
    key: "clipboardHistorySecretRejection.extraBlockedSourceApps",
    type: "string[]",
    default: [],
    description:
      "Additional bundle ID prefixes whose clipboard copies are never stored (hard secret rejection; defaults cover 1Password, Bitwarden, KeePassXC, Apple Passwords, Keychain Access)"
  },
  {
    key: "clipboardHistorySecretRejection.extraSecretPatterns",
    type: "string[]",
    default: [],
    description:
      "Additional regex patterns for secret-shaped clipboard text rejected before storage (conservative built-in defaults always apply)"
  },
  {
    key: "clipboardHistoryPostCopyMenu.enabled",
    type: "boolean",
    default: true,
    description: "Enable post-copy modifier-tap quick menu for annotate/reject (T12)"
  },
  {
    key: "clipboardHistoryPostCopyMenu.tapWindowMs",
    type: "number",
    default: 2500,
    description: "Milliseconds to watch for a bare modifier tap after copy before the quick menu window expires"
  },
  {
    key: "clipboardHistoryPostCopyMenu.triggerModifiers",
    type: "string[]",
    default: ["meta"],
    description: "Modifier keys that open the post-copy quick menu (default meta = Command)"
  },
  {
    key: "processLimits.maxMemoryMb",
    type: "number | undefined",
    default: undefined,
    description: "Maximum memory usage in MB (undefined = no limit)"
  },
  {
    key: "processLimits.maxRuntimeSeconds",
    type: "number | undefined",
    default: undefined,
    description: "Maximum runtime in seconds (undefined = no limit)"
  },
  {
    key: "processLimits.healthCheckIntervalMs",
    type: "number",
    default: 5000,
    description: "Health check interval in milliseconds"
  },
  // --- Auxiliary hotkeys ---
  {
    key: "notesHotkey",
    type: "HotkeyConfig",
    default: undefined,
    description: "Hotkey for opening the Notes window (no default; set explicitly)"
  },
  {
    key: "aiHotkey",
    type: "HotkeyConfig",
    default: undefined,
    description: "Hotkey for opening the AI chat (defaults to Cmd+Shift+Space when enabled)"
  },
  {
    key: "aiHotkeyEnabled",
    type: "boolean",
    default: true,
    description: "Whether the AI hotkey is registered"
  },
  {
    key: "logsHotkey",
    type: "HotkeyConfig",
    default: undefined,
    description: "Hotkey for toggling log capture (defaults to Cmd+Shift+L when enabled)"
  },
  {
    key: "logsHotkeyEnabled",
    type: "boolean",
    default: true,
    description: "Whether the logs hotkey is registered"
  },
  {
    key: "dictationHotkey",
    type: "HotkeyConfig",
    default: { modifiers: ["meta", "shift"], key: "Semicolon" },
    description: "Hotkey for toggling dictation (defaults to Cmd+Shift+; when enabled)"
  },
  {
    key: "dictationHotkeyEnabled",
    type: "boolean",
    default: true,
    description: "Whether the dictation hotkey is registered"
  },
  // --- Suggested ---
  {
    key: "suggested.enabled",
    type: "boolean",
    default: true,
    description: "Enable the Suggested section in the main menu"
  },
  {
    key: "suggested.maxItems",
    type: "number",
    default: 10,
    description: "Maximum number of suggested items shown"
  },
  {
    key: "suggested.minScore",
    type: "number",
    default: 0.1,
    description: "Minimum frecency score to include an item"
  },
  {
    key: "suggested.halfLifeDays",
    type: "number",
    default: 7,
    description: "Half-life (in days) for the frecency decay curve"
  },
  {
    key: "suggested.trackUsage",
    type: "boolean",
    default: true,
    description: "Track command usage for frecency scoring"
  },
  {
    key: "suggested.excludedCommands",
    type: "CommandId[]",
    default: ["builtin/quit-script-kit"],
    description: "Command IDs excluded from Suggested ranking"
  },
  // --- Watcher ---
  {
    key: "watcher.debounceMs",
    type: "number",
    default: 500,
    description: "File-watcher debounce interval in milliseconds"
  },
  {
    key: "watcher.stormThreshold",
    type: "number",
    default: 200,
    description: "Event count considered a storm within one debounce window"
  },
  {
    key: "watcher.initialBackoffMs",
    type: "number",
    default: 100,
    description: "Initial back-off delay in milliseconds after a storm"
  },
  {
    key: "watcher.maxBackoffMs",
    type: "number",
    default: 30000,
    description: "Maximum back-off delay in milliseconds"
  },
  {
    key: "watcher.maxNotifyErrors",
    type: "number",
    default: 10,
    description: "Max consecutive notify errors before the watcher stops"
  },
  // --- Layout ---
  {
    key: "layout.standardHeight",
    type: "number",
    default: 500,
    description: "Standard window height in pixels"
  },
  {
    key: "layout.maxHeight",
    type: "number",
    default: 700,
    description: "Maximum window height in pixels"
  },
  {
    key: "theme.presetId",
    type: "string | undefined",
    default: undefined,
    description: "Theme preset ID to apply before theme.json overrides"
  },
  {
    key: "dictation.selectedDeviceId",
    type: "string | undefined",
    default: undefined,
    description: "Preferred microphone device ID for dictation"
  },
  {
    key: "ai.selectedModelId",
    type: "string | undefined",
    default: undefined,
    description: "Last-selected Agent Chat model ID"
  },
  {
    key: "ai.selectedAgentChatAgentId",
    type: "string | undefined",
    default: undefined,
    description: "Legacy Agent Chat agent ID compatibility field"
  },
  {
    key: "windowManagement.snapMode",
    type: '"off" | "simple" | "expanded" | "precision" | undefined',
    default: undefined,
    description: "Drag-snap density for desktop tiling"
  },
  {
    key: "windowAppearance.vibrancy",
    type: '"default" | "none" | "hud" | "popover" | "sidebar" | undefined',
    default: undefined,
    description: "Schema-only window vibrancy material; runtime keeps built-in defaults until wired"
  },
  {
    key: "windowAppearance.animations",
    type: '"system" | "reduced" | "off" | undefined',
    default: undefined,
    description: "Schema-only show/hide animation mode; runtime keeps built-in defaults until wired"
  },
  {
    key: "powerSyntax.enabled",
    type: "boolean | undefined",
    default: undefined,
    description: "Schema-only Power Syntax master switch; runtime keeps built-in defaults until wired"
  },
  {
    key: "powerSyntax.captureSigil",
    type: '";" | "+" | "both" | undefined',
    default: undefined,
    description: 'Schema-only capture sigil preference ("both" keeps legacy + and prefers ;)',
    example: '"both"'
  },
  {
    key: "powerSyntax.commandSigil",
    type: '">" | "disabled" | undefined',
    default: undefined,
    description: 'Schema-only command invocation sigil preference ("disabled" turns off > invocation once wired)',
    example: '">"'
  },
  {
    key: "powerSyntax.cmdEnterAi.enabled",
    type: "boolean | undefined",
    default: undefined,
    description: "Schema-only Cmd+Enter AI enablement for Power Syntax"
  },
  {
    key: "powerSyntax.cmdEnterAi.modelId",
    type: "string | undefined",
    default: undefined,
    description: "Schema-only Cmd+Enter AI model override; falls back to active Agent Chat model"
  },
  {
    key: "powerSyntax.cmdEnterAi.systemPrompt",
    type: "string | undefined",
    default: undefined,
    description: "Schema-only Cmd+Enter AI system prompt override; falls back to active Agent Chat model"
  },
  {
    key: "tray.showCurrentAppCommands",
    type: "boolean | undefined",
    default: undefined,
    description: "Schema-only tray visibility for the dynamic <App> Commands header row; default true once wired"
  },
  {
    key: "tray.showNotes",
    type: "boolean | undefined",
    default: undefined,
    description: "Schema-only tray visibility for Open Notes; default true once wired"
  },
  {
    key: "tray.showAgentChat",
    type: "boolean | undefined",
    default: undefined,
    description: "Schema-only tray visibility for Open Agent Chat; default true once wired"
  },
  {
    key: "tray.showReloadScripts",
    type: "boolean | undefined",
    default: undefined,
    description: "Schema-only tray visibility for Reload Scripts; default true once wired"
  },
  {
    key: "tray.showHelp",
    type: "boolean | undefined",
    default: undefined,
    description: "Schema-only tray visibility for Help/Feedback; default true once wired"
  },
  {
    key: "tray.showSocialLinks",
    type: "boolean | undefined",
    default: undefined,
    description: "Schema-only tray visibility for Follow Us, GitHub, and Discord; default true once wired"
  },
  {
    key: "tray.showUpdateCheck",
    type: "boolean | undefined",
    default: undefined,
    description: "Schema-only tray visibility for Check for Updates and Version rows; default true once wired"
  },
  {
    key: "updates.autoCheck",
    type: "boolean | undefined",
    default: undefined,
    description: "Schema-only update auto-check preference; default true once wired"
  },
  // --- Commands & Claude Code ---
  {
    key: "commands",
    type: "Record<string, CommandConfig>",
    default: undefined,
    description: "Per-command shortcuts and visibility overrides"
  },
  {
    key: "promptTargets",
    type: "Record<string, PromptTargetConfig>",
    default: undefined,
    description: "Prompt handoff targets surfaced as prompt-target/<id> Actions and shortcut commands; built-in prompt actions use prompt-action/<id>"
  },
  {
    key: "claudeCode.enabled",
    type: "boolean",
    default: false,
    description: "Enable the Claude Code CLI provider"
  },
  {
    key: "claudeCode.path",
    type: "string | undefined",
    default: undefined,
    description: "Custom path to the claude executable"
  },
  {
    key: "claudeCode.permissionMode",
    type: '"plan" | "dontAsk"',
    default: "plan",
    description: 'Claude Code tool permission mode ("plan" asks first, "dontAsk" auto-runs tools)',
    example: '"plan"'
  },
  {
    key: "claudeCode.allowedTools",
    type: "string | undefined",
    default: undefined,
    description: "Comma-separated list of allowed Claude Code tools",
    example: '"Read,Edit,Bash(git:*)"'
  },
  {
    key: "claudeCode.addDirs",
    type: "string[]",
    default: [],
    description: "Additional directories passed to Claude Code with --add-dir",
    example: '["/Users/you/projects"]'
  }
];

// =============================================================================
// Utilities
// =============================================================================

const CONFIG_PATH = process.env.SCRIPT_KIT_CONFIG_PATH
  || path.join(os.homedir(), '.scriptkit', 'config.ts');

interface Result<T> {
  success: boolean;
  data?: T;
  error?: string;
  valid?: boolean;
  errors?: string[];
  warnings?: string[];
}

function output(result: Result<unknown>): void {
  console.log(JSON.stringify(result, null, 2));
}

function success<T>(data: T): void {
  output({ success: true, data });
}

function error(message: string): void {
  output({ success: false, error: message });
  process.exit(1);
}

/**
 * Get a nested value from an object using dot notation
 */
function getNestedValue(obj: Record<string, unknown>, key: string): unknown {
  const parts = key.split('.');
  let current: unknown = obj;
  
  for (const part of parts) {
    if (current === null || current === undefined) {
      return undefined;
    }
    if (typeof current !== 'object') {
      return undefined;
    }
    current = (current as Record<string, unknown>)[part];
  }
  
  return current;
}

/**
 * Set a nested value in an object using dot notation
 */
function setNestedValue(obj: Record<string, unknown>, key: string, value: unknown): void {
  const parts = key.split('.');
  let current = obj;
  
  for (let i = 0; i < parts.length - 1; i++) {
    const part = parts[i];
    if (!(part in current) || typeof current[part] !== 'object' || current[part] === null) {
      current[part] = {};
    }
    current = current[part] as Record<string, unknown>;
  }
  
  const lastPart = parts[parts.length - 1];
  current[lastPart] = value;
}

/**
 * Delete a nested value from an object using dot notation
 */
function deleteNestedValue(obj: Record<string, unknown>, key: string): boolean {
  const parts = key.split('.');
  let current = obj;
  
  for (let i = 0; i < parts.length - 1; i++) {
    const part = parts[i];
    if (!(part in current) || typeof current[part] !== 'object' || current[part] === null) {
      return false;
    }
    current = current[part] as Record<string, unknown>;
  }
  
  const lastPart = parts[parts.length - 1];
  if (lastPart in current) {
    delete current[lastPart];
    return true;
  }
  return false;
}

/**
 * Load and parse the current config
 */
async function loadConfig(): Promise<Config | null> {
  if (!fs.existsSync(CONFIG_PATH)) {
    return null;
  }
  
  // Use bun to transpile and evaluate the config
  const tmpJsPath = '/tmp/kit-config-cli.js';
  
  try {
    // Transpile TypeScript to JavaScript
    const buildResult = Bun.spawnSync(['bun', 'build', '--target=bun', CONFIG_PATH, `--outfile=${tmpJsPath}`]);
    if (buildResult.exitCode !== 0) {
      throw new Error(`Failed to transpile config: ${buildResult.stderr.toString()}`);
    }
    
    // Execute and extract default export
    const jsonResult = Bun.spawnSync(['bun', '-e', `console.log(JSON.stringify(require('${tmpJsPath}').default))`]);
    if (jsonResult.exitCode !== 0) {
      throw new Error(`Failed to evaluate config: ${jsonResult.stderr.toString()}`);
    }
    
    const jsonStr = jsonResult.stdout.toString().trim();
    return JSON.parse(jsonStr) as Config;
  } catch (e) {
    throw new Error(`Failed to load config: ${e instanceof Error ? e.message : String(e)}`);
  }
}

/**
 * Read the raw config.ts file content
 */
function readConfigFile(): string | null {
  if (!fs.existsSync(CONFIG_PATH)) {
    return null;
  }
  return fs.readFileSync(CONFIG_PATH, 'utf-8');
}

/**
 * Write the config.ts file content
 */
function writeConfigFile(content: string): void {
  const dir = path.dirname(CONFIG_PATH);
  if (!fs.existsSync(dir)) {
    fs.mkdirSync(dir, { recursive: true });
  }
  fs.writeFileSync(CONFIG_PATH, content, 'utf-8');
}

/**
 * Create a default config file
 */
function createDefaultConfig(): string {
  return `import type { Config } from "@scriptkit/sdk";

export default {
  hotkey: {
    modifiers: ["meta"],
    key: "Semicolon"
  }
} satisfies Config;
`;
}

/**
 * Parse a string value to the appropriate type
 */
function parseValue(value: string, key: string): unknown {
  // Find the schema entry for this key to determine type
  const schema = CONFIG_SCHEMA.find(s => s.key === key);
  if (!schema) {
    // Try to infer type from value
    if (value === 'true') return true;
    if (value === 'false') return false;
    if (value === 'undefined' || value === 'null') return undefined;
    const num = Number(value);
    if (!isNaN(num)) return num;
    // Try to parse as JSON (for arrays)
    try {
      return JSON.parse(value);
    } catch {
      return value;
    }
  }
  
  const type = schema.type;

  if (type.includes('undefined') && (value === 'undefined' || value === 'null')) {
    return undefined;
  }
  
  if (type.includes('boolean')) {
    if (value === 'true') return true;
    if (value === 'false') return false;
    throw new Error(`Invalid boolean value: ${value}. Use 'true' or 'false'.`);
  }
  
  if (type.includes('number')) {
    if (value === 'undefined' || value === 'null') return undefined;
    const num = Number(value);
    if (isNaN(num)) {
      throw new Error(`Invalid number value: ${value}`);
    }
    return num;
  }
  
  if (type.includes('[]')) {
    // Array type - parse as JSON
    try {
      const parsed = JSON.parse(value);
      if (!Array.isArray(parsed)) {
        throw new Error(`Expected array, got: ${typeof parsed}`);
      }
      return parsed;
    } catch (e) {
      throw new Error(`Invalid array value: ${value}. Use JSON format like '["meta", "shift"]'.`);
    }
  }
  
  // String or other - return as-is
  return value;
}

/**
 * Validate a config value against constraints
 */
function validateValue(key: string, value: unknown): { valid: boolean; error?: string } {
  // Validate command-ID arrays before falling through to generic handling
  if (key === "suggested.excludedCommands") {
    const errors = validateCommandIdList(value, key);
    return {
      valid: errors.length === 0,
      error: errors.length === 0 ? undefined : errors.map((e) => e.message).join("; "),
    };
  }

  const schema = CONFIG_SCHEMA.find(s => s.key === key);
  if (!schema) {
    return { valid: true }; // Unknown key - allow but warn
  }
  
  // Type validation
  const type = schema.type;
  
  if (key === 'hotkey.modifiers') {
    if (!Array.isArray(value)) {
      return { valid: false, error: 'hotkey.modifiers must be an array' };
    }
    const validMods: KeyModifier[] = ['meta', 'ctrl', 'alt', 'shift'];
    for (const mod of value) {
      if (!validMods.includes(mod as KeyModifier)) {
        return { valid: false, error: `Invalid modifier: ${mod}. Valid modifiers: ${validMods.join(', ')}` };
      }
    }
  }

  if (key === 'windowManagement.snapMode' && value !== undefined) {
    const validModes: SnapMode[] = ['off', 'simple', 'expanded', 'precision'];
    if (!validModes.includes(value as SnapMode)) {
      return {
        valid: false,
        error: `windowManagement.snapMode must be one of: ${validModes.join(', ')}`,
      };
    }
  }

  if (key === 'windowAppearance.vibrancy' && value !== undefined) {
    const validMaterials: WindowVibrancyMaterial[] = ['default', 'none', 'hud', 'popover', 'sidebar'];
    if (!validMaterials.includes(value as WindowVibrancyMaterial)) {
      return {
        valid: false,
        error: `windowAppearance.vibrancy must be one of: ${validMaterials.join(', ')}`,
      };
    }
  }

  if (key === 'windowAppearance.animations' && value !== undefined) {
    const validModes: WindowAnimationMode[] = ['system', 'reduced', 'off'];
    if (!validModes.includes(value as WindowAnimationMode)) {
      return {
        valid: false,
        error: `windowAppearance.animations must be one of: ${validModes.join(', ')}`,
      };
    }
  }

  if (key === 'powerSyntax.captureSigil' && value !== undefined) {
    const validSigils: PowerSyntaxCaptureSigil[] = [';', '+', 'both'];
    if (!validSigils.includes(value as PowerSyntaxCaptureSigil)) {
      return {
        valid: false,
        error: `powerSyntax.captureSigil must be one of: ${validSigils.join(', ')}`,
      };
    }
  }

  if (key === 'powerSyntax.commandSigil' && value !== undefined) {
    const validSigils: PowerSyntaxCommandSigil[] = ['>', 'disabled'];
    if (!validSigils.includes(value as PowerSyntaxCommandSigil)) {
      return {
        valid: false,
        error: `powerSyntax.commandSigil must be one of: ${validSigils.join(', ')}`,
      };
    }
  }
  
  if (key === 'hotkey.key') {
    const validKeys = [
      'KeyA', 'KeyB', 'KeyC', 'KeyD', 'KeyE', 'KeyF', 'KeyG',
      'KeyH', 'KeyI', 'KeyJ', 'KeyK', 'KeyL', 'KeyM', 'KeyN',
      'KeyO', 'KeyP', 'KeyQ', 'KeyR', 'KeyS', 'KeyT', 'KeyU',
      'KeyV', 'KeyW', 'KeyX', 'KeyY', 'KeyZ',
      'Digit0', 'Digit1', 'Digit2', 'Digit3', 'Digit4',
      'Digit5', 'Digit6', 'Digit7', 'Digit8', 'Digit9',
      'Space', 'Enter', 'Semicolon',
      'F1', 'F2', 'F3', 'F4', 'F5', 'F6',
      'F7', 'F8', 'F9', 'F10', 'F11', 'F12'
    ];
    if (!validKeys.includes(value as string)) {
      return { valid: false, error: `Invalid key: ${value}. Valid keys: ${validKeys.join(', ')}` };
    }
  }
  
  if (type.includes('number') && value !== undefined && value !== null) {
    if (typeof value !== 'number' || isNaN(value)) {
      return { valid: false, error: `${key} must be a number` };
    }
    // Range validations
    if (key === 'uiScale' && (value < 0.5 || value > 3.0)) {
      return { valid: false, error: 'uiScale must be between 0.5 and 3.0' };
    }
    if (key.includes('FontSize') && (value < 8 || value > 72)) {
      return { valid: false, error: 'Font size must be between 8 and 72' };
    }
    if (key.includes('padding') && value < 0) {
      return { valid: false, error: 'Padding cannot be negative' };
    }
  }
  
  if (type === 'boolean' && typeof value !== 'boolean') {
    return { valid: false, error: `${key} must be a boolean` };
  }
  
  return { valid: true };
}

/**
 * Update a value in config.ts while preserving formatting
 * Uses regex-based replacement for simple cases
 */
function updateConfigValue(key: string, value: unknown): void {
  let content = readConfigFile();
  
  if (!content) {
    // Create new config file
    content = createDefaultConfig();
  }
  
  const parts = key.split('.');
  const valueStr = JSON.stringify(value);
  
  // Strategy: For nested keys, we need to find and update the specific property
  // This is a simplified approach - for complex cases, consider ts-morph
  
  if (parts.length === 1) {
    // Top-level key
    const keyName = parts[0];
    // Try to find existing key and replace - use a more flexible pattern
    // Match the key name followed by colon and value, stopping at comma, newline, or closing brace
    const existingKeyRegex = new RegExp(`(\\s*["']?${keyName}["']?\\s*:\\s*)([^,}\\n]+(?:\\{[^}]*\\})?)`, 'g');
    
    if (existingKeyRegex.test(content)) {
      // Reset lastIndex since test() advances it
      existingKeyRegex.lastIndex = 0;
      content = content.replace(existingKeyRegex, `$1${valueStr}`);
    } else {
      // Key doesn't exist - need to add it
      // Find the content before the closing } satisfies/as Config
      // We need to ensure there's a comma after the last property
      const insertRegex = /(\s*)(})\s*(satisfies|as)\s+Config/;
      const match = content.match(insertRegex);
      
      if (match) {
        // Find position to insert
        const beforeClose = content.slice(0, content.indexOf(match[0]));
        
        // Check if we need to add a comma after the last property
        // Look for the last non-whitespace character before the closing brace
        const trimmedBefore = beforeClose.trimEnd();
        const needsComma = !trimmedBefore.endsWith(',') && !trimmedBefore.endsWith('{');
        
        const commaIfNeeded = needsComma ? ',' : '';
        content = content.replace(insertRegex, `${commaIfNeeded}\n  ${keyName}: ${valueStr}\n$2 $3 Config`);
      }
    }
  } else if (parts.length === 2) {
    // Nested key (e.g., hotkey.key, padding.top)
    const [parent, child] = parts;
    
    // Check if parent object exists - use a pattern that captures nested braces properly
    const parentRegex = new RegExp(`(["']?${parent}["']?\\s*:\\s*)\\{([^}]*)\\}`, 's');
    const parentMatch = content.match(parentRegex);
    
    if (parentMatch) {
      // Parent exists - update or add the child property
      const parentContent = parentMatch[2];
      const childRegex = new RegExp(`(["']?${child}["']?\\s*:\\s*)([^,}\\n]+)`);
      
      if (childRegex.test(parentContent)) {
        // Child exists - update it
        const newParentContent = parentContent.replace(childRegex, `$1${valueStr}`);
        content = content.replace(parentRegex, `$1{${newParentContent}}`);
      } else {
        // Child doesn't exist - add it at the end of the parent object
        const trimmedContent = parentContent.trimEnd();
        const needsComma = !trimmedContent.endsWith(',') && trimmedContent.length > 0;
        const commaIfNeeded = needsComma ? ',' : '';
        const newParentContent = parentContent.trimEnd() + commaIfNeeded + `\n    ${child}: ${valueStr}`;
        content = content.replace(parentRegex, `$1{${newParentContent}\n  }`);
      }
    } else {
      // Parent doesn't exist - create it with the child
      // Same logic as top-level insertion but with nested object
      const insertRegex = /(\s*)(})\s*(satisfies|as)\s+Config/;
      const match = content.match(insertRegex);
      
      if (match) {
        const beforeClose = content.slice(0, content.indexOf(match[0]));
        const trimmedBefore = beforeClose.trimEnd();
        const needsComma = !trimmedBefore.endsWith(',') && !trimmedBefore.endsWith('{');
        const commaIfNeeded = needsComma ? ',' : '';
        
        content = content.replace(insertRegex, `${commaIfNeeded}\n  ${parent}: {\n    ${child}: ${valueStr}\n  }\n$2 $3 Config`);
      }
    }
  } else {
    throw new Error(`Deep nesting (${parts.length} levels) not supported. Max 2 levels.`);
  }
  
  writeConfigFile(content);
}

/**
 * Reset a value to default in config.ts
 */
function resetConfigValue(key: string): void {
  const defaultValue = getNestedValue(DEFAULTS as unknown as Record<string, unknown>, key);
  
  if (defaultValue === undefined && !CONFIG_SCHEMA.some(s => s.key === key)) {
    throw new Error(`Unknown config key: ${key}`);
  }
  
  // For optional fields with undefined default, we remove the key
  if (defaultValue === undefined || defaultValue === '') {
    // Remove the key from config
    let content = readConfigFile();
    if (!content) {
      return; // Nothing to reset
    }
    
    const parts = key.split('.');
    if (parts.length === 1) {
      // Remove top-level key
      const regex = new RegExp(`\\s*["']?${parts[0]}["']?\\s*:\\s*[^,}\\n]+,?\\n?`, 'g');
      content = content.replace(regex, '');
    } else if (parts.length === 2) {
      // Remove nested key
      const regex = new RegExp(`\\s*["']?${parts[1]}["']?\\s*:\\s*[^,}\\n]+,?`, 'g');
      content = content.replace(regex, '');
    }
    
    writeConfigFile(content);
  } else {
    // Set to default value
    updateConfigValue(key, defaultValue);
  }
}

function normalizeShortcutKey(key: string): KeyCode {
  const trimmed = key.trim();
  if (/^[a-z]$/i.test(trimmed)) {
    return `Key${trimmed.toUpperCase()}` as KeyCode;
  }
  if (/^[0-9]$/.test(trimmed)) {
    return `Digit${trimmed}` as KeyCode;
  }

  const lower = trimmed.toLowerCase();
  const aliases: Record<string, KeyCode> = {
    space: "Space",
    enter: "Enter",
    return: "Enter",
    tab: "Tab",
    escape: "Escape",
    esc: "Escape",
    backspace: "Backspace",
    delete: "Delete",
    del: "Delete",
    up: "ArrowUp",
    arrowup: "ArrowUp",
    down: "ArrowDown",
    arrowdown: "ArrowDown",
    left: "ArrowLeft",
    arrowleft: "ArrowLeft",
    right: "ArrowRight",
    arrowright: "ArrowRight",
    home: "Home",
    end: "End",
    pageup: "PageUp",
    pgup: "PageUp",
    pagedown: "PageDown",
    pgdn: "PageDown",
    insert: "Insert",
    semicolon: "Semicolon",
    ";": "Semicolon",
    comma: "Comma",
    ",": "Comma",
    period: "Period",
    ".": "Period",
    slash: "Slash",
    "/": "Slash",
    quote: "Quote",
    "'": "Quote",
    backslash: "Backslash",
    "\\": "Backslash",
    bracketleft: "BracketLeft",
    "[": "BracketLeft",
    bracketright: "BracketRight",
    "]": "BracketRight",
    minus: "Minus",
    "-": "Minus",
    equal: "Equal",
    "=": "Equal",
    backquote: "Backquote",
    "`": "Backquote",
  };

  if (aliases[lower]) {
    return aliases[lower];
  }
  if (/^f([1-9]|1[0-2])$/i.test(trimmed)) {
    return trimmed.toUpperCase() as KeyCode;
  }

  const validKeys = new Set<string>([
    "KeyA", "KeyB", "KeyC", "KeyD", "KeyE", "KeyF", "KeyG",
    "KeyH", "KeyI", "KeyJ", "KeyK", "KeyL", "KeyM", "KeyN",
    "KeyO", "KeyP", "KeyQ", "KeyR", "KeyS", "KeyT", "KeyU",
    "KeyV", "KeyW", "KeyX", "KeyY", "KeyZ",
    "Digit0", "Digit1", "Digit2", "Digit3", "Digit4",
    "Digit5", "Digit6", "Digit7", "Digit8", "Digit9",
    "Space", "Enter", "Semicolon", "Comma", "Period", "Slash",
    "Tab", "Escape", "Backspace", "Delete",
    "ArrowUp", "ArrowDown", "ArrowLeft", "ArrowRight",
    "Home", "End", "PageUp", "PageDown", "Insert",
    "Quote", "Backslash", "BracketLeft", "BracketRight",
    "Minus", "Equal", "Backquote",
    "F1", "F2", "F3", "F4", "F5", "F6",
    "F7", "F8", "F9", "F10", "F11", "F12",
  ]);

  if (validKeys.has(trimmed)) {
    return trimmed as KeyCode;
  }

  throw new Error(`Invalid shortcut key: ${key}`);
}

function findCommandsPropertyRange(content: string): [number, number] | null {
  const match = content.match(/(^|\n)([ \t]*)commands\s*:\s*\{/);
  if (!match || match.index === undefined) {
    return null;
  }

  const start = match.index + match[1].length;
  const openBrace = content.indexOf("{", start);
  let depth = 0;
  let inString: string | null = null;
  let escaped = false;

  for (let i = openBrace; i < content.length; i++) {
    const char = content[i];
    if (inString) {
      if (escaped) {
        escaped = false;
      } else if (char === "\\") {
        escaped = true;
      } else if (char === inString) {
        inString = null;
      }
      continue;
    }

    if (char === '"' || char === "'" || char === "`") {
      inString = char;
      continue;
    }
    if (char === "{") {
      depth += 1;
    } else if (char === "}") {
      depth -= 1;
      if (depth === 0) {
        let end = i + 1;
        while (content[end] === " " || content[end] === "\t") end += 1;
        if (content[end] === ",") end += 1;
        return [start, end];
      }
    }
  }

  return null;
}

function formatCommandConfig(config: CommandConfig): string {
  const fields: string[] = [];
  if (config.shortcut) {
    fields.push(`shortcut: ${JSON.stringify(config.shortcut)}`);
  }
  if (config.hidden !== undefined) {
    fields.push(`hidden: ${JSON.stringify(config.hidden)}`);
  }
  if (config.confirmationRequired !== undefined) {
    fields.push(`confirmationRequired: ${JSON.stringify(config.confirmationRequired)}`);
  }
  return `{ ${fields.join(", ")} }`;
}

function formatCommandsProperty(commands: Record<string, CommandConfig>): string {
  const entries = Object.entries(commands).sort(([a], [b]) => a.localeCompare(b));
  if (entries.length === 0) {
    return "";
  }
  const lines = entries.map(
    ([commandId, commandConfig]) =>
      `    ${JSON.stringify(commandId)}: ${formatCommandConfig(commandConfig)},`,
  );
  return `  commands: {\n${lines.join("\n")}\n  },`;
}

function writeCommandsConfig(commands: Record<string, CommandConfig>): void {
  let content = readConfigFile() ?? createDefaultConfig();
  const formatted = formatCommandsProperty(commands);
  const range = findCommandsPropertyRange(content);

  if (range) {
    const prefix = content.slice(0, range[0]);
    const suffix = content.slice(range[1]);
    content = formatted
      ? `${prefix}${formatted}${suffix}`
      : `${prefix}${suffix.replace(/^\n?/, "")}`;
  } else if (formatted) {
    const insertRegex = /(\s*)(})\s*(satisfies|as)\s+Config/;
    const match = content.match(insertRegex);
    if (!match) {
      throw new Error("Could not find config export object terminator");
    }
    const beforeClose = content.slice(0, content.indexOf(match[0]));
    const needsComma = !beforeClose.trimEnd().endsWith(",") && !beforeClose.trimEnd().endsWith("{");
    content = content.replace(insertRegex, `${needsComma ? "," : ""}\n${formatted}\n$2 $3 Config`);
  }

  writeConfigFile(content);
}

async function getMutableCommandsConfig(): Promise<Record<string, CommandConfig>> {
  const config = await loadConfig();
  return { ...(config?.commands ?? {}) };
}

// =============================================================================
// Commands
// =============================================================================

async function cmdGet(key?: string): Promise<void> {
  try {
    const config = await loadConfig();
    
    if (!config) {
      success({
        exists: false,
        path: CONFIG_PATH,
        message: "Config file does not exist. Using defaults.",
        config: DEFAULTS
      });
      return;
    }
    
    if (key) {
      const value = getNestedValue(config as unknown as Record<string, unknown>, key);
      const defaultValue = getNestedValue(DEFAULTS as unknown as Record<string, unknown>, key);
      
      success({
        key,
        value: value ?? defaultValue,
        isDefault: value === undefined,
        default: defaultValue
      });
    } else {
      // Return full config merged with defaults
      const merged = { ...DEFAULTS, ...config };
      success({
        path: CONFIG_PATH,
        config: merged
      });
    }
  } catch (e) {
    error(e instanceof Error ? e.message : String(e));
  }
}

async function cmdSet(key: string, value: string): Promise<void> {
  if (!key || value === undefined) {
    error("Usage: bun scripts/config-cli.ts set <key> <value>");
  }
  
  try {
    // Parse the value
    const parsedValue = parseValue(value, key);
    
    // Validate the value
    const validation = validateValue(key, parsedValue);
    if (!validation.valid) {
      error(validation.error!);
    }
    
    // Update the config file
    updateConfigValue(key, parsedValue);
    
    // Read back to verify
    const config = await loadConfig();
    const newValue = config ? getNestedValue(config as unknown as Record<string, unknown>, key) : parsedValue;
    
    success({
      key,
      value: newValue,
      message: `Successfully set ${key} to ${JSON.stringify(parsedValue)}`
    });
  } catch (e) {
    error(e instanceof Error ? e.message : String(e));
  }
}

async function cmdSetCommandShortcut(args: string[]): Promise<void> {
  const [commandId, key, cmdStr, ctrlStr, altStr, shiftStr, ...rest] = args;
  if (!commandId || !key || cmdStr === undefined || ctrlStr === undefined || altStr === undefined || shiftStr === undefined) {
    error("Usage: bun scripts/config-cli.ts set-command-shortcut <command_id> <key> <cmd> <ctrl> <alt> <shift> [--skip-existing]");
  }

  const shortcut: HotkeyConfig = {
    key: normalizeShortcutKey(key),
    modifiers: [
      cmdStr === "true" ? "meta" : undefined,
      ctrlStr === "true" ? "ctrl" : undefined,
      altStr === "true" ? "alt" : undefined,
      shiftStr === "true" ? "shift" : undefined,
    ].filter(Boolean) as KeyModifier[],
  };

  const validation = validateCommandConfigValue({
    shortcut,
  }, `commands.${commandId}`);
  if (validation.length > 0) {
    error(validation.map((entry) => entry.message).join("; "));
  }

  const commands = await getMutableCommandsConfig();
  const existing = commands[commandId] ?? {};
  if (rest.includes("--skip-existing") && existing.shortcut) {
    success({
      commandId,
      skipped: true,
      shortcut: existing.shortcut,
      message: `Skipped ${commandId}; config.ts already has a shortcut`,
    });
    return;
  }

  commands[commandId] = {
    ...existing,
    shortcut,
  };
  writeCommandsConfig(commands);

  success({
    commandId,
    shortcut,
    message: `Successfully set shortcut for ${commandId}`,
  });
}

async function cmdRemoveCommandShortcut(commandId: string): Promise<void> {
  if (!commandId) {
    error("Usage: bun scripts/config-cli.ts remove-command-shortcut <command_id>");
  }

  const commands = await getMutableCommandsConfig();
  const existing = commands[commandId];
  if (!existing || !existing.shortcut) {
    success({
      commandId,
      removed: false,
      message: `No shortcut configured for ${commandId}`,
    });
    return;
  }

  const next: CommandConfig = { ...existing };
  delete next.shortcut;
  if (
    next.hidden === undefined &&
    next.confirmationRequired === undefined
  ) {
    delete commands[commandId];
  } else {
    commands[commandId] = next;
  }

  writeCommandsConfig(commands);

  success({
    commandId,
    removed: true,
    message: `Successfully removed shortcut for ${commandId}`,
  });
}

async function cmdList(): Promise<void> {
  try {
    const config = await loadConfig();
    
    const options = CONFIG_SCHEMA.map(schema => {
      const currentValue = config 
        ? getNestedValue(config as unknown as Record<string, unknown>, schema.key)
        : undefined;
      const effectiveValue = currentValue ?? schema.default;
      
      return {
        key: schema.key,
        type: schema.type,
        current: effectiveValue,
        default: schema.default,
        isCustom: currentValue !== undefined && currentValue !== schema.default,
        description: schema.description,
        example: schema.example
      };
    });
    
    success({
      path: CONFIG_PATH,
      exists: config !== null,
      options
    });
  } catch (e) {
    error(e instanceof Error ? e.message : String(e));
  }
}

async function cmdValidate(): Promise<void> {
  try {
    // Check if file exists
    if (!fs.existsSync(CONFIG_PATH)) {
      success({
        valid: true,
        exists: false,
        message: "Config file does not exist. Default config will be used."
      });
      return;
    }
    
    // Try to load and parse the config
    const config = await loadConfig();
    
    if (!config) {
      error("Failed to parse config file");
    }
    
    // Validate required fields
    const errors: string[] = [];
    const warnings: string[] = [];
    
    if (!config.hotkey) {
      errors.push("Missing required field: hotkey");
    } else {
      if (!config.hotkey.modifiers) {
        errors.push("Missing required field: hotkey.modifiers");
      }
      if (!config.hotkey.key) {
        errors.push("Missing required field: hotkey.key");
      }
    }
    
    // Validate all present values
    const configRecord = config as unknown as Record<string, unknown>;
    for (const schema of CONFIG_SCHEMA) {
      const value = getNestedValue(configRecord, schema.key);
      if (value !== undefined) {
        const validation = validateValue(schema.key, value);
        if (!validation.valid) {
          errors.push(validation.error!);
        }
      }
    }
    
    // Check for unknown keys
    const knownTopLevel = [
      'hotkey', 'bun_path', 'editor', 'padding', 'editorFontSize',
      'terminalFontSize', 'uiScale', 'builtIns', 'processLimits',
      'clipboardHistoryMaxTextLength', 'suggested', 'notesHotkey',
      'aiHotkey', 'aiHotkeyEnabled', 'logsHotkey', 'logsHotkeyEnabled',
      'dictationHotkey', 'dictationHotkeyEnabled', 'watcher', 'layout',
      'theme', 'dictation', 'ai', 'windowManagement', 'windowAppearance',
      'commands', 'claudeCode', 'mcp', 'powerSyntax', 'tray', 'updates',
    ];
    for (const key of Object.keys(config)) {
      if (!knownTopLevel.includes(key)) {
        warnings.push(`Unknown config key: ${key}`);
      }
    }
    
    if (errors.length > 0) {
      output({
        success: false,
        valid: false,
        errors,
        warnings: warnings.length > 0 ? warnings : undefined
      });
      process.exit(1);
    }
    
    success({
      valid: true,
      message: "Config is valid",
      warnings: warnings.length > 0 ? warnings : undefined
    });
  } catch (e) {
    output({
      success: false,
      valid: false,
      errors: [e instanceof Error ? e.message : String(e)]
    });
    process.exit(1);
  }
}

async function cmdReset(key?: string): Promise<void> {
  try {
    if (key) {
      // Reset specific key
      resetConfigValue(key);
      const defaultValue = getNestedValue(DEFAULTS as unknown as Record<string, unknown>, key);
      
      success({
        key,
        value: defaultValue,
        message: `Reset ${key} to default value`
      });
    } else {
      // Reset entire config
      const content = createDefaultConfig();
      writeConfigFile(content);
      
      success({
        message: "Reset config to defaults",
        config: {
          hotkey: DEFAULTS.hotkey
        }
      });
    }
  } catch (e) {
    error(e instanceof Error ? e.message : String(e));
  }
}

function debugLog(event: string, details: Record<string, unknown> = {}): void {
  if (process.env.SCRIPT_KIT_CONFIG_DEBUG !== '1') {
    return;
  }
  process.stderr.write(
    `[config-cli] ${JSON.stringify({ event, ...details })}\n`,
  );
}

function validateConfigChange(change: ConfigChange): ValidateConfigChangeResult {
  if (change.key === "commands") {
    const result = validateCommandsConfig(change.value);
    debugLog('validate_change_commands_root', {
      key: change.key,
      valid: result.valid,
      errorCount: result.errors.length,
    });
    return result;
  }

  const commandPath = analyzeCommandConfigPath(change.key);

  if (commandPath?.kind === 'parsed') {
    const errors = commandPath.fieldPath
      ? validateCommandConfigFieldValue(
          commandPath.fieldPath,
          change.value,
          change.key,
        )
      : validateCommandConfigValue(change.value, change.key);

    debugLog('validate_change_command_path', {
      key: change.key,
      commandId: commandPath.commandId,
      fieldPath: commandPath.fieldPath ?? null,
      valid: errors.length === 0,
      errorCount: errors.length,
    });

    return {
      valid: errors.length === 0,
      normalizedValue: errors.length === 0 ? change.value : undefined,
      errors,
      warnings: [],
    };
  }

  if (commandPath?.kind === 'invalidCommandId') {
    debugLog('validate_change_invalid_command_id', {
      key: change.key,
      rawCommandId: commandPath.rawCommandId,
      fieldPath: commandPath.fieldPath ?? null,
    });

    return {
      valid: false,
      errors: [{
        path: change.key,
        code: 'invalidCommandId',
        message: `Invalid command id: ${commandPath.rawCommandId}`,
      }],
      warnings: [],
    };
  }

  if (commandPath?.kind === 'invalidCommandPath') {
    debugLog('validate_change_invalid_command_path', {
      key: change.key,
    });

    return {
      valid: false,
      errors: [{
        path: change.key,
        code: 'invalidCommandPath',
        message: `Invalid commands path: ${change.key}`,
      }],
      warnings: [],
    };
  }

  // Validate command-ID arrays (e.g. suggested.excludedCommands)
  if (change.key === "suggested.excludedCommands") {
    const errors = validateCommandIdList(change.value, change.key);
    debugLog('validate_change_command_id_list', {
      key: change.key,
      valid: errors.length === 0,
      errorCount: errors.length,
    });
    return {
      valid: errors.length === 0,
      normalizedValue: errors.length === 0 ? change.value : undefined,
      errors,
      warnings: [],
    };
  }

  // Fall back to existing scalar validation for other keys
  const basic = validateValue(change.key, change.value);
  debugLog('validate_change_scalar', {
    key: change.key,
    valid: basic.valid,
    error: basic.error ?? null,
  });
  return {
    valid: basic.valid,
    normalizedValue: basic.valid ? change.value : undefined,
    errors: basic.valid
      ? []
      : [{
          path: change.key,
          code: "invalidValue",
          message: basic.error ?? `Invalid value for ${change.key}`,
        }],
    warnings: [],
  };
}

async function cmdValidateChange(payload: string): Promise<void> {
  let change: ConfigChange;
  try {
    change = JSON.parse(payload) as ConfigChange;
  } catch {
    error("Invalid JSON payload for validate-change");
    return;
  }

  const result = validateConfigChange(change);

  if (!result.valid) {
    output({ success: false, ...result });
    process.exit(1);
  }

  success(result);
}

function showHelp(): void {
  const help = `
Script Kit Config CLI

USAGE:
  bun scripts/config-cli.ts <command> [args]

COMMANDS:
  get [key]           Read a config value (or all values if no key specified)
  set <key> <value>   Set a config value
  set-command-shortcut <command_id> <key> <cmd> <ctrl> <alt> <shift>
                      Set a launcher command shortcut in config.ts
  remove-command-shortcut <command_id>
                      Remove only the shortcut field from a launcher command
  list                List all available config options with current values
  validate            Validate the current config file
  reset [key]         Reset a config value to default (or all values if no key)
  --help, -h          Show this help message

EXAMPLES:
  # Get the current hotkey
  bun scripts/config-cli.ts get hotkey.key

  # Get all config values
  bun scripts/config-cli.ts get

  # Set editor font size
  bun scripts/config-cli.ts set editorFontSize 16

  # Set hotkey to Cmd+K
  bun scripts/config-cli.ts set hotkey.key KeyK

  # Set hotkey modifiers
  bun scripts/config-cli.ts set hotkey.modifiers '["meta", "shift"]'

  # Disable clipboard history
  bun scripts/config-cli.ts set builtIns.clipboardHistory false

  # Set a command shortcut
  bun scripts/config-cli.ts set-command-shortcut builtin/clipboard-history KeyV true false false true

  # Remove only a command shortcut, preserving hidden/confirmation fields
  bun scripts/config-cli.ts remove-command-shortcut builtin/clipboard-history

  # List all available options
  bun scripts/config-cli.ts list

  # Check if config is valid
  bun scripts/config-cli.ts validate

  # Reset editor font size to default
  bun scripts/config-cli.ts reset editorFontSize

  # Reset entire config
  bun scripts/config-cli.ts reset

AVAILABLE CONFIG KEYS:
${CONFIG_SCHEMA.map(s => `  ${s.key.padEnd(35)} ${s.type.padEnd(25)} (default: ${JSON.stringify(s.default)})`).join('\n')}

OUTPUT:
  All output is JSON for easy parsing by AI agents.
  Check the "success" field to determine if the operation succeeded.

CONFIG FILE:
  Location: ${CONFIG_PATH}
`;
  
  console.log(help);
}

// =============================================================================
// Main
// =============================================================================

async function main(): Promise<void> {
  const args = process.argv.slice(2);
  
  if (args.length === 0 || args[0] === '--help' || args[0] === '-h') {
    showHelp();
    process.exit(0);
  }
  
  const command = args[0];
  
  switch (command) {
    case 'get':
      await cmdGet(args[1]);
      break;
    case 'set':
      await cmdSet(args[1], args[2]);
      break;
    case 'set-command-shortcut':
      await cmdSetCommandShortcut(args.slice(1));
      break;
    case 'remove-command-shortcut':
      await cmdRemoveCommandShortcut(args[1]);
      break;
    case 'list':
      await cmdList();
      break;
    case 'validate':
      await cmdValidate();
      break;
    case 'reset':
      await cmdReset(args[1]);
      break;
    case 'validate-change':
      await cmdValidateChange(args[1]);
      break;
    default:
      error(`Unknown command: ${command}. Use --help for usage.`);
  }
}

main().catch(e => {
  error(e instanceof Error ? e.message : String(e));
});
