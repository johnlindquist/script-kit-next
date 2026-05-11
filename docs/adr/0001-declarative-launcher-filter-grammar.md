# Declarative Launcher Filter Grammar

Launcher filters use declarative, order-independent trailing-colon heads so users can constrain search predictably while keeping `:` as a transient discovery trigger. Source heads such as `files:`/`f:`, `commands:`/`cmd:`, and `conversations:`/`ai:` are valueless universe selectors; property heads such as `type:`, `tag:`, and `shortcut:` require values and use scoped pickers.

We chose this over committed `:f` tokens, suffix add/subtract forms like `f+`, and procedural "filter the current visible list" semantics because async root sources must not change the meaning of an already-typed query. Multiple positive source heads are additive, leading-minus heads exclude, repeated values for one property are OR, different property heads are AND, and exclusion wins conflicts.

Filtered empty, unavailable, invalid, incomplete, and contradiction states remain structured states: they show filter-aware copy and explicit recovery actions rather than broadening to global search or executing generic fallbacks. Filter grammar only constrains retrieval and selection; asking an AI model requires explicit AI intent through an action or command, not inference from filter tokens.
