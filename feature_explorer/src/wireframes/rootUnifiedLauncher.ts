import { assign, setup } from "xstate";
import type { Feature } from "../state/featureMachine";
import type { WireframeRegistration, WireframeRenderInput } from "./registry";

export const ROOT_UNIFIED_FEATURE_IDS = ["001", "008", "009", "010", "011", "012"] as const;

type RootUnifiedFeatureId = (typeof ROOT_UNIFIED_FEATURE_IDS)[number];
export type RootLauncherSourceId = "commands" | "files" | "clipboard" | "dictation" | "conversations";
type LauncherRowKind = "primary" | "rootFile" | "rootPassive" | "status" | "fallback";
type PrivacyLevel = "publicMock" | "metadataOnly";

interface LauncherRow {
  stableKey: string;
  kind: LauncherRowKind;
  sourceId: RootLauncherSourceId;
  sourceName: string;
  typeLabel: string;
  title: string;
  subtitle: string;
  actionLabel: string;
  featureId: RootUnifiedFeatureId;
  selectable: boolean;
  privacy: PrivacyLevel;
}

export type RootActionId =
  | "openFile"
  | "revealInFinder"
  | "copyPath"
  | "copyName"
  | "quickLook"
  | "browseParentFolder"
  | "pasteClipboard"
  | "copyClipboard"
  | "attachClipboardToAi"
  | "pinClipboard"
  | "deleteClipboard"
  | "pasteDictation"
  | "copyTranscript"
  | "attachDictationToAi"
  | "createNoteFromDictation"
  | "deleteDictation"
  | "resumeConversation"
  | "copyConversationTitle"
  | "copySessionId"
  | "copyConversationPreview"
  | "runCommand"
  | "copyDeepLink"
  | "addShortcut";

interface LauncherAction {
  id: RootActionId;
  label: string;
  section: "Open" | "Copy" | "AI" | "Manage" | "Danger" | "Command";
  shortcut?: string;
  destructive?: boolean;
}

interface RootLauncherScenario {
  id: string;
  label: string;
  query: string;
  passiveRowsEnabled: boolean;
  selectedKey: string;
  actionsOpen?: boolean;
}

export interface RootLauncherContext {
  query: string;
  selectedKey: string;
  activeScenarioId: string;
  passiveRowsEnabled: boolean;
  actionsOpen: boolean;
  actionsSubjectKey: string | null;
  actionsQuery: string;
  selectedActionId: RootActionId | null;
  lastReceipt: {
    kind: "none" | "enter" | "action";
    subjectKey?: string;
    actionId?: RootActionId;
    message?: string;
  };
}

export type RootLauncherEvent =
  | { type: "LOAD_SCENARIO"; scenarioId: string }
  | { type: "TYPE_QUERY"; value: string }
  | { type: "CLICK_SOURCE"; sourceId: RootLauncherSourceId | "all" }
  | { type: "TOGGLE_PASSIVE_ROWS" }
  | { type: "SELECT_ROW"; stableKey: string }
  | { type: "MOVE_SELECTION"; delta: number }
  | { type: "PRESS_ENTER" }
  | { type: "TOGGLE_ACTIONS" }
  | { type: "CLOSE_ACTIONS" }
  | { type: "TYPE_ACTION_QUERY"; value: string }
  | { type: "MOVE_ACTION_SELECTION"; delta: number }
  | { type: "EXECUTE_ACTION"; actionId?: RootActionId };

export interface RootLauncherSnapshot {
  context: RootLauncherContext;
}

interface ParsedRootLauncherQuery {
  filterText: string;
  computedSearchText: string;
  include: RootLauncherSourceId[];
  exclude: RootLauncherSourceId[];
  sourceFilterMode: boolean;
  leadingColonDiscovery: boolean;
}

const sourceHeads: Record<string, RootLauncherSourceId> = {
  cmd: "commands",
  commands: "commands",
  f: "files",
  files: "files",
  c: "clipboard",
  clipboard: "clipboard",
  d: "dictation",
  dictation: "dictation",
  ai: "conversations",
  conversations: "conversations"
};

