import { createActor, type AnyActorRef } from "xstate";
import "./styles.css";
import {
  featureExplorerMachine,
  filteredFeatures,
  interactionLabel,
  atlasCoverage,
  selectedFeature,
  type ExplorerMode,
  type Feature,
  type FeatureExplorerContext
} from "./state/featureMachine";
import { createFeatureRuntimeMachine, runtimeModelForFeature } from "./state/featureRuntime";
import { rootUnifiedLauncherMachine, type RootLauncherEvent, type RootLauncherSnapshot } from "./wireframes/rootUnifiedLauncher";
import { wireframeForFeature, wireframeRegistrations } from "./wireframes/registry";

const appElement = document.querySelector<HTMLDivElement>("#app");
if (!appElement) throw new Error("Missing #app");
const app = appElement;

const actor = createActor(featureExplorerMachine);
let runtimeActor: AnyActorRef | null = null;
let runtimeFeatureId = "";
let rootUnifiedLauncherActor: AnyActorRef | null = null;

actor.subscribe((snapshot) => render(snapshot.context));
actor.start();

function sendMode(mode: ExplorerMode) {
  actor.send({ type: "SET_MODE", mode });
}

function render(context: FeatureExplorerContext) {
  const feature = selectedFeature(context);
  syncRuntimeActor(feature);
  syncRootUnifiedLauncherActor();
  const runtimeSnapshot = runtimeActor?.getSnapshot();
  const runtimeModel = runtimeModelForFeature(feature);
  const runtimeStateId = String(runtimeSnapshot?.value ?? runtimeModel.initial);
  const runtimeState = runtimeModel.states.find((state) => state.id === runtimeStateId) ?? runtimeModel.states[0];
  const features = filteredFeatures(context);
  const activeState = feature.stateRows.find((row) => row.State === context.selectedState);
  const activeWorkflow = feature.workflows.find((workflow) => workflow.title === context.selectedWorkflow);
  const activeInteraction = feature.interactions.find((row) => interactionLabel(row) === context.selectedInteraction);

  app.innerHTML = `
    <main class="shell">
      <aside class="sidebar">
        <div class="brand">
          <strong>Feature Explorer</strong>
          <span>${context.features.length}/${atlasCoverage.rawOracleFeatureCount} raw Oracle slices mapped</span>
        </div>
        <input class="search" value="${escapeHtml(context.filter)}" placeholder="Filter features" data-action="filter" />
        <div class="feature-list">
          ${features.map((item) => featureButton(item, item.id === feature.id)).join("")}
        </div>
      </aside>
      <section class="workspace">
        <header class="topbar">
          <div>
            <div class="eyebrow">${escapeHtml(feature.file)}</div>
            <h1>${escapeHtml(feature.title)}</h1>
            <p>${escapeHtml(feature.summary)}</p>
          </div>
          <div class="nav-buttons">
            <button data-action="prev">Prev</button>
            <button data-action="next">Next</button>
          </div>
        </header>
        <nav class="tabs">
          ${tab("overview", context.mode, "Overview")}
          ${tab("machine", context.mode, "Machine")}
          ${tab("wireframe", context.mode, "Wireframe")}
          ${tab("states", context.mode, "States")}
          ${tab("workflows", context.mode, "Workflows")}
          ${tab("interactions", context.mode, "Interactions")}
          ${tab("keystrokes", context.mode, "Keys")}
          ${tab("risks", context.mode, "Risks")}
        </nav>
        ${panelForMode(context.mode, feature, context.features, activeState, activeWorkflow, activeInteraction, runtimeModel, runtimeState)}
      </section>
    </main>
  `;

  bindHandlers();
}

function featureButton(feature: Feature, selected: boolean) {
  return `
    <button class="feature-button ${selected ? "selected" : ""}" data-action="feature" data-id="${feature.id}">
      <span>${feature.id}</span>
      <strong>${escapeHtml(feature.title.replace(/^\d+\s*/, ""))}</strong>
    </button>
  `;
}

function tab(mode: ExplorerMode, active: ExplorerMode, label: string) {
  return `<button class="${mode === active ? "active" : ""}" data-action="mode" data-mode="${mode}">${label}</button>`;
}

