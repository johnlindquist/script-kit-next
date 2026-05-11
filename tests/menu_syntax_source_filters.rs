use script_kit_gpui::menu_syntax::parse::{parse, MenuSyntaxParse};
use script_kit_gpui::menu_syntax::RootUnifiedSourceFilter;

#[test]
fn inline_source_filters_parse_for_files_notes_and_clipboard() {
    for (input, free_text, source) in [
        ("project :f", "project", RootUnifiedSourceFilter::Files),
        (":n meeting", "meeting", RootUnifiedSourceFilter::Notes),
        (
            "invoice :c",
            "invoice",
            RootUnifiedSourceFilter::ClipboardHistory,
        ),
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
    assert_eq!(parse("project \":f\""), MenuSyntaxParse::None);
    assert_eq!(parse("project :unknown"), MenuSyntaxParse::None);
}
