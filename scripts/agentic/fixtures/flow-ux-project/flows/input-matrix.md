---
engine: fasteng
description: One of every input type — proves roster input metadata + password redaction
_inputs:
  _target:
    type: text
    default: world
  _mode:
    type: select
    options:
      - fast
      - careful
    default: fast
  _count:
    type: number
    default: 3
  _dry:
    type: confirm
    default: true
  _token:
    type: password
---
Greet {{ _target }} in {{ _mode }} mode {{ _count }} times (dry={{ _dry }}), auth {{ _token }}.
