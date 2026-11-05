// Operator-only pairing. A personal view can ask to read an existing pairing; its declaration
// cannot supply a path, enroll a workspace, grant control, or discover an instance of Herdr.
import fs from "node:fs";
import path from "node:path";
import { createHash, randomBytes } from "node:crypto";
import {
  directName,
  directory,
  equal,
  failure,
  guarded,
  identity,
  inspectFile,
  isWorkspaceId,
  keys,
  listPaired,
  localId,
  locations,
  need,
  pairedFields,
  queryMetadata,
  Refusal,
  safeMode,
  snapshot,
  stored,
  token,
  workspace,
} from "./herdr-pane-read.mjs";

const SHARING = "kit-pane-share-v1", PRIVATE = "herdr-panes-v1";
const { O_WRONLY, O_CREAT, O_EXCL, O_NOFOLLOW } = fs.constants;
const mint = () => randomBytes(16).toString("hex");
function freshFile(p, bytes) {
  const fd = fs.openSync(p, O_WRONLY | O_NOFOLLOW | O_CREAT | O_EXCL, 0o600);
  try {
    fs.writeFileSync(fd, bytes);
    fs.fsyncSync(fd);
    return identity(fs.fstatSync(fd, { bigint: true }));
  } finally {
    fs.closeSync(fd);
  }
}
function retire(p, i) {
  const st = fs.lstatSync(p, { bigint: true });
  need(
    st.isFile() && st.nlink === 1n && equal(identity(st), i),
    "pairingChanged",
  );
  fs.unlinkSync(p);
}
const capability = "herdr.panes.metadata";
const allowed = JSON.stringify(capability);
// Keep this v2 document valid JSON, with the same file identity and size. A mounted
// provider compares the full original bytes; a fresh provider requires the capability.
const withheld = "null".padEnd(allowed.length);
function offerId(home, exchange, file, source, space, ws) {
  return createHash("sha256").update(
    JSON.stringify([
      "kit-pane-offer-v1",
      home,
      exchange,
      file,
      source,
      space,
      ws,
    ]),
  ).digest("hex");
}
function observeOffer(home, kit, ws) {
  const h = directory(home), root = path.join(home, SHARING);
  const sameHome = () => {
    need(equal(h, directory(home)), "offerChanged");
    need(workspace(kit) === ws, "workspaceChanged");
  };
  if (!fs.lstatSync(root, { throwIfNoEntry: false })) {
    sameHome();
    return { info: { state: "noOffer", workspace: ws } };
  }
  const d = directory(root, true), file = path.join(root, "grant.json");
  let a;
  try {
    a = inspectFile(file, 512);
  } catch (e) {
    if (e.code === "ENOENT") throw new Refusal("incompleteOffer");
    throw e;
  }
  let g;
  try {
    g = JSON.parse(a.text);
  } catch {
    throw new Refusal("unverifiedOffer");
  }
  if (g?.v === 1 && keys(g, "capability,source,space,v")) {
    throw new Refusal("renewalRequired");
  }
  need(
    keys(g, "capability,source,space,v,workspace") && g.v === 2 &&
      token(g.source) && localId(g.space) && isWorkspaceId(g.workspace),
    "unverifiedOffer",
  );
  need(g.workspace === ws, "offerWorkspaceMismatch");
  const live = JSON.stringify({
    v: 2,
    capability,
    source: g.source,
    space: g.space,
    workspace: ws,
  });
  const retired = live.replace(allowed, withheld);
  need(a.text === live || a.text === retired, "unverifiedOffer"); // never patch hand-edited or ambiguous bytes
  sameHome();
  need(
    equal(d, directory(root, true)) &&
      equal(a.at, identity(fs.lstatSync(file, { bigint: true }))),
    "offerChanged",
  );
  const fields = {
    workspace: ws,
    source: g.source,
    space: g.space,
    offerId: offerId(h, d, a.at, g.source, g.space, ws),
  };
  const info = a.text === live
    ? {
      state: "offered",
      ...fields,
      capability,
      marker: path.join(root, "out", g.source + "-listening.json"),
    }
    : { state: "withdrawn", ...fields };
  return { info, file, at: a.at, text: a.text, offset: live.indexOf(allowed) };
}
function withdrawOffer(home, expected, kit, ws) {
  need(
    typeof expected === "string" && /^[0-9a-f]{64}$/.test(expected),
    "invalidRequest",
  );
  const before = observeOffer(home, kit, ws);
  need(before.info.offerId === expected, "offerChanged");
  if (before.info.state === "withdrawn") return before.info;
  const fd = fs.openSync(
    before.file,
    fs.constants.O_RDWR | fs.constants.O_NONBLOCK | O_NOFOLLOW,
  );
  try {
    const held = fs.fstatSync(fd, { bigint: true });
    need(
      held.isFile() && held.nlink === 1n &&
        held.uid === BigInt(process.getuid()) &&
        (held.mode & 0o777n) === 0o600n && equal(identity(held), before.at),
      "offerChanged",
    );
    const current = observeOffer(home, kit, ws);
    need(
      current.info.offerId === expected && current.text === before.text,
      "offerChanged",
    );
    // Never unlink or replace a pathname. Even a rename after the last check cannot
    // send this write to a new grant. Report a displaced/uncertain result honestly.
    try {
      const b = Buffer.from(withheld);
      need(fs.writeSync(fd, b, 0, b.length, before.offset) === b.length);
      fs.fsyncSync(fd);
      const after = observeOffer(home, kit, ws).info;
      need(after.state === "withdrawn" && after.offerId === expected);
      return after;
    } catch {
      throw new Refusal("withdrawalUnverified");
    }
  } finally {
    fs.closeSync(fd);
  }
}
function providerOffer(home, space, kit, ws) {
  need(localId(space), "invalidRequest");
  const h = directory(home), root = path.join(home, SHARING), source = mint();
  try {
    fs.mkdirSync(root, { mode: 0o700 });
  } catch (e) {
    if (e.code === "EEXIST") throw new Refusal("offerExists");
    throw e;
  }
  // Create the grant last. A partial/abandoned offer is not repaired by replacing its files.
  const d = directory(root, true);
  for (const name of ["in", "out"]) {
    fs.mkdirSync(path.join(root, name), { mode: 0o700 });
  }
  freshFile(path.join(root, "in/request.json"), "{}");
  need(equal(h, directory(home)) && equal(d, directory(root, true)));
  need(workspace(kit) === ws, "workspaceChanged");
  const at = freshFile(
    path.join(root, "grant.json"),
    JSON.stringify({ v: 2, capability, source, space, workspace: ws }),
  );
  const made = observeOffer(home, kit, ws).info;
  need(
    made.state === "offered" &&
      made.offerId === offerId(h, d, at, source, space, ws),
    "offerChanged",
  );
  return made;
}
function enrollment(marker, space, ws) {
  need(directName(marker) && localId(space), "invalidRequest");
  const suffix = "-listening.json",
    name = path.basename(marker),
    source = name.slice(0, -suffix.length),
    out = path.dirname(marker),
    root = path.dirname(out);
  need(
    token(source) && name === source + suffix && path.basename(out) === "out" &&
      path.basename(root) === SHARING,
    "invalidRequest",
  );
  const dir = directory(root, true),
    inbox = path.join(root, "in"),
    i = directory(inbox, true),
    o = directory(out, true);
  const m = inspectFile(marker, 512, false), doc = JSON.parse(m.text);
  need(
    doc?.capability === "herdr.panes.metadata" && doc.state === "listening" &&
      doc.source === source && doc.space === space,
  );
  if (doc.v === 1 && keys(doc, "capability,source,space,state,v")) {
    throw new Refusal("renewalRequired");
  }
  need(
    keys(doc, "capability,source,space,state,v,workspace") && doc.v === 2 &&
      isWorkspaceId(doc.workspace),
  );
  // The marker names the original recipient before opening the request for writing.
  need(doc.workspace === ws, "offerWorkspaceMismatch");
  return {
    root,
    source,
    space,
    dir,
    in: i,
    out: o,
    marker: m.at,
    request: inspectFile(path.join(inbox, "request.json"), 512).at,
  };
}
function mutate(root, ws, fn) {
  need(workspace(root) === ws, "workspaceChanged");
  const db = path.join(root, "db");
  directory(db);
  try {
    fs.mkdirSync(path.join(db, PRIVATE), { mode: 0o700 });
  } catch (e) {
    if (e.code !== "EEXIST") throw e;
  }
  const l = locations(root);
  need(l);
  let lock;
  try {
    lock = freshFile(l.lock, mint());
  } catch (e) {
    if (e.code === "EEXIST") throw new Refusal("pairingBusy");
    throw e;
  }
  const check = () => {
    need(workspace(root) === ws, "workspaceChanged");
    need(
      equal(l.at, directory(l.dir, true)) &&
        equal(lock, inspectFile(l.lock, 64).at),
      "pairingChanged",
    );
  };
  let fd;
  try {
    fd = fs.openSync(
      l.dir,
      fs.constants.O_RDONLY | fs.constants.O_DIRECTORY | O_NOFOLLOW,
    );
    check();
    need(
      equal(l.at, identity(fs.fstatSync(fd, { bigint: true }))),
      "pairingChanged",
    );
    const result = fn(l, check);
    fs.fsyncSync(fd);
    return result;
  } finally {
    try {
      check();
      retire(l.lock, lock);
      if (fd !== undefined) fs.fsyncSync(fd);
    } finally {
      if (fd !== undefined) fs.closeSync(fd);
    }
  }
}
export function paneCommand(args, root) {
  try {
    if (process.platform !== "darwin") return { state: "unsupportedPlatform" };
    if (guarded()) return { state: "agentContextDenied" };
    const [cmd, ...v] = args;
    if (safeMode() && !["unpair", "withdraw", "offer-status"].includes(cmd)) {
      return { state: "safeMode" };
    }
    const ws = workspace(root);
    if (cmd === "offer-status" || cmd === "withdraw") {
      need(
        v.length === (cmd === "offer-status" ? 1 : 2) && directName(v[0]),
        "invalidRequest",
      );
      return cmd === "offer-status"
        ? observeOffer(v[0], root, ws).info
        : withdrawOffer(v[0], v[1], root, ws);
    }
    if (cmd === "offer" && v.length === 2) {
      return providerOffer(v[0], v[1], root, ws);
    }
    const s = snapshot(root, ws);
    if (cmd === "status" && !v.length) {
      return s
        ? {
          state: s.record.v === 1 ? "renewalRequired" : "paired",
          ...pairedFields(s.record),
        }
        : { state: "notPaired", workspace: ws };
    }
    if (cmd === "list" && (v.length === 0 || v.length === 3)) {
      const [filter = "all", sort = "title", limit = "48"] = v;
      need(/^[1-9][0-9]?$/.test(limit), "invalidRequest");
      return listPaired(root, ws, { filter, sort, limit: Number(limit) });
    }
    if (cmd === "pair" && v.length === 2) {
      need(!s, s?.record.v === 1 ? "renewalRequired" : "alreadyPaired");
      const b = enrollment(v[0], v[1], ws);
      const live = queryMetadata(
        b,
        { filter: "all", sort: "title", limit: 1 },
        ws,
      );
      if (live.state !== "ready") return { state: live.state };
      return mutate(root, ws, (l, check) => {
        need(!stored(l, ws), "alreadyPaired");
        const confirmed = enrollment(v[0], v[1], ws);
        need(JSON.stringify(confirmed) === JSON.stringify(b));
        const record = { v: 2, workspace: ws, id: mint(), binding: b };
        check();
        const at = freshFile(l.file, JSON.stringify(record));
        const actual = stored(l, ws);
        need(
          actual && equal(at, actual.at) &&
            actual.text === JSON.stringify(record),
          "unverifiedPairing",
        );
        return { state: "paired", ...pairedFields(record) };
      });
    }
    if (cmd === "unpair" && v.length === 1 && token(v[0])) {
      if (!s) return { state: "notPaired", workspace: ws };
      need(s.record.id === v[0], "pairingChanged");
      return mutate(root, ws, (l, check) => {
        check();
        const live = stored(l, ws);
        need(
          live && live.text === s.text && equal(live.at, s.at),
          "pairingChanged",
        );
        retire(l.file, live.at);
        return { state: "unpaired", workspace: ws, pairId: v[0] };
      });
    }
    return { state: "invalidRequest" };
  } catch (e) {
    return failure(e);
  }
}
