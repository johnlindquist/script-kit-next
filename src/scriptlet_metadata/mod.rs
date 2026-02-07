//! Scriptlet codefence metadata parser
//!
//! Parses `\`\`\`metadata` and `\`\`\`schema` codefence blocks from markdown scriptlets.
//! These blocks provide an alternative to the HTML comment metadata format, using
//! JSON directly in labeled code fences.
//!
//! # Example scriptlet with codefences:
//! ````markdown
//! # Quick Todo
//!
//! ```metadata
//! { "name": "Quick Todo", "description": "Add a todo item" }
//! ```
//!
//! ```schema
//! { "input": { "item": { "type": "string", "required": true } } }
//! ```
//!
//! ```ts
//! const { item } = await input();
//! await addTodo(item);
//! ```
//! ````

include!("part_000.rs");
include!("part_001.rs");
