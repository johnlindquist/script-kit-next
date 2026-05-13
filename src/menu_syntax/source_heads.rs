//! Single canonical source-of-truth for committed source-filter heads.
//!
//! Owns the parser-known committed-head list, the per-source labels, the
//! browse-mode blurb, the user-facing runnable example, and a stable lat
//! section id that backs the descriptor. Parser, input highlighting,
//! trigger popup, hint legend, and lat docs all consume this list.

use super::payload::RootUnifiedSourceFilter;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceHeadDescriptor {
    pub source: RootUnifiedSourceFilter,
    /// Long committed head including the trailing colon, e.g. `"vault:"`.
    pub long: &'static str,
    /// Short committed alias including the trailing colon, e.g. `"v:"`.
    pub short: Option<&'static str>,
    /// User-facing source label, e.g. `"AI Vault"`.
    pub label: &'static str,
    /// Short tooltip / picker subtitle (reused by trigger picker rows).
    pub description: &'static str,
    /// One-line explanation of what `<head>: ` browses with an empty
    /// stripped query.
    pub browse_blurb: &'static str,
    /// Runnable example for searching this source.
    pub example: &'static str,
    /// Durable lat section id backing this descriptor.
    pub lat_section_id: &'static str,
}

pub const ADVANCED_QUERY_LAT_SECTION: &str = "lat.md/menu-syntax#Menu Syntax#Advanced Query";

pub const SOURCE_HEAD_DESCRIPTORS: &[SourceHeadDescriptor] = &[
    SourceHeadDescriptor {
        source: RootUnifiedSourceFilter::Files,
        long: "files:",
        short: Some("f:"),
        label: "Files",
        description: "Search local file results",
        browse_blurb: "Browse recent file matches.",
        example: "f: budget.pdf",
        lat_section_id: ADVANCED_QUERY_LAT_SECTION,
    },
    SourceHeadDescriptor {
        source: RootUnifiedSourceFilter::Notes,
        long: "notes:",
        short: Some("n:"),
        label: "Notes",
        description: "Search note records",
        browse_blurb: "Browse recent notes.",
        example: "n: meeting agenda",
        lat_section_id: ADVANCED_QUERY_LAT_SECTION,
    },
    SourceHeadDescriptor {
        source: RootUnifiedSourceFilter::ClipboardHistory,
        long: "clipboard:",
        short: Some("c:"),
        label: "Clipboard",
        description: "Search clipboard history",
        browse_blurb: "Browse recent clipboard entries.",
        example: "c: order id",
        lat_section_id: ADVANCED_QUERY_LAT_SECTION,
    },
    SourceHeadDescriptor {
        source: RootUnifiedSourceFilter::BrowserTabs,
        long: "tabs:",
        short: Some("t:"),
        label: "Browser Tabs",
        description: "Search current browser tab metadata",
        browse_blurb: "Browse open browser tabs.",
        example: "t: github pr",
        lat_section_id: ADVANCED_QUERY_LAT_SECTION,
    },
    SourceHeadDescriptor {
        source: RootUnifiedSourceFilter::BrowserHistory,
        long: "history:",
        short: Some("h:"),
        label: "Browser History",
        description: "Search browser history metadata",
        browse_blurb: "Browse recent browser history.",
        example: "h: release notes",
        lat_section_id: ADVANCED_QUERY_LAT_SECTION,
    },
    SourceHeadDescriptor {
        source: RootUnifiedSourceFilter::Apps,
        long: "apps:",
        short: Some("a:"),
        label: "Apps",
        description: "Search installed apps",
        browse_blurb: "Browse installed apps.",
        example: "a: terminal",
        lat_section_id: ADVANCED_QUERY_LAT_SECTION,
    },
    SourceHeadDescriptor {
        source: RootUnifiedSourceFilter::Scripts,
        long: "scripts:",
        short: Some("s:"),
        label: "Scripts",
        description: "Search user-authored Kit scripts and scriptlets",
        browse_blurb: "Browse user scripts and scriptlets.",
        example: "s: deploy",
        lat_section_id: ADVANCED_QUERY_LAT_SECTION,
    },
    SourceHeadDescriptor {
        source: RootUnifiedSourceFilter::Commands,
        long: "commands:",
        short: Some("cmd:"),
        label: "Commands",
        description: "Search executable launcher commands",
        browse_blurb: "Browse executable launcher commands.",
        example: "cmd: open settings",
        lat_section_id: ADVANCED_QUERY_LAT_SECTION,
    },
    SourceHeadDescriptor {
        source: RootUnifiedSourceFilter::Conversations,
        long: "conversations:",
        short: Some("ai:"),
        label: "AI Conversations",
        description: "Search saved AI conversation records",
        browse_blurb: "Browse saved AI conversations.",
        example: "ai: refactor plan",
        lat_section_id: ADVANCED_QUERY_LAT_SECTION,
    },
    SourceHeadDescriptor {
        source: RootUnifiedSourceFilter::AiVault,
        long: "vault:",
        short: Some("v:"),
        label: "AI Vault",
        description: "Search cmux AI conversation vault sessions",
        browse_blurb: "Browse recent AI Vault sessions.",
        example: "v: project alpha",
        lat_section_id: ADVANCED_QUERY_LAT_SECTION,
    },
    SourceHeadDescriptor {
        source: RootUnifiedSourceFilter::Dictation,
        long: "dictation:",
        short: Some("d:"),
        label: "Dictation",
        description: "Search saved dictation records",
        browse_blurb: "Browse dictation transcripts.",
        example: "d: standup",
        lat_section_id: ADVANCED_QUERY_LAT_SECTION,
    },
    SourceHeadDescriptor {
        source: RootUnifiedSourceFilter::Windows,
        long: "windows:",
        short: Some("w:"),
        label: "Windows",
        description: "Search window records",
        browse_blurb: "Browse open windows.",
        example: "w: chrome",
        lat_section_id: ADVANCED_QUERY_LAT_SECTION,
    },
    SourceHeadDescriptor {
        source: RootUnifiedSourceFilter::Processes,
        long: "processes:",
        short: Some("p:"),
        label: "Processes",
        description: "Inspect running Kit and system processes",
        browse_blurb: "Browse running Kit and system processes.",
        example: "p: node",
        lat_section_id: ADVANCED_QUERY_LAT_SECTION,
    },
];

