use super::prelude;

#[test]
fn test_prompts_prelude_exports_core_prompt_types() {
    let parsed = prelude::parse_command("/explain hello").expect("slash command should parse");
    assert_eq!(parsed.kind, prelude::SlashCommandType::Explain);

    let path_info = prelude::PathInfo::new("notes", "/tmp/notes.md", false);
    assert_eq!(path_info.name, "notes");

    let container_options = prelude::ContainerOptions::default();
    assert!(container_options.background.is_none());

    let _container_padding = prelude::ContainerPadding::Pixels(8.0);
    let _submit_callback: Option<prelude::SubmitCallback> = None;
}
