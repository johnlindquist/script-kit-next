
#[test]
fn cat29_30_action_full_chain() {
    let a = Action::new("id", "T", None, ActionCategory::ScriptContext)
        .with_shortcut("⌘X")
        .with_icon(IconName::Trash)
        .with_section("Danger");
    assert_eq!(a.shortcut.as_deref(), Some("⌘X"));
    assert_eq!(a.icon, Some(IconName::Trash));
    assert_eq!(a.section.as_deref(), Some("Danger"));
}

#[test]
fn cat29_30_action_title_lower_computed() {
    let a = Action::new("id", "Hello World", None, ActionCategory::ScriptContext);
    assert_eq!(a.title_lower, "hello world");
}

#[test]
fn cat29_30_action_description_lower_computed() {
    let a = Action::new(
        "id",
        "T",
        Some("FoO BaR".into()),
        ActionCategory::ScriptContext,
    );
    assert_eq!(a.description_lower.as_deref(), Some("foo bar"));
}

#[test]
fn cat29_30_action_shortcut_lower_computed() {
    let a = Action::new("id", "T", None, ActionCategory::ScriptContext).with_shortcut("⌘E");
    assert_eq!(a.shortcut_lower.as_deref(), Some("⌘e"));
}
