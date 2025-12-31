# Script Kit env() System Expert Bundle

## Executive Summary

The env() system provides secure environment variable storage using the macOS Keychain via the `keyring` crate. However, there's a **critical configuration issue**: the `keyring` crate is included without the `apple-native` feature, which means it's using the **mock credential store** instead of the real macOS Keychain. This means secrets are NOT actually being persisted to the system keychain.

Additionally, the user is concerned about ensuring a **smooth experience without credential prompts** when running scripts - this is actually the intended design, but requires proper keyring feature configuration.

### Key Problems:

1. **CRITICAL: Keyring crate missing `apple-native` feature** - The `Cargo.toml` specifies `keyring = "3"` without features, defaulting to the mock store on macOS. The `Cargo.lock` confirms only `log` and `zeroize` dependencies (no `security-framework`).

2. **Non-secret keys not stored** - The `submit()` method only stores values in keyring when `secret: true`, but non-secret env vars are just set in `process.env` and lost between sessions.

3. **SDK secret detection is limited** - Only detects secrets by key name containing 'secret', 'password', 'token', or 'key'. Keys like `OPENAI_API_KEY` would match, but `GITHUB_TOKEN` wouldn't be stored if it doesn't contain those patterns (wait - it does contain 'token', so it would work).

### Required Fixes:

1. **`Cargo.toml` line 87**: Add `apple-native` feature to keyring dependency:
   ```toml
   keyring = { version = "3", features = ["apple-native"] }
   ```

2. **`src/prompts/env.rs` line 160-167**: Consider storing ALL env values (not just secrets) to ensure they persist across sessions.

3. **SDK `scripts/kit-sdk.ts` line 3168-3171**: The secret detection logic works but could be documented better.

### Files Included:

| File | Purpose |
|------|---------|
| `src/prompts/env.rs` | EnvPrompt UI component with keyring integration |
| `src/main.rs` (excerpts) | ShowEnv message handling and auto-submit flow |
| `src/protocol/message.rs` (excerpts) | Message::Env protocol definition |
| `scripts/kit-sdk.ts` (excerpts) | SDK env() function implementation |
| `Cargo.toml` | Dependencies (shows keyring without features) |
| `tests/sdk/test-env.ts` | SDK test file for env() |

---

## Full Code Context

### File: src/prompts/env.rs (375 lines - FULL FILE)

