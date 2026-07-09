---
engine: fasteng
description: Password input WITH a default — its value must never surface in app state or UI
_inputs:
  _name:
    type: text
    default: friend
  _token:
    type: password
    default: FIXTURE-SECRET-TOKEN-9F2
---
Greet {{ _name }} without revealing any credentials.