const scenarios: RootLauncherScenario[] = [
  {
    id: "plain-passive-history",
    label: "Plain search + passive rows",
    query: "standup",
    passiveRowsEnabled: true,
    selectedKey: "clipboard-history/clip-standup"
  },
  {
    id: "files-source",
    label: "Files source: f:s",
    query: "f:s",
    passiveRowsEnabled: false,
    selectedKey: "file/Users/demo/Documents/standup-notes.md"
  },
  {
    id: "clipboard-source-browse",
    label: "Clipboard browse: c:",
    query: "c:",
    passiveRowsEnabled: false,
    selectedKey: "clipboard-history/clip-standup"
  },
  {
    id: "dictation-source-search",
    label: "Dictation search: d:standup",
    query: "d:standup",
    passiveRowsEnabled: false,
    selectedKey: "dictation-history/dict-standup"
  },
  {
    id: "ai-conversation-source",
    label: "AI conversations: ai: launcher",
    query: "ai: launcher",
    passiveRowsEnabled: false,
    selectedKey: "acp-history/session-launcher-refactor"
  },
  {
    id: "captured-actions",
    label: "Cmd+K captured subject",
    query: "c:skip",
    passiveRowsEnabled: false,
    selectedKey: "clipboard-history/clip-standup",
    actionsOpen: true
  }
];

const mockRows: LauncherRow[] = [
  {
    stableKey: "command/open-main-menu",
    kind: "primary",
    sourceId: "commands",
    sourceName: "Commands",
    typeLabel: "Command",
    title: "Open Script Kit Main Menu",
    subtitle: "Primary launcher command",
    actionLabel: "Run Command",
    featureId: "001",
    selectable: true,
    privacy: "publicMock"
  },
  {
    stableKey: "file/Users/demo/Documents/standup-notes.md",
    kind: "rootFile",
    sourceId: "files",
    sourceName: "Files",
    typeLabel: "Markdown File",
    title: "standup-notes.md",
    subtitle: "~/Documents - modified recently",
    actionLabel: "Open File",
    featureId: "001",
    selectable: true,
    privacy: "publicMock"
  },
  {
    stableKey: "file/Users/demo/Projects/launcher-wireframe.md",
    kind: "rootFile",
    sourceId: "files",
    sourceName: "Files",
    typeLabel: "Markdown File",
    title: "launcher-wireframe.md",
    subtitle: "~/Projects - source-filter notes",
    actionLabel: "Open File",
    featureId: "001",
    selectable: true,
    privacy: "publicMock"
  },
  {
    stableKey: "clipboard-history/clip-standup",
    kind: "rootPassive",
    sourceId: "clipboard",
    sourceName: "Clipboard History",
    typeLabel: "Clipboard",
    title: "Clipboard text metadata match",
    subtitle: "Text - pinned - 12 minutes ago - content hidden",
    actionLabel: "Paste Clipboard",
    featureId: "008",
    selectable: true,
    privacy: "metadataOnly"
  },
  {
    stableKey: "dictation-history/dict-standup",
    kind: "rootPassive",
    sourceId: "dictation",
    sourceName: "Dictation History",
    typeLabel: "Dictation",
    title: "Standup follow-up dictation",
    subtitle: "Target Notes - 00:48 - transcript hidden",
    actionLabel: "Paste Dictation",
    featureId: "009",
    selectable: true,
    privacy: "metadataOnly"
  },
  {
    stableKey: "acp-history/session-launcher-refactor",
    kind: "rootPassive",
    sourceId: "conversations",
    sourceName: "AI Conversations",
    typeLabel: "AI Conversation",
    title: "Launcher refactor plan",
    subtitle: "8 messages - preview hidden",
    actionLabel: "Resume Conversation",
    featureId: "010",
    selectable: true,
    privacy: "metadataOnly"
  }
];

const initialScenario = scenarios[0];
const initialRootLauncherContext: RootLauncherContext = {
  query: initialScenario.query,
  selectedKey: initialScenario.selectedKey,
  activeScenarioId: initialScenario.id,
  passiveRowsEnabled: initialScenario.passiveRowsEnabled,
  actionsOpen: false,
  actionsSubjectKey: null,
  actionsQuery: "",
  selectedActionId: null,
  lastReceipt: { kind: "none" }
};

export const rootUnifiedLauncherMachine = setup({
  types: {
    context: {} as RootLauncherContext,
    events: {} as RootLauncherEvent
  }
}).createMachine({
  id: "rootUnifiedLauncherWireframe",
  initial: "ready",
  context: initialRootLauncherContext,
  states: {
    ready: {
      on: {
        LOAD_SCENARIO: { actions: assign(loadScenario) },
        TYPE_QUERY: { actions: assign(typeQuery) },
        CLICK_SOURCE: { actions: assign(clickSource) },
        TOGGLE_PASSIVE_ROWS: { actions: assign(togglePassiveRows) },
        SELECT_ROW: { actions: assign(selectRow) },
        MOVE_SELECTION: { actions: assign(moveSelection) },
        PRESS_ENTER: { actions: assign(pressEnter) },
        TOGGLE_ACTIONS: { actions: assign(toggleActions) },
        CLOSE_ACTIONS: { actions: assign(closeActions) },
        TYPE_ACTION_QUERY: { actions: assign(typeActionQuery) },
        MOVE_ACTION_SELECTION: { actions: assign(moveActionSelection) },
        EXECUTE_ACTION: { actions: assign(executeAction) }
      }
    }
  }
});

