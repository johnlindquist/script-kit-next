//! Core shortcut types with proper error handling and platform-aware display.
//!
//! This module provides:
//! - `Shortcut` - A keyboard shortcut (modifiers + key)
//! - `Modifiers` - Modifier key flags (cmd, ctrl, alt, shift)
//! - `ShortcutParseError` - Detailed parse errors for user feedback
//! - Platform-aware display (⌘⇧K on macOS, Ctrl+Shift+K on Windows/Linux)

// Allow dead code during incremental development - these types will be used
// by the context stack (task 2) and registry (task 3).
#![allow(dead_code)]

use std::fmt;
use thiserror::Error;

/// Errors that can occur when parsing a shortcut string.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum ShortcutParseError {
    #[error("shortcut string is empty")]
    Empty,
    #[error("shortcut has no key, only modifiers")]
    MissingKey,
    #[error("unknown token '{0}' in shortcut")]
    UnknownToken(String),
    #[error("unknown key '{0}'")]
    UnknownKey(String),
}

/// Modifier keys for a shortcut.
///
/// Note on `cmd` (platform accelerator):
/// - On macOS: Command (⌘)
/// - On Windows/Linux: Ctrl
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Modifiers {
    pub cmd: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
}

impl Modifiers {
    pub fn cmd() -> Self {
        Self {
            cmd: true,
            ..Default::default()
        }
    }
    pub fn shift() -> Self {
        Self {
            shift: true,
            ..Default::default()
        }
    }
    pub fn any(&self) -> bool {
        self.cmd || self.ctrl || self.alt || self.shift
    }
    pub fn none(&self) -> bool {
        !self.any()
    }
}

/// Platform enum for display formatting.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Platform {
    MacOS,
    Windows,
    Linux,
}

impl Platform {
    pub fn current() -> Self {
        #[cfg(target_os = "macos")]
        {
            Platform::MacOS
        }
        #[cfg(target_os = "windows")]
        {
            Platform::Windows
        }
        #[cfg(target_os = "linux")]
        {
            Platform::Linux
        }
        #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
        {
            Platform::Linux
        }
    }
}

/// A keyboard shortcut consisting of modifier keys and a main key.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Shortcut {
    pub key: String,
    pub modifiers: Modifiers,
}

impl Shortcut {
    pub fn new(key: impl Into<String>, modifiers: Modifiers) -> Self {
        Self {
            key: canonicalize_key(&key.into()),
            modifiers,
        }
    }

    pub fn parse(s: &str) -> Result<Self, ShortcutParseError> {
        let s = s.trim();
        if s.is_empty() {
            return Err(ShortcutParseError::Empty);
        }

        let normalized = s.replace('+', " ");
        let parts: Vec<&str> = normalized.split_whitespace().collect();
        if parts.is_empty() {
            return Err(ShortcutParseError::Empty);
        }

        let mut modifiers = Modifiers::default();
        let mut key_part: Option<&str> = None;

        for part in &parts {
            let part_lower = part.to_lowercase();
            match part_lower.as_str() {
                "cmd" | "command" | "meta" | "super" | "win" | "⌘" | "mod" => {
                    modifiers.cmd = true
                }
                "ctrl" | "control" | "ctl" | "^" => modifiers.ctrl = true,
                "alt" | "opt" | "option" | "⌥" => modifiers.alt = true,
                "shift" | "shft" | "⇧" => modifiers.shift = true,
                _ => {
                    if key_part.is_some() {
                        return Err(ShortcutParseError::UnknownToken(part.to_string()));
                    }
                    key_part = Some(part);
                }
            }
        }

        let key = key_part.ok_or(ShortcutParseError::MissingKey)?;
        let canonical_key = canonicalize_key(key);
        if !is_known_key(&canonical_key) {
            return Err(ShortcutParseError::UnknownKey(key.to_string()));
        }

        Ok(Self {
            key: canonical_key,
            modifiers,
        })
    }

    pub fn display(&self) -> String {
        self.display_for_platform(Platform::current())
    }

    pub fn display_for_platform(&self, platform: Platform) -> String {
        match platform {
            Platform::MacOS => self.display_macos(),
            Platform::Windows | Platform::Linux => self.display_other(),
        }
    }

    fn display_macos(&self) -> String {
        let mut s = String::new();
        if self.modifiers.ctrl {
            s.push('⌃');
        }
        if self.modifiers.alt {
            s.push('⌥');
        }
        if self.modifiers.shift {
            s.push('⇧');
        }
        if self.modifiers.cmd {
            s.push('⌘');
        }
        s.push_str(&self.key_display());
        s
    }

    fn display_other(&self) -> String {
        let mut parts: Vec<String> = Vec::new();
        if self.modifiers.ctrl {
            parts.push("Ctrl".to_string());
        }
        if self.modifiers.alt {
            parts.push("Alt".to_string());
        }
        if self.modifiers.shift {
            parts.push("Shift".to_string());
        }
        if self.modifiers.cmd {
            parts.push("Super".to_string());
        }
        parts.push(self.key_display_text());
        parts.join("+")
    }

