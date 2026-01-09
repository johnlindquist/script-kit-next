# Editor Find/Replace Feature Expert Bundle

## Original Goal

> Fixing the editor find/replace feature
>
> This is the original task description that prompted the creation of this bundle.

## Executive Summary

The editor component uses gpui-component's TextInput but the find/replace functionality may have issues with search state management, highlighting matches, or replace operations. The editor supports syntax highlighting and templates.

### Key Problems:
1. **Find state not persisting** - Search text may reset unexpectedly
2. **Match highlighting** - Visual indicators for matches may not render correctly
3. **Replace-all performance** - Large files may freeze on bulk replace

### Required Fixes:
1. **src/editor.rs** - Fix find/replace state management and UI binding
2. **src/prompt_handler.rs** - Ensure editor prompt correctly initializes find state
3. **src/syntax.rs** - Verify highlighting doesn't interfere with find

### Files Included:
- `src/editor.rs`: Main editor component with Find/Replace implementation
- `src/prompt_handler.rs`: Handles prompt messages including ShowEditor
- `src/syntax.rs`: Syntax highlighting using syntect
- `src/prompts/base.rs`: Base prompt structures

---