export const rootUnifiedLauncherRegistration: WireframeRegistration<RootLauncherSnapshot, RootLauncherEvent> = {
  id: "root-unified-launcher",
  title: "Root Unified Search",
  featureIds: [...ROOT_UNIFIED_FEATURE_IDS],
  summary: "ScriptList source filters, passive history rows, stable row identity, and captured MainList actions.",
  render(input) {
    return renderRootUnifiedLauncher(input);
  },
  bind(root, send, selectFeature) {
    bindRootUnifiedLauncher(root, send, selectFeature);
  }
};

function loadScenario({ event }: { event: RootLauncherEvent }) {
  if (event.type !== "LOAD_SCENARIO") return {};
  const scenario = scenarios.find((candidate) => candidate.id === event.scenarioId) ?? initialScenario;
  const context = {
    ...initialRootLauncherContext,
    query: scenario.query,
    selectedKey: scenario.selectedKey,
    activeScenarioId: scenario.id,
    passiveRowsEnabled: scenario.passiveRowsEnabled,
    actionsOpen: Boolean(scenario.actionsOpen)
  };
  const rows = launcherRowsForContext(context);
  const subject = rows.find((row) => row.stableKey === context.selectedKey);
  const actions = actionsForRow(subject);
  return {
    ...context,
    selectedKey: preserveOrFirstSelectable(context.selectedKey, rows),
    actionsSubjectKey: context.actionsOpen ? context.selectedKey : null,
    selectedActionId: context.actionsOpen ? actions[0]?.id ?? null : null
  };
}

function typeQuery({ context, event }: { context: RootLauncherContext; event: RootLauncherEvent }) {
  if (event.type !== "TYPE_QUERY") return {};
  const next = { ...context, query: event.value, activeScenarioId: "custom", actionsOpen: false, actionsSubjectKey: null };
  return {
    query: event.value,
    activeScenarioId: "custom",
    actionsOpen: false,
    actionsSubjectKey: null,
    selectedActionId: null,
    selectedKey: preserveOrFirstSelectable(context.selectedKey, launcherRowsForContext(next))
  };
}

function clickSource({ context, event }: { context: RootLauncherContext; event: RootLauncherEvent }) {
  if (event.type !== "CLICK_SOURCE") return {};
  if (event.sourceId === "all") {
    const next = { ...context, query: "standup", activeScenarioId: "custom", actionsOpen: false };
    return {
      query: next.query,
      activeScenarioId: next.activeScenarioId,
      actionsOpen: false,
      actionsSubjectKey: null,
      selectedActionId: null,
      selectedKey: preserveOrFirstSelectable(context.selectedKey, launcherRowsForContext(next))
    };
  }
  const prefixBySource: Record<RootLauncherSourceId, string> = {
    commands: "cmd:",
    files: "f:",
    clipboard: "c:",
    dictation: "d:",
    conversations: "ai:"
  };
  const parsed = parseRootLauncherQuery(context.query);
  const suffix = parsed.computedSearchText || (event.sourceId === "files" ? "s" : "");
  const nextQuery = `${prefixBySource[event.sourceId]}${suffix}`;
  const next = { ...context, query: nextQuery, activeScenarioId: "custom", actionsOpen: false };
  return {
    query: nextQuery,
    activeScenarioId: "custom",
    actionsOpen: false,
    actionsSubjectKey: null,
    selectedActionId: null,
    selectedKey: preserveOrFirstSelectable(context.selectedKey, launcherRowsForContext(next))
  };
}

function togglePassiveRows({ context }: { context: RootLauncherContext }) {
  const next = { ...context, passiveRowsEnabled: !context.passiveRowsEnabled, activeScenarioId: "custom" };
  return {
    passiveRowsEnabled: next.passiveRowsEnabled,
    activeScenarioId: "custom",
    selectedKey: preserveOrFirstSelectable(context.selectedKey, launcherRowsForContext(next))
  };
}

function selectRow({ event }: { event: RootLauncherEvent }) {
  if (event.type !== "SELECT_ROW") return {};
  return { selectedKey: event.stableKey };
}