    fn key_display(&self) -> String {
        match self.key.as_str() {
            "enter" => "↵",
            "escape" => "⎋",
            "tab" => "⇥",
            "space" => "␣",
            "backspace" => "⌫",
            "delete" => "⌦",
            "up" => "↑",
            "down" => "↓",
            "left" => "←",
            "right" => "→",
            "home" => "↖",
            "end" => "↘",
            "pageup" => "⇞",
            "pagedown" => "⇟",
            k => return k.to_uppercase(),
        }
        .to_string()
    }

    fn key_display_text(&self) -> String {
        match self.key.as_str() {
            "enter" => "Enter",
            "escape" => "Esc",
            "tab" => "Tab",
            "space" => "Space",
            "backspace" => "Backspace",
            "delete" => "Delete",
            "up" => "Up",
            "down" => "Down",
            "left" => "Left",
            "right" => "Right",
            "home" => "Home",
            "end" => "End",
            "pageup" => "PageUp",
            "pagedown" => "PageDown",
            k => return k.to_uppercase(),
        }
        .to_string()
    }

    pub fn to_canonical_string(&self) -> String {
        let mut parts: Vec<&str> = Vec::new();
        if self.modifiers.alt {
            parts.push("alt");
        }
        if self.modifiers.cmd {
            parts.push("cmd");
        }
        if self.modifiers.ctrl {
            parts.push("ctrl");
        }
        if self.modifiers.shift {
            parts.push("shift");
        }
        parts.push(&self.key);
        parts.join("+")
    }

    pub fn matches_keystroke(&self, keystroke: &gpui::Keystroke) -> bool {
        let canonical = canonicalize_key(&keystroke.key.to_lowercase());
        canonical == self.key
            && keystroke.modifiers.platform == self.modifiers.cmd
            && keystroke.modifiers.control == self.modifiers.ctrl
            && keystroke.modifiers.alt == self.modifiers.alt
            && keystroke.modifiers.shift == self.modifiers.shift
    }
}

impl fmt::Display for Shortcut {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display())
    }
}

/// Canonicalize a key name to the internal standard form.
pub fn canonicalize_key(key: &str) -> String {
    let key_lower = key.to_lowercase();
    match key_lower.as_str() {
        "arrowup" | "uparrow" => "up",
        "arrowdown" | "downarrow" => "down",
        "arrowleft" | "leftarrow" => "left",
        "arrowright" | "rightarrow" => "right",
        "return" => "enter",
        "esc" => "escape",
        "back" => "backspace",
        "del" => "delete",
        "/" | "forwardslash" => "slash",
        "\\" => "backslash",
        ";" => "semicolon",
        "'" | "apostrophe" => "quote",
        "," => "comma",
        "." | "dot" => "period",
        "[" | "leftbracket" => "bracketleft",
        "]" | "rightbracket" => "bracketright",
        "-" | "dash" | "hyphen" => "minus",
        "=" | "equals" => "equal",
        "`" | "backtick" | "grave" => "backquote",
        "pgup" => "pageup",
        "pgdn" | "pgdown" => "pagedown",
        _ => return key_lower,
    }
    .to_string()
}

/// Check if a key name is known/valid.
pub fn is_known_key(key: &str) -> bool {
    matches!(
        key,
        "a" | "b"
            | "c"
            | "d"
            | "e"
            | "f"
            | "g"
            | "h"
            | "i"
            | "j"
            | "k"
            | "l"
            | "m"
            | "n"
            | "o"
            | "p"
            | "q"
            | "r"
            | "s"
            | "t"
            | "u"
            | "v"
            | "w"
            | "x"
            | "y"
            | "z"
            | "0"
            | "1"
            | "2"
            | "3"
            | "4"
            | "5"
            | "6"
            | "7"
            | "8"
            | "9"
            | "f1"
            | "f2"
            | "f3"
            | "f4"
            | "f5"
            | "f6"
            | "f7"
            | "f8"
            | "f9"
            | "f10"
            | "f11"
            | "f12"
            | "f13"
            | "f14"
            | "f15"
            | "f16"
            | "f17"
            | "f18"
            | "f19"
            | "f20"
            | "f21"
            | "f22"
            | "f23"
            | "f24"
            | "space"
            | "enter"
            | "tab"
            | "escape"
            | "backspace"
            | "delete"
            | "up"
            | "down"
            | "left"
            | "right"
            | "home"
            | "end"
            | "pageup"
            | "pagedown"
            | "semicolon"
            | "quote"
            | "comma"
            | "period"
            | "slash"
            | "backslash"
            | "bracketleft"
            | "bracketright"
            | "minus"
            | "equal"
            | "backquote"
    )
}