function panelForMode(
  mode: ExplorerMode,
  feature: Feature,
  allFeatures: Feature[],
  activeState: Record<string, string> | undefined,
  activeWorkflow: { title: string; body: string } | undefined,
  activeInteraction: Record<string, string> | undefined,
  runtimeModel: ReturnType<typeof runtimeModelForFeature>,
  runtimeState: ReturnType<typeof runtimeModelForFeature>["states"][number]
) {
  if (mode === "machine") {
    const currentTransitions = runtimeModel.transitions.filter((transition) => transition.from === runtimeState.id);
    return `
      <div class="split wide">
        <div class="rail">
          <div class="runtime-state">
            <span>Current XState node</span>
            <strong>${escapeHtml(runtimeState.label)}</strong>
          </div>
          <div class="rail-heading">State transitions</div>
          ${
            currentTransitions.length
              ? currentTransitions.map((transition) => `<button data-action="runtime-event" data-id="${escapeAttr(transition.id)}">${escapeHtml(transition.label)}<small>${escapeHtml(transition.detail.guard || "state exit")}</small></button>`).join("")
              : `<p class="empty-note">No explicit exits from this state. Scenario events use inferred or fallback targets.</p>`
          }
          <div class="rail-heading">Scenario events</div>
          ${runtimeModel.events.map((event) => `<button data-action="runtime-event" data-id="${escapeAttr(event.id)}">${escapeHtml(event.label)}<small>${event.source}${event.target ? ` -> ${stateLabel(runtimeModel, event.target)}` : " -> fallback"}</small></button>`).join("")}
        </div>
        <article class="detail">
          <h2>${escapeHtml(runtimeState.label)}</h2>
          <div class="metric-grid">
            <span><strong>${runtimeModel.coverage.stateCount}</strong> states</span>
            <span><strong>${runtimeModel.coverage.explicitTransitionCount}</strong> state exits</span>
            <span><strong>${runtimeModel.coverage.inferredEventTargetCount}</strong> inferred events</span>
            <span><strong>${runtimeModel.coverage.fallbackEventCount}</strong> fallback events</span>
          </div>
          ${
            runtimeModel.authored
              ? `<div class="authored-note">Authored machine: ${escapeHtml(runtimeModel.authored.meta.slug)} · receipts ${escapeHtml(runtimeModel.authored.meta.receipts.join(", "))}</div>`
              : `<div class="authored-note fallback">Derived fallback machine from chapter tables.</div>`
          }
          ${keyValueGrid(runtimeState.row)}
          <h2>Current State Exits</h2>
          ${table(currentTransitions.map((transition) => ({
            Event: transition.label,
            Target: stateLabel(runtimeModel, transition.target),
            Guard: transition.detail.guard,
            Source: transition.detail.rawExit
          })))}
          <h2>Derived Machine Definition</h2>
          <pre>${escapeHtml(JSON.stringify({
            initial: runtimeModel.initial,
            states: runtimeModel.states.map((state) => ({
              id: state.id,
              exits: runtimeModel.transitions
                .filter((transition) => transition.from === state.id)
                .map((transition) => ({ event: transition.id, target: transition.target }))
            })),
            events: runtimeModel.events.map((event) => ({ id: event.id, source: event.source, target: event.target ?? "fallback-next-state" })),
            coverage: runtimeModel.coverage
          }, null, 2))}</pre>
        </article>
      </div>
    `;
  }
  if (mode === "wireframe") {
    const registration = wireframeForFeature(feature.id);
    if (!registration) return genericFeatureWireframe(feature, activeState, activeWorkflow, activeInteraction);
    return registration.render({
      activeFeature: feature,
      features: allFeatures,
      snapshot: rootUnifiedLauncherActor?.getSnapshot() as RootLauncherSnapshot | undefined
    });
  }
  if (mode === "states") {
    return `
      <div class="split">
        <div class="rail">${feature.stateRows.map((row) => stateButton(row, row.State === activeState?.State)).join("")}</div>
        <article class="detail">
          <h2>${escapeHtml(activeState?.State ?? "No state rows")}</h2>
          ${keyValueGrid(activeState)}
        </article>
      </div>
    `;
  }
  if (mode === "workflows") {
    return `
      <div class="split">
        <div class="rail">${feature.workflows.map((workflow) => workflowButton(workflow, workflow.title === activeWorkflow?.title)).join("")}</div>
        <article class="detail">
          <h2>${escapeHtml(activeWorkflow?.title ?? "No workflows")}</h2>
          <p>${escapeHtml(activeWorkflow?.body ?? "")}</p>
        </article>
      </div>
    `;
  }
  if (mode === "interactions") {
    return `
      <div class="split wide">
        <div class="rail">${feature.interactions.map((row) => interactionButton(row, interactionLabel(row) === interactionLabel(activeInteraction))).join("")}</div>
        <article class="detail">
          <h2>${escapeHtml(interactionLabel(activeInteraction) || "No interactions")}</h2>
          ${keyValueGrid(activeInteraction)}
        </article>
      </div>
    `;
  }
  if (mode === "keystrokes") {
    return `<article class="detail">${table(feature.keystrokes)}</article>`;
  }
  if (mode === "risks") {
    return `
      <div class="cards">
        ${listCard("Invariants And Regression Risks", feature.risks)}
        ${listCard("Open Questions And Gaps", feature.gaps)}
        ${listCard("Visual And Focus States", feature.visualStates)}
      </div>
    `;
  }
  return `
    <div class="overview-grid">
      <article class="detail">
        <h2>Atlas Coverage</h2>
        <dl>
          <dt>Indexed features</dt><dd>${atlasCoverage.indexedFeatureCount}</dd>
          <dt>Raw Oracle slices</dt><dd>${atlasCoverage.rawOracleFeatureCount}</dd>
          <dt>Mapped chapters</dt><dd>${atlasCoverage.chapterFeatureCount}</dd>
          <dt>Pending raw slices</dt><dd>${atlasCoverage.pendingRawOracleRows.map((row) => row.slug).join(", ") || "None"}</dd>
        </dl>
      </article>
      ${listCard("What Users Can Do", feature.capabilities)}
      <article class="detail">
        <h2>Entry Points</h2>
        ${table(feature.entryPoints)}
      </article>
      <article class="detail">
        <h2>Core Concepts</h2>
        ${table(feature.concepts)}
      </article>
    </div>
  `;
}