function moveSelection({ context, event }: { context: RootLauncherContext; event: RootLauncherEvent }) {
  if (event.type !== "MOVE_SELECTION") return {};
  const rows = launcherRowsForContext(context).filter((row) => row.selectable);
  if (!rows.length) return {};
  const index = rows.findIndex((row) => row.stableKey === context.selectedKey);
  const nextIndex = (index + event.delta + rows.length) % rows.length;
  return { selectedKey: rows[nextIndex]?.stableKey ?? context.selectedKey };
}

function pressEnter({ context }: { context: RootLauncherContext }) {
  const selected = launcherRowsForContext(context).find((row) => row.stableKey === context.selectedKey);
  return {
    lastReceipt: {
      kind: "enter" as const,
      subjectKey: selected?.stableKey,
      message: selected ? `${selected.actionLabel} routed for ${selected.sourceName}` : "No selectable row"
    }
  };
}

function toggleActions({ context }: { context: RootLauncherContext }) {
  if (context.actionsOpen) return closeActions();
  const selected = launcherRowsForContext(context).find((row) => row.stableKey === context.selectedKey);
  const actions = actionsForRow(selected);
  return {
    actionsOpen: true,
    actionsSubjectKey: selected?.stableKey ?? null,
    actionsQuery: "",
    selectedActionId: actions[0]?.id ?? null
  };
}

function closeActions() {
  return {
    actionsOpen: false,
    actionsSubjectKey: null,
    actionsQuery: "",
    selectedActionId: null
  };
}

function typeActionQuery({ context, event }: { context: RootLauncherContext; event: RootLauncherEvent }) {
  if (event.type !== "TYPE_ACTION_QUERY") return {};
  const next = { ...context, actionsQuery: event.value };
  const actions = visibleActionsForContext(next, launcherRowsForContext(next));
  return {
    actionsQuery: event.value,
    selectedActionId: actions[0]?.id ?? null
  };
}

function moveActionSelection({ context, event }: { context: RootLauncherContext; event: RootLauncherEvent }) {
  if (event.type !== "MOVE_ACTION_SELECTION") return {};
  const actions = visibleActionsForContext(context, launcherRowsForContext(context));
  if (!actions.length) return {};
  const index = actions.findIndex((action) => action.id === context.selectedActionId);
  const nextIndex = (index + event.delta + actions.length) % actions.length;
  return { selectedActionId: actions[nextIndex]?.id ?? context.selectedActionId };
}

function executeAction({ context, event }: { context: RootLauncherContext; event: RootLauncherEvent }) {
  if (event.type !== "EXECUTE_ACTION") return {};
  const rows = launcherRowsForContext(context);
  const subject = rows.find((row) => row.stableKey === context.actionsSubjectKey);
  const actionId = event.actionId ?? context.selectedActionId ?? undefined;
  const action = actionsForRow(subject).find((candidate) => candidate.id === actionId);
  return {
    actionsOpen: false,
    actionsSubjectKey: null,
    selectedActionId: null,
    lastReceipt: {
      kind: "action" as const,
      subjectKey: subject?.stableKey,
      actionId,
      message: action && subject ? `${action.label} executed against captured ${subject.sourceName} row` : "No captured action subject"
    }
  };
}

export function parseRootLauncherQuery(input: string): ParsedRootLauncherQuery {
  const trimmed = input.trim();
  if (trimmed.startsWith(":")) {
    return {
      filterText: input,
      computedSearchText: trimmed,
      include: [],
      exclude: [],
      sourceFilterMode: false,
      leadingColonDiscovery: true
    };
  }

  const include = new Set<RootLauncherSourceId>();
  const exclude = new Set<RootLauncherSourceId>();
  const freeText: string[] = [];
  for (const rawToken of trimmed.split(/\s+/).filter(Boolean)) {
    const negative = rawToken.startsWith("-");
    const token = negative ? rawToken.slice(1) : rawToken;
    const match = token.match(/^([a-z]+):(.*)$/i);
    if (!match) {
      freeText.push(rawToken);
      continue;
    }
    const source = sourceHeads[match[1].toLowerCase()];
    if (!source) {
      freeText.push(rawToken);
      continue;
    }
    if (negative) exclude.add(source);
    else include.add(source);
    if (match[2]) freeText.push(match[2]);
  }

  return {
    filterText: input,
    computedSearchText: freeText.join(" "),
    include: [...include],
    exclude: [...exclude],
    sourceFilterMode: include.size > 0 || exclude.size > 0,
    leadingColonDiscovery: false
  };
}

