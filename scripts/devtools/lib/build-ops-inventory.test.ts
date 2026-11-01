import { afterEach, beforeEach, expect, spyOn, test } from "bun:test";
import * as fs from "node:fs";
import * as fsPromises from "node:fs/promises";
import { EventEmitter } from "node:events";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import * as workerThreads from "node:worker_threads";
import { allocation, buildStorage, BuildResourceError, requireBuildAdmission, startBuildResourceGuard } from "./build-ops-inventory.ts";
import type { AllocationObservation, BuildResourceObservation, BuildResourceSummary } from "./build-ops-inventory.ts";
import { registerManagedStorageRoute } from "../../agentic/managed-storage.ts";

// Bun spies invoke constructor bodies as functions; this view preserves the actual module property.
interface WorkerMockBoundary {
  Worker(filename: string | URL, options?: workerThreads.WorkerOptions): workerThreads.Worker;
}
interface NumberStatfsBoundary {
  statfs(path: fs.PathLike, options?: fs.StatFsOptions & { bigint?: false }): Promise<fs.StatsFs>;
}
const workerMockBoundary = workerThreads as unknown as WorkerMockBoundary;
// The resource guard exercises only the numeric statfs overload, never bigint observations.
const numberStatfsBoundary: NumberStatfsBoundary = fsPromises;

const GiB = 1024 ** 3;
const roots: string[] = [], restore: Array<() => void> = [];
const environment = ["SCRIPT_KIT_AGENT_TARGET_BUDGET_GB", "SCRIPT_KIT_AGENT_MIN_FREE_GB", "SCRIPT_KIT_AGENT_FREE_SAMPLE_TIMEOUT_MS", "SCRIPT_KIT_AGENT_ALLOW_LOW_DISK", "SCRIPT_KIT_NONINTERACTIVE", "CARGO_HOME", "RUSTUP_HOME", "SK_PATH", "SCCACHE_DIR"];
beforeEach(() => {
  const saved = environment.map(name => [name, process.env[name]] as const);
  restore.push(() => { for (const [name, value] of saved) { if (value === undefined) delete process.env[name]; else process.env[name] = value; } });
  process.env.SCRIPT_KIT_AGENT_TARGET_BUDGET_GB = "1";
  process.env.SCRIPT_KIT_AGENT_MIN_FREE_GB = "0";
  delete process.env.SCRIPT_KIT_AGENT_ALLOW_LOW_DISK;
  delete process.env.SCRIPT_KIT_AGENT_FREE_SAMPLE_TIMEOUT_MS;
});
afterEach(() => {
  for (const undo of restore.splice(0).reverse()) undo();
  for (const root of roots.splice(0)) fs.rmSync(root, { recursive: true, force: true });
});
function fixture(): string {
  const root = fs.realpathSync(fs.mkdtempSync(join(tmpdir(), "build-resource-behavior-")));
  roots.push(root); return root;
}
function file(root: string, child: string): string {
  const path = join(root, child);
  fs.mkdirSync(dirname(path), { recursive: true }); fs.writeFileSync(path, Buffer.alloc(8192, 97)); return path;
}
function volume(bytes: number) { return { bavail: bytes, bsize: 1 } as fs.StatsFs; }
function observedVolume(bytes: number): void {
  const mock = spyOn(fs, "statfsSync").mockReturnValue(volume(bytes)); restore.push(() => mock.mockRestore());
}

test("a trusted linked tree is budgeted and either volume can refuse free-space admission", () => {
  const root = fixture(), destination = fixture();
  fs.symlinkSync(destination, join(root, "target-agent"));
  registerManagedStorageRoute(root, destination);
  file(destination, "shared/payload"); file(destination, "artifacts/payload");
  process.env.SCRIPT_KIT_AGENT_MIN_FREE_GB = "1";
  const query = spyOn(fs, "statfsSync").mockImplementation(path => volume(String(path) === destination ? GiB - 1 : 2 * GiB));
  restore.push(() => query.mockRestore());
  const storage = buildStorage(root, false, true);
  expect(storage.targetAgentComplete).toBe(true);
  expect(storage.targetAgentAllocatedBytes).toBeGreaterThanOrEqual(16_384);
  expect(refusal(() => requireBuildAdmission(root)).code).toBe("resource_free_space_below_floor");
  query.mockImplementation(path => volume(String(path) === root ? GiB - 1 : 2 * GiB));
  expect(refusal(() => requireBuildAdmission(root)).code).toBe("resource_free_space_below_floor");
});
function refusal(fn: () => unknown): BuildResourceError {
  try { fn(); } catch (error) { expect(error).toBeInstanceOf(BuildResourceError); return error as BuildResourceError; }
  throw new Error("expected resource refusal");
}
function sample(allocatedBytes: number, errors: string[] = []): AllocationObservation {
  return { present: true, complete: errors.length === 0, logicalBytes: allocatedBytes, allocatedBytes, entries: 1, linksNotFollowed: 0, errors };
}
function externalFixture(allocatedBytes = 42 * GiB) {
  const root = fixture(), destination = fixture(), payload = file(destination, "unknown/payload");
  const original = fs.lstatSync, device = Number(fs.lstatSync(root).dev) + 1;
  const stat = spyOn(fs, "lstatSync").mockImplementation(((candidate: fs.PathLike, ...args: unknown[]) => {
    const value = Reflect.apply(original, fs, [candidate, ...args]), path = String(candidate);
    if (path !== destination && !path.startsWith(destination + "/")) return value;
    return Object.assign(Object.create(Object.getPrototypeOf(value)), value, { dev: device },
      path === payload ? { blocks: allocatedBytes / 512, size: allocatedBytes } : {});
  }) as typeof fs.lstatSync);
  restore.push(() => stat.mockRestore());
  fs.symlinkSync(destination, join(root, "target-agent"));
  registerManagedStorageRoute(root, destination);
  delete process.env.SCRIPT_KIT_AGENT_TARGET_BUDGET_GB;
  return { root, destination };
}

test("trusted cross-volume storage is uncapped but publication still reserves free space", () => {
  const f = externalFixture(); observedVolume(GiB);
  process.env.SCRIPT_KIT_AGENT_MIN_FREE_GB = "1";
  const admitted = requireBuildAdmission(f.root);
  expect(admitted.targetAgentAllocatedBytes).toBeGreaterThan(40 * GiB);
  expect(admitted.targetAgentBudgetBytes).toBeNull();
  expect(admitted.withinLimits).toBe(true);
  expect(refusal(() => requireBuildAdmission(f.root, { phase: "pre-publication", reserveBytes: 1 })).code).toBe("resource_free_space_below_floor");
});

test("an explicit allocation budget still limits trusted cross-volume storage", () => {
  const f = externalFixture(); observedVolume(2 * GiB);
  process.env.SCRIPT_KIT_AGENT_TARGET_BUDGET_GB = "1";
  expect(refusal(() => requireBuildAdmission(f.root)).code).toBe("resource_budget_exceeded");
});

test("uncapped storage still refuses incomplete allocation observations", () => {
  const f = externalFixture(); observedVolume(2 * GiB);
  fs.symlinkSync(f.root, join(f.destination, "shared"));
  const denied = refusal(() => requireBuildAdmission(f.root));
  expect(denied.code).toBe("resource_observation_incomplete");
  expect(denied.observation.targetAgentBudgetBytes).toBeNull();
  expect(denied.observation.complete).toBe(false);
});


