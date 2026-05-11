# Script Kit Launcher

Script Kit Launcher is the command surface where users combine intent text and filters to choose commands, files, notes, and other local results predictably.

## Language

**Declarative Filter**:
A token that constrains the eligible candidate set regardless of where it appears in the search input.
_Avoid_: procedural filter, current-list filter

**Free Text**:
The non-filter portion of the search input that ranks or matches candidates inside the active constraints.
_Avoid_: remainder, raw search

**Filter Indicator**:
A visible chip or status element that reflects an active declarative filter parsed from the input.
_Avoid_: hidden filter, implicit-only filter state

**Filtered Universe**:
The bounded candidate set made eligible by active declarative filters before free text is applied.
_Avoid_: current list, temporary result set

**Filtered Empty State**:
The no-results message shown when free text produces no matches inside an active filtered universe.
_Avoid_: generic no results, filter-agnostic empty copy

**Canonical Filter Syntax**:
The only documented and supported spelling for declarative filters before the feature ships.
_Avoid_: legacy alias, compatibility shortcut

**Filter Discovery Trigger**:
A transient `:` interaction that opens filter help and inserts canonical filter syntax without becoming part of committed source-filter tokens.
_Avoid_: source-filter sigil, permanent filter prefix

**Valueless Source Head**:
A source filter head whose presence alone selects a result universe, such as `files:` or `f:`.
_Avoid_: source value, source argument

**Valued Property Head**:
A property filter head that is incomplete until it has a supported value, such as `type:script`.
_Avoid_: free-form property prefix

**Source Exclusion**:
A leading-minus source filter that removes a result universe from the otherwise eligible source set.
_Avoid_: source minus suffix, subtractive source shortcut

**Property Exclusion**:
A leading-minus property filter that removes candidates with a matching property value.
_Avoid_: negative property suffix, anti-tag

**Incomplete Filter**:
A known valued property head without a committed value.
_Avoid_: literal partial filter, failed search token

**Invalid Filter Value**:
A value supplied to a known property head that is outside that head's supported value set.
_Avoid_: unknown free text, ignored filter value

**Planned Source Head**:
A canonical source filter included in the grammar so coverage, empty states, and availability behavior are designed before every source is equally mature.
_Avoid_: hidden future source, undocumented source placeholder

**Unavailable Source State**:
The structured no-results state for a known source filter whose provider is disabled, missing permissions, or not indexed yet.
_Avoid_: silent global fallback, unknown source token

**Explicit AI Intent**:
A user action or command that asks an AI model to operate on text, context, or selected results.
_Avoid_: inferred AI intent, filter-implied AI

**Source Head Catalog**:
The canonical set of long and short valueless source heads exposed by filter discovery.
_Avoid_: ad hoc source aliases, hidden source names

**Command Source**:
The broad executable launcher catalog, including built-ins, scripts, scriptlets, app commands, AI commands, and extension commands.
_Avoid_: scripts-only command source

**Script Source**:
The user-authored Kit script and scriptlet catalog.
_Avoid_: all commands, built-ins

**Row Kind**:
The kind of result row after sources have produced candidates, such as Script, App, File, or Command.
_Avoid_: source alias, provider name

**Scoped Value Picker**:
A valued-filter picker whose suggestions are limited to values that can apply inside the active filtered universe.
_Avoid_: global-only value picker, impossible-first suggestions

**Filter Contradiction**:
A structured filter combination that is syntactically valid but cannot produce candidates in the active filtered universe.
_Avoid_: parser error, literal fallback

**Contradiction Empty State**:
An actionable empty state explaining that valid filters cannot produce results together.
_Avoid_: syntax error, fallback execution state

**Filtered Empty Action**:
An explicit action shown inside a filtered empty, unavailable, invalid, incomplete, or contradiction state.
_Avoid_: generic fallback action, implicit broadening

**Filter Removal**:
An explicit edit, chip removal, or empty-state action that relaxes active declarative filters.
_Avoid_: Escape-to-clear, hidden broadening

## Relationships