/// Compatibility alias for code that still names the type `SourceHeadSpec`.
pub type SourceHeadSpec = SourceHeadDescriptor;

/// Compatibility alias for the legacy const name. Kept so call sites that
/// reference `SOURCE_HEAD_SPECS` (parser, input span highlighter, trigger
/// picker, payload helpers) keep working while a single list backs them.
pub const SOURCE_HEAD_SPECS: &[SourceHeadDescriptor] = SOURCE_HEAD_DESCRIPTORS;

pub fn iter_source_heads() -> impl Iterator<Item = &'static SourceHeadDescriptor> {
    SOURCE_HEAD_DESCRIPTORS.iter()
}

pub fn source_head_for_token(token: &str) -> Option<&'static SourceHeadDescriptor> {
    let lower = token.to_ascii_lowercase();
    SOURCE_HEAD_DESCRIPTORS.iter().find(|descriptor| {
        lower == descriptor.long || descriptor.short.is_some_and(|short| lower == short)
    })
}

pub fn source_head_for_source(
    source: RootUnifiedSourceFilter,
) -> Option<&'static SourceHeadDescriptor> {
    SOURCE_HEAD_DESCRIPTORS
        .iter()
        .find(|descriptor| descriptor.source == source)
}

pub fn source_for_head(head_with_colon: &str) -> Option<RootUnifiedSourceFilter> {
    source_head_for_token(head_with_colon.trim()).map(|descriptor| descriptor.source)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn descriptors_have_required_strings() {
        for descriptor in iter_source_heads() {
            assert!(descriptor.long.ends_with(':'), "{:?}", descriptor);
            assert!(!descriptor.label.is_empty());
            assert!(!descriptor.browse_blurb.is_empty());
            assert!(!descriptor.example.is_empty());
            assert!(!descriptor.lat_section_id.is_empty());
            if let Some(short) = descriptor.short {
                assert!(short.ends_with(':'));
            }
        }
    }

    #[test]
    fn token_lookup_matches_long_and_short() {
        let vault = source_head_for_token("vault:").unwrap();
        assert_eq!(vault.source, RootUnifiedSourceFilter::AiVault);
        let short = source_head_for_token("V:").unwrap();
        assert_eq!(short.source, RootUnifiedSourceFilter::AiVault);
    }

    #[test]
    fn processes_head_resolves_long_and_short() {
        let long = source_head_for_token("processes:").expect("processes: must resolve");
        assert_eq!(long.source, RootUnifiedSourceFilter::Processes);
        let short = source_head_for_token("p:").expect("p: must resolve");
        assert_eq!(short.source, RootUnifiedSourceFilter::Processes);
        // Mixed case still resolves.
        let mixed = source_head_for_token("P:").expect("P: must resolve");
        assert_eq!(mixed.source, RootUnifiedSourceFilter::Processes);
    }

    #[test]
    fn process_source_head_resolves_through_payload_alias() {
        assert_eq!(
            source_for_head("processes:"),
            Some(RootUnifiedSourceFilter::Processes)
        );
        assert_eq!(
            source_for_head("p:"),
            Some(RootUnifiedSourceFilter::Processes)
        );
    }
}
