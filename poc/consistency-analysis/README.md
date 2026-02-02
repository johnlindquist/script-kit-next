# Code Consistency Analysis - Script Kit GPUI

This directory contains detailed analyses of code consistency patterns across the Script Kit GPUI codebase.

## Reports

### 03-logging-practices.md (921 lines)

Comprehensive analysis of logging practices in Script Kit GPUI.

**Coverage:**
- Log level usage distribution (238 calls analyzed)
- Structured logging patterns and field naming consistency
- Correlation ID implementation and adoption gaps
- Message formatting standards (JSONL, compact AI format)
- Tracing spans usage (`#[instrument]` macro - only 3 files)
- Performance logging and benchmarking system
- Error handling in logs
- Recommendations with code examples
- Testing strategy

**Key Findings:**

✅ **Strengths:**
- World-class logging infrastructure (2,300 lines in logging.rs)
- Excellent correlation ID system (thread-safe, well-documented)
- Dual-output design (JSONL + human-readable stderr)
- Compact AI format (81% token savings)
- Comprehensive benchmarking utilities
- Extensive unit test coverage (40+ tests)

⚠️ **Gaps:**
- Correlation ID adoption: only 1 location actually sets it
- Tracing spans: only 3 files use `#[instrument]` macro (needs 30+ more)
- Field naming: inconsistent mix of `event_type`, `category`, `action`, `component`
- Debug level: 45% of logs (potentially too verbose)
- Structured fields: inconsistent across codebase (should be in every log)

**Recommendations (Priority Order):**
1. Add correlation ID at critical entry points (hotkeys, protocol, window events)
2. Add structured fields to all logs (standardized field order)
3. Instrument critical paths with `#[instrument]` macro
4. Standardize field naming (deprecate `category` in favor of `event_type` + `action`)
5. Convert high-frequency debug logs to trace level
6. Audit modules for missing event_type fields

**Statistics:**
- Total logging calls analyzed: 238
- Log levels: debug (107), info (39), error (46), warn (23), trace (10)
- Files using tracing: 35+
- Files using `#[instrument]`: 3 (text_injector.rs only)
- Correlation ID usages: 1 (execute_script.rs)

---

## How to Use These Reports

1. **Read the executive summary** (first 2-3 sections) for overview
2. **Review findings tables** for quick reference
3. **Study code examples** (section 11) to see before/after patterns
4. **Check recommendations** (section 10) for implementation priority
5. **Share with team** for discussion on logging standards

## References

- CLAUDE.md logging requirements
- `/src/logging.rs` - Main logging implementation (2,300 lines)
- `/src/logging.rs` tests - Comprehensive test suite (631 lines of tests)
- Example logs in hot paths: hotkeys.rs, execute_script.rs, main.rs

## Implementation Checklist

Using this analysis as reference, the team can track adoption:

- [ ] Add correlation ID at hotkey dispatch entry point
- [ ] Add correlation ID at protocol message entry point  
- [ ] Add correlation ID at window event entry point
- [ ] Audit all 35+ files using tracing:: for structured fields
- [ ] Add `#[instrument]` to 30+ critical functions
- [ ] Standardize on event_type + action field naming
- [ ] Reduce debug-level logging (move frequent calls to trace)
- [ ] Add integration tests for correlation ID persistence
- [ ] Create log query dashboard for dev/staging

---

Generated: 2026-01-30
Analysis tool: Script Kit GPUI Codebase Analysis
