## Role Findings

The design task is missing. `Original task: --input -` appears to be a placeholder or stdin marker, not an actionable requirement.

As architect, I can only evaluate the process contract from the supplied repo instructions: source-first, narrow edits, owner imp consultation for non-trivial owned surfaces, shared UI/token reuse, and smallest failing verification.

## Evidence And Assumptions

Assumption: the intended task text was omitted.

Evidence: no feature, bug, file path, behavior, or desired outcome is included after `--input -`.

## Failure Modes

Proceeding anyway would risk inventing requirements, touching the wrong surface, or giving architectural advice detached from the actual code path.

In this repo specifically, that could also skip the required owner-surface routing through `.agents/imps` and miss the relevant shared component/theme contracts.

## Recommendation

Resend the actual task text after `--input -`, or paste the feature/bug you want reviewed. I can then respond in this architect panel format with concrete design tradeoffs, implementation shape, risks, and verification strategy.

## Self Score

9/10: Correctly blocked on missing input rather than fabricating a design.