function launcherRowsForContext(context: RootLauncherContext): LauncherRow[] {
  const parsed = parseRootLauncherQuery(context.query);
  const q = parsed.computedSearchText.toLowerCase();
  const sourceOnlyBrowse = parsed.sourceFilterMode && q.length === 0;
  const rows: LauncherRow[] = [];

  if (sourceAllowed(parsed, "commands", !parsed.sourceFilterMode)) rows.push(...matchingRows("commands", q, sourceOnlyBrowse));
  if (sourceAllowed(parsed, "files", q.length >= 2 || sourceOnlyBrowse || parsed.include.includes("files"))) {
    rows.push(...matchingRows("files", q, sourceOnlyBrowse));
    rows.push(sourceStatus("files", "Showing root Files mock rows", "001"));
  }
  if (sourceAllowed(parsed, "clipboard", context.passiveRowsEnabled || parsed.include.includes("clipboard"))) {
    rows.push(...matchingRows("clipboard", q, sourceOnlyBrowse));
    rows.push(sourceStatus("clipboard", "Clipboard metadata only", "008"));
  }
  if (sourceAllowed(parsed, "dictation", context.passiveRowsEnabled || parsed.include.includes("dictation"))) {
    rows.push(...matchingRows("dictation", q, sourceOnlyBrowse));
    rows.push(sourceStatus("dictation", "Dictation transcript hidden until action", "009"));
  }
  if (sourceAllowed(parsed, "conversations", context.passiveRowsEnabled || parsed.include.includes("conversations"))) {
    rows.push(...matchingRows("conversations", q, sourceOnlyBrowse));
    rows.push(sourceStatus("conversations", "Saved conversation metadata", "010"));
  }
  if (!parsed.sourceFilterMode && q.length > 0) {
    rows.push({
      stableKey: `fallback/search-files/${q}`,
      kind: "fallback",
      sourceId: "files",
      sourceName: "Fallback",
      typeLabel: "Fallback",
      title: `Search files for "${q}"`,
      subtitle: "Fallback handoff row",
      actionLabel: "Search Files",
      featureId: "001",
      selectable: true,
      privacy: "publicMock"
    });
  }
  return rows;
}

function sourceAllowed(parsed: ParsedRootLauncherQuery, sourceId: RootLauncherSourceId, ordinaryAllowed: boolean) {
  if (parsed.exclude.includes(sourceId)) return false;
  if (parsed.include.length > 0) return parsed.include.includes(sourceId);
  return ordinaryAllowed;
}

function matchingRows(sourceId: RootLauncherSourceId, query: string, sourceOnlyBrowse: boolean) {
  return mockRows.filter((row) => {
    if (row.sourceId !== sourceId) return false;
    if (sourceOnlyBrowse || !query) return true;
    return [row.title, row.subtitle, row.sourceName, row.typeLabel].join(" ").toLowerCase().includes(query);
  });
}

function sourceStatus(sourceId: RootLauncherSourceId, title: string, featureId: RootUnifiedFeatureId): LauncherRow {
  const sourceName = sourceNameFor(sourceId);
  return {
    stableKey: `status/${sourceId}`,
    kind: "status",
    sourceId,
    sourceName,
    typeLabel: "Status",
    title,
    subtitle: "Non-selectable source status row",
    actionLabel: "Not actionable",
    featureId,
    selectable: false,
    privacy: "metadataOnly"
  };
}

function preserveOrFirstSelectable(previousKey: string, rows: LauncherRow[]) {
  if (rows.some((row) => row.selectable && row.stableKey === previousKey)) return previousKey;
  return rows.find((row) => row.selectable)?.stableKey ?? "";
}

