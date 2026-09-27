use script_kit_gpui::protocol::transaction_executor::{
    execute_batch, stable_transaction_fingerprint, TransactionStateProvider,
};
use script_kit_gpui::protocol::transaction_trace::read_latest_transaction_trace;
use script_kit_gpui::protocol::{
    BatchCommand, TransactionTraceMode, UiStateSnapshot, WaitCondition, WaitNamedCondition,
    TRANSACTION_TRACE_SCHEMA_VERSION,
};

#[derive(Default)]
struct CountingProvider {
    snapshot: UiStateSnapshot,
}

impl TransactionStateProvider for CountingProvider {
    fn snapshot(&self) -> UiStateSnapshot {
        self.snapshot.clone()
    }

    fn set_input(&mut self, text: &str) -> anyhow::Result<()> {
        self.snapshot.input_value = Some(text.to_string());
        Ok(())
    }

    fn select_by_value(&mut self, _value: &str, _submit: bool) -> anyhow::Result<Option<String>> {
        Ok(None)
    }

    fn select_by_semantic_id(
        &mut self,
        _semantic_id: &str,
        _submit: bool,
    ) -> anyhow::Result<Option<String>> {
        Ok(None)
    }
}

#[test]
fn trace_contains_command_payloads_and_schema_version() {
    let request_id = unique_request_id("tx-trace-payload");
    let commands = vec![BatchCommand::SetInput {
        text: "alpha".to_string(),
    }];
    let mut provider = CountingProvider::default();

    let result = execute_batch(
        &mut provider,
        request_id,
        &commands,
        None,
        TransactionTraceMode::On,
    )
    .expect("batch executes");
    let trace = result.trace.expect("trace included");
    assert_eq!(trace.schema_version, TRANSACTION_TRACE_SCHEMA_VERSION);
    assert!(!trace.command_fingerprint.is_empty());
    let Some(BatchCommand::SetInput { text }) = &trace.commands[0].command_payload else {
        panic!("transaction trace must preserve the setInput command shape");
    };
    assert!(text.starts_with("[REDACTED sha256:"));
    assert!(!text.contains("alpha"));
    assert!(trace.command_fingerprint.starts_with("sha256:"));
}

#[test]
fn trace_log_reader_does_not_collect_entire_file() {
    let source = std::fs::read_to_string("src/protocol/transaction_trace.rs")
        .expect("read transaction trace source");
    assert!(
        !source.contains("let lines: Vec<String> = reader.lines().collect"),
        "trace reader must not collect the whole file before scanning"
    );
}

#[test]
fn stable_fingerprint_changes_when_payload_changes() {
    let a = stable_transaction_fingerprint(
        &[BatchCommand::SetInput {
            text: "alpha".to_string(),
        }],
        None,
    )
    .expect("fingerprint a");
    let b = stable_transaction_fingerprint(
        &[BatchCommand::SetInput {
            text: "beta".to_string(),
        }],
        None,
    )
    .expect("fingerprint b");
    assert_ne!(a, b);
}

#[test]
fn malformed_trace_log_lines_do_not_block_latest_valid_match() {
    let request_id = unique_request_id("tx-malformed-log");
    let commands = vec![
        BatchCommand::SetInput {
            text: String::new(),
        },
        BatchCommand::WaitFor {
            condition: WaitCondition::Named(WaitNamedCondition::InputEmpty),
            timeout: Some(1),
            poll_interval: Some(1),
        },
    ];
    let mut provider = CountingProvider::default();

    execute_batch(
        &mut provider,
        request_id.clone(),
        &commands,
        None,
        TransactionTraceMode::On,
    )
    .expect("trace written");

    let latest = read_latest_transaction_trace(None, Some(&request_id))
        .expect("read trace log despite unrelated malformed lines")
        .expect("latest trace exists");
    assert_eq!(latest.request_id, request_id);
}

fn unique_request_id(prefix: &str) -> String {
    format!(
        "{prefix}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    )
}
