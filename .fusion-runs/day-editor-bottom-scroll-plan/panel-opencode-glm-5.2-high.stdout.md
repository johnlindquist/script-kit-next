## Role Findings
The "task" is literally the string `--input -` with no further content. There is no actionable work specified — no file to edit, no bug to fix, no feature to build, no question to answer. `--input -` is a CLI-style flag (read from stdin), but no body followed it, so there is nothing for me to implement or verify.

## Evidence And Assumptions
- Evidence: The only task text provided is `--input -` (a flag/argument fragment), with no accompanying instruction, target file, or desired outcome.
- Assumption: This is either a paste error, a placeholder that was never filled in, or a meta-prompt where the literal flag *is* the input to reason about.
- I cannot find any referenced file, since no path or content was supplied alongside the flag.

## Failure Modes
- Fabricating a task to look busy (high cost: wrong work, wasted edits).
- Treating `--input -` as a request to read stdin in *this* CLI session (not applicable — I receive messages, not a piped stream here).
- Over-interpreting and producing speculative code/changes against an imagined requirement.

## Recommendation
Do nothing substantive. Ask the user to re-send the actual task content. The smallest implementation that "fully satisfies" an empty task is zero changes — and that is the correct pragmatist answer until a real requirement arrives.

## Self Score
Confidence: high that the task is empty/unparseable as-is. Low risk of being wrong, because acting on nothing is the safe default and the only verifiable outcome here is "no change made, clarification requested."
