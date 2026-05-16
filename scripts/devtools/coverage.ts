#!/usr/bin/env bun

type CoverageStatus = "supported" | "partial" | "missing" | "planned";

type Domain = {
  id: string;
  name: string;
  chromeAnalogue: string;
  purpose: string;
  currentPrimitives: string[];
  nextPrimitives: string[];
};

type Surface = {
  id: string;
  name: string;
  status: CoverageStatus;
  domains: string[];
  sourceFiles: string[];
  features: string[];
  shortcuts: string[];
  supportedNow: string[];
  missingRuntimePrimitives: string[];
  regressionRecipeRole: string;
};

const notesShortcutCoverage = [
  "Cmd+K",
  "Cmd+P",
  "Cmd+Shift+P",
  "Cmd+F",
  "Cmd+Shift+F",
  "Cmd+N",
  "Cmd+Shift+N",
  "Cmd+Shift+T",
  "Cmd+W",
  "Cmd+.",
  "Cmd+Shift+.",
  "Cmd+Shift+S",
  "Cmd+Z",
  "Cmd+D",
  "Cmd+Shift+D",
  "Cmd+Shift+X",
  "Cmd+Shift+L",
  "Cmd+L",
  "Cmd+Shift+-",
  "Cmd+Shift+H",
  "Cmd+V",
  "Cmd+Shift+C",
  "Cmd+E",
  "Cmd+/",
  "Cmd+J",
  "Cmd+Shift+U",
  "Cmd+B",
  "Cmd+I",
  "Cmd+Shift+I",
  "Cmd+Enter",
  "Cmd+Shift+A",
  "Cmd+Shift+O",
  "Cmd+Up",
  "Cmd+Down",
  "Cmd+Shift+Up",
  "Cmd+Shift+Down",
  "Cmd+[",
  "Cmd+]",
  "Cmd+Shift+Backspace",
  "Cmd+Shift+Delete",
  "Cmd+Shift+7",
  "Cmd+Shift+8",
  "Cmd+1..Cmd+9",
  "Tab",
  "Shift+Tab",
  "Alt+Up",
  "Alt+Down",
  "Alt+Shift+Up",
  "Alt+Shift+Down",
  "Ctrl+Shift+K",
  "Escape",
  "Enter",
  "ArrowUp",
  "ArrowDown",
  "Home",
  "End",
  "PageUp",
  "PageDown",
  "Backspace",
  "Delete",
];

const dictationPhaseCoverage = [
  "idle/hidden",
  "recording",
  "quiet recording",
  "active speech",
  "confirming",
  "stop confirmation",
  "transcribing",
  "delivering",
  "finished",
  "failed/error",
  "Idle -> Recording",
  "Recording -> Confirming",
  "Recording -> Transcribing",
  "Recording -> Failed",
  "Confirming -> Recording",
  "Confirming -> Transcribing",
  "Transcribing -> Delivering",
  "Transcribing -> Failed",
  "Delivering -> Finished",
  "Delivering -> Failed",
  "Finished -> Idle",
  "Failed -> Idle",
];

