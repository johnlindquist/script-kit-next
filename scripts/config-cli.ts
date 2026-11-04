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

import { configPath, DisplacedSettingsWrite, editLiteral, mutate, readSource, revision, type SettingsWriteErrorCode, StaleConfigRevision, values } from './config-document.ts';

import {
  analyzeCommandConfigPath,
  isValidCommandId,
  validateCommandConfigFieldValue,
  validateCommandConfigValue,
  validateCommandIdList,
  validateCommandsConfig,
} from './config-schema.ts';
import type {
  CommandId,
  AnimationPreferences,
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
  screenSpelling?: "off" | "accessibility" | "accessibilityAndOcr";
  selectedDeviceId?: string;
  model?: string;
  language?: string;
  saveHistory?: boolean;
  silenceRms?: number;
  maxDurationSecs?: number;
  livePreview?: boolean;
  pushToTalk?: boolean;
}

type AgentChatBackend = "goose" | "pi" | "agent_chat";

interface AgentChatToolPolicyConfig {
  allow?: string[];
}

interface AgentChatPathPolicyConfig {
  allowRead?: string[];
  allowWrite?: string[];
  deny?: string[];
}

interface AiProfile {
  id?: string;
  name: string;
  iconName?: string;
  backend?: AgentChatBackend;
  gooseBinary?: string;
  agent?: string;
  provider?: string;
  model?: string;
  systemPrompt?: string;
  appendSystemPrompt?: string;
  cwd?: string;
  tools?: string[];
  toolPolicy?: AgentChatToolPolicyConfig;
  pathPolicy?: AgentChatPathPolicyConfig;
  blockedActionMessage?: string;
  disableExtensions?: boolean;
  disableSkills?: boolean;
  disablePromptTemplates?: boolean;
  disableContextFiles?: boolean;
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
  quickAiProfileId?: string;
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

interface EffectsPreferences {
  background?: string;
  intensity?: number;
}

interface SpineStyleConfig {
  id: string;
  title?: string;
  description?: string;
  icon?: string;
  instruction: string;
}

interface SpineCommandConfig {
  name: string;
  description?: string;
  icon?: string;
}

interface BrainRemoteConfig {
  enabled?: boolean;
  telegramBotToken?: string;
  telegramAllowedUserIds?: number[];
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
  jevIntentEnabled?: boolean;
  clipboardHistoryMaxTextLength?: number;
  clipboardHistorySecretRejection?: ClipboardHistorySecretRejectionConfig;
  processLimits?: ProcessLimits;
  suggested?: SuggestedConfig;
  notesHotkey?: HotkeyConfig;
  notesHotkeyEnabled?: boolean;
  notesFolder?: string;
  aiHotkey?: HotkeyConfig;
  aiHotkeyEnabled?: boolean;
  logsHotkey?: HotkeyConfig;
  logsHotkeyEnabled?: boolean;
  dictationHotkey?: HotkeyConfig;
  dictationHotkeyEnabled?: boolean;
  inlineAiHotkey?: HotkeyConfig;
  inlineAiHotkeyEnabled?: boolean;
  rewriteHotkey?: HotkeyConfig;
  rewriteHotkeyEnabled?: boolean;
  watcher?: WatcherConfig;
  layout?: LayoutConfig;
  theme?: ThemeSelectionPreferences;
  animation?: AnimationPreferences;
  designs?: DesignsConfig;
  dictation?: DictationPreferences;
  ai?: AiPreferences;
  spineStyles?: SpineStyleConfig[];
  spineCommands?: SpineCommandConfig[];
  windowManagement?: WindowManagementPreferences;
  windowAppearance?: WindowAppearanceConfig;
  effects?: EffectsPreferences;
  commands?: Record<string, CommandConfig>;
  promptTargets?: Record<string, PromptTargetConfig>;
  unifiedSearch?: UnifiedSearchConfig;
  claudeCode?: ClaudeCodeConfig;
  mcp?: McpConfig;
  hiddenCommands?: string[];
  brainRemote?: BrainRemoteConfig;
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
  jevIntentEnabled: false,
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
  notesHotkey: { modifiers: ["meta", "ctrl"], key: "KeyN" },
  notesHotkeyEnabled: true,
  aiHotkeyEnabled: true,
  logsHotkeyEnabled: true,
  dictationHotkey: { modifiers: ["meta", "shift"], key: "Semicolon" },
  dictationHotkeyEnabled: true,
  inlineAiHotkey: { modifiers: ["meta", "ctrl"], key: "KeyI" },
  inlineAiHotkeyEnabled: true,
  rewriteHotkey: { modifiers: ["meta", "ctrl"], key: "KeyR" },
  rewriteHotkeyEnabled: true,
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
  animation: undefined,
  designs: undefined,
  dictation: { screenSpelling: "accessibility" },
  ai: undefined,
  windowManagement: undefined,
  effects: undefined,
  hiddenCommands: undefined,
  brainRemote: undefined,
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
    key: "jevIntentEnabled",
    type: "boolean",
    default: false,
    description: "Send launcher words, app names and command/menu choices to TypeSafe for remote readings"
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
    default: { modifiers: ["meta", "ctrl"], key: "KeyN" },
    description: "Hotkey for opening today's note under the main input (defaults to Cmd+Ctrl+N when enabled)",
    example: '{"modifiers": ["meta", "ctrl"], "key": "KeyN"}'
  },
  {
    key: "notesHotkeyEnabled",
    type: "boolean",
    default: true,
    description: "Whether the Notes hotkey is registered"
  },
  {
    key: "notesFolder",
    type: "string",
    default: "~/.scriptkit/brain",
    description: "Folder holding days/YYYY-MM-DD.md and notes/<Title>.md (plain Markdown, Obsidian-compatible); takes effect on the next start",
    example: '"~/Documents/Notes"'
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
  {
    key: "inlineAiHotkey",
    type: "HotkeyConfig",
    default: { modifiers: ["meta", "ctrl"], key: "KeyI" },
    description: "Hotkey for inline AI focused-text editing (defaults to Cmd+Ctrl+I when enabled)",
    example: '{"modifiers": ["meta", "ctrl"], "key": "KeyI"}'
  },
  {
    key: "inlineAiHotkeyEnabled",
    type: "boolean",
    default: true,
    description: "Whether the inline AI focused-text hotkey is registered"
  },
  {
    key: "rewriteHotkey",
    type: "HotkeyConfig",
    default: { modifiers: ["meta", "ctrl"], key: "KeyR" },
    description: "Hotkey for the instant rewrite flow (defaults to Cmd+Ctrl+R when enabled)",
    example: '{"modifiers": ["meta", "ctrl"], "key": "KeyR"}'
  },
  {
    key: "rewriteHotkeyEnabled",
    type: "boolean",
    default: true,
    description: "Whether the instant rewrite hotkey is registered"
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
    key: "animation.preset",
    type: "string | undefined",
    default: undefined,
    description: "Window entry fade (sharp_presence or fixed_frame; retired ids load as sharp_presence)"
  },
  {
    key: "dictation.startInKit",
    type: "boolean",
    default: undefined,
    description: "Start contextual dictation in the Kit input rather than the captured cursor"
  },
  {
    key: "dictation.screenSpelling",
    type: '"off" | "accessibility" | "accessibilityAndOcr"',
    default: DEFAULTS.dictation!.screenSpelling,
    description: "Local spelling clues from the original external dictation window; optional OCR requires Screen Recording"
  },
  {
    key: "dictation.selectedDeviceId",
    type: "string | undefined",
    default: undefined,
    description: "Preferred microphone device ID for dictation"
  },
  {
    key: "dictation.model",
    type: "string | undefined",
    default: undefined,
    description: "Dictation model catalog ID (undefined = default model)",
    example: '"whisper-large-v3-turbo"'
  },
  {
    key: "dictation.language",
    type: "string | undefined",
    default: undefined,
    description: "Language hint for engines that support one (Whisper only; Parakeet auto-detects)",
    example: '"en"'
  },
  {
    key: "dictation.saveHistory",
    type: "boolean",
    default: true,
    description: "Append transcripts to dictation-history.jsonl (disable to keep dictated text out of plaintext history)"
  },
  {
    key: "dictation.pushToTalk",
    type: "boolean",
    default: true,
    description: "Hold the dictation hotkey as push-to-talk (release to transcribe); a quick tap keeps toggle behavior"
  },
  {
    key: "dictation.livePreview",
    type: "boolean",
    default: true,
    description: "Show live partial transcripts in the recording overlay"
  },
  {
    key: "dictation.silenceRms",
    type: "number",
    default: 0.01,
    description: "RMS energy threshold below which a capture is skipped as silence (0.0-0.5; lower for quiet microphones)"
  },
  {
    key: "dictation.maxDurationSecs",
    type: "number",
    default: 600,
    description: "Auto-stop guard for runaway recordings in seconds (0 disables; runtime clamps to 30-14400)"
  },
  // --- Designs & effects ---
  {
    key: "designs.activeId",
    type: "string | undefined",
    default: undefined,
    description: "Stable catalog ID of the active design",
    example: '"script-kit-classic"'
  },
  {
    key: "designs.cmd1Behavior",
    type: '"picker" | "cycle" | undefined',
    default: undefined,
    description: 'Cmd+1 behavior: open the design picker or cycle designs ("picker" is the default behavior)',
    example: '"picker"'
  },
  {
    key: "effects.background",
    type: "string | undefined",
    default: undefined,
    description: 'Background shader effect slug (for example "aurora"); undefined disables the effect',
    example: '"aurora"'
  },
  {
    key: "effects.intensity",
    type: "number",
    default: 0.5,
    description: "Background effect strength from 0.0 to 1.0"
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
    description: "NOT YET WIRED — accepted by the schema but ignored by the app at runtime. Window vibrancy material; runtime keeps built-in defaults until wired"
  },
  {
    key: "windowAppearance.animations",
    type: '"system" | "reduced" | "off" | undefined',
    default: undefined,
    description: "NOT YET WIRED — accepted by the schema but ignored by the app at runtime. Show/hide animation mode; runtime keeps built-in defaults until wired"
  },
  {
    key: "powerSyntax.enabled",
    type: "boolean | undefined",
    default: undefined,
    description: "NOT YET WIRED — accepted by the schema but ignored by the app at runtime. Power Syntax master switch; runtime keeps built-in defaults until wired"
  },
  {
    key: "powerSyntax.captureSigil",
    type: '";" | "+" | "both" | undefined',
    default: undefined,
    description: 'NOT YET WIRED — accepted by the schema but ignored by the app at runtime. Capture sigil preference ("both" keeps legacy + and prefers ;)',
    example: '"both"'
  },
  {
    key: "powerSyntax.commandSigil",
    type: '">" | "disabled" | undefined',
    default: undefined,
    description: 'NOT YET WIRED — accepted by the schema but ignored by the app at runtime. Command invocation sigil preference ("disabled" turns off > invocation once wired)',
    example: '">"'
  },
  {
    key: "powerSyntax.cmdEnterAi.enabled",
    type: "boolean | undefined",
    default: undefined,
    description: "NOT YET WIRED — accepted by the schema but ignored by the app at runtime. Cmd+Enter AI enablement for Power Syntax"
  },
  {
    key: "powerSyntax.cmdEnterAi.modelId",
    type: "string | undefined",
    default: undefined,
    description: "NOT YET WIRED — accepted by the schema but ignored by the app at runtime. Cmd+Enter AI model override; falls back to active Agent Chat model"
  },
  {
    key: "powerSyntax.cmdEnterAi.systemPrompt",
    type: "string | undefined",
    default: undefined,
    description: "NOT YET WIRED — accepted by the schema but ignored by the app at runtime. Cmd+Enter AI system prompt override; falls back to active Agent Chat model"
  },
  {
    key: "tray.showCurrentAppCommands",
    type: "boolean | undefined",
    default: undefined,
    description: "NOT YET WIRED — accepted by the schema but ignored by the app at runtime. Tray visibility for the dynamic <App> Commands header row; default true once wired"
  },
  {
    key: "tray.showNotes",
    type: "boolean | undefined",
    default: undefined,
    description: "NOT YET WIRED — accepted by the schema but ignored by the app at runtime. Tray visibility for Open Notes; default true once wired"
  },
  {
    key: "tray.showAgentChat",
    type: "boolean | undefined",
    default: undefined,
    description: "NOT YET WIRED — accepted by the schema but ignored by the app at runtime. Tray visibility for Open Agent Chat; default true once wired"
  },
  {
    key: "tray.showReloadScripts",
    type: "boolean | undefined",
    default: undefined,
    description: "NOT YET WIRED — accepted by the schema but ignored by the app at runtime. Tray visibility for Reload Scripts; default true once wired"
  },
  {
    key: "tray.showHelp",
    type: "boolean | undefined",
    default: undefined,
    description: "NOT YET WIRED — accepted by the schema but ignored by the app at runtime. Tray visibility for Help/Feedback; default true once wired"
  },
  {
    key: "tray.showSocialLinks",
    type: "boolean | undefined",
    default: undefined,
    description: "NOT YET WIRED — accepted by the schema but ignored by the app at runtime. Tray visibility for Follow Us, GitHub, and Discord; default true once wired"
  },
  {
    key: "tray.showUpdateCheck",
    type: "boolean | undefined",
    default: undefined,
    description: "NOT YET WIRED — accepted by the schema but ignored by the app at runtime. Tray visibility for Check for Updates and Version rows; default true once wired"
  },
  {
    key: "updates.autoCheck",
    type: "boolean | undefined",
    default: undefined,
    description: "NOT YET WIRED — accepted by the schema but ignored by the app at runtime. Update auto-check preference; default true once wired"
  },
  // --- Commands & Claude Code ---
  {
    key: "commands",
    type: "Record<string, CommandConfig>",
    default: undefined,
    description: "Per-command shortcuts and visibility overrides"
  },
  {
    key: "mcp.enabled",
    type: "boolean",
    default: true,
    description: "Enable configured external MCP servers"
  },
  {
    key: "mcp.servers",
    type: "Record<string, McpServerConfig>",
    default: {},
    description: "External server definitions (may launch executables or send credentials)"
  },
  {
    key: "promptTargets",
    type: "Record<string, PromptTargetConfig>",
    default: undefined,
    description: "Prompt handoff targets surfaced as prompt-target/<id> Actions and shortcut commands; built-in prompt actions use prompt-action/<id>"
  },
  {
    key: "hiddenCommands",
    type: "string[]",
    default: undefined,
    description: "Canonical command IDs hidden from the launcher main menu (still resolvable via triggerBuiltin, hotkeys, and deeplinks)",
    example: '["builtin/clipboard-history", "script/deprecated-tool"]'
  },
  // --- Brain remote access ---
  {
    key: "brainRemote.enabled",
    type: "boolean",
    default: false,
    description: "Master switch for the Telegram brain bridge (inactive without a bot token AND a non-empty telegramAllowedUserIds allowlist)"
  },
  {
    key: "brainRemote.telegramBotToken",
    type: "string | undefined",
    default: undefined,
    description: "Telegram bot token from @BotFather. SECURITY: grants remote capture/queries into local memory; treated as a secret and never logged"
  },
  {
    key: "brainRemote.telegramAllowedUserIds",
    type: "number[]",
    default: [],
    description: "Numeric Telegram user IDs allowed to query the brain (empty disables the bridge entirely)",
    example: '[123456789]'
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

let CONFIG_PATH: string;

interface Result<T> {
  success: boolean;
  errorCode?: SettingsWriteErrorCode;
  data?: T;
  error?: string;
  valid?: boolean;
  errors?: string[];
  warnings?: string[];
}

function output(result: Result<unknown> | (ValidateConfigChangeResult & { success: false })): void {
  console.log(JSON.stringify(result, null, 2));
}

function success<T>(data: T): void {
  output({ success: true, data });
}

function error(message: string, errorCode?: SettingsWriteErrorCode): void {
  output({ success: false, error: message, errorCode });
  process.exit(1);
}

/**
 * Get a nested value from an object using dot notation
 */
function pathParts(key: string) {
  const cmd = analyzeCommandConfigPath(key);
  return cmd?.kind === 'parsed'
    ? ['commands', cmd.commandId, ...(cmd.fieldPath?.split('.') ?? [])]
    : key.split('.');
}
function getNestedValue(obj: Record<string, unknown>, key: string) {
  let value: unknown = obj;
  for (const part of pathParts(key)) {
    if (!isRecord(value) || !Object.hasOwn(value, part)) return undefined;
    value = value[part];
  }
  return value;
}
function isRecord(value: unknown): value is Record<string, unknown> {
  return value !== null && typeof value === 'object' && !Array.isArray(value);
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
function parseValue(raw: string, key: string) {
  const type = CONFIG_SCHEMA.find((s) => s.key === key)?.type ?? '';
  if (
    (type.startsWith('string') && !type.startsWith('string[]')) ||
    type.startsWith('"') ||
    type === 'KeyCode'
  )
    return raw;
  try {
    return JSON.parse(raw);
  } catch {
    return raw;
  }
}

// Describes consequences; it does not grant permission or change MCP auth.
function impact(key: string) {
  if (/^(bun_path|editor$|mcp(?:\.|$)|promptTargets|claudeCode)/.test(key)) return 'execution-policy';
  if (
    /^(jevIntentEnabled|brainRemote|builtIns.clipboardHistory|clipboardHistory|dictation\.(screenSpelling|saveHistory)|suggested.trackUsage)/.test(
      key,
    )
  )
    return 'privacy';
  if (/^(commands|.*[Hh]otkey|hiddenCommands)/.test(key)) return 'command-routing';
  if (key === 'notesFolder') return 'filesystem';
  if (/^(processLimits|powerSyntax\.cmdEnterAi)/.test(key)) return 'execution-policy';
  return 'preference';
}

// Local runtime config remains fully configurable by its owner. A scoped
// settings tool isn't a grant to launch an executable, move private files,
// grant remote access, or disable confirmations. Check group ancestors too.
function manualOnly(key: string): boolean {
  if (['execution-policy', 'filesystem'].includes(impact(key))) return true;
  if (/^(?:brainRemote|jevIntentEnabled)(?:\.|$)/.test(key)) return true;
  if (
    ['ai', 'powerSyntax', 'dictation.screenSpelling', 'clipboardHistorySecretRejection', 'clipboardHistoryMaxTextLength'].includes(key) ||
    key.startsWith('clipboardHistorySecretRejection.')
  )
    return true;
  return analyzeCommandConfigPath(key)?.kind === 'parsed' && key.endsWith('.confirmationRequired');
}
// A scoped tool may turn collection off, but cannot opt the user back in.
const disableOnly = new Set(['builtIns.clipboardHistory', 'dictation.saveHistory', 'suggested.trackUsage']);
function permissionError(key: string, value: unknown): string | undefined {
  if (disableOnly.has(key) && value !== false) return `Enabling ${key} requires manual opt-in`;
  if (manualOnly(key)) return `Manual-only setting: ${key}; edit config.ts yourself`;
  // A whole-group replacement could drop unrelated manual-only settings.
  if (key !== 'hotkey' && CONFIG_SCHEMA.some((s) => s.key.startsWith(key + '.')))
    return `Change individual fields of ${key}; replacing the group could discard unreviewed settings`;
  if (
    key === 'commands' &&
    isRecord(value) &&
    Object.values(value).some((v) => isRecord(v) && Object.hasOwn(v, 'confirmationRequired'))
  )
    return 'Command confirmation policy must be edited manually';
  if (key.startsWith('commands.') && isRecord(value) && Object.hasOwn(value, 'confirmationRequired'))
    return 'Command confirmation policy must be edited manually';
}
function protectedConfirmations(before: Record<string, unknown>, after: Record<string, unknown>) {
  if (!isRecord(before.commands)) return;
  for (const [id, entry] of Object.entries(before.commands)) {
    if (!isRecord(entry) || !Object.hasOwn(entry, 'confirmationRequired')) continue;
    const updated = isRecord(after.commands) ? after.commands[id] : undefined;
    if (!isRecord(updated) || updated.confirmationRequired !== entry.confirmationRequired)
      throw new Error('Cannot remove a command confirmation policy via a parent edit');
  }
}
function validateEndpointMap(key: string, v: unknown): boolean {
  if (!isRecord(v)) return false;
  return Object.values(v).every((entry) => {
    if (!isRecord(entry)) return false;
    const prompt = key === 'promptTargets';
    const http = !prompt && entry.transport === 'http';
    if (!prompt && !http && entry.transport !== 'stdio') return false;
    if (typeof entry[http ? 'endpoint' : 'command'] !== 'string') return false;
    const strings = prompt
      ? ['title', 'description', 'command', 'cwd']
      : http
        ? ['transport', 'name', 'description', 'endpoint']
        : ['transport', 'name', 'description', 'command', 'cwd'];
    return Object.entries(entry).every(([k, x]) =>
      strings.includes(k)
        ? typeof x === 'string'
        : k === 'enabled' && !prompt
          ? typeof x === 'boolean'
          : k === 'args' && !http
            ? Array.isArray(x) && x.every((a) => typeof a === 'string')
            : k === (http ? 'headers' : 'env') &&
              isRecord(x) &&
              Object.values(x).every((a) => typeof a === 'string'),
    );
  });
}
function validateValue(key: string, value: unknown): { valid: boolean; error?: string } {
  const invalid = (message: string) => ({ valid: false, error: message });
  const schema = CONFIG_SCHEMA.find((s) => s.key === key);
  if (
    key === 'hotkey' ||
    schema?.type === 'HotkeyConfig' ||
    key === 'hotkey.key' ||
    key === 'hotkey.modifiers'
  ) {
    const field = key.startsWith('hotkey.') ? key.replace('hotkey.', 'shortcut.') : 'shortcut';
    const errors = validateCommandConfigFieldValue(field, value, key);
    if (errors.length) return invalid(errors.map((e) => e.message).join('; '));
    return { valid: true };
  }
  if (!schema) {
    // Allow replacing a documented group only when *every* child has a rule.
    if (CONFIG_SCHEMA.some((s) => s.key.startsWith(key + '.')) && isRecord(value)) {
      for (const [child, v] of Object.entries(value)) {
        if (child.includes('.') || !child) return invalid('Invalid property in ' + key);
        const check = validateConfigChange({ key: key + '.' + child, value: v }, false);
        if (!check.valid) return invalid(check.errors.map((e) => e.message).join('; '));
      }
      return { valid: true };
    }
    return invalid('Unknown or unsupported config key: ' + key);
  }
  const type = schema.type.replace(' | undefined', '');
  let ok: boolean;
  if (type === 'number') ok = typeof value === 'number' && Number.isFinite(value);
  else if (type === 'boolean' || type === 'string') ok = typeof value === type;
  else if (type === 'string[]') ok = Array.isArray(value) && value.every((x) => typeof x === 'string');
  else if (type === 'number[]') ok = Array.isArray(value) && value.every(Number.isSafeInteger);
  else if (type.startsWith('"')) ok = type.split(' | ').some((t) => JSON.parse(t) === value);
  else if (key === 'promptTargets' || key === 'mcp.servers') ok = validateEndpointMap(key, value);
  else ok = false;
  if (!ok) return invalid(key + ' must match ' + type);
  if (typeof value === 'number') {
    if (
      /^(processLimits\.|watcher\.|clipboardHistoryMaxTextLength$|suggested\.maxItems$|dictation\.maxDurationSecs$)/.test(
        key,
      ) &&
      (!Number.isSafeInteger(value) || value < 0)
    )
      return invalid(key + ' must be a nonnegative integer');
    if (key === 'watcher.maxNotifyErrors' && value > 0xffffffff) return invalid(key + ' exceeds u32');
    if (
      /^(?:effects\.|padding\.|layout\.|dictation\.silenceRms|.*FontSize$|uiScale$)/.test(key) &&
      !Number.isFinite(Math.fround(value))
    )
      return invalid(key + ' exceeds f32');
    if (key.includes('FontSize') && (value < 8 || value > 72))
      return invalid('Font size must be between 8 and 72');
    if (key === 'uiScale' && (value < 0.5 || value > 3))
      return invalid('uiScale must be between 0.5 and 3.0');
    if (key.startsWith('padding.') && value < 0) return invalid('Padding cannot be negative');
    if (key === 'effects.intensity' && (value < 0 || value > 1))
      return invalid('effects.intensity must be between 0 and 1');
    if (key === 'dictation.silenceRms' && (value < 0 || value > 0.5))
      return invalid('dictation.silenceRms must be between 0 and 0.5');
  }
  return { valid: true };
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

  if (Object.hasOwn(aliases,lower)) {
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

function readConfig() {
  const source = readSource(CONFIG_PATH);
  return {
    config: source === null ? null : values(source),
    revision: revision(source),
    path: CONFIG_PATH,
    exists: source !== null,
  };
}
// The SDK and MCP results are tool-visible. Never include configured bot
// tokens, target environments or server credentials in their get/list output.
function safeValue(key: string, value: unknown): unknown {
  if (value === undefined) return value;
  if (/^(?:brainRemote\.telegramBotToken|mcp\.servers|promptTargets|ai\.profiles)(?:\.|$)/.test(key))
    return '[redacted]';
  if (!isRecord(value)) return value;
  return Object.fromEntries(
    Object.entries(value).map(([k, v]) => [k, safeValue(key ? key + '.' + k : k, v)]),
  );
}
function cmdGet(key?: string) {
  const { config, ...meta } = readConfig();
  if (!key)
    return success({
      ...meta,
      config: safeValue('', {
        ...DEFAULTS,
        ...config,
        dictation: { ...DEFAULTS.dictation, ...(config?.dictation as object) },
      }),
    });
  const current = getNestedValue(config ?? {}, key),
    def = defaultValue(key);
  success({
    ...meta,
    key,
    value: safeValue(key, current ?? def),
    isDefault: current === undefined,
    default: def,
    impact: impact(key),
    manualOnly: manualOnly(key),
    disableOnly: disableOnly.has(key),
  });
}
function defaultValue(key: string) {
  return CONFIG_SCHEMA.find((s) => s.key === key)?.default ?? getNestedValue(DEFAULTS, key);
}
function assertChange(key: string, value: unknown) {
  const result = validateConfigChange({ key, value });
  if (!result.valid) throw new Error(result.errors.map((e) => e.message).join('; '));
}
function cmdSet(key: string, raw: string, json: boolean, expected?: string) {
  if (!key || raw === undefined) throw new Error('set requires a key and value');
  const value = json ? JSON.parse(raw) : parseValue(raw, key);
  assertChange(key, value);
  const rev = mutate(CONFIG_PATH, expected, (source) => {
    const updated = editLiteral(source ?? createDefaultConfig(), pathParts(key), value);
    const after = values(updated);
    // Partial hotkey writes must still produce a complete shortcut.
    const cmd = analyzeCommandConfigPath(key);
    if (cmd?.kind === 'parsed' && cmd.fieldPath?.startsWith('shortcut.')) {
      assertChange(
        'commands.' + cmd.commandId + '.shortcut',
        getNestedValue(after, 'commands.' + cmd.commandId + '.shortcut'),
      );
    }
    if (key.startsWith('hotkey.')) assertChange('hotkey', after.hotkey);
    if (key === 'commands' || key.startsWith('commands.'))
      protectedConfirmations(values(source ?? createDefaultConfig()), after);
    if (JSON.stringify(getNestedValue(after, key)) !== JSON.stringify(value))
      throw new Error('Config edit failed verification');
    return updated;
  });
  success({ key, value, revision: rev, impact: impact(key), message: 'Set ' + key });
}
function cmdSetCommandShortcut(args: string[], expected?: string) {
  const [id, key, ...flags] = args;
  if (
    !id ||
    !isValidCommandId(id) ||
    !key ||
    ![4, 5].includes(flags.length) ||
    (flags.length === 5 && flags[4] !== '--skip-existing') ||
    flags.slice(0, 4).some((f) => f !== 'true' && f !== 'false')
  )
    throw new Error('Invalid set-command-shortcut arguments');
  const shortcut = {
    key: normalizeShortcutKey(key),
    modifiers: ['meta', 'ctrl', 'alt', 'shift'].filter((_, i) => flags[i] === 'true'),
  };
  assertChange('commands.' + id + '.shortcut', shortcut);
  let result: unknown = shortcut,
    skipped = false;
  const rev = mutate(CONFIG_PATH, expected, (source) => {
    const before = source ?? createDefaultConfig();
    const current = getNestedValue(values(before), 'commands.' + id + '.shortcut');
    if (flags.includes('--skip-existing') && current !== undefined) {
      skipped = true;
      result = current;
      return before;
    }
    return editLiteral(before, ['commands', id, 'shortcut'], shortcut);
  });
  success({
    commandId: id,
    shortcut: result,
    skipped,
    revision: rev,
    message: skipped ? 'Skipped existing shortcut' : 'Set shortcut',
  });
}
function cmdRemoveCommandShortcut(id: string, expected?: string) {
  if (!id || !isValidCommandId(id)) throw new Error('Invalid command ID');
  let removed = false;
  const rev = mutate(CONFIG_PATH, expected, (source) => {
    if (source === null) return null;
    const commands = values(source).commands;
    if (commands !== undefined && !isRecord(commands)) throw new Error('Expected commands object');
    const entry = isRecord(commands) ? commands[id] : undefined;
    if (entry !== undefined && !isRecord(entry)) throw new Error('Expected command object');
    if (!isRecord(entry) || !Object.hasOwn(entry, 'shortcut')) return source;
    removed = true;
    return editLiteral(
      source,
      Object.keys(entry).length === 1 ? ['commands', id] : ['commands', id, 'shortcut'],
      undefined,
    );
  });
  success({
    commandId: id,
    removed,
    revision: rev,
    message: removed ? 'Removed shortcut' : 'No shortcut configured',
  });
}
function cmdList() {
  const { config, ...meta } = readConfig();
  success({
    ...meta,
    options: CONFIG_SCHEMA.map((s) => {
      const current = getNestedValue(config ?? {}, s.key);
      return {
        ...s,
        current: safeValue(s.key, current ?? s.default),
        isCustom: current !== undefined && JSON.stringify(current) !== JSON.stringify(s.default),
        impact: impact(s.key),
        manualOnly: manualOnly(s.key),
        disableOnly: disableOnly.has(s.key),
      };
    }),
  });
}
function cmdValidate() {
  try {
    const { config, ...meta } = readConfig();
    const errors = [];
    for (const [key, value] of Object.entries(config ?? {})) {
      const result = validateConfigChange({ key, value }, false);
      for (const e of result.errors) errors.push(e.message);
    }
    if (errors.length) {
      output({ success: false, valid: false, errors });
      process.exitCode = 1;
    } else success({ ...meta, valid: true });
  } catch (e) {
    output({ success: false, valid: false, errors: [String(e)] });
    process.exitCode = 1;
  }
}
function cmdReset(key?: string, expected?: string) {
  if (
    key !== undefined &&
    (!key ||
      (!CONFIG_SCHEMA.some((s) => s.key === key) && analyzeCommandConfigPath(key)?.kind !== 'parsed'))
  )
    throw new Error('Unknown or unsupported config key: ' + key);
  if (key && (manualOnly(key) || disableOnly.has(key))) throw new Error('Manual-only reset: ' + key);
  const cmd = key ? analyzeCommandConfigPath(key) : null;
  if (cmd?.kind === 'parsed' && cmd.fieldPath?.startsWith('shortcut.'))
    throw new Error('Reset the whole shortcut, not one required field');
  const def = key ? defaultValue(key) : undefined;
  const rev = mutate(CONFIG_PATH, expected, (source) => {
    if (source === null) return null;
    const before = values(source);
    if (!key) {
      function canReset(k: string, v: unknown): boolean {
        if (manualOnly(k) || disableOnly.has(k)) return false;
        if (k === 'commands' || k === 'hotkey' || CONFIG_SCHEMA.some((s) => s.key === k))
          return validateConfigChange({ key: k, value: v }).valid;
        return (
          CONFIG_SCHEMA.some((s) => s.key.startsWith(k + '.')) &&
          isRecord(v) &&
          Object.entries(v).every(([child, x]) => !child.includes('.') && canReset(k + '.' + child, x))
        );
      }
      if (!Object.entries(before).every(([k, v]) => canReset(k, v)))
        throw new Error(
          'Full reset includes manual-only or unsupported settings; edit config.ts yourself',
        );
      protectedConfirmations(before, {});
      return createDefaultConfig();
    }
    // Reset means inherit again, not freeze the CLI's current default. The
    // main hotkey's two required fields must still form a valid hotkey.
    const updated = editLiteral(source, pathParts(key), key.startsWith('hotkey.') ? def : undefined);
    if (key.startsWith('commands.') || key === 'commands')
      protectedConfirmations(before, values(updated));
    return updated;
  });
  success({
    key,
    value: def,
    revision: rev,
    message: key ? 'Reset ' + key : 'Reset config to defaults',
  });
}

function debugLog(event: string, details: Record<string, unknown> = {}): void {
  if (process.env.SCRIPT_KIT_CONFIG_DEBUG !== '1') {
    return;
  }
  process.stderr.write(
    `[config-cli] ${JSON.stringify({ event, ...details })}\n`,
  );
}

function validateConfigChange(change: ConfigChange, scoped = true): ValidateConfigChangeResult {
  if (!isRecord(change) || typeof change.key !== 'string' || !change.key || !Object.hasOwn(change,'value') || change.value === undefined) {
    return {valid:false,errors:[{path:'',code:'invalidChange',message:'Expected a key and JSON value'}],warnings:[]};
  }
  const denied = scoped ? permissionError(change.key, change.value) : undefined;
  if (denied) return {valid:false,errors:[{path:change.key,code:'manualOnly',message:denied}],warnings:[]};
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

  // Validate command-ID arrays (e.g. suggested.excludedCommands, hiddenCommands)
  if (change.key === "suggested.excludedCommands" || change.key === "hiddenCommands") {
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
  set <key> <value>   Set a config value (human-friendly syntax)
  set-json <key> <JSON> [--if-revision <revision>]
                      Set a typed value, optionally only from a get/list revision
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
  CONFIG_PATH = configPath();
  const args = process.argv.slice(2);

  if (args.length === 0 || args[0] === '--help' || args[0] === '-h') {
    showHelp();
    process.exit(0);
  }

  const command = args[0];
  // Flags are only parsed after required positionals, never out of a JSON value.
  let expected: string | undefined;
  if (
    ['set', 'set-json', 'reset', 'set-command-shortcut', 'remove-command-shortcut'].includes(command) &&
    args.at(-2) === '--if-revision'
  ) {
    expected = args.pop();
    args.pop();
    if (!expected || !/^(?:missing|[a-f0-9]{64})$/.test(expected))
      throw new Error('Expected a valid revision');
  }
  const max: Record<string, number> = {
    get: 2,
    set: 3,
    'set-json': 3,
    reset: 2,
    'remove-command-shortcut': 2,
    list: 1,
    validate: 1,
    'validate-change': 2,
  };
  if (Object.hasOwn(max, command) && args.length > max[command])
    throw new Error('Unexpected arguments for ' + command);

  switch (command) {
    case 'get':
      await cmdGet(args[1]);
      break;
    case 'set':
    case 'set-json':
      await cmdSet(args[1], args[2], command === 'set-json', expected);
      break;
    case 'set-command-shortcut':
      await cmdSetCommandShortcut(args.slice(1), expected);
      break;
    case 'remove-command-shortcut':
      await cmdRemoveCommandShortcut(args[1], expected);
      break;
    case 'list':
      await cmdList();
      break;
    case 'validate':
      await cmdValidate();
      break;
    case 'reset':
      await cmdReset(args[1], expected);
      break;
    case 'validate-change':
      await cmdValidateChange(args[1]);
      break;
    default:
      error(`Unknown command: ${command}. Use --help for usage.`);
  }
}

main().catch((e) => {
  const code = e instanceof StaleConfigRevision || e instanceof DisplacedSettingsWrite ? e.code : undefined;
  error(e instanceof Error ? e.message : String(e), code);
});
