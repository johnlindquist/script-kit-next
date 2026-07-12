import type { Config } from "@scriptkit/sdk";

/**
 * Script Kit Configuration
 * ========================
 *
 * This file controls Script Kit's behavior, appearance, and built-in features.
 * It's loaded on startup from ~/.scriptkit/config.ts.
 *
 * HOW TO CUSTOMIZE:
 * 1. Uncomment the options you want to change
 * 2. Modify the values to your preference
 * 3. Save the file - Script Kit reloads config automatically
 *
 * DOCUMENTATION:
 * - Full schema with all options: See Config interface in kit-sdk.ts
 * - Type definitions provide inline documentation via your editor's hover
 *
 * TYPE SAFETY:
 * This file uses `satisfies Config` for compile-time type checking.
 * Your editor will warn you about invalid options or values.
 *
 * TIP:
 * You can ask Agent Chat to change any of these for you — it edits this
 * file with validation (kit/config_set). Everything Script Kit persists
 * as a setting lives in this one file.
 */
export default {
  // ===========================================================================
  // REQUIRED: Global Hotkey
  // ===========================================================================
  // hotkey: Global keyboard shortcut used to open the Script Kit launcher.
  // hotkey.modifiers: Array of modifier keys to hold.
  // Valid values: 'meta', 'ctrl', 'alt', 'shift'
  // hotkey.key: KeyboardEvent.code string for the non-modifier key.
  // Common key values: 'Semicolon', 'KeyK', 'Digit1', 'Space', 'Enter'
  // Example hotkey configs:
  // - { modifiers: ['meta'], key: 'Semicolon' } // Cmd + ;
  // - { modifiers: ['meta', 'shift'], key: 'KeyK' } // Cmd + Shift + K
  // - { modifiers: ['ctrl', 'alt'], key: 'Digit1' } // Ctrl + Alt + 1
  // - { modifiers: ['ctrl', 'alt'], key: 'Space' } // Ctrl + Alt + Space

  hotkey: {
    // hotkey.modifiers: Ordered list of modifier keys that must be held.
    // Allowed values: 'meta', 'ctrl', 'alt', 'shift'
    // - meta: Command on macOS, Windows key on Windows/Linux
    // - ctrl: Control key
    // - alt: Option on macOS, Alt on Windows/Linux
    // - shift: Shift key
    // Examples: ['meta'], ['meta', 'shift'], ['ctrl', 'alt']
    modifiers: ["meta"],

    // hotkey.key: Non-modifier key using KeyboardEvent.code values.
    // Common values: 'Semicolon', 'KeyK', 'Digit1', 'Space', 'Enter'
    // Other valid patterns include letters ("KeyA"..."KeyZ"), numbers
    // ("Digit0"..."Digit9"), punctuation keys, and function keys ("F1"..."F12").
    key: "Semicolon", // Cmd+; on Mac, Win+; on Windows
  },

  // ===========================================================================
  // UI Settings
  // ===========================================================================
  // Customize the appearance of Script Kit's interface.

  // Font size for the Monaco-style code editor (in pixels)
  // editorFontSize: 16,

  // Font size for the integrated terminal (in pixels)
  // terminalFontSize: 14,

  // UI scale factor (1.0 = 100%, 1.5 = 150%, etc.)
  // Useful for HiDPI displays or accessibility
  // uiScale: 1.0,

  // Window appearance: vibrancy material and animation behavior. Panel
  // level / non-activating behavior stay fixed for focus safety.
  // Reserved for future runtime wiring; the launcher currently honors the
  // built-in defaults.
  //
  // windowAppearance: {
  //   vibrancy: "default",     // "default" | "none" | "hud" | "popover" | "sidebar"
  //   animations: "system",    // "system" | "reduced" | "off"
  // },

  // Content padding for prompts (terminal, editor, etc.)
  // All values in pixels
  // padding: {
  //   top: 8,    // Inner top spacing
  //   left: 12,  // Inner left spacing
  //   right: 12, // Inner right spacing
  // },

  // ===========================================================================
  // Editor Settings
  // ===========================================================================
  // Configure the external editor used for "Open in Editor" actions.

  // Editor command (falls back to $EDITOR env var, then "code")
  // Examples: "code", "vim", "nvim", "subl", "zed", "cursor"
  // editor: "code",

  // ===========================================================================
  // Built-in Features
  // ===========================================================================
  // Enable or disable Script Kit's built-in productivity features.

  // builtIns: {
  //   // Clipboard history - tracks clipboard changes with searchable history
  //   clipboardHistory: true,
  //
  //   // App launcher - search and launch applications
  //   appLauncher: true,
  //
  //   // Window switcher - manage open windows across applications
  //   windowSwitcher: true,
  // },
  //
  // Max text size (bytes) stored per clipboard history entry
  // Set to 0 to disable the limit
  // clipboardHistoryMaxTextLength: 100000,
  //
  // Hard clipboard secret rejection: extend the built-in blocklists.
  // Copies from blocked apps (bundle-ID prefix match) and text matching
  // secret patterns are never stored. Conservative defaults always apply;
  // these lists only add to them.
  // clipboardHistorySecretRejection: {
  //   extraBlockedSourceApps: ["com.example.passwordmanager"],
  //   extraSecretPatterns: ["^corp-token-[A-Za-z0-9]{32}$"],
  // },

  // ===========================================================================
  // Auxiliary Window / Tool Hotkeys
  // ===========================================================================

  // Notes falls back to Cmd+Ctrl+N when enabled and not explicitly set.
  // notesHotkey: { modifiers: ["meta", "ctrl"], key: "KeyN" },
  // notesHotkeyEnabled: true,

  // AI falls back to Cmd+Shift+Space when enabled and not explicitly set.
  // aiHotkey: { modifiers: ["meta", "shift"], key: "Space" },
  // aiHotkeyEnabled: true,

  // Logs fall back to Cmd+Shift+L when enabled and not explicitly set.
  // logsHotkey: { modifiers: ["meta", "shift"], key: "KeyL" },
  // logsHotkeyEnabled: true,

  // Dictation defaults to Cmd+Shift+; when enabled.
  // Change this value to customize the global dictation shortcut.
  dictationHotkey: { modifiers: ["meta", "shift"], key: "Semicolon" },
  dictationHotkeyEnabled: true,

  // Inline AI focused-text editing falls back to Cmd+Ctrl+I when enabled.
  // Captures the focused text field in the frontmost app, then shows the
  // inline agent overlay.
  // inlineAiHotkey: { modifiers: ["meta", "ctrl"], key: "KeyI" },
  // inlineAiHotkeyEnabled: true,

  // Instant rewrite falls back to Cmd+Ctrl+R when enabled. Captures the
  // focused text and immediately streams three rewrite variations in the
  // mini UI. Change it if you need Xcode's Cmd+Ctrl+R.
  // rewriteHotkey: { modifiers: ["meta", "ctrl"], key: "KeyR" },
  // rewriteHotkeyEnabled: true,
  // Disable the footer tips on the main menu.
  // tips: { enabled: false },
  //
  // Runtime preferences also live here:
  // theme: { presetId: "nord" },
  // dictation: {
  //   selectedDeviceId: "usb-mic",
  //   // Where transcripts go when the global shortcut starts dictation:
  //   // - "sticky" (default): the last destination picked via an overlay
  //   //   chip (Paste / Today / Ask / Send); falls back to context capture
  //   //   until one has been picked. The pick persists as lastTarget.
  //   // - "context": the active Script Kit surface at start time.
  //   // - explicit: "frontmost" | "today" | "ask" | "agent" | "notes"
  //   target: "sticky",
  //   // What the "Ask" destination does with the transcript:
  //   // - "answer" (default): submit immediately and stream the answer in
  //   //   the mini AI window (fire-and-show).
  //   // - "composer": stage it in the AI composer without sending.
  //   quickAi: "answer",
  // },
  // ai: {
  //   // Last-selected model and profile (the profile id wins when both match)
  //   selectedModelId: "gpt-5.4",
  //   selectedProfileId: "writing",
  //   // Named Agent Chat profiles for quick switching
  //   profiles: [
  //     { id: "writing", name: "Long-form writing",
  //       model: "gpt-5.4", systemPrompt: "You are a long-form writing partner." },
  //     { id: "code", name: "Code review",
  //       provider: "openai-codex", model: "gpt-5.6-terra", thinking: "medium" },
  //   ],
  // },
  // windowManagement: { snapMode: "expanded" },
  //
  // Design picker: active design plus per-design token overrides.
  // designs: {
  //   activeId: "script-kit-classic",
  //   overrides: {
  //     "script-kit-classic": { density: "comfortable" },
  //   },
  // },
  //
  // Background shader effect for the launcher window.
  // Defaults to "starfield"; set "off" to disable effects entirely.
  // intensity ranges 0.0-1.0 (default 0.5).
  // effects: { background: "aurora", intensity: 0.5 },
  // effects: { background: "off" },
  //
  // Behavior:
  // - No selectedDeviceId means use the macOS default microphone
  // - Missing saved microphone falls back to the best available device
  // - The app clears stale microphone preferences automatically
  // - Use the built-in "Select Microphone" action to change it

  // ===========================================================================
  // Unified Search
  // ===========================================================================
  // Passive extra sources (files, notes, browser history, clipboard, ...)
  // in root launcher search. Each source has enabled/maxResults/minQueryChars
  // knobs; see the UnifiedSearchConfig type for the full tree.
  //
  // unifiedSearch: {
  //   enabled: true,
  //   browserHistory: { enabled: true, maxAgeDays: 90 },
  //   clipboardHistory: { enabled: false },
  // },

  // ===========================================================================
  // Power Syntax
  // ===========================================================================
  // Power Syntax controls launcher sigils for capture (;), refine (:), and
  // command (>) input. These knobs are reserved for future runtime wiring;
  // for now Script Kit honors the built-in defaults shown below.
  //
  // powerSyntax: {
  //   enabled: true,
  //   captureSigil: "both",     // ";" | "+" | "both"
  //   commandSigil: ">",        // ">" | "disabled"
  //   cmdEnterAi: {
  //     enabled: true,
  //     modelId: undefined,     // fall back to active Agent Chat model
  //     systemPrompt: undefined,
  //   },
  // },

  // ===========================================================================
  // Tray Menu
  // ===========================================================================
  // Tray menu visibility. Each row defaults to shown; set false to hide.
  // Reserved for future runtime wiring; the menu currently honors the
  // built-in defaults shown below.
  //
  // tray: {
  //   showCurrentAppCommands: true,  // dynamic "<App> Commands" header row
  //   showNotes: true,
  //   showAgentChat: true,
  //   showReloadScripts: true,
  //   showHelp: true,                // Send Feedback...
  //   showSocialLinks: true,         // Follow Us / GitHub / Discord
  //   showUpdateCheck: true,         // Check for Updates... + Version row
  // },

  // ===========================================================================
  // Updates
  // ===========================================================================
  // Auto-check for new releases on launch.
  // Reserved for future runtime wiring; the checker currently runs ~5s
  // after launch regardless of this setting.
  //
  // updates: {
  //   autoCheck: true,
  // },

  // ===========================================================================
  // Command Configuration
  // ===========================================================================
  // Configure shortcuts and visibility for any command in Script Kit.
  // Commands are identified by category-prefixed IDs: {category}/{identifier}
  //
  // CATEGORIES:
  //   builtin/   - Built-in features (clipboard-history, app-launcher, etc.)
  //   app/       - macOS apps by bundle ID (com.apple.Safari, etc.)
  //   script/    - User scripts by filename without .ts (my-script, etc.)
  //   scriptlet/ - Inline scriptlets by UUID or name
  //
  // DEEPLINKS: Each command maps to scriptkit://commands/{id}
  //   Example: "builtin/clipboard-history" → scriptkit://commands/builtin/clipboard-history
  //
  // OPTIONS:
  //   shortcut - Global keyboard shortcut to invoke directly
  //   hidden   - Hide from main menu (still accessible via shortcut/deeplink)

  // commands: {
  //   // ─────────────────────────────────────────────────────────────────────
  //   // BUILT-IN FEATURES
  //   // ─────────────────────────────────────────────────────────────────────
  //
  //   // Quick access to clipboard history with Cmd+Shift+V
  //   "builtin/clipboard-history": {
  //     shortcut: { modifiers: ["meta", "shift"], key: "KeyV" }
  //   },
  //
  //   // Require a confirmation dialog for a destructive built-in
  //   // "builtin/empty-trash": {
  //   //   confirmationRequired: true,
  //   // },
  //
  //   // Hide app launcher if you prefer Spotlight/Raycast
  //   // "builtin/app-launcher": {
  //   //   hidden: true
  //   // },
  //
  //   // Emoji picker with Cmd+Ctrl+Space
  //   // "builtin/emoji-picker": {
  //   //   shortcut: { modifiers: ["meta", "ctrl"], key: "Space" }
  //   // },
  //
  //   // ─────────────────────────────────────────────────────────────────────
  //   // APPLICATIONS (by macOS bundle identifier)
  //   // ─────────────────────────────────────────────────────────────────────
  //   // Find bundle IDs with: osascript -e 'id of app "App Name"'
  //
  //   // Quick launch Safari with Cmd+Shift+S
  //   // "app/com.apple.Safari": {
  //   //   shortcut: { modifiers: ["meta", "shift"], key: "KeyS" }
  //   // },
  //
  //   // Quick launch VS Code with Cmd+Shift+C
  //   // "app/com.microsoft.VSCode": {
  //   //   shortcut: { modifiers: ["meta", "shift"], key: "KeyC" }
  //   // },
  //
  //   // ─────────────────────────────────────────────────────────────────────
  //   // USER SCRIPTS (by filename without .ts extension)
  //   // ─────────────────────────────────────────────────────────────────────
  //   // Scripts are in ~/.scriptkit/plugins/main/scripts/
  //
  //   // Add shortcut to a frequently-used script
  //   // "script/my-workflow": {
  //   //   shortcut: { modifiers: ["meta", "shift"], key: "KeyW" }
  //   // },
  //
  //   // Hide a deprecated script but keep it accessible via deeplink
  //   // "script/deprecated-helper": {
  //   //   hidden: true
  //   // },
  //
  //   // ─────────────────────────────────────────────────────────────────────
  //   // SCRIPTLETS (inline scripts by UUID or name)
  //   // ─────────────────────────────────────────────────────────────────────
  //
  //   // Add shortcut to a scriptlet
  //   // "scriptlet/clipboard-to-uppercase": {
  //   //   shortcut: { modifiers: ["meta", "shift"], key: "KeyU" }
  //   // },
  // },

  // Hide commands from the launcher main menu by canonical command ID.
  // Hidden commands stay resolvable via shortcuts, deeplinks, and
  // triggerBuiltin — this only filters them from visible lists.
  // hiddenCommands: [
  //   "builtin/clipboard-history",
  //   "script/deprecated-tool",
  // ],

  // ===========================================================================
  // Prompt Targets
  // ===========================================================================
  // Hand the built prompt off to an external tool. Targets appear in the
  // Actions menu as prompt-target/<id> commands (assign shortcuts via
  // `commands`). Prompt text arrives through the SCRIPT_KIT_PROMPT env var
  // or {prompt} / {promptFile} placeholders in args/env.

  // promptTargets: {
  //   "my-app": {
  //     title: "My App",
  //     command: "/usr/local/bin/my-app",
  //     args: ["--prompt", "{prompt}"],
  //   },
  // },

  // ===========================================================================
  // Process Limits
  // ===========================================================================
  // Control resource usage for running scripts.
  // Leave undefined for no limits.

  // processLimits: {
  //   // Maximum memory usage in MB (scripts exceeding this may be terminated)
  //   maxMemoryMb: 512,
  //
  //   // Maximum runtime in seconds (scripts running longer will be terminated)
  //   maxRuntimeSeconds: 300,  // 5 minutes
  //
  //   // How often to check script health (in milliseconds)
  //   healthCheckIntervalMs: 5000,  // 5 seconds
  // },

  // ===========================================================================
  // Suggested Commands (Frecency)
  // ===========================================================================
  // Controls the "Suggested" section in the main menu.

  // suggested: {
  //   enabled: true,       // Show suggested section
  //   maxItems: 10,        // Max items in the section
  //   minScore: 0.1,       // Minimum frecency score to include
  //   halfLifeDays: 7,     // Decay half-life in days
  //   trackUsage: true,    // Track command usage
  //   excludedCommands: ["builtin-quit-script-kit"] // Command IDs to exclude
  // },

  // ===========================================================================
  // File Watcher
  // ===========================================================================
  // Debounce and back-off settings for the file watcher.

  // watcher: {
  //   debounceMs: 500,
  //   stormThreshold: 200,
  //   initialBackoffMs: 100,
  //   maxBackoffMs: 30000,
  //   maxNotifyErrors: 10,
  // },

  // ===========================================================================
  // Window Layout
  // ===========================================================================
  // Sizing defaults for the launcher window.

  // layout: {
  //   standardHeight: 500,
  //   maxHeight: 700,
  // },

  // ===========================================================================
  // Agent Chat CLI Provider
  // ===========================================================================
  // Controls Agent Chat and compatibility harness launch settings.
  // When Agent Chat is invoked, Script Kit writes context to ~/.scriptkit/context/
  // and spawns the claude CLI with --append-system-prompt and the user intent.

  // claudeCode: {
  //   enabled: true,
  //   path: "/opt/homebrew/bin/claude",   // default: "claude" from PATH
  //   permissionMode: "plan",             // "plan" | "dontAsk"
  //   allowedTools: "Read,Edit,Bash(git:*)",
  //   addDirs: ["/Users/you/projects"],
  // },

  // ===========================================================================
  // MCP Servers
  // ===========================================================================
  // Configure Model Context Protocol servers available to AI tooling.
  // enabled: false is safe and is the default until you opt in.

  mcp: {
    enabled: false,
    servers: {
      // example: {
      //   "filesystem": {
      //     command: "npx",
      //     args: ["-y", "@modelcontextprotocol/server-filesystem"],
      //   },
      // }
    },
  },

  // ===========================================================================
  // Brain Remote Access (Telegram)
  // ===========================================================================
  // Opt-in remote capture/queries into the brain (local memory) via a
  // Telegram bot. SECURITY: the bot token grants remote access into your
  // local memory — treat it like a password. The bridge only runs when
  // enabled is true, a bot token is set, AND telegramAllowedUserIds is
  // non-empty; an empty allowlist disables the bot entirely.

  // brainRemote: {
  //   enabled: false,
  //   telegramBotToken: "123456:ABC-DEF...",  // from @BotFather; keep secret
  //   telegramAllowedUserIds: [123456789],     // numeric Telegram user IDs
  // },

  // ===========================================================================
  // Advanced Settings
  // ===========================================================================
  // These settings are rarely needed but available for special cases.

  // Custom path to the bun executable (auto-detected by default)
  // bun_path: "/opt/homebrew/bin/bun",
} satisfies Config;