test("complete scope includes exports, pending, shared, quarantine, runtime, leases and unknown entries", () => {
  const root = fixture();
  for (const child of ["pools/agent-debug/debug/deps/output", "agents/old/debug/output", "artifacts/export/executable", "artifacts/.pending-copy/executable", "shared/compiler/cache", "runtime/task/log", ".locks/lease/owner", ".quarantine/object/payload", "unknown/payload"]) file(root, `target-agent/${child}`);
  const expected = allocation(join(root, "target-agent"));
  const before = buildStorage(root, false, true);
  expect(before.targetAgentComplete).toBe(true);
  expect(before.targetAgentAllocatedBytes).toBe(expected.allocatedBytes);
  expect(before.targetAgentAllocatedBytes).toBeGreaterThan(before.agentPoolAllocatedBytes);
  file(root, "target/dev-output"); file(root, ".test-output/evidence/payload"); file(root, "target/goose-sidecar/sidecar");
  for (const name of ["cargo", "rustup", "kit", "sccache"]) file(root, `outside/${name}/payload`);
  process.env.CARGO_HOME = join(root, "outside/cargo"); process.env.RUSTUP_HOME = join(root, "outside/rustup");
  process.env.SK_PATH = join(root, "outside/kit"); process.env.SCCACHE_DIR = join(root, "outside/sccache");
  const full = buildStorage(root, true);
  expect(full.targetAgentAllocatedBytes).toBe(before.targetAgentAllocatedBytes);
  expect(full.categories.find(item => item.category === "dev-target")!.allocatedBytes).toBeGreaterThan(0);
  expect(full.categories.find(item => item.category === "evidence-and-tasks")!.allocatedBytes).toBeGreaterThan(0);
  expect(full.external.find(item => item.category === "goose-sidecar")!.allocatedBytes).toBeGreaterThan(0);
  expect(full.external.find(item => item.category === "configured-sccache")!.allocatedBytes).toBeGreaterThan(0);
});

test("budget and pool subtotal cannot lose hardlinked bytes to diagnostic category order", () => {
  const root = fixture(), payload = file(root, "target/dev-output");
  for (const child of ["pools/agent-debug/debug/deps/output", "agents/old/debug/output", "artifacts/export/executable", "shared/cache/output"]) {
    const path = join(root, "target-agent", child); fs.mkdirSync(dirname(path), { recursive: true }); fs.linkSync(payload, path);
  }
  const full = buildStorage(root, false), budget = buildStorage(root, false, true);
  const expected = allocation(join(root, "target-agent"));
  expect(full.targetAgentAllocatedBytes).toBe(expected.allocatedBytes);
  expect(budget.targetAgentAllocatedBytes).toBe(full.targetAgentAllocatedBytes);
  expect(full.agentPoolAllocatedBytes).toBe(budget.agentPoolAllocatedBytes);
  const poolSeen = new Set<string>();
  const poolBytes = allocation(join(root, "target-agent/agents"), [], poolSeen).allocatedBytes + allocation(join(root, "target-agent/pools"), [], poolSeen).allocatedBytes;
  expect(full.agentPoolAllocatedBytes).toBe(poolBytes);
  const budgetSeen = new Set<string>();
  allocation(join(root, "target"), [], budgetSeen);
  expect(allocation(join(root, "target-agent"), [], budgetSeen).allocatedBytes).toBe(expected.allocatedBytes - fs.lstatSync(payload).blocks * 512);
});

test("missing roots are absent but permission failures and disappearing children are incomplete", () => {
  const root = fixture(); observedVolume(2 * GiB);
  expect(allocation(join(root, "missing"))).toMatchObject({ present: false, complete: true, allocatedBytes: 0 });
  expect(requireBuildAdmission(root)).toMatchObject({ complete: true, targetAgentAllocatedBytes: 0, withinLimits: true });
  const path = file(root, "target-agent/unknown/payload"), original = fs.lstatSync;
  let failurePath = join(root, "target-agent"), code = "EACCES";
  const stat = spyOn(fs, "lstatSync").mockImplementation(((candidate: fs.PathLike, ...args: unknown[]) => {
    if (String(candidate) === failurePath) throw Object.assign(new Error(code), { code });
    return Reflect.apply(original, fs, [candidate, ...args]);
  }) as typeof fs.lstatSync);
  restore.push(() => stat.mockRestore());
  expect(allocation(failurePath)).toMatchObject({ present: false, complete: false });
  expect(refusal(() => requireBuildAdmission(root)).code).toBe("resource_observation_incomplete");
  failurePath = path; code = "ENOENT";
  expect(buildStorage(root, false, true)).toMatchObject({ targetAgentComplete: false });
  expect(refusal(() => requireBuildAdmission(root)).observation.complete).toBe(false);
});

// Mutate only these private fixtures, at the precise boundary after readdir and before lstat.
function beforeStat(effect: (path: string) => void): void {
  const original = fs.lstatSync;
  const spy = spyOn(fs, "lstatSync").mockImplementation(((path: fs.PathLike, ...args: unknown[]) => {
    effect(String(path)); return Reflect.apply(original, fs, [path, ...args]);
  }) as typeof fs.lstatSync);
  restore.push(() => spy.mockRestore());
}

const compilerDeps = "target-agent/pools/agent-debug/debug/deps";
for (const stage of ["initial", "empty EOF", "nonempty EOF", "unknown scope", "boundary replacement"] as const) test(`enumerated compiler directory: ${stage}`, () => {
  const root = fixture(), deps = join(root, stage === "unknown scope" ? "target-agent/unknown" : compilerDeps);
  const target = join(deps, "arbitrary-name"); fs.mkdirSync(target, { recursive: true });
  if (stage === "nonempty EOF") file(target, "payload");
  const keep = file(deps, "stable"); fs.linkSync(keep, join(deps, "alias"));
  let visits = 0, moved = false;
  beforeStat(path => {
    if (path !== target) return;
    visits++;
    if (!moved && visits === (stage.endsWith("EOF") ? 2 : 1)) {
      moved = true; fs.renameSync(target, join(root, "outside")); file(deps, "late");
    } else if (moved && stage === "boundary replacement") fs.symlinkSync(join(root, "outside"), target);
  });
  observedVolume(2 * GiB);
  if (stage === "initial" || stage === "empty EOF") {
    const recovered = requireBuildAdmission(root);
    expect(recovered).toEqual(requireBuildAdmission(root));
    expect(recovered.targetAgentAllocatedBytes).toBe(allocation(join(root, "target-agent")).allocatedBytes);
  } else expect(refusal(() => requireBuildAdmission(root)).code).toBe("resource_observation_incomplete");
  expect(moved).toBe(true);
});

