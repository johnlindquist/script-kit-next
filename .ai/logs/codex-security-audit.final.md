Implemented the security audit deliverable and wrote the report to:

- `plans/security-audit.md`

The report includes severity-ranked findings for:

- injection vulnerabilities
- path/file-write risks
- insecure deserialization/input-size hardening gaps
- stdin protocol privilege escalation surface
- concrete remediation steps

**Verification run**

I ran:

```bash
cargo check && cargo clippy --all-targets -- -D warnings && cargo test
```

Results:

- `cargo check`: passed
- `cargo clippy --all-targets -- -D warnings`: failed due pre-existing unrelated issues in the repo (examples: duplicate `tests` module, many unused imports in `src/actions/dialog_builtin_action_validation_tests_*.rs`, missing `AiApp::message_body_content` in `src/ai/window.rs`, clippy `needless_return`)
- `cargo test`: not reached because clippy failed in the chained command

**Risks / Known gaps**

- This was a static audit (no exploit harness/fuzzing run).
- Some medium findings depend on deployment trust boundaries for stdin control.
- Highest-risk confirmed issue: unescaped AppleScript interpolation in `src/app_impl.rs` path actions.

**Commits**

- No commits made. (I did not commit because full verification gate is currently failing from pre-existing repo errors.)