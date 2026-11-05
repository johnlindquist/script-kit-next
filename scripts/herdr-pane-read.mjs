// Only static Kit code runs in the metadata reader. Its seatbelt permits one pre-existing
// request and one fresh reply; there is no Herdr socket, terminal-content or action API.
import fs from "node:fs";
import path from "node:path";
import { createHash, randomBytes } from "node:crypto";
import { spawnSync } from "node:child_process";

const PRIVATE = "herdr-panes-v1";
const { O_RDONLY, O_NOFOLLOW, O_NONBLOCK } = fs.constants;
export const token = (s) => typeof s === "string" && /^[0-9a-f]{32}$/.test(s);
export const isWorkspaceId = (s) =>
  typeof s === "string" && s.length === 64 && /^[0-9a-f]+$/.test(s);
export const localId = (s) =>
  typeof s === "string" && /^[A-Za-z0-9][A-Za-z0-9_-]{0,63}$/.test(s);
export const keys = (o, csv) =>
  o !== null && typeof o === "object" && !Array.isArray(o) &&
  Object.keys(o).sort().join(",") === csv;
const validIdentity = (a) =>
  Array.isArray(a) && a.length === 2 &&
  a.every((s) => typeof s === "string" && /^[0-9]{1,20}$/.test(s));
export const directName = (s) =>
  typeof s === "string" && s.length <= 1024 && s.startsWith("/") &&
  path.resolve(s) === s && !/[\x00-\x1f\x7f]/.test(s);
export const validBinding = (b) =>
  keys(b, "dir,in,marker,out,request,root,source,space") &&
  directName(b.root) && path.basename(b.root) === "kit-pane-share-v1" &&
  token(b.source) && localId(b.space) &&
  [b.dir, b.in, b.out, b.request, b.marker].every(validIdentity);
export const validQuery = (q) =>
  keys(q, "filter,limit,sort") && Number.isInteger(q.limit) && q.limit >= 1 &&
  q.limit <= 48 && ["all", "needsAttention"].includes(q.filter) &&
  ["title", "attentionFirst"].includes(q.sort);