const domains: Domain[] = [
  {
    id: "targets",
    name: "Targets and Windows",
    chromeAnalogue: "Target/Page",
    purpose: "Discover exact app windows, attached popups, detached panels, parentage, bounds, and screenshot identity.",
    currentPrimitives: ["listAutomationWindows", "inspectAutomationWindow", "devtools.inspect"],
    nextPrimitives: ["devtools.targets.watch", "target capability discovery", "window lifetime timeline"],
  },
  {
    id: "elements",
    name: "Elements and Semantics",
    chromeAnalogue: "DOM/Accessibility tree",
    purpose: "Expose visible UI nodes, roles, labels, selected/focused ids, disabled reasons, actions, owners, and stable semantic ids.",
    currentPrimitives: ["getElements", "inspectAutomationWindow.semanticQuality"],
    nextPrimitives: ["target-scoped semantic collectors", "stable action ids", "semantic tree diff"],
  },
  {
    id: "layout",
    name: "Layout and Box Model",
    chromeAnalogue: "Elements box model / Overlay",
    purpose: "Measure bounds, scroll extents, anchor rects, safe areas, overlap pairs, footer/input/list geometry, and resize deltas.",
    currentPrimitives: ["getLayoutInfo"],
    nextPrimitives: ["devtools.measure.layout", "target-scoped layout info", "scroll geometry", "anchor and overlap reports"],
  },
  {
    id: "styles",
    name: "Styles, Theme, and Text Fit",
    chromeAnalogue: "CSS/Computed styles",
    purpose: "Expose theme tokens, foreground/background colors, contrast, font metrics, wrap lines, truncation intent, and text clipping.",
    currentPrimitives: ["theme contrast source audits", "screenshot pixel probes"],
    nextPrimitives: ["devtools.measure.text", "devtools.measure.contrast", "computed theme tokens per node"],
  },
  {
    id: "console",
    name: "Console, Logs, and Events",
    chromeAnalogue: "Console/Log",
    purpose: "Correlate user actions with app logs, protocol parse failures, warnings, event traces, and structured diagnostics.",
    currentPrimitives: ["scripts/agentic/session.sh logs", "response logs", "app logs"],
    nextPrimitives: ["devtools.events.tail", "action-correlated log spans", "warning taxonomy"],
  },
  {
    id: "sources",
    name: "Sources, Scripts, and Owners",
    chromeAnalogue: "Sources",
    purpose: "Map observed UI nodes and failed measurements to script metadata, prompt type, source files, and likely Rust owners.",
    currentPrimitives: ["promptType", "surfaceContract", "lat.md refs"],
    nextPrimitives: ["owner metadata on semantic nodes", "script provenance receipts", "source jump hints"],
  },
  {
    id: "performance",
    name: "Performance and Timeline",
    chromeAnalogue: "Performance",
    purpose: "Capture resize, filtering, provider refresh, render, async delivery, and focus-transition timelines.",
    currentPrimitives: ["trace logs", "FILTER_PERF logs", "scenario receipts"],
    nextPrimitives: ["devtools.timeline.record", "layout shift timeline", "input-to-paint timings"],
  },
  {
    id: "storage",
    name: "Storage, Resources, and Privacy",
    chromeAnalogue: "Application/Storage",
    purpose: "Inspect redacted resource rows, cache/store identities, context resources, attachment provenance, and privacy boundaries.",
    currentPrimitives: ["kit://context resources", "surface-specific state receipts"],
    nextPrimitives: ["devtools.resources.inspect", "redaction fingerprints", "cache/store generation ids"],
  },
  {
    id: "accessibility",
    name: "Accessibility",
    chromeAnalogue: "Accessibility",
    purpose: "Compare semantic nodes with AX roles, labels, focus order, disabled state, activation affordances, and keyboard reachability.",
    currentPrimitives: ["native computer observation", "semantic roles"],
    nextPrimitives: ["devtools.ax.snapshot", "semantic-to-AX parity diff", "tab order graph"],
  },
  {
    id: "input",
    name: "Input, Focus, and Actions",
    chromeAnalogue: "Input/Runtime",
    purpose: "Drive user-like keys, text, selection, safe clicks, popup dismissal, focus ownership, and wrong-target refusal.",
    currentPrimitives: ["batch", "waitFor", "simulateKey", "target-scoped batch.setInput"],
    nextPrimitives: ["devtools.act", "focus owner transitions", "safe click receipts", "shortcut registry snapshot"],
  },
  {
    id: "media",
    name: "Media, Sensors, and Permissions",
    chromeAnalogue: "Media/Sensors/Permissions",
    purpose: "Inspect microphone readiness, dictation recording states, model readiness, target delivery, permission status, and media cleanup.",
    currentPrimitives: ["dictation story states", "dictation fail-closed scenario specs"],
    nextPrimitives: ["devtools.media.inspect", "passive permission receipts", "transcript delivery generation ids"],
  },
  {
    id: "screenshots",
    name: "Screenshots and Visual Proof",
    chromeAnalogue: "Page.captureScreenshot / Overlay",
    purpose: "Capture strict target screenshots, crop identity, nonblank checks, pixel probes, visual agreement with semantic state, and before/after evidence.",
    currentPrimitives: ["captureScreenshot", "captureWindow", "verify-shot.ts", "inspectAutomationWindow screenshot metadata"],
    nextPrimitives: ["devtools.visual.compare", "semantic text agreement", "occlusion candidates"],
  },
  {
    id: "investigation",
    name: "Investigation Records",
    chromeAnalogue: "Recorder/Protocol Monitor",
    purpose: "Store bug intake, hypotheses, actions, receipts, missing primitives, classification, likely owner, and red/green proof.",
    currentPrimitives: ["manual reports", "scenario receipts"],
    nextPrimitives: ["devtools.investigate", "paired red/green artifact schema", "missing primitive backlog export"],
  },
];