for (const lostKind of ["directory", "file"] as const) test(`a lost compiler ${lostKind} cannot conceal a move into a previously measured part of the managed tree`, () => {
  const root = fixture(), tree = join(root, "target-agent"), prior = join(tree, "already-measured");
  const gone = join(root, compilerDeps, "arbitrary-name");
  fs.mkdirSync(prior, { recursive: true }); fs.mkdirSync(lostKind === "directory" ? gone : dirname(gone), { recursive: true });
  fs.writeFileSync(lostKind === "directory" ? join(gone, "data") : gone, Buffer.alloc(512 * 1024, 97));
  const limit = allocation(tree).allocatedBytes - 128 * 1024;
  const open = fs.opendirSync.bind(fs);
  // Both are real entries on disk. Force one permitted traversal order instead
  // of relying on APFS/ext4 ordering to visit the destination before deps.
  const ordered = spyOn(fs, "opendirSync").mockImplementation(((path: fs.PathLike) => {
    const dir = open(path);
    if (String(path) !== tree) return dir;
    const rows: fs.Dirent[] = []; let e;
    while ((e = dir.readSync()) !== null) rows.push(e);
    rows.sort((a, b) => Number(b.name === "already-measured") - Number(a.name === "already-measured"));
    return { readSync: () => rows.shift() ?? null, closeSync: () => dir.closeSync() } as fs.Dir;
  }) as typeof fs.opendirSync);
  restore.push(() => ordered.mockRestore());
  let moved = false;
  beforeStat(path => {
    if (!moved && path === gone) { moved = true; fs.renameSync(gone, join(prior, "still-budgeted")); }
  });
  observedVolume(2 * GiB);
  const admit = () => requireBuildAdmission(root, { reserveBytes: GiB - limit });
  expect(["resource_observation_incomplete", "resource_budget_exceeded"]).toContain(refusal(admit).code);
  expect(moved).toBe(true);
  expect(refusal(admit).code).toBe("resource_budget_exceeded");
});

test("compiler replay measures concurrent fingerprint outputs and their hardlinks exactly once", () => {
  const root = fixture(), tree = join(root, "target-agent"), deps = join(root, compilerDeps);
  const debug = dirname(deps), fingerprint = join(debug, ".fingerprint/crate-hash");
  file(fingerprint, "invoked.timestamp");
  const gone = join(deps, "rmeta-work"); fs.mkdirSync(gone, { recursive: true });
  const seed = file(deps, "libcrate.rmeta");
  const open = fs.opendirSync.bind(fs);
  const ordered = spyOn(fs, "opendirSync").mockImplementation(((path: fs.PathLike) => {
    const dir = open(path);
    if (String(path) !== debug) return dir;
    const rows: fs.Dirent[] = []; let e;
    while ((e = dir.readSync()) !== null) rows.push(e);
    rows.sort((a,b) => Number(b.name === ".fingerprint") - Number(a.name === ".fingerprint"));
    return { readSync: () => rows.shift() ?? null, closeSync: () => dir.closeSync() } as fs.Dir;
  }) as typeof fs.opendirSync);
  restore.push(() => ordered.mockRestore());
  observedVolume(2 * GiB);
  const baseline = requireBuildAdmission(root).targetAgentAllocatedBytes;
  expect(baseline).toBe(allocation(tree).allocatedBytes);
  let changed = false;
  beforeStat(path => {
    if (changed || path !== gone) return;
    changed = true; fs.rmdirSync(gone);
    fs.writeFileSync(join(fingerprint, "lib-crate.json"), Buffer.alloc(128 * 1024, 97));
    fs.linkSync(seed, join(fingerprint, "alias"));
    fs.utimesSync(fingerprint, 100, 100);
  });
  const recovered = requireBuildAdmission(root);
  expect(changed).toBe(true);
  expect(recovered.targetAgentAllocatedBytes).toBeGreaterThan(baseline);
  expect(recovered.targetAgentAllocatedBytes).toBe(allocation(tree).allocatedBytes);
});

test("a probed sibling cannot be replaced by a cheaper namesake after moving into measured compiler output", () => {
  const root = fixture(), tree = join(root, "target-agent"), deps = join(root, compilerDeps);
  const prior = join(tree, "probed-first"), gone = join(deps, "gone");
  fs.mkdirSync(gone, { recursive: true }); fs.writeFileSync(prior, Buffer.alloc(512 * 1024, 97));
  const limit = allocation(tree).allocatedBytes - 128 * 1024;
  const open = fs.opendirSync.bind(fs); let opens = 0, lost = false, moved = false;
  const ordered = spyOn(fs, "opendirSync").mockImplementation(((path: fs.PathLike) => {
    if (String(path) !== tree) return open(path);
    if (++opens === 2) {
      fs.renameSync(prior, join(deps, "still-budgeted")); file(tree, "probed-first"); moved = true;
    }
    const dir = open(path); const rows: fs.Dirent[] = []; let e;
    while ((e = dir.readSync()) !== null) rows.push(e);
    rows.sort((a, b) => Number(b.name === "probed-first") - Number(a.name === "probed-first"));
    return { readSync: () => rows.shift() ?? null, closeSync: () => dir.closeSync() } as fs.Dir;
  }) as typeof fs.opendirSync); restore.push(() => ordered.mockRestore());
  beforeStat(path => { if (!lost && path === gone) { lost = true; fs.rmdirSync(gone); } });
  observedVolume(2 * GiB);
  const admit = () => requireBuildAdmission(root, { reserveBytes: GiB - limit });
  const denied = refusal(admit);
  // Real budget breaches take precedence, but the failed identity proof is retained too.
  expect(denied.code).toBe("resource_budget_exceeded");
  expect(denied.observation.complete).toBe(false);
  expect(denied.observation.failureCodes).toContain("resource_observation_incomplete");
  expect(denied.observation.failureCodes).toContain("resource_budget_exceeded");
  expect(denied.observation.targetAgentAllocatedBytes + denied.observation.reserveBytes).toBeGreaterThan(GiB);
  expect(lost && moved).toBe(true);
  expect(refusal(admit).code).toBe("resource_budget_exceeded");
});

for (const fault of ["missing", "replacement"] as const) test(`compiler-first parent reconciliation rejects ${fault}`, () => {
  const root = fixture(), deps = join(root, compilerDeps), payload = file(deps, "output");
  const open = fs.opendirSync.bind(fs);
  let parentOpens = 0;
  const spy = spyOn(fs, "opendirSync").mockImplementation(((path: fs.PathLike) => {
    if (String(path) === dirname(deps) && ++parentOpens === 2) {
      fs.renameSync(deps, join(root, "outside"));
      if (fault === "replacement") fs.mkdirSync(deps);
    }
    return open(path);
  }) as typeof fs.opendirSync);
  restore.push(() => spy.mockRestore());
  const denied = refusal(() => requireBuildAdmission(root));
  expect(denied.code).toBe("resource_observation_incomplete");
  expect(denied.observation.errors).toContain(`Error: inventory_probe_${fault === "missing" ? "missing" : "identity"}`);
  expect(fs.existsSync(payload)).toBe(false);
});

test("a new sibling arriving after compiler deps replay EOF invalidates that observation", () => {
  const root = fixture(), deps = join(root, compilerDeps), gone = join(deps, "gone");
  fs.mkdirSync(gone, { recursive: true });
  file(deps, "keep");
  let calls = 0;
  beforeStat(path => {
    if (path !== gone) return;
    if (++calls === 1) {
      fs.renameSync(gone, join(root, "outside"));
      fs.utimesSync(deps, 100, 100); // The replay takes its starting stamp after this.
    } else if (calls === 2) {
      // The replay reached EOF and is checking the vanished name. This new
      // allocation wasn't enumerated, even though the original name stays gone.
      file(deps, "arrived-after-eof");
    }
  });
  observedVolume(2 * GiB);
  expect(refusal(() => requireBuildAdmission(root)).code).toBe("resource_observation_incomplete");
  expect(calls).toBe(2);
  expect(fs.existsSync(join(deps, "arrived-after-eof"))).toBe(true);
});

