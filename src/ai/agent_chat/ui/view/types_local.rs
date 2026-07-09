//! View-local value types and parsers for Agent Chat.

/// Parsed `SCRIPT_READY path=... validated=true` receipt from assistant output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ScriptReadyReceipt {
    pub path: std::path::PathBuf,
    pub validated: bool,
}

/// Parse the last `SCRIPT_READY path=<path> validated=true` line from text.
pub(crate) fn parse_script_ready_receipt(text: &str) -> Option<ScriptReadyReceipt> {
    let line = text
        .lines()
        .rev()
        .find(|line| line.trim_start().starts_with("SCRIPT_READY "))?;
    let mut path: Option<std::path::PathBuf> = None;
    let mut validated = false;
    for token in line.split_whitespace().skip(1) {
        if let Some(rest) = token.strip_prefix("path=") {
            path = Some(std::path::PathBuf::from(rest));
        } else if token == "validated=true" {
            validated = true;
        }
    }
    Some(ScriptReadyReceipt {
        path: path?,
        validated,
    })
}