const surfaces: Surface[] = [
  {
    id: "main",
    name: "Main launcher and prompt host",
    status: "partial",
    domains: ["targets", "elements", "layout", "input", "screenshots", "console", "sources"],
    sourceFiles: ["src/app.rs", "src/app_impl/render_impl.rs", "src/widgets/script_list.rs"],
    features: ["script list", "prompt state", "footer", "input", "preview", "surface contract", "source chips"],
    shortcuts: ["Cmd+K", "Escape", "Enter", "Tab", "ArrowUp", "ArrowDown"],
    supportedNow: ["devtools.inspect --main", "getState", "getElements", "getLayoutInfo", "captureScreenshot"],
    missingRuntimePrimitives: ["text fit", "scroll geometry", "layout overlap pairs", "focus ring bounds"],
    regressionRecipeRole: "Use recipes only for stable launcher regressions after direct measurements isolate the bug.",
  },
  {
    id: "actions-dialog",
    name: "Actions dialog and attached action menus",
    status: "partial",
    domains: ["targets", "elements", "layout", "input", "screenshots", "accessibility"],
    sourceFiles: ["src/actions/window.rs", "src/actions/command_bar.rs", "src/actions/types/action_model.rs"],
    features: ["route stack", "sections", "filter input", "shortcut hints", "disabled reasons", "anchor placement", "resize"],
    shortcuts: ["Cmd+K", "Escape", "Enter", "Backspace", "ArrowUp", "ArrowDown"],
    supportedNow: ["inspectAutomationWindow target kind actionsDialog", "getElements(target)", "target bounds"],
    missingRuntimePrimitives: ["anchor rect", "route stack", "section bounds", "hover row", "shortcut layout bounds"],
    regressionRecipeRole: "Smoke actions menu invariants only after devtools.measure can prove anchor and clipping failures.",
  },
  {
    id: "notes",
    name: "Notes window",
    status: "partial",
    domains: ["targets", "elements", "layout", "input", "storage", "screenshots", "accessibility", "investigation"],
    sourceFiles: [
      "src/notes/window.rs",
      "src/notes/window/keyboard.rs",
      "src/notes/window/acp_host.rs",
      "src/notes/window/window_ops.rs",
      "src/notes/window/render_ui.rs",
      "src/notes/actions_panel.rs",
      "src/notes/browse_panel.rs",
      "src/notes/storage.rs",
      "src/notes/model.rs",
    ],
    features: [
      "floating notes host",
      "editor mode",
      "browse/list mode",
      "trash mode",
      "markdown editor",
      "markdown preview",
      "editor find",
      "global search",
      "format toolbar",
      "focus mode",
      "pinning",
      "sort cycling",
      "command bar",
      "actions panel",
      "recent note switcher",
      "note cart",
      "clipboard-backed note creation",
      "embedded ACP mode",
      "ACP actions popup",
      "ACP history portal",
      "attachment/context chips",
      "draft snapshots",
      "auto-resize",
      "autosave and dirty state",
      "history back/forward",
      "scroll collapse after deleting trailing lines",
      "independent app-hide behavior",
    ],
    shortcuts: notesShortcutCoverage,
    supportedNow: [
      "stable notes automation parent",
      "inspectAutomationWindow target id notes",
      "getElements(target) for notes-owned surfaces when registered",
      "notes-window-resize-stress regression receipt",
    ],
    missingRuntimePrimitives: [
      "Notes target-scoped layout info",
      "editor and preview scroll anchors",
      "cursor and selection ranges",
      "note store generation and sandbox identity",
      "active note id and dirty state",
      "command bar route stack",
      "ACP embedded generation and origin receipts",
      "portal session provenance",
      "draft snapshot fingerprint",
      "Notes shortcut registry and focus owner transitions",
      "auto-resize before/after compare",
    ],
    regressionRecipeRole: "Keep notes recipes as regression guards for resize, ACP handoff, preview sync, and origin safety after DevTools receipts exist.",
  },
  {
    id: "notes-acp",
    name: "Notes-hosted embedded ACP",
    status: "partial",
    domains: ["targets", "elements", "input", "storage", "screenshots", "investigation"],
    sourceFiles: ["src/notes/window/acp_host.rs", "src/notes/window/keyboard.rs", "src/ai/acp/view.rs", "src/ai/acp/portal_contract.rs"],
    features: ["composer", "streaming turn", "attach menu", "history portal", "agent switch", "draft snapshot", "actions popup", "host callback routing"],
    shortcuts: ["Escape", "Cmd+K", "Cmd+W", "Tab", "Enter"],
    supportedNow: ["Notes host callbacks", "targeted popup parent notes", "fail-closed origin-generation stress spec"],
    missingRuntimePrimitives: ["getAcpState target notes", "turn generation", "composer caret", "pending context parts", "wrong-host negative proof"],
    regressionRecipeRole: "Use recipes to guard delayed actions and portal restoration only after generation receipts exist.",
  },
  {
    id: "dictation",
    name: "Dictation window and media flow",
    status: "planned",
    domains: ["targets", "elements", "media", "input", "storage", "screenshots", "accessibility", "investigation"],
    sourceFiles: [
      "src/dictation/window.rs",
      "src/dictation/runtime.rs",
      "src/dictation/types.rs",
      "src/dictation/setup.rs",
      "src/dictation/capture.rs",
      "src/dictation/device.rs",
      "src/dictation/transcription.rs",
      "src/main_entry/runtime_tray_hotkeys.rs",
    ],
    features: [
      ...dictationPhaseCoverage,
      "Script Kit target delivery",
      "ACP target delivery",
      "external app target delivery",
      "Notes editor target delivery",
      "Tab AI target delivery",
      "frontmost app paste delivery",
      "waveform/audio level bars",
      "microphone permission",
      "microphone device",
      "preferred device fallback",
      "model readiness",
      "model download/extract/failure status",
      "hotkey readiness",
      "hotkey registration",
      "hotkey conflict detection",
      "target identity",
      "transcript generation",
      "cursor insertion range",
      "wrong-target rejection",
      "cleanup without TCC/System Settings mutation",
    ],
    shortcuts: ["dictation hotkey", "Escape", "Enter", "Space", "Cmd+W", "target badge click"],
    supportedNow: ["dictation story states", "kit://dictation", "kit://dictation-history", "fail-closed dictation stress specs"],
    missingRuntimePrimitives: [
      "devtools.media.inspect",
      "passive microphone permission status",
      "microphone device snapshot",
      "model readiness generation",
      "recording state generation",
      "audio level metrics",
      "target delivery generation",
      "transcript fingerprint",
      "cursor insertion range",
      "wrong-target refusal receipt",
      "hotkey binding snapshot",
      "media cleanup receipt",
    ],
    regressionRecipeRole: "Do not use live dictation recipes as proof until passive media receipts can avoid permission prompts and target mutations.",
  },
  {
    id: "dictation-history",
    name: "Dictation History surface",
    status: "planned",
    domains: ["targets", "elements", "layout", "storage", "input", "screenshots", "accessibility"],
    sourceFiles: ["src/dictation/history.rs", "src/dictation/types.rs", "src/builtin/resources.rs"],
    features: ["transcript rows", "search/filter", "preview", "redaction", "missing audio fallback", "selection reanchor", "portal attachment"],
    shortcuts: ["Enter", "Escape", "Tab", "ArrowUp", "ArrowDown"],
    supportedNow: ["kit://dictation-history", "filterable surface architecture"],
    missingRuntimePrimitives: [
      "fixture dictation store identity",
      "transcript row generation",
      "preview generation",
      "redacted transcript fingerprint",
      "audio path redaction proof",
      "scroll and selection anchor metrics",
    ],
    regressionRecipeRole: "Use history recipes to prevent privacy and selection regressions once resource receipts are first-class.",
  },
];