for (const kind of ["deps root", "nested directory", "dangling symlink", "second disappearance"] as const) test(`compiler directory replay refuses ${kind}`, () => {
  const root = fixture(), deps = join(root, compilerDeps);
  const target = kind === "deps root" ? deps : join(deps, kind === "nested directory" ? "sub/child" : "sub");
  fs.mkdirSync(dirname(target), { recursive: true });
  if (kind === "dangling symlink") fs.symlinkSync(join(root, "missing"), target); else fs.mkdirSync(target);
  let moved = 0;
  beforeStat(path => {
    if ((!moved && path === target) || (kind === "second disappearance" && moved === 1 && path === join(deps, "late"))) {
      fs.renameSync(path, join(root, `outside-${++moved}`));
      if (kind === "second disappearance" && moved === 1) fs.mkdirSync(join(deps, "late"));
    }
  });
  expect(refusal(() => requireBuildAdmission(root)).code).toBe("resource_observation_incomplete");
  expect(moved).toBe(kind === "second disappearance" ? 2 : 1);
});

for (const failedReplay of [false, true]) test(`a directory replay cannot erase observed over-cap bytes (failed=${failedReplay})`, () => {
  const root = fixture(), deps = join(root, compilerDeps);
  for (const name of ["one", "two"]) file(join(deps, name), "payload");
  const dirs = [join(deps, "one"), join(deps, "two")], visited = new Set<string>();
  let moved = false, validations = 0;
  beforeStat(path => {
    if (moved && path === deps && ++validations === 2 && failedReplay) throw Object.assign(new Error("EACCES"), { code: "EACCES" });
    if (moved || !dirs.includes(path)) return;
    visited.add(path);
    if (visited.size === 2) {
      moved = true;
      for (const [i, dir] of dirs.entries()) fs.renameSync(dir, join(root, `moved-${i}`));
    }
  });
  observedVolume(2 * GiB);
  // Managed directories can themselves own blocks: use the unchanged empty tree as baseline.
  const empty = fixture(); fs.mkdirSync(join(empty, compilerDeps), { recursive: true });
  const baseline = allocation(join(empty, "target-agent")).allocatedBytes;
  const denied = refusal(() => requireBuildAdmission(root, { reserveBytes: GiB - baseline }));
  expect(moved).toBe(true);
  expect(denied.code).toBe("resource_budget_exceeded");
  expect(denied.observation.complete).toBe(!failedReplay);
  expect(denied.observation.targetAgentAllocatedBytes).toBeGreaterThan(allocation(join(root, "target-agent")).allocatedBytes);
});

test("one vanished object re-enumerates only its directory and commits exactly the clean rescan", () => {
  const root = fixture(), tree = join(root, "target-agent"), deps = join(tree, "debug/deps");
  const seed = file(root, "outside/seed"), stable = file(deps, "nested/keep");
  fs.linkSync(seed, join(deps, "already-seen")); fs.linkSync(stable, join(deps, "alias"));
  fs.symlinkSync(stable, join(deps, "count-the-link"));
  file(deps, "a.rcgu.o"); file(deps, "b.rcgu.o");
  const before = fs.lstatSync(seed), seeded = `${before.dev}:${before.ino}`, seen = new Set([seeded]);
  const opens = spyOn(fs, "opendirSync"); restore.push(() => opens.mockRestore());
  const objects: string[] = [];
  beforeStat(path => {
    if (!path.endsWith(".rcgu.o")) return;
    objects.push(path);
    if (objects.length === 2) {
      // One object has already contributed bytes and an inode; the next now fails lstat.
      fs.renameSync(objects[0]!, join(root, "outside/first")); fs.renameSync(path, join(root, "outside/second"));
      file(deps, "only-on-rescan");
    }
  });
  const recovered = allocation(tree, [], seen);
  expect(objects).toHaveLength(2);
  expect(opens.mock.calls.filter(([path]) => path === deps)).toHaveLength(2);
  expect(opens.mock.calls.filter(([path]) => path === tree)).toHaveLength(1);
  expect(recovered).toMatchObject({ complete: true, errors: [], linksNotFollowed: 1 });
  const cleanSeen = new Set([seeded]);
  expect(recovered).toEqual(allocation(tree, [], cleanSeen));
  expect(seen).toEqual(cleanSeen);
});

for (const fault of ["replacement", "symlink", "reappeared-link", "permission", "churn"] as const) test(`disappearance cannot recover from ${fault}`, () => {
  const root = fixture(), target = file(root, "target-agent/deps/1.rcgu.o"), deps = dirname(target);
  let misses = 0;
  beforeStat(path => {
    if (path === deps && misses && fault === "permission") throw Object.assign(new Error("EACCES"), { code: "EACCES" });
    if (!path.startsWith(deps + "/") || !path.endsWith(".rcgu.o")) return;
    misses++;
    if (fault === "replacement" || fault === "symlink") {
      fs.renameSync(deps, join(root, "old"));
      if (fault === "replacement") fs.mkdirSync(deps); else fs.symlinkSync(join(root, "old"), deps);
      // The redirected link still resolves the old child; model the observed lstat ENOENT.
      throw Object.assign(new Error("ENOENT"), { code: "ENOENT" });
    }
    fs.renameSync(path, join(root, `old-${misses}`));
    if (fault === "reappeared-link") {
      // The name is reused between listings. Do not follow or silently accept that link.
      fs.symlinkSync(join(root, `old-${misses}`), path);
      throw Object.assign(new Error("ENOENT"), { code: "ENOENT" });
    }
    if (fault === "churn") file(deps, `${misses + 1}.rcgu.o`);
  });
  const denied = refusal(() => requireBuildAdmission(root));
  expect(denied.code).toBe("resource_observation_incomplete");
  expect(denied.observation.complete).toBe(false);
  expect(denied.observation.errors!.join(" ")).toContain(fault === "churn" || fault === "reappeared-link" ? "inventory_unstable_directory" : fault === "permission" ? "EACCES" : "inventory_directory_changed");
  expect(misses).toBe(fault === "churn" ? 2 : 1);
});

test("a recovered allocation must still fit the publication reserve", () => {
  const root = fixture(), deps = join(root, "target-agent/deps"), target = file(deps, "gone.rcgu.o");
  observedVolume(2 * GiB);
  beforeStat(path => {
    if (path === target) { fs.renameSync(target, join(root, "old")); file(deps, "new-object"); }
  });
  const denied = refusal(() => requireBuildAdmission(root, { phase: "pre-publication", reserveBytes: GiB }));
  expect(denied.code).toBe("resource_budget_exceeded");
  expect(denied.observation).toMatchObject({ complete: true, withinLimits: false, reserveBytes: GiB,
    targetAgentAllocatedBytes: allocation(join(root, "target-agent")).allocatedBytes });
});