- A **Declarative Filter** constrains the candidate set before **Free Text** ranks or matches candidates.
- **Free Text** must behave the same whether the **Declarative Filter** appears before or after it.
- A **Filter Indicator** reflects a parsed **Declarative Filter** but does not replace the editable input as the source text.
- A completed **Declarative Filter** followed by space shows the default view of its **Filtered Universe** until **Free Text** narrows it.
- A **Filtered Empty State** names both the active **Filtered Universe** and the **Free Text** that produced no matches.
- Because the source-filter feature has not shipped, **Canonical Filter Syntax** should replace experimental spellings instead of preserving aliases.
- The **Filter Discovery Trigger** helps users insert **Canonical Filter Syntax** such as `files:` or `type:` but is not itself the committed source-filter syntax.
- A **Valueless Source Head** becomes active as soon as its head is complete, while a **Valued Property Head** must keep the picker scoped to valid values until a value is committed.
- Multiple positive **Valueless Source Head** tokens are additive, while **Source Exclusion** removes matching sources and wins conflicts.
- Repeated positive values for the same **Valued Property Head** are OR constraints, different property heads are AND constraints, and **Property Exclusion** removes matching values with exclusion winning conflicts.
- Unknown filter-looking heads stay **Free Text**, while an **Incomplete Filter** or **Invalid Filter Value** remains structured and should show picker or error feedback.
- The filter catalog should surface every **Planned Source Head** so source coverage is designed and tested explicitly.
- A **Planned Source Head** that cannot currently provide rows remains an active filter and shows an **Unavailable Source State** instead of falling back to global search.
- Filter grammar only constrains retrieval and selection; **Explicit AI Intent** must come from an action or command rather than being inferred from filter tokens.
- The **Source Head Catalog** includes canonical long heads and memorable short heads, including `files:`/`f:`, `notes:`/`n:`, `clipboard:`/`c:`, `tabs:`/`t:`, `history:`/`h:`, `apps:`/`a:`, `scripts:`/`s:`, `commands:`/`cmd:`, `conversations:`/`ai:`, `dictation:`/`d:`, `windows:`/`w:`, and `processes:`/`p:`.
- The **Command Source** is broader than the **Script Source** today, though future architecture may move more command implementations into script-backed entries.
- Source heads choose where to search, while `type:` filters by **Row Kind** after candidates exist.
- A **Scoped Value Picker** should suggest values that can apply inside the current **Filtered Universe**, while a manually typed valid-but-impossible value remains structured as a **Filter Contradiction**.
- A **Filter Contradiction** renders a **Contradiction Empty State** with removal or source-switch actions, not a hard parse error or silent fallback.
- Filtered empty, unavailable, invalid, incomplete, and contradiction states suppress generic fallback execution; Enter may only run a visible **Filtered Empty Action**.
- **Filter Removal** happens through input editing, removable indicators, or explicit actions; Escape remains dismiss-first instead of clearing filters.

## Example Dialogue

> **Dev:** "Should `png f:` and `f: png` search different result sets?"
> **Domain expert:** "No. `f:` is a **Declarative Filter**, so both inputs constrain the candidate set to Files and use `png` as **Free Text**."

> **Dev:** "If the user types `png f:`, should the UI only show that raw text?"
> **Domain expert:** "No. Keep the input editable, but show a **Filter Indicator** so Files is visibly active."

> **Dev:** "What should `f: ` show before the user types a filename?"
> **Domain expert:** "It should show the Files **Filtered Universe**, then use later **Free Text** to narrow that universe."

> **Dev:** "What should `zzzz f:` say if no files match?"
> **Domain expert:** "Show a **Filtered Empty State** like `No Files results for \"zzzz\"`, not a generic no-results message."

> **Dev:** "Should we keep both `:f` and `f:`?"
> **Domain expert:** "No. The feature has not shipped, so choose one **Canonical Filter Syntax** and avoid aliases."

> **Dev:** "How does the user discover filters without making every filter start with `:`?"
> **Domain expert:** "Use `:` as a **Filter Discovery Trigger** that inserts committed heads like `files:` or `type:`."

> **Dev:** "Does `type:` behave like `files:`?"
> **Domain expert:** "No. `files:` is a **Valueless Source Head**, while `type:` is a **Valued Property Head** that needs a value such as `script`."

> **Dev:** "What does `files: notes: -notes: invoice` search?"
> **Domain expert:** "Positive source heads are additive, then **Source Exclusion** wins, so this searches Files for `invoice`."

> **Dev:** "What does `type:script type:app -tag:deprecated deploy` mean?"
> **Domain expert:** "Script OR App candidates matching `deploy`, excluding anything tagged deprecated through **Property Exclusion**."

> **Dev:** "Should `typ:script` fail as a filter?"
> **Domain expert:** "No. Unknown heads remain **Free Text**, but `type:` is an **Incomplete Filter** and `type:banana` is an **Invalid Filter Value**."

> **Dev:** "Should the picker hide future sources until each one is fully baked?"
> **Domain expert:** "No. Include every **Planned Source Head** so we can design coverage, unavailable states, and tests now."

> **Dev:** "What if `dictation:` is known but disabled?"
> **Domain expert:** "Keep the filter active and show an **Unavailable Source State** rather than broadening to all sources."

> **Dev:** "Does `ai:` mean ask AI?"
> **Domain expert:** "No. It can search AI-related records, but asking a model requires **Explicit AI Intent**."

> **Dev:** "Should every planned source get a short alias?"
> **Domain expert:** "Use the **Source Head Catalog** with memorable aliases, including `ai:`, `cmd:`, and `p:` where the product meaning is clear."

> **Dev:** "Are `cmd:` and `s:` the same?"
> **Domain expert:** "No. `cmd:` searches the broad **Command Source**, while `s:` searches the user-authored **Script Source**."

> **Dev:** "Is `type:file` another way to say `files:`?"
> **Domain expert:** "No. `files:` chooses the Files source; `type:file` filters by **Row Kind**."

> **Dev:** "What does `files: type:script deploy` mean?"
> **Domain expert:** "It is a **Filter Contradiction** because Script is a valid **Row Kind** but not producible by the Files universe."

> **Dev:** "Should a contradiction be a syntax error?"
> **Domain expert:** "No. Show a **Contradiction Empty State** because the syntax is valid but the filters cannot produce rows together."

> **Dev:** "Should Enter on `zzzz files:` run a generic fallback?"
> **Domain expert:** "No. With filters active, Enter is reserved for a visible **Filtered Empty Action**."

> **Dev:** "Should Escape clear filters?"
> **Domain expert:** "No. Escape dismisses; **Filter Removal** is explicit through editing, chips, or empty-state actions."

## Flagged Ambiguities

- "Search based on the current list" can sound procedural. Resolved: filters are declarative constraints, not operations on whatever rows are currently visible.