```rust
//! EnvPrompt - Environment variable prompt with keyring storage
//!
//! Features:
//! - Prompt for environment variable values
//! - Secure storage via system keyring (keychain on macOS)
//! - Mask input for secret values
//! - Remember values for future sessions

use gpui::{
    div, prelude::*, px, rgb, Context, FocusHandle, Focusable, Render, SharedString, Window,
};
use std::sync::Arc;

use crate::designs::{get_tokens, DesignVariant};
use crate::logging;
use crate::theme;

use super::SubmitCallback;

/// Service name for keyring storage
const KEYRING_SERVICE: &str = "com.scriptkit.env";

/// Get a secret from the system keyring
pub fn get_secret(key: &str) -> Option<String> {
    let entry = keyring::Entry::new(KEYRING_SERVICE, key);
    match entry {
        Ok(entry) => match entry.get_password() {
            Ok(value) => {
                logging::log("KEYRING", &format!("Retrieved secret for key: {}", key));
                Some(value)
            }
            Err(keyring::Error::NoEntry) => {
                logging::log("KEYRING", &format!("No entry found for key: {}", key));
                None
            }
            Err(e) => {
                logging::log(
                    "KEYRING",
                    &format!("Error retrieving secret for key {}: {}", key, e),
                );
                None
            }
        },
        Err(e) => {
            logging::log(
                "KEYRING",
                &format!("Error creating keyring entry for key {}: {}", key, e),
            );
            None
        }
    }
}

/// Set a secret in the system keyring
pub fn set_secret(key: &str, value: &str) -> Result<(), String> {
    let entry = keyring::Entry::new(KEYRING_SERVICE, key)
        .map_err(|e| format!("Failed to create keyring entry: {}", e))?;

    entry
        .set_password(value)
        .map_err(|e| format!("Failed to store secret: {}", e))?;

    logging::log("KEYRING", &format!("Stored secret for key: {}", key));
    Ok(())
}

/// Delete a secret from the system keyring
#[allow(dead_code)]
pub fn delete_secret(key: &str) -> Result<(), String> {
    let entry = keyring::Entry::new(KEYRING_SERVICE, key)
        .map_err(|e| format!("Failed to create keyring entry: {}", e))?;

    entry
        .delete_credential()
        .map_err(|e| format!("Failed to delete secret: {}", e))?;

    logging::log("KEYRING", &format!("Deleted secret for key: {}", key));
    Ok(())
}

/// EnvPrompt - Environment variable prompt with secure storage
///
/// Prompts for environment variable values and stores them securely
/// using the system keyring. Useful for API keys, tokens, and secrets.
pub struct EnvPrompt {
    /// Unique ID for this prompt instance
    pub id: String,
    /// Environment variable key name
    pub key: String,
    /// Custom prompt text (defaults to "Enter value for {key}")
    pub prompt: Option<String>,
    /// Whether to mask input (for secrets)
    pub secret: bool,
    /// Current input value
    pub input_text: String,
    /// Focus handle for keyboard input
    pub focus_handle: FocusHandle,
    /// Callback when user submits a value
    pub on_submit: SubmitCallback,
    /// Theme for styling
    pub theme: Arc<theme::Theme>,
    /// Design variant for styling
    pub design_variant: DesignVariant,
    /// Whether we checked the keyring already
    checked_keyring: bool,
}

impl EnvPrompt {
    pub fn new(
        id: String,
        key: String,
        prompt: Option<String>,
        secret: bool,
        focus_handle: FocusHandle,
        on_submit: SubmitCallback,
        theme: Arc<theme::Theme>,
    ) -> Self {
        logging::log(
            "PROMPTS",
            &format!("EnvPrompt::new for key: {} (secret: {})", key, secret),
        );

        EnvPrompt {
            id,
            key,
            prompt,
            secret,
            input_text: String::new(),
            focus_handle,
            on_submit,
            theme,
            design_variant: DesignVariant::Default,
            checked_keyring: false,
        }
    }

    /// Check keyring and auto-submit if value exists
    /// Returns true if value was found and submitted
    pub fn check_keyring_and_auto_submit(&mut self) -> bool {
        if self.checked_keyring {
            return false;
        }
        self.checked_keyring = true;

        if let Some(value) = get_secret(&self.key) {
            logging::log(
                "PROMPTS",
                &format!("Found existing value in keyring for key: {}", self.key),
            );
            // Auto-submit the stored value
            (self.on_submit)(self.id.clone(), Some(value));
            return true;
        }
        false
    }

    /// Submit the entered value
    fn submit(&mut self) {
        if !self.input_text.is_empty() {
            // Store in keyring if this is a secret
            if self.secret {
                if let Err(e) = set_secret(&self.key, &self.input_text) {
                    logging::log("ERROR", &format!("Failed to store secret: {}", e));
                }
            }
            (self.on_submit)(self.id.clone(), Some(self.input_text.clone()));
        }
    }

    /// Set the input text programmatically
    pub fn set_input(&mut self, text: String, cx: &mut Context<Self>) {
        if self.input_text == text {
            return;
        }

        self.input_text = text;
        cx.notify();
    }

    /// Cancel - submit None
    fn submit_cancel(&mut self) {
        (self.on_submit)(self.id.clone(), None);
    }

    /// Handle character input
    fn handle_char(&mut self, ch: char, cx: &mut Context<Self>) {
        self.input_text.push(ch);
        cx.notify();
    }

    /// Handle backspace
    fn handle_backspace(&mut self, cx: &mut Context<Self>) {
        if !self.input_text.is_empty() {
            self.input_text.pop();
            cx.notify();
        }
    }

    /// Get display text (masked if secret)
    fn display_text(&self) -> String {
        if self.secret && !self.input_text.is_empty() {
            "•".repeat(self.input_text.len())
        } else {
            self.input_text.clone()
        }
    }
}

impl Focusable for EnvPrompt {
    fn focus_handle(&self, _cx: &gpui::App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for EnvPrompt {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let tokens = get_tokens(self.design_variant);
        let colors = tokens.colors();
        let spacing = tokens.spacing();

        let handle_key = cx.listener(
            |this: &mut Self,
             event: &gpui::KeyDownEvent,
             _window: &mut Window,
             cx: &mut Context<Self>| {
                let key_str = event.keystroke.key.to_lowercase();

                match key_str.as_str() {
                    "enter" => this.submit(),
                    "escape" => this.submit_cancel(),
                    "backspace" => this.handle_backspace(cx),
                    _ => {
                        if let Some(ref key_char) = event.keystroke.key_char {
                            if let Some(ch) = key_char.chars().next() {
                                if !ch.is_control() {
                                    this.handle_char(ch, cx);
                                }
                            }
                        }
                    }
                }
            },
        );

        let (main_bg, text_color, muted_color, search_box_bg, border_color) =
            if self.design_variant == DesignVariant::Default {
                (
                    rgb(self.theme.colors.background.main),
                    rgb(self.theme.colors.text.secondary),
                    rgb(self.theme.colors.text.muted),
                    rgb(self.theme.colors.background.search_box),
                    rgb(self.theme.colors.ui.border),
                )
            } else {
                (
                    rgb(colors.background),
                    rgb(colors.text_secondary),
                    rgb(colors.text_muted),
                    rgb(colors.background_secondary),
                    rgb(colors.border),
                )
            };

        let prompt_text = self
            .prompt
            .clone()
            .unwrap_or_else(|| format!("Enter value for {}", self.key));

        let display_text = self.display_text();
        let input_display = if display_text.is_empty() {
            SharedString::from("Type here...")
        } else {
            SharedString::from(display_text)
        };

        // Icon based on secret mode
        let icon = if self.secret { "🔐" } else { "📝" };

        div()
            .id(gpui::ElementId::Name("window:env".into()))
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .bg(main_bg)
            .text_color(text_color)
            .p(px(spacing.padding_lg))
            .key_context("env_prompt")
            .track_focus(&self.focus_handle)
            .on_key_down(handle_key)
            // Header with icon and key name
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_2()
                    .child(div().text_xl().child(icon))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .child(
                                div()
                                    .text_lg()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .child(prompt_text),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(muted_color)
                                    .child(format!("Key: {}", self.key)),
                            ),
                    ),
            )
            // Input field
            .child(
                div()
                    .mt(px(spacing.padding_lg))
                    .px(px(spacing.item_padding_x))
                    .py(px(spacing.padding_md))
                    .bg(search_box_bg)
                    .border_1()
                    .border_color(border_color)
                    .rounded(px(6.))
                    .text_color(if self.input_text.is_empty() {
                        muted_color
                    } else {
                        text_color
                    })
                    .child(input_display),
            )
            // Footer hint
            .child(
                div()
                    .mt(px(spacing.padding_md))
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .text_sm()
                            .text_color(muted_color)
                            .child(if self.secret {
                                "🔒 Value will be stored securely in system keychain"
                            } else {
                                "Value will be saved to environment"
                            }),
                    ),
            )
            // Keyboard hints
            .child(
                div()
                    .mt(px(spacing.padding_sm))
                    .flex()
                    .flex_row()
                    .gap_4()
                    .child(
                        div()
                            .text_xs()
                            .text_color(muted_color)
                            .child("Enter to submit"),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(muted_color)
                            .child("Esc to cancel"),
                    ),
            )
    }
}
```

