use script_kit_gpui::menu_syntax::parse::{parse, MenuSyntaxParse};
use script_kit_gpui::menu_syntax::RootUnifiedSourceFilter;

#[test]
fn inline_source_filters_parse_for_files_notes_and_clipboard() {
    for (input, free_text, source) in [
        ("project files:", "project", RootUnifiedSourceFilter::Files),
        ("f: project", "project", RootUnifiedSourceFilter::Files),
        ("n: meeting", "meeting", RootUnifiedSourceFilter::Notes),
        ("notes: meeting", "meeting", RootUnifiedSourceFilter::Notes),
        (
            "invoice c:",
            "invoice",
            RootUnifiedSourceFilter::ClipboardHistory,
        ),
        (
            "clipboard: invoice",
            "invoice",
            RootUnifiedSourceFilter::ClipboardHistory,
        ),
        ("tabs: docs", "docs", RootUnifiedSourceFilter::BrowserTabs),
        ("t: docs", "docs", RootUnifiedSourceFilter::BrowserTabs),
        (
            "history: docs",
            "docs",
            RootUnifiedSourceFilter::BrowserHistory,
        ),
        ("h: docs", "docs", RootUnifiedSourceFilter::BrowserHistory),
        ("apps: zed", "zed", RootUnifiedSourceFilter::Apps),
        ("a: zed", "zed", RootUnifiedSourceFilter::Apps),
        ("scripts: build", "build", RootUnifiedSourceFilter::Scripts),
        ("s: build", "build", RootUnifiedSourceFilter::Scripts),
        (
            "commands: build",
            "build",
            RootUnifiedSourceFilter::Commands,
        ),
        ("cmd: build", "build", RootUnifiedSourceFilter::Commands),
        (
            "conversations: plan",
            "plan",
            RootUnifiedSourceFilter::Conversations,
        ),
        ("ai: plan", "plan", RootUnifiedSourceFilter::Conversations),
        (
            "dictation: note",
            "note",
            RootUnifiedSourceFilter::Dictation,
        ),
        ("d: note", "note", RootUnifiedSourceFilter::Dictation),
        ("windows: zed", "zed", RootUnifiedSourceFilter::Windows),
        ("w: zed", "zed", RootUnifiedSourceFilter::Windows),
    ] {
        match parse(input) {
            MenuSyntaxParse::AdvancedQuery(query) => {
                assert_eq!(query.free_text, free_text);
                assert!(query.predicates.is_empty());
                assert!(query.source_filters.allows(source));
            }
            other => panic!("expected source-filter query for {input}, got {other:?}"),
        }
    }
}

#[test]
fn source_filters_are_standalone_tokens_only() {
    assert_eq!(parse("project:x"), MenuSyntaxParse::None);
    assert_eq!(parse("project \"f:\""), MenuSyntaxParse::None);
    assert_eq!(parse("project unknown:"), MenuSyntaxParse::None);
    assert_eq!(parse("project :f"), MenuSyntaxParse::None);
    assert_eq!(parse("project p:"), MenuSyntaxParse::None);
    assert_eq!(parse("project processes:"), MenuSyntaxParse::None);
}

#[test]
fn source_filter_exclusion_is_structured_and_exclusion_wins() {
    match parse("files: -files: png") {
        MenuSyntaxParse::AdvancedQuery(query) => {
            assert_eq!(query.free_text, "png");
            assert!(query
                .source_filters
                .includes(RootUnifiedSourceFilter::Files));
            assert!(query
                .source_filters
                .excludes(RootUnifiedSourceFilter::Files));
            assert!(!query.source_filters.allows(RootUnifiedSourceFilter::Files));
        }
        other => panic!("expected source-filter query, got {other:?}"),
    }
}
