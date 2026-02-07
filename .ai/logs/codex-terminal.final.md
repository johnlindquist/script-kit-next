Implemented the audit deliverable and wrote the report at:

- `crates-audits/terminal-integration.md`

Key findings captured in the report:

1. `AlacrittyEvent::PtyWrite` is currently dropped in `src/terminal/alacritty.rs:116`, which is a protocol-correctness gap.
2. PTY reader thread is detached and not joined (`src/terminal/alacritty.rs:464`, `src/terminal/alacritty.rs:951`), so teardown is nondeterministic.
3. Alacritty request events (`ClipboardLoad/Store`, `ColorRequest`, `TextAreaSizeRequest`) are ignored (`src/terminal/alacritty.rs:129`).
4. PTY lifecycle is mostly correct, but exit-code fidelity can degrade to synthetic `0` fallback (`src/term_prompt.rs:580`).

Verification run:

- `cargo check`  
  - Failed due to existing repo errors outside this audit change:
    - `src/watcher.rs:23` (and nearby): private `config::defaults` usage.
    - `src/hotkeys.rs:367` (and nearby): `&Option<HotkeyConfig>` passed where `&HotkeyConfig` expected.
- `cargo test --lib terminal::`  
  - Failed with the same pre-existing compile errors above.

How to test:

1. Open `crates-audits/terminal-integration.md` and review severity-ranked findings and remediation plan.
2. After baseline compile issues are fixed, rerun:
   - `cargo check`
   - `cargo test --lib terminal::`

Risks / known gaps:

- This task produced an audit report only; no runtime terminal behavior changes were made yet.
- Build/test verification is currently blocked by unrelated workspace compile failures.

Commits made:

- None.