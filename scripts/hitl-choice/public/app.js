const params = new URLSearchParams(location.search);
const token = params.get("token") || "";
const clientId = params.get("clientId") || "default";
const api = (path) => `${path}${path.includes("?") ? "&" : "?"}token=${encodeURIComponent(token)}`;

const state = {
  job: null,
  selected: new Set(),
  optionFeedback: {},
  remoteSaveTimer: null,
};

const els = {
  title: document.querySelector("#job-title"),
  description: document.querySelector("#job-description"),
  selectedCount: document.querySelector("#selected-count"),
  filter: document.querySelector("#filter"),
  options: document.querySelector("#options"),
  overall: document.querySelector("#overall-feedback"),
  submit: document.querySelector("#submit"),
  message: document.querySelector("#message"),
  saveStatus: document.querySelector("#save-status"),
  template: document.querySelector("#option-template"),
  form: document.querySelector("#review-form"),
};

function draftKey() {
  return `hitl-choice-draft:${state.job?.jobId ?? "unknown"}`;
}

function loadDraft() {
  try {
    const raw = localStorage.getItem(draftKey());
    if (!raw) return;
    const draft = JSON.parse(raw);
    state.selected = new Set(draft.selectedOptionIds ?? []);
    state.optionFeedback = draft.optionFeedback ?? {};
    els.overall.value = draft.overallFeedback ?? "";
  } catch {
    // Ignore broken local drafts.
  }
}

function applyDraft(draft) {
  if (!draft) return;
  state.selected = new Set(draft.selectedOptionIds ?? []);
  state.optionFeedback = draft.optionFeedback ?? {};
  els.overall.value = draft.overallFeedback ?? "";
}

function saveDraft() {
  if (!state.job) return;
  localStorage.setItem(
    draftKey(),
    JSON.stringify({
      selectedOptionIds: [...state.selected],
      optionFeedback: state.optionFeedback,
      overallFeedback: els.overall.value,
      savedAt: new Date().toISOString(),
    }),
  );
  scheduleRemoteDraftSave();
}

function draftBody() {
  return {
    jobId: state.job.jobId,
    selectedOptionIds: [...state.selected],
    optionFeedback: Object.fromEntries(
      Object.entries(state.optionFeedback).filter(([, value]) => value.trim().length > 0),
    ),
    overallFeedback: els.overall.value,
    client: {
      userAgent: navigator.userAgent,
      timezone: Intl.DateTimeFormat().resolvedOptions().timeZone,
      href: location.href.replace(token, "<redacted-token>"),
      clientId,
    },
  };
}

function setSaveStatus(message, isError = false) {
  els.saveStatus.textContent = message;
  els.saveStatus.classList.toggle("error", isError);
}

function scheduleRemoteDraftSave() {
  if (!state.job || !token) return;
  setSaveStatus("Saving draft...");
  clearTimeout(state.remoteSaveTimer);
  state.remoteSaveTimer = setTimeout(() => {
    void saveRemoteDraft();
  }, 350);
}

async function saveRemoteDraft() {
  if (!state.job) return;
  try {
    const res = await fetch(api(`/api/jobs/${encodeURIComponent(state.job.jobId)}/draft?clientId=${encodeURIComponent(clientId)}`), {
      method: "PUT",
      headers: {
        "content-type": "application/json",
        "x-hitl-token": token,
      },
      body: JSON.stringify(draftBody()),
    });
    const payload = await res.json();
    if (!res.ok) throw new Error(payload.error ?? `HTTP ${res.status}`);
    setSaveStatus(`Draft saved ${new Date(payload.savedAt).toLocaleTimeString()}`);
  } catch (error) {
    setSaveStatus(`Draft save failed: ${error instanceof Error ? error.message : String(error)}`, true);
  }
}

async function loadRemoteDraft() {
  if (!state.job || !token) return;
  const res = await fetch(api(`/api/jobs/${encodeURIComponent(state.job.jobId)}/draft?clientId=${encodeURIComponent(clientId)}`), {
    headers: { "x-hitl-token": token },
  });
  if (!res.ok) return;
  const payload = await res.json();
  if (payload && payload.clientId && Array.isArray(payload.selectedOptionIds)) {
    applyDraft(payload);
    localStorage.setItem(
      draftKey(),
      JSON.stringify({
        selectedOptionIds: [...state.selected],
        optionFeedback: state.optionFeedback,
        overallFeedback: els.overall.value,
        savedAt: payload.savedAt,
      }),
    );
    setSaveStatus(`Draft restored ${new Date(payload.savedAt).toLocaleTimeString()}`);
  }
}