function actionsForRow(row: LauncherRow | undefined): LauncherAction[] {
  if (!row || !row.selectable) return [];
  if (row.kind === "rootFile") {
    return [
      { id: "openFile", label: "Open File", section: "Open" },
      { id: "revealInFinder", label: "Reveal in Finder", section: "Open", shortcut: "Shift Cmd F" },
      { id: "copyPath", label: "Copy Path", section: "Copy", shortcut: "Shift Cmd C" },
      { id: "copyName", label: "Copy Name", section: "Copy" },
      { id: "quickLook", label: "Quick Look", section: "Open", shortcut: "Cmd Y" },
      { id: "browseParentFolder", label: "Browse Parent Folder", section: "Open" }
    ];
  }
  if (row.sourceId === "clipboard") {
    return [
      { id: "pasteClipboard", label: "Paste Clipboard", section: "Open" },
      { id: "copyClipboard", label: "Copy to Clipboard", section: "Copy" },
      { id: "attachClipboardToAi", label: "Attach to AI", section: "AI" },
      { id: "pinClipboard", label: "Pin / Unpin", section: "Manage" },
      { id: "quickLook", label: "Quick Look", section: "Open" },
      { id: "deleteClipboard", label: "Delete Clipboard Entry", section: "Danger", destructive: true }
    ];
  }
  if (row.sourceId === "dictation") {
    return [
      { id: "pasteDictation", label: "Paste Dictation", section: "Open" },
      { id: "copyTranscript", label: "Copy Transcript", section: "Copy" },
      { id: "attachDictationToAi", label: "Attach to AI", section: "AI" },
      { id: "createNoteFromDictation", label: "Create Note from Transcript", section: "Manage" },
      { id: "deleteDictation", label: "Delete Dictation", section: "Danger", destructive: true }
    ];
  }
  if (row.sourceId === "conversations") {
    return [
      { id: "resumeConversation", label: "Resume Conversation", section: "Open" },
      { id: "copyConversationTitle", label: "Copy Conversation Title", section: "Copy" },
      { id: "copySessionId", label: "Copy Session ID", section: "Copy" },
      { id: "copyConversationPreview", label: "Copy Preview", section: "Copy" }
    ];
  }
  return [
    { id: "runCommand", label: "Run Command", section: "Command" },
    { id: "addShortcut", label: "Add Shortcut", section: "Command" },
    { id: "copyDeepLink", label: "Copy Deep Link", section: "Copy" }
  ];
}

function visibleActionsForContext(context: RootLauncherContext, rows: LauncherRow[]) {
  const subject = rows.find((row) => row.stableKey === context.actionsSubjectKey);
  const query = context.actionsQuery.trim().toLowerCase();
  return actionsForRow(subject).filter((action) =>
    !query || [action.id, action.label, action.section].join(" ").toLowerCase().includes(query)
  );
}

function renderRootUnifiedLauncher({ activeFeature, features, snapshot }: WireframeRenderInput<RootLauncherSnapshot>) {
  const context = snapshot?.context ?? initialRootLauncherContext;
  const parsed = parseRootLauncherQuery(context.query);
  const rows = launcherRowsForContext(context);
  const selected = rows.find((row) => row.stableKey === context.selectedKey);
  const receipt = receiptForContext(context, rows);
  return `
    <div class="launcher-wireframe" data-wireframe-root tabindex="0">
      <section class="wireframe-stage">
        <div class="scenario-bar">
          ${scenarios.map((scenario) => `<button class="${scenario.id === context.activeScenarioId ? "active" : ""}" data-wireframe-action="scenario" data-scenario-id="${escapeAttr(scenario.id)}">${escapeHtml(scenario.label)}</button>`).join("")}
        </div>
        <div class="launcher-card">
          <div class="launcher-titlebar">
            <span>Script Kit</span>
            <span>Root Unified Search Mock</span>
          </div>
          <div class="launcher-input-shell">
            <input class="launcher-input" value="${escapeAttr(context.query)}" data-wireframe-action="query" aria-label="Mock launcher query" />
            <div class="computed-query">computedSearchText: <code>${escapeHtml(parsed.computedSearchText || "empty")}</code></div>
            <div class="source-heads">
              ${sourceButton("all", "All", !parsed.sourceFilterMode)}
              ${sourceButton("files", "f:", parsed.include.includes("files"))}
              ${sourceButton("clipboard", "c:", parsed.include.includes("clipboard"))}
              ${sourceButton("dictation", "d:", parsed.include.includes("dictation"))}
              ${sourceButton("conversations", "ai:", parsed.include.includes("conversations"))}
              ${sourceButton("commands", "cmd:", parsed.include.includes("commands"))}
            </div>
          </div>
          <div class="launcher-list">
            ${rows.map((row) => renderLauncherRow(row, row.stableKey === context.selectedKey)).join("") || `<p class="empty-note">No mock rows match this source/query.</p>`}
          </div>
          <div class="launcher-footer">
            <span>Up/Down Select</span>
            <span>Enter ${escapeHtml(selected?.actionLabel ?? "No row")}</span>
            <span>Cmd+K Actions</span>
            <span>Esc Close popup</span>
          </div>
        </div>
        <div class="launcher-toolbar">
          <button data-wireframe-action="enter">Enter</button>
          <button data-wireframe-action="actions">Cmd+K Actions</button>
          <button data-wireframe-action="passive-toggle">${context.passiveRowsEnabled ? "Disable" : "Enable"} passive rows</button>
        </div>
        ${context.actionsOpen ? renderActionsPopover(context, rows) : ""}
      </section>
      <aside class="wireframe-inspector">
        ${renderFeatureScope(features, activeFeature.id)}
        ${renderReceiptPanel(receipt)}
      </aside>
    </div>
  `;
}