test("mounted subtrees refuse admission rather than disappearing from the budget", () => {
  const root = fixture(), payload = file(root, "target-agent/unknown/payload"), original = fs.lstatSync;
  observedVolume(2 * GiB);
  const stat = spyOn(fs, "lstatSync").mockImplementation(((candidate: fs.PathLike, ...args: unknown[]) => {
    const value = Reflect.apply(original, fs, [candidate, ...args]);
    return String(candidate) === payload ? Object.assign(Object.create(Object.getPrototypeOf(value)), value, { dev: Number(value.dev) + 1 }) : value;
  }) as typeof fs.lstatSync);
  restore.push(() => stat.mockRestore());
  expect(buildStorage(root, false, true).targetAgentErrors.join(" ")).toContain("inventory_unexpected_mount");
  expect(refusal(() => requireBuildAdmission(root)).code).toBe("resource_observation_incomplete");
});

for (const child of ["target-agent", "target-agent/pools", "target-agent/pools/agent-debug", "target-agent/pools/agent-debug/debug", "target-agent/shared", "target-agent/artifacts", "target-agent/runtime"]) test(`protected write root ${child} cannot redirect outside accounting`, () => {
  const root = fixture(); file(root, "outside/payload"); observedVolume(2 * GiB);
  fs.mkdirSync(dirname(join(root, child)), { recursive: true }); fs.symlinkSync(join(root, "outside"), join(root, child));
  const storage = buildStorage(root, false, true);
  expect(storage.targetAgentComplete).toBe(false);
  expect(storage.targetAgentErrors.join(" ")).toContain("inventory_output_symlink");
  expect(refusal(() => requireBuildAdmission(root)).code).toBe("resource_observation_incomplete");
});

test("unprotected links are counted without traversing their targets", () => {
  const root = fixture(); file(root, "outside/payload"); fs.mkdirSync(join(root, "target-agent/unknown"), { recursive: true });
  fs.symlinkSync(join(root, "outside"), join(root, "target-agent/unknown/link"));
  const result = allocation(join(root, "target-agent"));
  expect(result).toMatchObject({ complete: true, linksNotFollowed: 1, entries: 3 });
  expect(buildStorage(root, false, true).targetAgentAllocatedBytes).toBe(result.allocatedBytes);
});

test("entry and time limits are incomplete, not successful truncated inventories", () => {
  const root = fixture(), payload = file(root, "target-agent/payload"), tree = join(root, "target-agent");
  const originalStat = fs.lstatSync, originalOpen = fs.opendirSync, treeStat = fs.lstatSync(tree), payloadStat = fs.lstatSync(payload);
  const stat = spyOn(fs, "lstatSync").mockImplementation(((path: fs.PathLike, ...args: unknown[]) => String(path) === tree ? treeStat : String(path) === payload ? payloadStat : Reflect.apply(originalStat, fs, [path, ...args])) as typeof fs.lstatSync);
  let closed = 0;
  const open = spyOn(fs, "opendirSync").mockImplementation(((path: fs.PathLike, ...args: unknown[]) => String(path) === tree ? { readSync: () => ({ name: "payload" }), closeSync: () => { closed++; } } : Reflect.apply(originalOpen, fs, [path, ...args])) as typeof fs.opendirSync);
  const clock = spyOn(performance, "now").mockReturnValue(0);
  restore.push(() => stat.mockRestore(), () => open.mockRestore(), () => clock.mockRestore());
  const entries = allocation(tree);
  expect(entries.complete).toBe(false); expect(entries.errors.join(" ")).toContain("inventory_entry_limit"); expect(closed).toBe(1);
  let now = 0; clock.mockImplementation(() => { const value = now; now += 9_000; return value; });
  const expired = allocation(tree);
  expect(expired.complete).toBe(false); expect(expired.errors.join(" ")).toContain("inventory_time_limit");
});

for (const limit of ["entry", "time"]) test(`local rescan shares the original ${limit} limit`, () => {
  const tree = fixture(), payload = file(tree, "keep"), payloadStat = fs.lstatSync(payload), original = fs.lstatSync;
  const stat = spyOn(fs, "lstatSync").mockImplementation(((path: fs.PathLike, ...args: unknown[]) => String(path) === payload ? payloadStat : Reflect.apply(original, fs, [path, ...args])) as typeof fs.lstatSync);
  const timeout = Number(process.env.SCRIPT_KIT_AGENT_INVENTORY_TIMEOUT_MS ?? 8_000);
  let now = 0, opened = 0, closed = 0;
  const clock = spyOn(performance, "now").mockImplementation(() => now);
  const open = spyOn(fs, "opendirSync").mockImplementation((() => {
    const pass = ++opened;
    let reads = 0;
    return { readSync() {
      if (++reads <= (limit === "entry" ? 130_000 : 1)) return { name: "keep" };
      if (pass === 1) { now = timeout - 1; return { name: "vanished", isFile: () => true }; }
      if (limit === "time") now += 2;
      return null;
    }, closeSync() { closed++; } };
  }) as unknown as typeof fs.opendirSync);
  restore.push(() => stat.mockRestore(), () => clock.mockRestore(), () => open.mockRestore());
  const result = allocation(tree);
  expect(result.complete).toBe(false); expect(result.errors.join(" ")).toContain(`inventory_${limit}_limit`);
  expect(opened).toBe(2); expect(closed).toBe(2);
});

test("publication reserve needs both total-budget and free-space headroom, with inclusive boundaries", () => {
  const root = fixture(); file(root, "target-agent/unknown/payload");
  const allocated = buildStorage(root, false, true).targetAgentAllocatedBytes, reserveBytes = GiB - allocated;
  observedVolume(reserveBytes);
  expect(requireBuildAdmission(root, { phase: "before-copy", reserveBytes })).toMatchObject({ phase: "before-copy", reserveBytes, withinLimits: true, targetAgentAllocatedBytes: allocated, scope: "target-agent", measurement: "allocated-blocks", hardQuota: false, automaticEviction: false });
  const over = refusal(() => requireBuildAdmission(root, { phase: "before-copy", reserveBytes: reserveBytes + 1 }));
  expect(over).toMatchObject({ code: "resource_budget_exceeded", exitCode: 75 });
  expect(over.observation.failureCodes).toEqual(["resource_budget_exceeded", "resource_free_space_below_floor"]);
});

test("free-space floor includes reserve and refuses unavailable or invalid observations", () => {
  const root = fixture(); process.env.SCRIPT_KIT_AGENT_MIN_FREE_GB = "1";
  const stat = spyOn(fs, "statfsSync").mockReturnValue(volume(GiB)); restore.push(() => stat.mockRestore());
  expect(requireBuildAdmission(root).withinLimits).toBe(true);
  expect(refusal(() => requireBuildAdmission(root, { reserveBytes: 1 })).code).toBe("resource_free_space_below_floor");
  stat.mockImplementation(() => { throw new Error("EIO"); });
  expect(refusal(() => requireBuildAdmission(root)).code).toBe("resource_observation_incomplete");
  stat.mockReturnValue(volume(Number.NaN));
  expect(refusal(() => requireBuildAdmission(root)).observation.availableBytes).toBeNull();
});

for (const interactive of ["0", "1"]) test(`managed low-disk bypass is a policy conflict with NONINTERACTIVE=${interactive}`, () => {
  const root = fixture(); observedVolume(2 * GiB);
  process.env.SCRIPT_KIT_NONINTERACTIVE = interactive; process.env.SCRIPT_KIT_AGENT_ALLOW_LOW_DISK = "1";
  expect(refusal(() => requireBuildAdmission(root)).code).toBe("resource_policy_conflict");
});

