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
fn source_filters_accept_attached_query_text() {
    for (input, free_text, source) in [
        ("c:skip", "skip", RootUnifiedSourceFilter::ClipboardHistory),
        (
            "clipboard:skip",
            "skip",
            RootUnifiedSourceFilter::ClipboardHistory,
        ),
        ("f:report", "report", RootUnifiedSourceFilter::Files),
        ("files:report", "report", RootUnifiedSourceFilter::Files),
        ("n:meeting", "meeting", RootUnifiedSourceFilter::Notes),
        ("notes:meeting", "meeting", RootUnifiedSourceFilter::Notes),
        ("t:docs", "docs", RootUnifiedSourceFilter::BrowserTabs),
        ("tabs:docs", "docs", RootUnifiedSourceFilter::BrowserTabs),
        ("ai:plan", "plan", RootUnifiedSourceFilter::Conversations),
        (
            "conversations:plan",
            "plan",
            RootUnifiedSourceFilter::Conversations,
        ),
        ("d:note", "note", RootUnifiedSourceFilter::Dictation),
        ("dictation:note", "note", RootUnifiedSourceFilter::Dictation),
        ("a:calendar", "calendar", RootUnifiedSourceFilter::Apps),
        ("apps:calendar", "calendar", RootUnifiedSourceFilter::Apps),
        ("s:build", "build", RootUnifiedSourceFilter::Scripts),
        ("scripts:build", "build", RootUnifiedSourceFilter::Scripts),
        ("cmd:build", "build", RootUnifiedSourceFilter::Commands),
        ("commands:build", "build", RootUnifiedSourceFilter::Commands),
        ("w:finder", "finder", RootUnifiedSourceFilter::Windows),
        ("windows:finder", "finder", RootUnifiedSourceFilter::Windows),
        (
            "h:https://example.com",
            "https://example.com",
            RootUnifiedSourceFilter::BrowserHistory,
        ),
    ] {
        match parse(input) {
            MenuSyntaxParse::AdvancedQuery(query) => {
                assert_eq!(query.free_text, free_text);
                assert!(query.predicates.is_empty());
                assert!(query.source_filters.allows(source));
            }
            other => panic!("expected attached source-filter query for {input}, got {other:?}"),
        }
    }
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

#[test]
fn source_only_filters_parse_as_empty_advanced_queries() {
    for (input, source) in [
        ("f: ", RootUnifiedSourceFilter::Files),
        ("files: ", RootUnifiedSourceFilter::Files),
        ("c: ", RootUnifiedSourceFilter::ClipboardHistory),
        ("clipboard: ", RootUnifiedSourceFilter::ClipboardHistory),
        ("n: ", RootUnifiedSourceFilter::Notes),
        ("notes: ", RootUnifiedSourceFilter::Notes),
        ("t: ", RootUnifiedSourceFilter::BrowserTabs),
        ("h: ", RootUnifiedSourceFilter::BrowserHistory),
        ("ai: ", RootUnifiedSourceFilter::Conversations),
        ("d: ", RootUnifiedSourceFilter::Dictation),
    ] {
        match parse(input) {
            MenuSyntaxParse::AdvancedQuery(query) => {
                assert_eq!(query.free_text, "");
                assert!(query.predicates.is_empty());
                assert!(query.source_filters.allows(source));
            }
            other => panic!("expected empty source-filter query for {input}, got {other:?}"),
        }
    }
}