function sourceButton(sourceId: RootLauncherSourceId | "all", label: string, active: boolean) {
  return `<button class="${active ? "active" : ""}" data-wireframe-action="source" data-source-id="${sourceId}">${label}</button>`;
}

function renderLauncherRow(row: LauncherRow, selected: boolean) {
  return `
    <button class="launcher-row ${selected ? "selected" : ""} ${row.kind}" data-wireframe-action="select-row" data-key="${escapeAttr(row.stableKey)}" ${row.selectable ? "" : "disabled"}>
      <span class="row-icon">${iconForSource(row.sourceId)}</span>
      <span class="row-copy">
        <strong>${escapeHtml(row.title)}</strong>
        <small>${escapeHtml(row.subtitle)}</small>
      </span>
      <span class="row-meta">
        <span>${escapeHtml(row.typeLabel)}</span>
        <em>${escapeHtml(row.actionLabel)}</em>
      </span>
    </button>
  `;
}

function renderActionsPopover(context: RootLauncherContext, rows: LauncherRow[]) {
  const subject = rows.find((row) => row.stableKey === context.actionsSubjectKey);
  const actions = visibleActionsForContext(context, rows);
  return `
    <div class="actions-popover">
      <header>
        <strong>MainList Actions</strong>
        <small>Captured: ${escapeHtml(subject?.stableKey ?? "none")}</small>
      </header>
      <input class="action-search" value="${escapeAttr(context.actionsQuery)}" data-wireframe-action="action-query" placeholder="Filter actions" />
      <div class="actions-list">
        ${actions.map((action) => `
          <button class="action-row ${action.id === context.selectedActionId ? "selected" : ""} ${action.destructive ? "destructive" : ""}" data-wireframe-action="execute-action" data-action-id="${escapeAttr(action.id)}">
            <span>${escapeHtml(action.label)}<small>${escapeHtml(action.section)}</small></span>
            <em>${escapeHtml(action.shortcut ?? "")}</em>
          </button>
        `).join("") || `<p class="empty-note">No actions for the captured row.</p>`}
      </div>
      <footer>Actions execute against the captured subject, not the current live selection.</footer>
    </div>
  `;
}

function renderFeatureScope(features: Feature[], activeFeatureId: string) {
  return `
    <article class="receipt-card">
      <h2>Feature map slice</h2>
      <p>Root Unified Search / passive history rows / root source actions.</p>
      <div class="feature-chip-grid">
        ${ROOT_UNIFIED_FEATURE_IDS.map((id) => {
          const feature = features.find((candidate) => candidate.id === id);
          return `
            <button class="${id === activeFeatureId ? "active" : ""}" data-wireframe-feature-id="${id}">
              <strong>${id}</strong>
              <span>${escapeHtml(feature?.title.replace(/^\d+\s*/, "") ?? "Missing feature")}</span>
            </button>
          `;
        }).join("")}
      </div>
    </article>
  `;
}

function renderReceiptPanel(receipt: unknown) {
  return `
    <article class="receipt-card">
      <h2>Mock state receipt</h2>
      <pre>${escapeHtml(JSON.stringify(receipt, null, 2))}</pre>
    </article>
  `;
}

function receiptForContext(context: RootLauncherContext, rows: LauncherRow[]) {
  const parsed = parseRootLauncherQuery(context.query);
  const selected = rows.find((row) => row.stableKey === context.selectedKey);
  const captured = rows.find((row) => row.stableKey === context.actionsSubjectKey);
  const visibleActions = visibleActionsForContext(context, rows);
  return {
    surface: "ScriptList",
    filterText: parsed.filterText,
    computedSearchText: parsed.computedSearchText,
    sourceFilters: { include: parsed.include, exclude: parsed.exclude },
    sourceFilterMode: parsed.sourceFilterMode,
    leadingColonDiscovery: parsed.leadingColonDiscovery,
    selectedResultKey: selected?.stableKey ?? null,
    selectedResultRole: selected?.kind ?? null,
    selectedSource: selected?.sourceName ?? null,
    actionsDialog: context.actionsOpen
      ? {
          open: true,
          host: "MainList",
          contextStableKey: captured?.stableKey ?? context.actionsSubjectKey,
          contextSource: captured?.sourceName ?? null,
          selectedActionId: context.selectedActionId,
          visibleActions: visibleActions.map((action) => ({
            id: action.id,
            label: action.label,
            section: action.section,
            shortcut: action.shortcut ?? null,
            destructive: Boolean(action.destructive)
          }))
        }
      : { open: false },
    visibleRows: rows.map((row) => ({
      stableKey: row.stableKey,
      role: row.kind,
      sourceName: row.sourceName,
      selectable: row.selectable,
      privacy: row.privacy
    })),
    lastReceipt: context.lastReceipt
  };
}