test("invalid limits and reserves fail with a typed policy refusal", () => {
  const root = fixture(); observedVolume(2 * GiB);
  for (const reserveBytes of [-1, Number.NaN, Number.POSITIVE_INFINITY, 0.5]) expect(refusal(() => requireBuildAdmission(root, { reserveBytes })).code).toBe("resource_policy_conflict");
  for (const value of ["bad", "-1", String(Number.MAX_SAFE_INTEGER)]) {
    process.env.SCRIPT_KIT_AGENT_TARGET_BUDGET_GB = value;
    expect(refusal(() => requireBuildAdmission(root)).code).toBe("resource_policy_conflict");
  }
});

// Only module-boundary filesystem/worker/timer substitutes; no production test mode or real-disk growth.
class FixtureWorker extends EventEmitter {
  readonly threadId = 71;
  readonly requests: number[] = [];
  terminated = 0;
  unreferenced = false;
  termination: Promise<number> = Promise.resolve(0);
  postMessage(id: number): void { this.requests.push(id); }
  terminate(): Promise<number> { this.terminated++; return this.termination; }
  unref(): this { this.unreferenced = true; return this; }
  answer(value: AllocationObservation): void { this.emit("message", { id: this.requests.at(-1), allocation: value }); }
}
async function microtasks(): Promise<void> { for (let i = 0; i < 8; i++) await Promise.resolve(); }
function guardFixture(root = fixture()) {
  const worker = new FixtureWorker(), refusals: BuildResourceError[] = [];
  let now = 0, next = 0;
  const timers = new Map<number, { at: number; callback: () => void }>();
  const timer = spyOn(globalThis, "setTimeout").mockImplementation(((callback: () => void, ms = 0) => {
    const id = ++next; timers.set(id, { at: now + ms, callback }); return id;
  }) as unknown as typeof setTimeout);
  const clear = spyOn(globalThis, "clearTimeout").mockImplementation(id => { timers.delete(Number(id)); });
  const clock = spyOn(performance, "now").mockImplementation(() => now);
  const constructor = spyOn(workerMockBoundary, "Worker").mockImplementation(function () { return worker as unknown as workerThreads.Worker; });
  const free = spyOn(numberStatfsBoundary, "statfs").mockResolvedValue(volume(2 * GiB));
  restore.push(() => timer.mockRestore(), () => clear.mockRestore(), () => clock.mockRestore(), () => constructor.mockRestore(), () => free.mockRestore());
  const advance = async (ms: number): Promise<void> => {
    const end = now + ms;
    await microtasks();
    for (let i = 0; i < 1_000; i++) {
      const next = [...timers].filter(([, timer]) => timer.at <= end).sort((a, b) => a[1].at - b[1].at)[0];
      if (!next) { now = end; await microtasks(); return; }
      now = next[1].at; timers.delete(next[0]); next[1].callback(); await microtasks();
    }
    throw new Error("unbounded fixture timers");
  };
  return { root, worker, refusals, timers, free, advance, start: (initial?: BuildResourceObservation) => startBuildResourceGuard(root, error => refusals.push(error), initial) };
}

test("an uncapped guard refuses when its pinned external route becomes local storage", async () => {
  const f = externalFixture(8192), g = guardFixture(f.root); observedVolume(2 * GiB);
  const guard = g.start(requireBuildAdmission(f.root));
  try {
    g.worker.answer(sample(8192)); await microtasks();
    expect(g.refusals).toEqual([]);
    fs.unlinkSync(join(f.root, "target-agent"));
    fs.mkdirSync(join(f.root, "target-agent"));
    await g.advance(1_000);
    expect(g.refusals.map(error => error.code)).toEqual(["resource_policy_conflict"]);
    expect(await guard.stop()).toMatchObject({ complete: false, workerClosed: true });
  } finally { await guard.stop(); }
});

test("guard samples bounded summaries, admits one scan in flight, and closes timers and worker once", async () => {
  const f = guardFixture(), guard = f.start();
  await microtasks(); expect(f.worker.requests).toHaveLength(1);
  await f.advance(5_000); expect(f.worker.requests).toHaveLength(1);
  f.worker.answer(sample(8192)); await f.advance(5_000);
  expect(f.worker.requests).toHaveLength(2); f.worker.answer(sample(16_384));
  const stopping = guard.stop(); expect(guard.stop()).toBe(stopping);
  const summary = await stopping;
  expect(summary).toMatchObject({ complete: true, workerClosed: true, trigger: null, maximumSampledAllocatedBytes: 16_384, minimumSampledAvailableBytes: 2 * GiB });
  expect(summary.sampleCount).toBeGreaterThan(2); expect(f.refusals).toEqual([]);
  expect(f.worker.terminated).toBe(1); expect(f.timers.size).toBe(0);
  const count = summary.sampleCount; await f.advance(60_000);
  expect(summary.sampleCount).toBe(count); expect(f.worker.requests).toHaveLength(2);
});

for (const breach of ["allocation", "free", "incomplete"] as const) test(`guard requests exact one-shot cancellation for ${breach} refusal`, async () => {
  const f = guardFixture();
  if (breach === "free") { process.env.SCRIPT_KIT_AGENT_MIN_FREE_GB = "1"; f.free.mockResolvedValue(volume(GiB - 1)); }
  const guard = f.start();
  f.worker.answer(breach === "allocation" ? sample(GiB + 1) : breach === "incomplete" ? sample(0, ["EACCES"]) : sample(8192));
  await microtasks();
  const code = breach === "allocation" ? "resource_budget_exceeded" : breach === "free" ? "resource_free_space_below_floor" : "resource_observation_incomplete";
  expect(f.refusals).toHaveLength(1); expect(f.refusals[0]).toBeInstanceOf(BuildResourceError);
  expect(f.refusals[0]).toMatchObject({ code, exitCode: 75, observation: { phase: "in-build", withinLimits: false, hardQuota: false, automaticEviction: false } });
  if (breach === "incomplete") expect(f.refusals[0]!.observation.errors).toEqual(["EACCES"]);
  const summary = await guard.stop(); expect(summary.trigger).toBe(f.refusals[0]!.observation);
  expect(summary).toMatchObject({ complete: false, workerClosed: true });
  await f.advance(60_000); expect(f.refusals).toHaveLength(1); expect(f.timers.size).toBe(0); expect(f.worker.terminated).toBe(1);
});

test("one transient allocation race retries without masking persistent incompleteness", async () => {
  const f = guardFixture(), guard = f.start();
  f.worker.answer(sample(0, ["EAGAIN"])); await microtasks();
  expect(f.refusals).toEqual([]); await f.advance(250); expect(f.worker.requests).toHaveLength(2);
  f.worker.answer(sample(4096)); await f.advance(10_000); expect(f.worker.requests).toHaveLength(3);
  f.worker.answer(sample(0, ["ESTALE"])); await f.advance(250); expect(f.worker.requests).toHaveLength(4);
  f.worker.answer(sample(0, ["ESTALE"])); await microtasks();
  expect(f.refusals.map(error => error.code)).toEqual(["resource_observation_incomplete"]);
  expect(await guard.stop()).toMatchObject({ transientFailureCount: 2, complete: false, workerClosed: true });
  expect(f.timers.size).toBe(0);
});

