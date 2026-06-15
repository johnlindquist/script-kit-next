The consensus mostly survives, but the judge overstates one causal claim: `--input -` strongly suggests missing piped stdin, yet no wrapper logs prove stdin forwarding failed. Treat that as the leading hypothesis, not established fact.

Kimi’s parser-edge analysis is useful only if the literal string `--input -` was intended as the subject. It should not drive implementation advice, tests, or parser changes unless the user confirms that is the actual task. Agy/Gemini should be discounted entirely for synthesis; it violated the artifact contract and supplied process claims without evidence.

## Critic JSON

```json
{
  "claims": [
    {
      "claim": "The supplied original task contains no actionable content beyond `--input -`.",
      "source": "consensus",
      "verdict": "survived",
      "evidence_status": "cited",
      "counterargument": "The literal string could itself be the intended topic, but the prompt provides no instruction that it should be analyzed as a parser artifact.",
      "synthesis_instruction": "may_assert"
    },
    {
      "claim": "The run likely represents missing stdin from an automated `--input -` invocation.",
      "source": "synthesis_instructions",
      "verdict": "weakened",
      "evidence_status": "unverified",
      "counterargument": "No wrapper logs, raw stdin transcript, or invocation trace proves that stdin was expected and dropped.",
      "synthesis_instruction": "label_as_hypothesis"
    },
    {
      "claim": "No implementation or repo review should proceed from the provided payload alone.",
      "source": "consensus",
      "verdict": "survived",
      "evidence_status": "cited",
      "counterargument": "Only a confirmed literal-parser-task interpretation would justify further analysis, and that confirmation is absent.",
      "synthesis_instruction": "may_assert"
    },
    {
      "claim": "`--input -` parser-edge guidance is relevant to the final response.",
      "source": "unique_insights",
      "verdict": "weakened",
      "evidence_status": "unverified",
      "counterargument": "No specific parser, CLI, or code path was nominated, so generic parser advice risks becoming invented scope.",
      "synthesis_instruction": "label_as_hypothesis"
    },
    {
      "claim": "Agy/Gemini inspected workspace files and identified relevant context.",
      "source": "unsupported_claims",
      "verdict": "refuted",
      "evidence_status": "contradicted",
      "counterargument": "Its output does not provide the requested panel artifact or verifiable evidence, and it discusses model/process details despite instructions forbidding that.",
      "synthesis_instruction": "drop"
    }
  ],
  "missed_constraints": [
    "Do not write a substantive repo review or implementation plan from ambient repository context.",
    "Do not present stdin-forwarding failure as verified without wrapper evidence.",
    "Do not convert Kimi's generic parser notes into concrete test or code recommendations unless the literal flag string was intended as the task."
  ],
  "synthesis_must_include": [
    "The only provided original task content is `--input -`.",
    "The actual task or artifact should be resent.",
    "Checking stdin forwarding is a conditional recommendation if this came from an automated pipeline run."
  ],
  "synthesis_must_avoid": [
    "Claiming the repo was inspected or that any source facts are established.",
    "Relying on Agy/Gemini's output as evidence.",
    "Over-explaining parser behavior as though a specific parser implementation was under review."
  ]
}
```