function bindRootUnifiedLauncher(
  root: ParentNode,
  send: (event: RootLauncherEvent) => void,
  selectFeature: (id: string) => void
) {
  const frame = root.querySelector<HTMLElement>("[data-wireframe-root]");
  if (!frame) return;
  frame.querySelector<HTMLInputElement>("[data-wireframe-action='query']")?.addEventListener("input", (event) => {
    send({ type: "TYPE_QUERY", value: (event.target as HTMLInputElement).value });
  });
  frame.querySelectorAll<HTMLElement>("[data-wireframe-action='source']").forEach((button) => {
    button.addEventListener("click", () => {
      send({ type: "CLICK_SOURCE", sourceId: (button.dataset.sourceId ?? "all") as RootLauncherSourceId | "all" });
    });
  });
  frame.querySelectorAll<HTMLElement>("[data-wireframe-action='scenario']").forEach((button) => {
    button.addEventListener("click", () => send({ type: "LOAD_SCENARIO", scenarioId: button.dataset.scenarioId ?? "" }));
  });
  frame.querySelectorAll<HTMLElement>("[data-wireframe-action='select-row']").forEach((button) => {
    button.addEventListener("click", () => {
      const stableKey = button.dataset.key;
      if (stableKey) send({ type: "SELECT_ROW", stableKey });
    });
  });
  frame.querySelector<HTMLElement>("[data-wireframe-action='enter']")?.addEventListener("click", () => send({ type: "PRESS_ENTER" }));
  frame.querySelector<HTMLElement>("[data-wireframe-action='actions']")?.addEventListener("click", () => send({ type: "TOGGLE_ACTIONS" }));
  frame.querySelector<HTMLElement>("[data-wireframe-action='passive-toggle']")?.addEventListener("click", () => send({ type: "TOGGLE_PASSIVE_ROWS" }));
  frame.querySelector<HTMLInputElement>("[data-wireframe-action='action-query']")?.addEventListener("input", (event) => {
    send({ type: "TYPE_ACTION_QUERY", value: (event.target as HTMLInputElement).value });
  });
  frame.querySelectorAll<HTMLElement>("[data-wireframe-action='execute-action']").forEach((button) => {
    button.addEventListener("click", () => send({ type: "EXECUTE_ACTION", actionId: button.dataset.actionId as RootActionId }));
  });
  frame.querySelectorAll<HTMLElement>("[data-wireframe-feature-id]").forEach((button) => {
    button.addEventListener("click", () => selectFeature(button.dataset.wireframeFeatureId ?? ""));
  });
  frame.addEventListener("keydown", (event) => {
    if (event.metaKey && event.key.toLowerCase() === "k") {
      event.preventDefault();
      send({ type: "TOGGLE_ACTIONS" });
      return;
    }
    if (event.key === "ArrowDown") {
      event.preventDefault();
      send({ type: "MOVE_SELECTION", delta: 1 });
      return;
    }
    if (event.key === "ArrowUp") {
      event.preventDefault();
      send({ type: "MOVE_SELECTION", delta: -1 });
      return;
    }
    if (event.key === "Enter") {
      event.preventDefault();
      send({ type: "PRESS_ENTER" });
      return;
    }
    if (event.key === "Escape") {
      event.preventDefault();
      send({ type: "CLOSE_ACTIONS" });
    }
  });
}

function sourceNameFor(sourceId: RootLauncherSourceId) {
  return {
    commands: "Commands",
    files: "Files",
    clipboard: "Clipboard History",
    dictation: "Dictation History",
    conversations: "AI Conversations"
  }[sourceId];
}

function iconForSource(sourceId: RootLauncherSourceId) {
  return {
    commands: "Cmd",
    files: "File",
    clipboard: "Clip",
    dictation: "Dict",
    conversations: "AI"
  }[sourceId];
}

function escapeHtml(value: string) {
  return value.replace(/[&<>"']/g, (char) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[char] ?? char);
}

function escapeAttr(value: string) {
  return escapeHtml(value);
}