for (const error of ["ENOENT", "Error: inventory_directory_changed", "Error: inventory_unstable_directory: /ESTALE/object"]) test(`guard does not retry a refused directory: ${error}`, async () => {
  const f = guardFixture(), guard = f.start(); f.worker.answer(sample(4096, [error]));
  await f.advance(250);
  expect(f.worker.requests).toHaveLength(1); expect(f.refusals).toHaveLength(1);
  expect((await guard.stop()).trigger?.errors).toEqual([error]);
});

test("worker refuses an over-cap partial observation even when replay completes smaller", async () => {
  const f = guardFixture(), guard = f.start();
  f.worker.answer({ ...sample(0), priorAllocatedBytes: GiB + 1 }); await microtasks();
  const trigger = (await guard.stop()).trigger;
  expect(trigger?.targetAgentAllocatedBytes).toBe(GiB + 1);
  expect(trigger?.failureCodes[0]).toBe("resource_budget_exceeded");
});

test("a transient-looking partial scan already above budget records an immediate terminal trigger", async () => {
  const f = guardFixture(), guard = f.start(); f.worker.answer(sample(GiB + 1, ["ESTALE"]));
  const summary = await guard.stop();
  expect(summary.trigger).toBe(f.refusals[0]!.observation);
  expect(summary.trigger?.failureCodes).toEqual(["resource_budget_exceeded", "resource_observation_incomplete"]);
  expect(summary).toMatchObject({ workerClosed: true, complete: false, transientFailureCount: 0 });
  expect(f.worker.requests).toHaveLength(1);
});

for (const outcome of ["complete", "budget", "floor", "incomplete", "unclosed"] as const) test(`stop before transient retry requires a fresh pair and worker closure: ${outcome}`, async () => {
  const f = guardFixture(); observedVolume(2 * GiB); process.env.SCRIPT_KIT_AGENT_MIN_FREE_GB = "1";
  const initial = requireBuildAdmission(f.root), oldFree = Promise.withResolvers<fs.StatsFs>();
  f.free.mockImplementationOnce(() => oldFree.promise);
  const guard = f.start(initial);
  f.worker.answer(sample(0, ["ESTALE"]));
  const stopping = guard.stop(); expect(guard.stop()).toBe(stopping);
  expect(f.worker.requests).toHaveLength(2); expect(f.free).toHaveBeenCalledTimes(1);
  if (outcome === "unclosed") f.worker.termination = Promise.withResolvers<number>().promise;
  f.worker.answer(outcome === "incomplete" ? sample(0, ["ESTALE"]) : sample(outcome === "budget" ? GiB + 1 : 8192));
  if (outcome === "complete" || outcome === "floor" || outcome === "unclosed") {
    expect(f.worker.terminated).toBe(0); // A fresh allocation and old free-space claim are not enough.
    f.free.mockResolvedValue(volume(outcome === "floor" ? GiB - 1 : GiB));
    oldFree.resolve(volume(2 * GiB)); await microtasks();
    expect(f.free).toHaveBeenCalledTimes(2);
  }
  await f.advance(1_000);
  const summary = await stopping;
  expect(summary).toMatchObject({ complete: outcome === "complete", workerClosed: outcome !== "unclosed", transientFailureCount: 1 });
  if (outcome === "complete") expect(summary).toMatchObject({ trigger: null, maximumSampledAllocatedBytes: 8192, minimumSampledAvailableBytes: GiB });
  if (["budget", "floor", "incomplete"].includes(outcome)) {
    expect(summary.trigger).toBe(f.refusals[0]!.observation);
    expect(f.refusals.map(e => e.code)).toEqual([outcome === "budget" ? "resource_budget_exceeded" : outcome === "floor" ? "resource_free_space_below_floor" : "resource_observation_incomplete"]);
  }
  expect(f.timers.size).toBe(0); expect(f.worker.terminated).toBe(1);
});

test("stop after transient free failure cannot accept an allocation scan started before stop", async () => {
  const f = guardFixture(); f.free.mockRejectedValueOnce(new Error("EBUSY"));
  const guard = f.start(); await microtasks();
  const stopping = guard.stop(); expect(f.worker.requests).toHaveLength(1);
  const deadline = Number(process.env.SCRIPT_KIT_AGENT_INVENTORY_TIMEOUT_MS ?? 8_000) + 1_000;
  await f.advance(deadline - 30); f.worker.answer(sample(4096)); await microtasks();
  expect(f.worker.requests).toHaveLength(2); expect(f.worker.terminated).toBe(0);
  // Waiting for the old request used most of the final pair's deadline.
  await f.advance(29); expect(f.worker.terminated).toBe(0); await f.advance(1);
  expect((await stopping).trigger?.errors).toEqual(["inventory_time_limit"]);
  expect(f.worker.terminated).toBe(1); expect(f.timers.size).toBe(0);
});

test("free observation retries are bounded and do not retain stale available-space claims", async () => {
  const f = guardFixture(); f.free.mockRejectedValue(Object.assign(new Error("ESTALE"), { code: "ESTALE" }));
  const guard = f.start(); f.worker.answer(sample(4096)); await microtasks();
  expect(f.refusals).toEqual([]); await f.advance(250);
  expect(f.refusals).toHaveLength(1); expect(f.refusals[0]!.observation.availableBytes).toBeNull();
  expect(await guard.stop()).toMatchObject({ transientFailureCount: 1, complete: false, workerClosed: true });
  expect(f.free).toHaveBeenCalledTimes(2); expect(f.timers.size).toBe(0);
});

test("a hung scan is cancelled on deadline, never overlapped by another allocation scan", async () => {
  const f = guardFixture(), guard = f.start();
  const deadline = Number(process.env.SCRIPT_KIT_AGENT_INVENTORY_TIMEOUT_MS ?? 8_000) + 1_000;
  await f.advance(deadline - 1);
  expect(f.refusals).toEqual([]); expect(f.worker.requests).toHaveLength(1);
  await f.advance(1);
  expect(f.worker.requests).toHaveLength(1); expect(f.refusals.map(error => error.code)).toEqual(["resource_observation_incomplete"]);
  expect(await guard.stop()).toMatchObject({ complete: false, workerClosed: true }); expect(f.timers.size).toBe(0);
});

test("stop during in-flight observations is bounded and ignores late completions", async () => {
  const f = guardFixture(), free = Promise.withResolvers<fs.StatsFs>();
  f.free.mockImplementation(() => free.promise);
  const guard = f.start(), summary = await guard.stop();
  expect(summary).toMatchObject({ complete: false, workerClosed: true, sampleCount: 0 });
  free.resolve(volume(0)); f.worker.answer(sample(GiB + 1)); await microtasks(); await f.advance(60_000);
  expect(f.refusals).toEqual([]); expect(summary.sampleCount).toBe(0); expect(f.timers.size).toBe(0);
});

test("failed worker termination has bounded stop and cannot claim complete monitoring", async () => {
  const f = guardFixture(); f.worker.termination = Promise.withResolvers<number>().promise;
  const guard = f.start(); f.worker.answer(sample(4096)); await microtasks();
  let result: BuildResourceSummary | undefined;
  const stopping = guard.stop().then(value => { result = value; });
  await f.advance(999); expect(result).toBeUndefined(); await f.advance(1); await stopping;
  expect(result).toMatchObject({ complete: false, workerClosed: false, workerThreadId: f.worker.threadId });
  expect(f.worker.terminated).toBe(1); expect(f.worker.unreferenced).toBe(true); expect(f.timers.size).toBe(0);
});

