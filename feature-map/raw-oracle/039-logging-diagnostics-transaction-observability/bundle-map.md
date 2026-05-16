# Bundle Map

- Feature id: `039-logging-diagnostics-transaction-observability`
- Oracle slug: `logging-diagnostics-observability-atlas`
- Bundle path: `/Users/johnlindquist/.oracle/bundles/logging-diagnostics-observability-atlas.txt`
- Bundle size: `74990` bytes
- Bundle SHA-256: `f83e61d730932a36f18a27369307a393469e894af58593888752bd241c2ec03e`
- Pack summary: 32 files, 227 matches, 75 context windows, about 19.6K exact tokens.
- Pack command:

```bash
packx AGENTS.md CLAUDE.md lat.md/logging.md lat.md/protocol.md lat.md/automation.md lat.md/acp-chat.md lat.md/verification.md .agents/skills/dev-loop-observability/SKILL.md .agents/skills/protocol-automation/SKILL.md .agents/skills/agentic-testing/SKILL.md .agents/skills/testing-quality-gates/SKILL.md .claude/skills/dev-loop/SKILL.md .claude/skills/script-kit-logging/SKILL.md dev.sh src/logging src/protocol/transaction_trace.rs src/protocol/transaction_executor.rs src/protocol_stats.rs src/mcp_resources/transaction_resources.rs src/ai/preflight_audit.rs src/ai/acp/preflight.rs src/ai/window/context_preflight.rs src/main_entry/preflight.rs src/main_window_preflight tests/transaction_trace_contract.rs tests/transaction_trace_resources.rs tests/tx_trace_replay_idempotency_contract.rs tests/tx_trace_wait_for_runtime_contract.rs tests/ai_preflight_persistent_audit_contract.rs tests/protocol_stats_report_contract.rs tests/source_audits/structured_logging.rs tests/source_audits/trace_propagation.rs tests/context_preflight_source_audits.rs scripts/agentic/tx_trace_replay_idempotency.ts --no-interactive --limit 49k -l 5 -s "log_user_value" -s "log_rate_limit" -s "TransactionTrace" -s "append_transaction_trace" -s "read_latest_transaction_trace" -s "compact_transaction_trace_log_if_needed" -s "preflight_audit" -s "AI_PREFLIGHT_AUDIT" -s "protocol_stats" -s "DO_IN_TRACE" -s "SCROLL_TRACE" -s "SCRIPT_KIT_AI_LOG" -s "compact" -o /Users/johnlindquist/.oracle/bundles/logging-diagnostics-observability-atlas.txt
```