function stateButton(row: Record<string, string>, selected: boolean) {
  return `<button class="${selected ? "selected" : ""}" data-action="state" data-id="${escapeAttr(row.State)}">${escapeHtml(row.State)}</button>`;
}

function workflowButton(workflow: { title: string }, selected: boolean) {
  return `<button class="${selected ? "selected" : ""}" data-action="workflow" data-id="${escapeAttr(workflow.title)}">${escapeHtml(workflow.title)}</button>`;
}

function interactionButton(row: Record<string, string>, selected: boolean) {
  const label = interactionLabel(row);
  return `<button class="${selected ? "selected" : ""}" data-action="interaction" data-id="${escapeAttr(label)}">${escapeHtml(label)}</button>`;
}

function listCard(title: string, items: string[]) {
  return `
    <article class="detail">
      <h2>${escapeHtml(title)}</h2>
      <ul>${items.map((item) => `<li>${escapeHtml(item)}</li>`).join("")}</ul>
    </article>
  `;
}

function genericFeatureWireframe(
  feature: Feature,
  activeState: Record<string, string> | undefined,
  activeWorkflow: { title: string; body: string } | undefined,
  activeInteraction: Record<string, string> | undefined
) {
  const layout = layoutForFeature(feature);
  const primaryWorkflow = activeWorkflow ?? feature.workflows[0];
  const primaryInteraction = activeInteraction ?? feature.interactions[0];
  const primaryState = activeState ?? feature.stateRows[0];
  return `
    <div class="generic-wireframe">
      <section class="mock-app-window ${layout.kind}">
        <header class="mock-titlebar">
          <span>Script Kit</span>
          <strong>${escapeHtml(layout.title)}</strong>
          <span>${escapeHtml(feature.id)}</span>
        </header>
        <div class="mock-body">
          <aside class="mock-left-rail">
            <div class="mock-search">${escapeHtml(layout.inputLabel)}</div>
            ${feature.entryPoints.slice(0, 5).map((entry, index) => {
              const label = entry["Entry point"] ?? entry.Entry ?? entry["User input"] ?? Object.values(entry)[0] ?? `Entry ${index + 1}`;
              return `<button data-action="workflow" data-id="${escapeAttr(feature.workflows[index]?.title ?? feature.workflows[0]?.title ?? "")}">${escapeHtml(label)}</button>`;
            }).join("") || `<button>${escapeHtml(feature.title.replace(/^\d+\s*/, ""))}</button>`}
          </aside>
          <main class="mock-main-surface">
            <div class="mock-surface-heading">
              <h2>${escapeHtml(primaryWorkflow?.title ?? feature.title.replace(/^\d+\s*/, ""))}</h2>
              <span>${escapeHtml(layout.badge)}</span>
            </div>
            <p>${escapeHtml(primaryWorkflow?.body ?? feature.summary)}</p>
            <div class="mock-flow-grid">
              ${feature.interactions.slice(0, 6).map((interaction) => {
                const label = interactionLabel(interaction);
                const selected = label === interactionLabel(primaryInteraction);
                return `
                  <button class="${selected ? "selected" : ""}" data-action="interaction" data-id="${escapeAttr(label)}">
                    <strong>${escapeHtml(label || "Interaction")}</strong>
                    <small>${escapeHtml(interaction.Result ?? interaction["Expected behavior"] ?? interaction["UI state"] ?? "Mock behavior")}</small>
                  </button>
                `;
              }).join("") || `<div class="mock-empty">No interaction rows found.</div>`}
            </div>
          </main>
          <aside class="mock-inspector">
            <h2>Mock receipt</h2>
            <pre>${escapeHtml(JSON.stringify({
              featureId: feature.id,
              layout: layout.kind,
              activeState: primaryState?.State ?? null,
              workflow: primaryWorkflow?.title ?? null,
              interaction: interactionLabel(primaryInteraction) || null,
              proofHint: primaryInteraction?.Proof ?? primaryState?.Proof ?? "feature-map derived mock"
            }, null, 2))}</pre>
          </aside>
        </div>
        <footer class="mock-footer">
          <span>${escapeHtml(layout.footer)}</span>
          <span>${feature.capabilities.length} capabilities</span>
          <span>${feature.risks.length} risks</span>
        </footer>
      </section>
    </div>
  `;
}