function matchesFilter(option, query) {
  if (!query) return true;
  const haystack = [
    option.id,
    option.title,
    option.description,
    option.surface,
    ...(option.tags ?? []),
  ].join(" ").toLowerCase();
  return haystack.includes(query.toLowerCase());
}

function updateCount() {
  els.selectedCount.textContent = `${state.selected.size} selected`;
}

function syncRow(row, id) {
  const selected = state.selected.has(id);
  row.setAttribute("aria-checked", String(selected));
  row.classList.toggle("selected", selected);
  row.querySelector(".check").textContent = selected ? "[x]" : "[ ]";
}

function toggleOption(id, row) {
  if (state.selected.has(id)) state.selected.delete(id);
  else state.selected.add(id);
  if (row) syncRow(row, id);
  updateCount();
  saveDraft();
}

function render() {
  const query = els.filter.value.trim();
  els.options.replaceChildren();

  for (const option of state.job.options.filter((item) => matchesFilter(item, query))) {
    const node = els.template.content.firstElementChild.cloneNode(true);
    node.dataset.optionId = option.id;
    node.querySelector("h2").textContent = option.title;
    node.querySelector(".description").textContent = option.description;
    node.querySelector(".goal").textContent = option.goal ? `Goal: ${option.goal}` : "";
    node.querySelector(".risk").textContent = option.risk ? `Risk: ${option.risk}` : "";
    node.querySelector(".option-meta").textContent = [
      option.id,
      option.surface,
      ...(option.tags ?? []),
    ].filter(Boolean).join(" / ");
    node.querySelector(".proof").textContent = (option.suggestedProofCommands ?? []).join("\n");
    node.querySelector(".evidence").textContent = option.expectedEvidence
      ? `PASS: ${option.expectedEvidence.pass}\nFAIL: ${option.expectedEvidence.fail}`
      : "";

    const textarea = node.querySelector("textarea");
    textarea.value = state.optionFeedback[option.id] ?? "";
    textarea.setAttribute("aria-label", `Feedback for ${option.title}`);
    textarea.addEventListener("input", () => {
      state.optionFeedback[option.id] = textarea.value;
      saveDraft();
    });

    node.addEventListener("keydown", (event) => {
      if (event.target !== node) return;
      if (event.code === "Space") {
        event.preventDefault();
        toggleOption(option.id, node);
      }
    });
    node.addEventListener("click", (event) => {
      if (event.target.closest("textarea, button, a, input, summary, details")) return;
      toggleOption(option.id, node);
    });

    syncRow(node, option.id);
    els.options.appendChild(node);
  }

  updateCount();
}

async function submit() {
  els.submit.disabled = true;
  els.message.textContent = "Submitting...";
  await saveRemoteDraft();
  const body = draftBody();

  const res = await fetch(api(`/api/jobs/${encodeURIComponent(state.job.jobId)}/submissions`), {
    method: "POST",
    headers: {
      "content-type": "application/json",
      "x-hitl-token": token,
    },
    body: JSON.stringify(body),
  });
  const payload = await res.json();
  if (!res.ok) {
    els.message.textContent = `Submit failed: ${payload.error ?? res.status}`;
    els.submit.disabled = false;
    return;
  }

  localStorage.removeItem(draftKey());
  els.message.textContent = `Submitted ${payload.selectedOptionIds.length} selection(s). Submission: ${payload.submissionId}`;
}

async function init() {
  if (!token) {
    els.title.textContent = "Missing token";
    els.description.textContent = "Open the URL printed by the server, including ?token=...";
    return;
  }

  const res = await fetch(api("/api/jobs/current"), {
    headers: { "x-hitl-token": token },
  });
  if (!res.ok) throw new Error(`job load failed: ${res.status}`);

  state.job = await res.json();
  els.title.textContent = state.job.title;
  els.description.textContent = state.job.description ?? "";
  loadDraft();
  await loadRemoteDraft();
  render();

  els.filter.addEventListener("input", render);
  els.overall.addEventListener("input", saveDraft);
  els.submit.addEventListener("click", submit);
  els.form.addEventListener("submit", (event) => event.preventDefault());
}

init().catch((error) => {
  els.title.textContent = "Load failed";
  els.description.textContent = String(error);
});