// Everything this function uses is explicit so the child never loads an editable helper or
// dependencies from the workspace. Input is JSON on stdin, never interpolated in executable JS.
async function readChannel(fs, path, b, q, nonce, ws) {
  const need = (ok) => {
    if (!ok) throw Error("unverifiedSource");
  };
  const shape = (o, csv) =>
    o && typeof o === "object" && !Array.isArray(o) &&
    Object.keys(o).sort().join(",") === csv;
  const id = (s) =>
    typeof s === "string" && /^[A-Za-z0-9][A-Za-z0-9_-]{0,63}$/.test(s);
  const same = (s, i) => String(s.dev) === i[0] && String(s.ino) === i[1];
  const mode = (s) => Number(s.mode & 0o777n);
  const uid = BigInt(process.getuid());
  const { O_NOFOLLOW, O_NONBLOCK, O_RDWR, O_RDONLY } = fs.constants;
  const inbox = path.join(b.root, "in"), outbox = path.join(b.root, "out");
  const request = path.join(inbox, "request.json"),
    prefix = path.join(outbox, b.source + "-");
  const marker = prefix + "listening.json", response = prefix + nonce + ".json";
  function withFile(name, flags, privateFile, action) {
    const fd = fs.openSync(name, flags | O_NOFOLLOW | O_NONBLOCK);
    try {
      const st = fs.fstatSync(fd, { bigint: true });
      need(
        st.isFile() && st.nlink === 1n && st.uid === uid &&
          (privateFile
            ? mode(st) === 0o600
            : [0o600, 0o640, 0o644].includes(mode(st))),
      );
      return action(fd, st);
    } finally {
      fs.closeSync(fd);
    }
  }
  function bytes(fd, st, cap) {
    need(st.size <= BigInt(cap));
    const buf = Buffer.alloc(cap + 1),
      n = fs.readSync(fd, buf, 0, buf.length, 0);
    need(n <= cap);
    return buf.subarray(0, n);
  }
  function paths() {
    for (let d = b.root; d !== "/"; d = path.dirname(d)) {
      need(fs.lstatSync(d).isDirectory());
    }
    for (const [p, i] of [[b.root, b.dir], [inbox, b.in], [outbox, b.out]]) {
      const s = fs.lstatSync(p, { bigint: true });
      need(s.isDirectory() && s.uid === uid && mode(s) === 0o700 && same(s, i));
    }
    withFile(marker, O_RDONLY, false, (fd, s) => {
      need(same(s, b.marker));
      const m = JSON.parse(
        new TextDecoder("utf-8", { fatal: true }).decode(bytes(fd, s, 512)),
      );
      need(
        shape(m, "capability,source,space,state,v,workspace") && m.v === 2 &&
          m.workspace === ws &&
          m.capability === "herdr.panes.metadata" && m.state === "listening" &&
          m.space === b.space && m.source === b.source,
      );
    });
  }
  function verify(body) {
    need(
      body.v === 2 && body.workspace === ws && body.source === b.source &&
        body.space === b.space &&
        body.nonce === nonce,
    );
    if (body.state === "unavailable" || body.state === "unsupportedRequest") {
      need(shape(body, "nonce,source,space,state,v,workspace"));
      return { state: body.state };
    }
    need(
      body.state === "ready" &&
        shape(body, "more,nonce,rows,source,space,state,v,workspace") &&
        typeof body.more === "boolean" && Array.isArray(body.rows) &&
        body.rows.length <= q.limit &&
        (!body.more || body.rows.length === q.limit),
    );
    const seen = new Set(), rank = { blocked: 0, done: 1, working: 2, idle: 3 };
    let last = null;
    for (const r of body.rows) {
      need(
        shape(
          r,
          "createdMs,focused,pane,pid,status,tab,terminal,title,zoomed",
        ) && id(r.pane) && id(r.tab) &&
          /^t[1-9][0-9]{0,11}$/.test(r.terminal) &&
          r.title === "Pane " + r.terminal.slice(1) &&
          Number.isSafeInteger(r.pid) && r.pid > 0 &&
          Number.isSafeInteger(r.createdMs) && r.createdMs > 0 &&
          typeof r.focused === "boolean" && typeof r.zoomed === "boolean" &&
          [null, "working", "idle", "done", "blocked"].includes(r.status) &&
          (q.filter !== "needsAttention" ||
            ["done", "blocked"].includes(r.status)) &&
          !seen.has(r.pane),
      );
      const order = [
        q.sort === "attentionFirst" ? (rank[r.status] ?? 4) : 0,
        Number(r.terminal.slice(1)),
        r.pane,
      ];
      need(
        !last || last[0] < order[0] ||
          (last[0] === order[0] &&
            (last[1] < order[1] ||
              (last[1] === order[1] && last[2] < order[2]))),
      );
      last = order;
      seen.add(r.pane);
    }
    return { state: "ready", rows: body.rows, more: body.more };
  }
  paths();
  const raw = Buffer.from(
    JSON.stringify({
      v: 2,
      op: "list",
      source: b.source,
      workspace: ws,
      nonce,
      ...q,
    }),
  );
  withFile(request, O_RDWR, true, (fd, s) => {
    need(same(s, b.request) && s.size <= 512n);
    need(fs.writeSync(fd, raw, 0, raw.length, 0) === raw.length);
    fs.ftruncateSync(fd, raw.length);
  });
  const end = performance.now() + 1800;
  do {
    paths();
    try {
      const result = withFile(response, O_RDONLY, false, (fd, s) => {
        need(String(s.dev) === b.out[0]);
        const content = bytes(fd, s, 24576);
        if (!content.length) return null;
        const result = verify(
          JSON.parse(new TextDecoder("utf-8", { fatal: true }).decode(content)),
        );
        paths();
        const current = withFile(request, O_RDONLY, true, (qfd, st) => {
          need(same(st, b.request));
          return raw.equals(bytes(qfd, st, 512));
        });
        const at = fs.lstatSync(response, { bigint: true });
        need(
          at.dev === s.dev && at.ino === s.ino && at.nlink === 1n &&
            at.size === BigInt(content.length),
        );
        fs.unlinkSync(response);
        return current ? result : { state: "supersededRequest" };
      });
      if (result) return result;
    } catch (e) {
      if (e.code !== "ENOENT" && !(e instanceof SyntaxError)) throw e;
    }
    await new Promise((r) => setTimeout(r, 25));
  } while (performance.now() < end);
  return { state: "noFreshReply" };
}