### File: Cargo.toml (keyring section - THE PROBLEM)

```toml
# Keyring integration for secure secret storage (env prompt)
keyring = "3"
```

**ISSUE**: No features specified. The keyring crate requires platform-specific features:
- `apple-native` for macOS Keychain
- `windows-native` for Windows Credential Manager
- `linux-native` or `sync-secret-service` for Linux

Without these, keyring uses a **mock credential store** that doesn't persist anything!

### File: SDK env() Implementation (scripts/kit-sdk.ts lines 3133-3176)

```typescript
globalThis.env = async function env(
  key: string,
  promptFn?: () => Promise<string>
): Promise<string> {
  // First check if the env var is already set
  const existingValue = process.env[key];
  if (existingValue !== undefined && existingValue !== '') {
    return existingValue;
  }

  // If a prompt function is provided, use it to get the value
  if (promptFn) {
    const value = await promptFn();
    process.env[key] = value;
    return value;
  }

  // Otherwise, send a message to GPUI to prompt for the value
  const id = nextId();

  return new Promise((resolve) => {
    pending.set(id, (msg: SubmitMessage) => {
      // If user pressed Escape (value is null), exit the script
      if (msg.value === null) {
        process.exit(0);
      }
      const value = msg.value ?? '';
      process.env[key] = value;
      resolve(value);
    });

    const message: EnvMessage = {
      type: 'env',
      id,
      key,
      secret: key.toLowerCase().includes('secret') || 
              key.toLowerCase().includes('password') ||
              key.toLowerCase().includes('token') ||
              key.toLowerCase().includes('key'),
    };

    send(message);
  });
};
```