function layoutForFeature(feature: Feature) {
  const text = [feature.title, feature.summary, feature.file].join(" ").toLowerCase();
  if (text.includes("agent") || text.includes("acp") || text.includes("chat")) {
    return { kind: "chat", title: "Agent Chat Surface", inputLabel: "Ask or stage context", badge: "Composer", footer: "Context parts / send / cancel" };
  }
  if (text.includes("terminal") || text.includes("term")) {
    return { kind: "terminal", title: "Terminal Surface", inputLabel: "Shell command", badge: "PTY", footer: "Input owned by terminal" };
  }
  if (text.includes("prompt") || text.includes("form") || text.includes("editor") || text.includes("template")) {
    return { kind: "prompt", title: "Prompt Surface", inputLabel: "Prompt input", badge: "SDK Prompt", footer: "Submit / actions / escape" };
  }
  if (text.includes("notes")) {
    return { kind: "notes", title: "Notes Surface", inputLabel: "Search notes", badge: "Notes", footer: "Editor / browse / ACP" };
  }
  if (text.includes("settings") || text.includes("theme") || text.includes("permission")) {
    return { kind: "settings", title: "Settings Surface", inputLabel: "Filter settings", badge: "Preferences", footer: "Config / status / actions" };
  }
  return { kind: "launcher", title: "Launcher Surface", inputLabel: "Type to search", badge: "ScriptList", footer: "Enter / Cmd+K / Escape" };
}