// Only OS-supplied runtime files and the exact enrollment files. In particular the child
// cannot enumerate out/, read the grant, open a socket, create a file, or replace a reply.
export function readerPolicy(b, nonce, exe) {
  if (!validBinding(b) || !token(nonce) || !directName(exe)) {
    throw Error("unverifiedSource");
  }
  const literal = (s) => "(literal " + JSON.stringify(s) + ")";
  const inbox = path.join(b.root, "in"), outbox = path.join(b.root, "out");
  const request = path.join(inbox, "request.json"),
    prefix = path.join(outbox, b.source + "-"),
    reply = prefix + nonce + ".json";
  const parents = new Set([outbox, inbox]);
  for (const start of [b.root, path.dirname(exe)]) {
    for (let d = start; d !== "/"; d = path.dirname(d)) {
      parents.add(d);
    }
  }
  return [
    "(version 1)(deny default)(allow sysctl-read)(allow process-info* (target self))",
    `(allow process-exec ${
      literal(exe)
    })(allow file-map-executable (subpath "/System/Library") (subpath "/usr/lib") ${
      literal(exe)
    })`,
    `(allow file-read* (literal "/") (subpath "/System/Library") (subpath "/usr/lib") (subpath "/usr/share/icu") (literal "/dev/urandom") ${
      literal(exe)
    })`,
    '(allow file-read* file-write-data (literal "/dev/null") (literal "/dev/zero"))',
    `(allow file-read-metadata ${[...parents].map(literal).join(" ")})`,
    `(allow file-read* ${
      [request, prefix + "listening.json", reply].map(literal).join(" ")
    })`,
    `(allow file-write-data ${literal(request)})(allow file-write-unlink ${
      literal(reply)
    })`,
  ].join("\n");
}
export function queryMetadata(binding, query, ws) {
  if (process.platform !== "darwin" || !process.versions.bun) {
    return { state: "readerUnavailable" };
  }
  if (!validBinding(binding) || !validQuery(query) || !isWorkspaceId(ws)) {
    return { state: "invalidRequest" };
  }
  const exe = fs.realpathSync(process.execPath),
    nonce = randomBytes(16).toString("hex");
  const code =
    `import fs from 'node:fs';import path from 'node:path';try{const [b,q,n,w]=JSON.parse(await Bun.stdin.text());console.log(JSON.stringify(await (${readChannel.toString()})(fs,path,b,q,n,w)))}catch{console.log('{"state":"unverifiedSource"}')}`;
  const run = spawnSync("/usr/bin/sandbox-exec", [
    "-p",
    readerPolicy(binding, nonce, exe),
    exe,
    "--config=/dev/null",
    "--no-env-file",
    "--no-install",
    "--no-macros",
    "--log-level=error",
    "-e",
    code,
  ], {
    cwd: "/",
    env: {
      HOME: "/dev/null",
      PATH: "/usr/bin:/bin",
      BUN_RUNTIME_TRANSPILER_CACHE_PATH: "0",
    },
    input: JSON.stringify([binding, query, nonce, ws]),
    encoding: "utf8",
    timeout: 3600,
    killSignal: "SIGKILL",
    maxBuffer: 32768,
  });
  if (run.error || run.status !== 0) return { state: "readerUnavailable" };
  try {
    return JSON.parse(run.stdout);
  } catch {
    return { state: "readerUnavailable" };
  }
}

// Workspace-side reads use the same ownership and generation checks in the CLI and native host.
export class Refusal extends Error {
  constructor(state) {
    super(state);
    this.state = state;
  }
}
export function need(ok, state = "unverifiedSource") {
  if (!ok) throw new Refusal(state);
}
export const identity = (s) => [String(s.dev), String(s.ino)];
export const equal = (a, b) => a[0] === b[0] && a[1] === b[1];
export const failure = (e) => ({
  state: e instanceof Refusal ? e.state : "unavailable",
});
export const guarded = () =>
  [
    "SCRIPT_KIT_AGENT_CREDENTIAL",
    "SCRIPT_KIT_MCP_TOKEN",
    "SCRIPT_KIT_MCP_ENDPOINT",
    "SCRIPT_KIT_MCP_SERVER_JSON",
  ].some((s) => Object.hasOwn(process.env, s));
export const safeMode = () =>
  process.env.SCRIPT_KIT_PERSONAL_SAFE_MODE !== undefined &&
  process.env.SCRIPT_KIT_PERSONAL_SAFE_MODE !== "0";