test("guard policy refusal schedules no worker or filesystem sample", async () => {
  const f = guardFixture(); process.env.SCRIPT_KIT_AGENT_ALLOW_LOW_DISK = "1";
  const guard = f.start(); await microtasks();
  expect(f.refusals.map(error => error.code)).toEqual(["resource_policy_conflict"]);
  expect(f.worker.requests).toHaveLength(0); expect(f.free).not.toHaveBeenCalled();
  expect(await guard.stop()).toMatchObject({ complete: false, workerClosed: true }); expect(f.timers.size).toBe(0);
});

test("a stalled free-space query refuses on its deadline without launching a second query", async () => {
  const f = guardFixture(); f.free.mockImplementation(() => Promise.withResolvers<fs.StatsFs>().promise);
  const guard = f.start(); f.worker.answer(sample(4096)); await f.advance(2_000);
  expect(f.refusals.map(error => error.code)).toEqual(["resource_observation_incomplete"]);
  expect(f.free).toHaveBeenCalledTimes(1);
  expect(f.refusals[0]!.observation.errors).toContain("inventory_free_space_time_limit");
  expect(await guard.stop()).toMatchObject({ complete: false, workerClosed: true }); expect(f.timers.size).toBe(0);
});

test("a bounded free-space deadline accepts a delayed authoritative measurement", async () => {
  const f = guardFixture(), free = Promise.withResolvers<fs.StatsFs>();
  process.env.SCRIPT_KIT_AGENT_FREE_SAMPLE_TIMEOUT_MS = "10000";
  f.free.mockImplementation(() => free.promise);
  const guard = f.start(); f.worker.answer(sample(4096)); await f.advance(3_000);
  expect(f.refusals).toEqual([]);
  free.resolve(volume(2 * GiB)); await microtasks();
  expect(await guard.stop()).toMatchObject({ complete: true, minimumSampledAvailableBytes: 2 * GiB });
});

test("an extended free-space deadline still refuses a stalled query exactly once", async () => {
  const f = guardFixture();
  process.env.SCRIPT_KIT_AGENT_FREE_SAMPLE_TIMEOUT_MS = "10000";
  f.free.mockImplementation(() => Promise.withResolvers<fs.StatsFs>().promise);
  const guard = f.start(); f.worker.answer(sample(4096)); await f.advance(9_999);
  expect(f.refusals).toEqual([]); await f.advance(1);
  expect(f.refusals.map(error => error.code)).toEqual(["resource_observation_incomplete"]);
  expect(f.free).toHaveBeenCalledTimes(1);
  expect(await guard.stop()).toMatchObject({ complete: false, workerClosed: true });
});

for (const value of ["0", "-1", "disabled", "60001"]) test(`invalid free-space deadline ${value} starts no worker`, () => {
  const f = guardFixture(); process.env.SCRIPT_KIT_AGENT_FREE_SAMPLE_TIMEOUT_MS = value;
  expect(() => f.start()).toThrow("invalid_SCRIPT_KIT_AGENT_FREE_SAMPLE_TIMEOUT_MS");
  expect(f.worker.requests).toHaveLength(0); expect(f.free).not.toHaveBeenCalled();
});

test("the real request-owned worker scans a tiny fixture and closes its thread", async () => {
  const root = fixture(); file(root, "target-agent/unknown/payload"); file(root, `${compilerDeps}/live-directory/payload`);
  const expected = allocation(join(root, "target-agent")).allocatedBytes;
  const OriginalWorker = workerThreads.Worker, received = Promise.withResolvers<void>();
  const constructor = spyOn(workerMockBoundary, "Worker").mockImplementation(function (filename: string | URL, options?: workerThreads.WorkerOptions) {
    const worker = new OriginalWorker(filename, options);
    worker.once("message", () => received.resolve());
    worker.once("error", received.reject);
    return worker;
  });
  restore.push(() => constructor.mockRestore());
  const free = spyOn(numberStatfsBoundary, "statfs").mockResolvedValue(volume(2 * GiB)); restore.push(() => free.mockRestore());
  const errors: BuildResourceError[] = [], guard = startBuildResourceGuard(root, error => errors.push(error));
  const deadline = setTimeout(() => received.reject(new Error("fixture_worker_deadline")), 5_000);
  try {
    await received.promise;
    expect(await guard.stop()).toMatchObject({ complete: true, workerClosed: true, maximumSampledAllocatedBytes: expected, minimumSampledAvailableBytes: 2 * GiB });
    expect(errors).toEqual([]);
  } finally { clearTimeout(deadline); await guard.stop(); }
});

test("fast operations retain real preflight evidence while cancelling an unfinished first scan", async () => {
  const f = guardFixture(); file(f.root, "target-agent/unknown/payload"); observedVolume(2 * GiB);
  const initial = requireBuildAdmission(f.root, { phase: "preflight" });
  const free = Promise.withResolvers<fs.StatsFs>(); f.free.mockImplementation(() => free.promise);
  const guard = f.start(initial), summary = await guard.stop();
  expect(summary).toMatchObject({ complete: true, workerClosed: true, initialObservationIncluded: true, sampleCount: 1,
    maximumSampledAllocatedBytes: initial.targetAgentAllocatedBytes, minimumSampledAvailableBytes: initial.availableBytes });
  expect(f.worker.requests).toHaveLength(1); expect(f.worker.terminated).toBe(1);
  free.resolve(volume(0)); await microtasks();
  expect(summary.sampleCount).toBe(1); expect(f.refusals).toEqual([]); expect(f.timers.size).toBe(0);
});

for (const patch of [
  { complete: false }, { withinLimits: false }, { reserveBytes: 1 }, { targetAgentBudgetBytes: 2 * GiB },
  { minimumFreeBytes: GiB }, { targetAgentAllocatedBytes: GiB + 1 }, { targetAgentAllocatedBytes: -1 },
  { availableBytes: null }, { failureCodes: ["resource_observation_incomplete"] },
] satisfies Partial<BuildResourceObservation>[]) test(`guard rejects unproved or policy-mismatched seed ${JSON.stringify(patch)}`, async () => {
  const f = guardFixture(); observedVolume(2 * GiB);
  const initial = { ...requireBuildAdmission(f.root), ...patch };
  const guard = f.start(initial); await microtasks();
  expect(f.refusals.map(error => error.code)).toEqual(["resource_policy_conflict"]);
  expect(await guard.stop()).toMatchObject({ complete: false, initialObservationIncluded: false });
  expect(f.worker.requests).toHaveLength(0); expect(f.free).not.toHaveBeenCalled(); expect(f.timers.size).toBe(0);
});

test("preflight seed cannot conceal a later failed sample or failed worker closure", async () => {
  const f = guardFixture(); observedVolume(2 * GiB);
  const guard = f.start(requireBuildAdmission(f.root)); f.worker.termination = Promise.withResolvers<number>().promise;
  f.worker.answer(sample(0, ["EACCES"])); await f.advance(1_000);
  const summary = await guard.stop();
  expect(summary).toMatchObject({ complete: false, workerClosed: false, workerThreadId: f.worker.threadId, initialObservationIncluded: true });
  expect(f.refusals.map(error => error.code)).toEqual(["resource_observation_incomplete"]); expect(f.timers.size).toBe(0);
});