function keyValueGrid(row: Record<string, string> | undefined) {
  if (!row) return "<p>No structured rows found in this chapter.</p>";
  return `<dl>${Object.entries(row)
    .map(([key, value]) => `<dt>${escapeHtml(key)}</dt><dd>${escapeHtml(value)}</dd>`)
    .join("")}</dl>`;
}

function table(rows: Record<string, string>[]) {
  if (!rows.length) return "<p>No table rows found in this chapter.</p>";
  const headers = Object.keys(rows[0]);
  return `
    <div class="table-wrap">
      <table>
        <thead><tr>${headers.map((header) => `<th>${escapeHtml(header)}</th>`).join("")}</tr></thead>
        <tbody>
          ${rows
            .map((row) => `<tr>${headers.map((header) => `<td>${escapeHtml(row[header] ?? "")}</td>`).join("")}</tr>`)
            .join("")}
        </tbody>
      </table>
    </div>
  `;
}

function stateLabel(runtimeModel: ReturnType<typeof runtimeModelForFeature>, stateId: string) {
  return runtimeModel.states.find((state) => state.id === stateId)?.label ?? stateId;
}

function bindHandlers() {
  app.querySelector<HTMLInputElement>("[data-action='filter']")?.addEventListener("input", (event) => {
    actor.send({ type: "FILTER", value: (event.target as HTMLInputElement).value });
  });
  app.querySelectorAll<HTMLElement>("[data-action='feature']").forEach((button) => {
    button.addEventListener("click", () => actor.send({ type: "SELECT_FEATURE", id: button.dataset.id ?? "" }));
  });
  app.querySelectorAll<HTMLElement>("[data-action='mode']").forEach((button) => {
    button.addEventListener("click", () => sendMode((button.dataset.mode ?? "overview") as ExplorerMode));
  });
  app.querySelectorAll<HTMLElement>("[data-action='state']").forEach((button) => {
    button.addEventListener("click", () => actor.send({ type: "SELECT_STATE", id: button.dataset.id ?? "" }));
  });
  app.querySelectorAll<HTMLElement>("[data-action='workflow']").forEach((button) => {
    button.addEventListener("click", () => actor.send({ type: "SELECT_WORKFLOW", id: button.dataset.id ?? "" }));
  });
  app.querySelectorAll<HTMLElement>("[data-action='interaction']").forEach((button) => {
    button.addEventListener("click", () => actor.send({ type: "SELECT_INTERACTION", id: button.dataset.id ?? "" }));
  });
  app.querySelector<HTMLElement>("[data-action='prev']")?.addEventListener("click", () => actor.send({ type: "PREV_FEATURE" }));
  app.querySelector<HTMLElement>("[data-action='next']")?.addEventListener("click", () => actor.send({ type: "NEXT_FEATURE" }));
  app.querySelectorAll<HTMLElement>("[data-action='runtime-event']").forEach((button) => {
    button.addEventListener("click", () => {
      const type = button.dataset.id;
      if (type) runtimeActor?.send({ type });
      render(actor.getSnapshot().context);
    });
  });
  const feature = selectedFeature(actor.getSnapshot().context);
  const registration = wireframeForFeature(feature.id);
  registration?.bind(
    app,
    (event) => rootUnifiedLauncherActor?.send(event as RootLauncherEvent),
    (id) => actor.send({ type: "SELECT_FEATURE", id })
  );
}

function syncRuntimeActor(feature: Feature) {
  if (runtimeActor && runtimeFeatureId === feature.id) return;
  runtimeActor?.stop();
  runtimeActor = createActor(createFeatureRuntimeMachine(feature));
  runtimeFeatureId = feature.id;
  runtimeActor.start();
}

function syncRootUnifiedLauncherActor() {
  if (rootUnifiedLauncherActor) return;
  rootUnifiedLauncherActor = createActor(rootUnifiedLauncherMachine);
  rootUnifiedLauncherActor.subscribe(() => render(actor.getSnapshot().context));
  rootUnifiedLauncherActor.start();
}

function escapeHtml(value: string) {
  return value.replace(/[&<>"']/g, (char) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[char] ?? char);
}

function escapeAttr(value: string) {
  return escapeHtml(value);
}
