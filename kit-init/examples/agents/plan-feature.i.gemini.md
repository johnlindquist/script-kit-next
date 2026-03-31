---
_sk_name: "Plan Feature"
_sk_description: "Turn a feature request into an implementation plan"
_sk_icon: "map"
_interactive: true
_inputs:
  feature_name:
    type: text
    message: "Feature name?"
  risk_tolerance:
    type: select
    message: "Risk tolerance?"
    choices: ["low", "medium", "high"]
model: gemini-2.0-flash
---

Create an implementation plan for {{ feature_name }}.

Risk tolerance: {{ risk_tolerance }}.

Include:
- files to change
- data flow
- validation strategy
- tests to add
- rollback plan