function parseArgs(argv: string[]) {
  const args = {
    surface: "",
    domain: "",
    markdown: false,
  };
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === "--surface") {
      args.surface = argv[++index] ?? "";
    } else if (arg === "--domain") {
      args.domain = argv[++index] ?? "";
    } else if (arg === "--markdown") {
      args.markdown = true;
    }
  }
  return args;
}

function filteredCoverage(args: ReturnType<typeof parseArgs>) {
  const filteredDomains = args.domain ? domains.filter((domain) => domain.id === args.domain) : domains;
  const filteredSurfaces = args.surface ? surfaces.filter((surface) => surface.id === args.surface) : surfaces;
  const referencedDomainIds = new Set(filteredSurfaces.flatMap((surface) => surface.domains));
  const scopedDomains = args.domain
    ? filteredDomains
    : filteredDomains.filter((domain) => referencedDomainIds.has(domain.id) || !args.surface);

  return {
    schemaVersion: 1,
    tool: "script-kit-devtools.coverage",
    generatedAt: new Date().toISOString(),
    philosophy: "Chrome DevTools-style protocol and API coverage first; recipes are smoke/regression wrappers after direct primitives exist.",
    primitiveFamilies: ["devtools.inspect", "devtools.measure", "devtools.act", "devtools.compare", "devtools.investigate"],
    domains: scopedDomains,
    surfaces: filteredSurfaces,
    criticalGaps: [
      "target-scoped layout and scroll geometry for Notes, popups, detached ACP, prompt containers, and Dictation",
      "text-fit, contrast, overlap, and occlusion measurements tied to semantic ids",
      "passive media permission/model readiness and transcript delivery receipts for Dictation",
      "red/green investigation artifacts with stable metric names and missing-primitive classification",
      "semantic-to-AX parity and tab-order graphs for keyboard and accessibility bugs",
    ],
    recommendedNext: [
      "Build devtools.measure layout/text/scroll/contrast around stable target ids.",
      "Build devtools.act with safe protocol-first user actions and explicit native escalation.",
      "Build devtools.media.inspect before treating live Dictation bugs as verifiable.",
      "Add Notes target-scoped layout, editor selection, scroll anchors, and ACP generation receipts.",
    ],
  };
}

function markdown(report: ReturnType<typeof filteredCoverage>) {
  const lines = [
    "# Script Kit DevTools Coverage",
    "",
    report.philosophy,
    "",
    "## Domains",
    "",
    "| Domain | Chrome analogue | Current primitives | Next primitives |",
    "| --- | --- | --- | --- |",
    ...report.domains.map((domain) =>
      `| ${domain.name} | ${domain.chromeAnalogue} | ${domain.currentPrimitives.join(", ")} | ${domain.nextPrimitives.join(", ")} |`
    ),
    "",
    "## Surfaces",
    "",
    "| Surface | Status | Features | Shortcuts | Missing runtime primitives |",
    "| --- | --- | --- | --- | --- |",
    ...report.surfaces.map((surface) =>
      `| ${surface.name} | ${surface.status} | ${surface.features.join(", ")} | ${surface.shortcuts.join(", ")} | ${surface.missingRuntimePrimitives.join(", ")} |`
    ),
  ];
  return lines.join("\n");
}

const args = parseArgs(Bun.argv.slice(2));
const report = filteredCoverage(args);
if (args.markdown) {
  console.log(markdown(report));
} else {
  console.log(JSON.stringify(report, null, 2));
}
