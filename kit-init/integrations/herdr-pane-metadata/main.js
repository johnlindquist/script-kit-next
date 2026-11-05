// On-demand local metadata adapter. An operator supplies one grant outside the requester's inbox.
import { div, View } from "gpui-kit";
import {
  appHome,
  machines,
  panes,
  readTextFile,
  spaces,
  terminals,
  watchFolder,
  writeTextFile,
} from "herdr";

const hex = (v) => typeof v === "string" && /^[a-f0-9]{32}$/.test(v);
const localId = (v) =>
  typeof v === "string" && /^[a-zA-Z0-9][a-zA-Z0-9_-]{0,63}$/.test(v);
const shape = (v, keys) =>
  v && typeof v === "object" && !Array.isArray(v) &&
  Object.keys(v).sort().join(",") === keys;
function parse(s, max) {
  if (typeof s !== "string" || !s.length || s.length > max) return null;
  try {
    return JSON.parse(s);
  } catch {
    return null;
  }
}
function grantOf(raw) {
  const g = parse(raw, 512);
  return shape(g, "capability,source,space,v,workspace") && g.v === 2 &&
      typeof g.workspace === "string" && g.workspace.length === 64 &&
      /^[0-9a-f]+$/.test(g.workspace) &&
      g.capability === "herdr.panes.metadata" && hex(g.source) &&
      localId(g.space)
    ? g
    : null;
}
function requestOf(raw, g) {
  const r = parse(raw, 512);
  return shape(r, "filter,limit,nonce,op,sort,source,v,workspace") &&
      r.v === 2 && r.workspace === g.workspace &&
      r.op === "list" && r.source === g.source && hex(r.nonce) &&
      Number.isInteger(r.limit) && r.limit >= 1 && r.limit <= 48 &&
      ["all", "needsAttention"].includes(r.filter) &&
      ["title", "attentionFirst"].includes(r.sort)
    ? r
    : null;
}
const urgency = { blocked: 0, done: 1, working: 2, idle: 3 };
function project(g, r) {
  const unavailable = { state: "unavailable" };
  const hosts = machines().filter((m) => m.name === null);
  if (hosts.length !== 1 || hosts[0].layout !== "ready") return unavailable;
  const allSpaces = spaces(),
    chosen = allSpaces.filter((s) => s.id === g.space);
  if (chosen.length !== 1 || chosen[0].machine !== null) return unavailable;
  const tabs = new Set(chosen[0].tabs.map((t) => t.id));
  if (
    tabs.size !== chosen[0].tabs.length || [...tabs].some((t) => !localId(t)) ||
    allSpaces.some((s) => s !== chosen[0] && s.tabs.some((t) => tabs.has(t.id)))
  ) return unavailable;
  const ps = panes(), ts = terminals();
  if (ps.length > 2048 || ts.length > 2048) return unavailable;
  const byAddress = new Map();
  for (const t of ts) {
    byAddress.set(t.address, byAddress.has(t.address) ? null : t);
  }
  const seen = new Set(), rows = [];
  for (const p of ps) {
    if (!tabs.has(p.tab) || p.terminal === null) continue;
    if (
      !localId(p.pane) || !/^t[1-9][0-9]{0,11}$/.test(p.terminal) ||
      seen.has(p.pane) || typeof p.focused !== "boolean" ||
      typeof p.zoomed !== "boolean"
    ) return unavailable;
    seen.add(p.pane);
    const t = byAddress.get(p.terminal);
    if (!t) return unavailable;
    if (t.state === "finished") continue;
    if (
      t.state !== "live" || !Number.isSafeInteger(t.pid) || t.pid <= 0 ||
      !Number.isSafeInteger(t.created_ms) || t.created_ms <= 0
    ) return unavailable;
    if (p.status !== null && !Object.hasOwn(urgency, p.status)) {
      return unavailable;
    }
    // 'needsAttention' means blocked or finished work not yet acknowledged by the person.
    if (
      r.filter === "needsAttention" && p.status !== "blocked" &&
      p.status !== "done"
    ) continue;
    // Never use a terminal-controlled title, command line, folder, branch, report, or screen text as a label.
    rows.push({
      pane: p.pane,
      tab: p.tab,
      terminal: p.terminal,
      pid: t.pid,
      createdMs: t.created_ms,
      title: "Pane " + p.terminal.slice(1),
      status: p.status,
      focused: p.focused,
      zoomed: p.zoomed,
    });
  }
  rows.sort((a, b) =>
    (r.sort === "attentionFirst"
      ? (urgency[a.status] ?? 4) - (urgency[b.status] ?? 4)
      : 0) ||
    Number(a.terminal.slice(1)) - Number(b.terminal.slice(1)) ||
    (a.pane < b.pane ? -1 : a.pane > b.pane ? 1 : 0)
  );
  return {
    state: "ready",
    more: rows.length > r.limit,
    rows: rows.slice(0, r.limit),
  };
}

export default class PaneSource extends View {
  init(_props, cx) {
    const root = appHome() + "/kit-pane-share-v1/",
      inbox = root + "in/",
      request = inbox + "request.json",
      grant = root + "grant.json";
    cx.spawn(async () => {
      const authority = await readTextFile(grant), g = grantOf(authority);
      if (!g) return;
      const prefix = root + "out/" + g.source + "-";
      const envelope = (data) =>
        JSON.stringify({
          v: 2,
          source: g.source,
          space: g.space,
          workspace: g.workspace,
          ...data,
        });
      let previous = null, observed = await readTextFile(request);
      // A fresh enrollment belongs to exactly one mounted copy. A marker alone never proves liveness.
      await writeTextFile(
        prefix + "listening.json",
        envelope({ state: "listening", capability: "herdr.panes.metadata" }),
        null,
      );
      for (;;) {
        const change = watchFolder(inbox);
        const raw = await readTextFile(request);
        if (raw !== observed) {
          observed = raw;
          const header = parse(raw, 512);
          if (
            header && header.v === 2 && header.workspace === g.workspace &&
            header.source === g.source &&
            hex(header.nonce)
          ) {
            if (await readTextFile(grant) !== authority) return;
            // At most one unread reply per source. If its reader disappeared, stop rather than fill the disk.
            if (previous === null || await readTextFile(previous) === null) {
              const r = requestOf(raw, g);
              const next = envelope({
                nonce: header.nonce,
                ...(r ? project(g, r) : { state: "unsupportedRequest" }),
              });
              if (await readTextFile(grant) !== authority) return;
              if (await readTextFile(request) === raw) {
                const path = prefix + header.nonce + ".json";
                // Always create exclusively. Even a symlink to a copy of a previous reply cannot redirect a write.
                try {
                  await writeTextFile(path, next, null);
                  previous = path;
                } catch { /* occupied or unavailable: no reply */ }
              }
            }
          }
        }
        await change;
      }
    });
  }
  render() {
    return div().child("Pane metadata · read only");
  }
}
