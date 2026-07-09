---
engine: fasteng
description: Three-step DAG — proves step.started/step.completed ordering
_steps:
  - id: gather
    run: gather the facts
  - id: draft
    run: draft the answer
    needs: [gather]
  - id: polish
    run: polish the draft
    needs: [draft]
---
