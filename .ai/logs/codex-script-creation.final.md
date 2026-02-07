Completed the audit and wrote the report to `plans/script-creation-improvements.md`.

**Changed files**
- `plans/script-creation-improvements.md`

**What’s in the report**
- UX issues in current script/extension creation flow
- Template quality improvements
- Validation gaps
- Filesystem edge cases
- Streamlining opportunities for authoring workflow
- Prioritized implementation phases and proposed test cases

**How to test**
1. Open `plans/script-creation-improvements.md` and review recommendations.
2. (Optional) Run the relevant test command I used:
   - `cargo test script_creation::tests --lib`

**Verification result**
- `cargo test script_creation::tests --lib` failed due to an unrelated existing compile error:
  - `src/ai/window.rs:8549` references missing `AiApp::message_body_content`
- This blocked full test verification for this task.

**Risks / known gaps**
- No runtime/code behavior changed in this task (report-only deliverable).
- Build/test health is currently impacted by unrelated in-progress changes in `src/ai/window.rs`.

**Commits**
- None.