export function directory(p, privateDir = false) {
  need(directName(p));
  for (let d = p; d !== "/"; d = path.dirname(d)) {
    need(fs.lstatSync(d).isDirectory());
  }
  const st = fs.lstatSync(p, { bigint: true });
  need(
    st.uid === BigInt(process.getuid()) &&
      (privateDir ? (st.mode & 0o777n) === 0o700n : (st.mode & 0o022n) === 0n),
  );
  return identity(st);
}
export function workspace(root) {
  const [d, i] = directory(root);
  return createHash("sha256").update(`kit-personal-v1:${d}:${i}`).digest("hex");
}
export function inspectFile(p, cap, privateFile = true) {
  const fd = fs.openSync(p, O_RDONLY | O_NOFOLLOW | O_NONBLOCK);
  try {
    const s = fs.fstatSync(fd, { bigint: true });
    need(
      s.isFile() && s.uid === BigInt(process.getuid()) && s.nlink === 1n &&
        (privateFile
          ? (s.mode & 0o777n) === 0o600n
          : [0o600n, 0o640n, 0o644n].includes(s.mode & 0o777n)) &&
        s.size <= BigInt(cap),
    );
    const buf = Buffer.alloc(cap + 1),
      n = fs.readSync(fd, buf, 0, buf.length, 0);
    need(n <= cap && BigInt(n) === s.size);
    return {
      at: identity(s),
      text: new TextDecoder("utf-8", { fatal: true }).decode(
        buf.subarray(0, n),
      ),
    };
  } finally {
    fs.closeSync(fd);
  }
}
export function locations(root) {
  const db = path.join(root, "db");
  directory(db);
  const privateDir = path.join(db, PRIVATE);
  try {
    const at = directory(privateDir, true);
    return {
      dir: privateDir,
      at,
      file: path.join(privateDir, "pair.json"),
      lock: path.join(privateDir, "writer"),
    };
  } catch (e) {
    if (
      e.code === "ENOENT" &&
      fs.lstatSync(privateDir, { throwIfNoEntry: false }) === undefined
    ) return null;
    throw e;
  }
}
export function stored(loc, ws) {
  if (!loc) return null;
  let f;
  try {
    f = inspectFile(loc.file, 4096);
  } catch (e) {
    if (e.code === "ENOENT" && equal(loc.at, directory(loc.dir, true))) {
      return null;
    }
    throw e;
  }
  const record = JSON.parse(f.text);
  need(
    keys(record, "binding,id,v,workspace") && [1, 2].includes(record.v) &&
      token(record.id) && validBinding(record.binding),
    "unverifiedPairing",
  );
  need(record.workspace === ws, "workspaceChanged");
  return { record, ...f };
}
export function snapshot(root, ws) {
  const l = locations(root);
  if (!l) return null;
  need(!fs.lstatSync(l.lock, { throwIfNoEntry: false }), "pairingBusy");
  const s = stored(l, ws);
  need(
    !fs.lstatSync(l.lock, { throwIfNoEntry: false }) &&
      equal(l.at, directory(l.dir, true)),
    "pairingBusy",
  );
  return s;
}
export function pairedFields(r) {
  return {
    workspace: r.workspace,
    pairId: r.id,
    source: r.binding.source,
    space: r.binding.space,
    capability: "herdr.panes.metadata",
  };
}
function unchanged(root, ws, prev) {
  const now = snapshot(root, workspace(root));
  return now && now.record.workspace === ws && equal(now.at, prev.at) &&
    now.text === prev.text;
}
export function listPaired(
  root,
  expectedWorkspace,
  query,
  expectedPair = null,
) {
  try {
    if (process.platform !== "darwin") return { state: "unsupportedPlatform" };
    if (guarded()) return { state: "agentContextDenied" };
    if (safeMode()) return { state: "safeMode" };
    need(
      validQuery(query) && (expectedPair === null || token(expectedPair)),
      "invalidRequest",
    );
    const ws = workspace(root);
    need(ws === expectedWorkspace, "workspaceChanged");
    const s = snapshot(root, ws);
    if (!s) return { state: "notPaired" };
    need(
      expectedPair === null || expectedPair === s.record.id,
      "pairingChanged",
    );
    // A pre-workspace offer is not authority to read. Keep the old record only for explicit unpair.
    need(s.record.v === 2, "renewalRequired");
    const result = queryMetadata(s.record.binding, query, ws);
    need(unchanged(root, ws, s), "pairingChanged");
    return { ...result, ...pairedFields(s.record) };
  } catch (e) {
    return failure(e);
  }
}
