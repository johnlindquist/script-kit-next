use std::fs;

fn read_act() -> String {
    fs::read_to_string("scripts/devtools/act.ts").expect("failed to read scripts/devtools/act.ts")
}

fn slice_between<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    let start_ix = source.find(start).expect("start marker missing");
    let rest = &source[start_ix..];
    let end_ix = rest.find(end).expect("end marker missing");
    &rest[..end_ix]
}

#[test]
fn act_input_actions_do_not_wait_on_stdin_parse_receipts() {
    let source = read_act();

    let dispatch = slice_between(&source, "let actionEnvelope", "const afterArgs");

    assert!(
        !dispatch.contains("shouldUseLauncherSetFilter"),
        "act.set-input must not route ScriptList through setFilter + send --await-parse"
    );
    assert!(
        !dispatch.contains("type: \"setFilter\""),
        "act input dispatch must not depend on stdin_command_parsed logs"
    );
    assert!(
        !source.contains("function printableTextKeyAction"),
        "printable act.key must use target-scoped batch.setInput, not setFilter send --await-parse"
    );
    assert!(
        source.contains("commands: [{ type: \"setInput\", text: args.text }]"),
        "act.set-input must keep using target-scoped batch.setInput"
    );
    assert!(
        source.contains("function textEditingKeyPayload"),
        "printable and Backspace act.key must share target-scoped batch.setInput payload construction"
    );
    assert!(
        source.contains("Array.from(currentInput).slice(0, -1).join(\"\")"),
        "Backspace act.key must delete through target-scoped batch.setInput"
    );
    assert!(
        source.contains(
            "return args.actionKind === \"key\" && !isTextEditingKey(args) ? \"externalCommandResult\" : \"batchResult\";"
        ),
        "navigation act.key must expect externalCommandResult; text edits and non-key actions must expect batchResult"
    );
}

#[test]
fn act_receipt_exposes_dispatch_timing() {
    let source = read_act();
    assert!(
        source.contains("actionTiming") && source.contains("dispatchElapsedMs"),
        "act receipts must expose reliable command/action timing for input responsiveness proof"
    );
}