### File: main.rs ShowEnv Handler (lines 5432-5484)

```rust
PromptMessage::ShowEnv {
    id,
    key,
    prompt,
    secret,
} => {
    tracing::info!(id, key, ?prompt, secret, "ShowEnv received");
    logging::log(
        "UI",
        &format!(
            "ShowEnv prompt received: {} (key: {}, secret: {})",
            id, key, secret
        ),
    );

    // Create submit callback for env prompt
    let response_sender = self.response_sender.clone();
    let submit_callback: std::sync::Arc<dyn Fn(String, Option<String>) + Send + Sync> =
        std::sync::Arc::new(move |id, value| {
            if let Some(ref sender) = response_sender {
                let response = Message::Submit { id, value };
                if let Err(e) = sender.send(response) {
                    logging::log("UI", &format!("Failed to send env response: {}", e));
                }
            }
        });

    // Create EnvPrompt entity
    let focus_handle = self.focus_handle.clone();
    let mut env_prompt = prompts::EnvPrompt::new(
        id.clone(),
        key,
        prompt,
        secret,
        focus_handle,
        submit_callback,
        std::sync::Arc::new(self.theme.clone()),
    );

    // Check keyring first - if value exists, auto-submit without showing UI
    if env_prompt.check_keyring_and_auto_submit() {
        logging::log("UI", "EnvPrompt: value found in keyring, auto-submitted");
        // Don't switch view, the callback already submitted
        cx.notify();
        return;
    }

    let entity = cx.new(|_| env_prompt);
    self.current_view = AppView::EnvPrompt { id, entity };
    self.focused_input = FocusedInput::None; // EnvPrompt has its own focus handling

    defer_resize_to_view(ViewType::ArgPromptNoChoices, 0, cx);
    cx.notify();
}
```

### File: Message::Env Protocol Definition (message.rs lines 202-209)

```rust
/// Environment variable prompt
#[serde(rename = "env")]
Env {
    id: String,
    key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    secret: Option<bool>,
},
```

### File: tests/sdk/test-env.ts (key excerpts)

```typescript
// Name: SDK Test - env()
// Description: Tests env() prompt for environment variable input with keychain storage

/**
 * SDK TEST: test-env.ts
 * 
 * Tests the env() function for secure environment variable prompts:
 * 1. First call: Prompts for value (password-masked for secrets), stores in keychain
 * 2. Subsequent calls: Retrieves from keychain silently without showing UI
 * 
 * Expected behavior:
 * - env(key) sends JSONL message with type: 'env'
 * - Secret detection: keys containing 'secret', 'password', 'token', or 'key' are masked
 * - Values stored securely in system keychain (macOS Keychain)
 * - Subsequent calls return cached value without prompting
 */

import '../../scripts/kit-sdk';

// Test 1: env() with a regular key (non-secret)
const result = await env('MY_CONFIG_VALUE');

// Test 2: env() with a secret key (contains 'secret' in name)
const result = await env('MY_SECRET_VALUE');

// Test 3: env() with custom prompt function
const result = await env('CUSTOM_PROMPTED_VALUE', customPrompt);

// Test 4: env() with existing process.env value
process.env['PRESET_VALUE'] = 'already-set-value';
const result = await env('PRESET_VALUE');
```

---

## Implementation Guide

### Step 1: Fix Keyring Feature Flag (CRITICAL)

```toml
# File: Cargo.toml
# Location: Line 86-87

# BEFORE (BROKEN):
# Keyring integration for secure secret storage (env prompt)
keyring = "3"

# AFTER (FIXED):
# Keyring integration for secure secret storage (env prompt)
# apple-native: Uses macOS Keychain for secure storage
# windows-native: Uses Windows Credential Manager
keyring = { version = "3", features = ["apple-native"] }
```

After this change, run:
```bash
cargo update -p keyring
cargo build
```

Verify the fix by checking `Cargo.lock` for `security-framework` dependency under keyring.

### Step 2: Store ALL env values (not just secrets)

Currently, only secrets are stored in keyring. Consider storing all env values for persistence:

```rust
// File: src/prompts/env.rs
// Location: Line 156-167 (submit function)

// BEFORE:
fn submit(&mut self) {
    if !self.input_text.is_empty() {
        // Store in keyring if this is a secret
        if self.secret {
            if let Err(e) = set_secret(&self.key, &self.input_text) {
                logging::log("ERROR", &format!("Failed to store secret: {}", e));
            }
        }
        (self.on_submit)(self.id.clone(), Some(self.input_text.clone()));
    }
}

// AFTER (stores ALL values):
fn submit(&mut self) {
    if !self.input_text.is_empty() {
        // Store in keyring for persistence (both secrets and regular values)
        if let Err(e) = set_secret(&self.key, &self.input_text) {
            logging::log("ERROR", &format!("Failed to store value: {}", e));
        }
        (self.on_submit)(self.id.clone(), Some(self.input_text.clone()));
    }
}
```

### Step 3: Add SDK functions for managing secrets (Optional)

Add these to `scripts/kit-sdk.ts`:

```typescript
// File: scripts/kit-sdk.ts
// Location: After env() function (around line 3176)

/**
 * Delete a stored environment variable from the keychain
 * @param key - Environment variable key to delete
 */
globalThis.envDelete = async function envDelete(key: string): Promise<void> {
  delete process.env[key];
  const id = nextId();
  return new Promise((resolve) => {
    pending.set(id, () => resolve());
    send({ type: 'envDelete', id, key });
  });
};

/**
 * Check if an env var exists in keychain without prompting
 * @param key - Environment variable key to check
 * @returns true if the value exists in keychain
 */
globalThis.envExists = async function envExists(key: string): Promise<boolean> {
  // First check process.env
  if (process.env[key]) return true;
  
  const id = nextId();
  return new Promise((resolve) => {
    pending.set(id, (msg: { exists: boolean }) => resolve(msg.exists));
    send({ type: 'envCheck', id, key });
  });
};
```

**Note:** This requires corresponding protocol messages and handlers in Rust.

### Step 4: Verify Smooth UX (No Prompts After First Entry)

The current design already supports this when keyring is working:

1. **First call to `env('API_KEY')`**: Shows prompt, user enters value, stored in keychain
2. **Subsequent calls**: `check_keyring_and_auto_submit()` finds value, auto-submits without UI
3. **No macOS permission prompts**: The keyring crate uses the app's keychain access, which is granted automatically to the signed app bundle

For unsigned development builds, the first access may show a macOS dialog asking to allow keychain access - this is a one-time prompt per key and is unavoidable for unsigned apps.

### Testing

After applying fixes:

1. **Verify keyring works:**
```bash
# Build with new feature
cargo build

# Run a test script that uses env()
echo '{"type": "run", "path": "'$(pwd)'/tests/sdk/test-env.ts"}' | \
  SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1 | \
  grep -iE 'keyring|secret|env'
```

2. **Check keychain storage:**
```bash
# On macOS, check Keychain Access.app for "com.scriptkit.env" entries
# Or use security command:
security find-generic-password -s "com.scriptkit.env" -a "MY_API_KEY" -w 2>/dev/null && echo "Found" || echo "Not found"
```

3. **Verify no prompts on subsequent runs:**
- Run a script with `env('TEST_KEY')` twice
- First run: should show prompt
- Second run: should auto-submit without showing UI

---

## Instructions For The Next AI Agent

You are reading the "Script Kit env() System Expert Bundle". This file is self-contained and includes all the context you should assume you have.

Your job:

* Design and describe the minimal, safe changes needed to fully resolve the issues described in the Executive Summary and Key Problems.
* Operate **only** on the files and code snippets included in this bundle. If you need additional files or context, clearly say so.

When you propose changes, follow these rules strictly:

1. Always provide **precise code snippets** that can be copy-pasted directly into the repo.
2. Always include **exact file paths** (e.g. `Cargo.toml` line 87) and, when possible, line numbers or a clear description of the location.
3. Never describe code changes only in prose. Show the full function or block as it should look **after** the change, or show both "before" and "after" versions.
4. Keep instructions **unmistakable and unambiguous**. A human or tool following your instructions should not need to guess what to do.
5. Assume you cannot see any files outside this bundle. If you must rely on unknown code, explicitly note assumptions and risks.

**Key Implementation Details:**

- The keyring crate's `apple-native` feature provides macOS Keychain integration via the `security-framework` crate
- Once enabled, stored credentials persist across app restarts
- The `check_keyring_and_auto_submit()` method is already implemented and working - it just needs real keychain backend
- macOS will NOT prompt for credentials when the app accesses its own keychain entries (only cross-app access requires prompts)
- For production, the app should be code-signed to get automatic keychain access without user prompts

When you answer, work directly with the code and instructions it contains and return a clear, step-by-step plan plus exact code edits.

---
