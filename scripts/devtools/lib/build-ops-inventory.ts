import { accessSync, constants, existsSync, lstatSync, opendirSync, readFileSync, statfsSync } from "node:fs";
import type { Dirent, Stats } from "node:fs";
import * as fsPromises from "node:fs/promises";
import { homedir } from "node:os";
import { join, resolve } from "node:path";
import * as workerThreads from "node:worker_threads";
import { managedStorageRoot, trustedManagedRootLink } from "../../agentic/managed-storage.ts";

export interface AllocationObservation {
  present: boolean; complete: boolean; logicalBytes: number; allocatedBytes: number;
  entries: number; linksNotFollowed: number; errors: string[]; priorAllocatedBytes?: number;
}
export function allocation(path: string, excluded: readonly string[] = [], seen = new Set<string>()): AllocationObservation {
  return scanAllocation(path, excluded, seen);
}

const ALLOCATION_ENTRY_LIMIT = 250_000;
const ALLOCATION_TIMEOUT_MS = positiveLimit("SCRIPT_KIT_AGENT_INVENTORY_TIMEOUT_MS", 8_000, 1, 300_000);

const compilerRoute = new Map([
  ["", "pools"], ["pools", "agent-debug"], ["pools/agent-debug", "debug"],
  ["pools/agent-debug/debug", "deps"],
]);

function protectedOutputRoot(child: string): boolean {
  if (!child) return true;
  const parts = child.split("/");
  if (["pools", "agents"].includes(parts[0]!)) return parts.length <= 4;
  return ["shared", "artifacts", "runtime", ".locks"].includes(parts[0]!) && parts.length <= 2;
}

function scanAllocation(path: string, excluded: readonly string[], seen: Set<string>, expectedDevice?: number, protectOutputs = false): AllocationObservation {
  const result: AllocationObservation = { present: false, complete: true, logicalBytes: 0, allocatedBytes: 0, entries: 0, linksNotFollowed: 0, errors: [] };
  const started = performance.now();
  let device = expectedDevice, work = 0, priorAllocatedBytes = 0;
  const added: string[] = []; // Undo only identities this walk inserted after a directory checkpoint.
  // Start-of-enumeration witnesses for retained directories, in parent-first order.
  // A lost entry can have moved behind an earlier iterator in the same managed tree.
  const directories: Array<[string, number, number, number, number, number, number]> = [];
  const checkTime = () => { if (performance.now() - started > ALLOCATION_TIMEOUT_MS) throw new Error("inventory_time_limit"); };
  const charge = () => { if (++work > ALLOCATION_ENTRY_LIMIT) throw new Error("inventory_entry_limit"); checkTime(); };
  const verifyReplayFrontier = (): void => {
    for (const [dir, dev, ino, ctime, mtime, links, size] of directories) {
      charge();
      const now = lstatSync(dir);
      if (!now.isDirectory() || !Number.isSafeInteger(dev) || !Number.isSafeInteger(ino) || ino <= 0
        || !Number.isFinite(ctime) || !Number.isFinite(mtime) || now.dev !== dev || now.ino !== ino
        || now.ctimeMs !== ctime || now.mtimeMs !== mtime || now.nlink !== links || now.size !== size) {
        throw new Error(now.isDirectory() && now.dev === dev && now.ino === ino
          ? "inventory_unstable_replay_scope:membership" : "inventory_unstable_replay_scope:identity");
      }
    }
  };
  const account = (stat: Stats) => {
    result.entries++;
    const identity = `${stat.dev}:${stat.ino}`;
    if (!seen.has(identity)) {
      seen.add(identity); added.push(identity);
      const bytes = Number(stat.blocks) * 512;
      if (!Number.isSafeInteger(bytes) || bytes < 0 || !Number.isSafeInteger(result.allocatedBytes + bytes)) throw new Error("inventory_invalid_allocation");
      result.allocatedBytes += bytes;
      if (!stat.isDirectory()) result.logicalBytes += Number(stat.size);
    }
  };
  // Only the authoritative managed walk knows the exact compiler deps scope.
  const visit = (current: string, child: string, depth: number, listed?: Dirent, compilerDirectory = false, expected?: Stats): boolean => {
    if (excluded.some(value => child === value || child.startsWith(value + "/"))) return true;
    charge();
    if (depth > 256) throw new Error("inventory_depth_limit");
    const compilerDeps = protectOutputs && child === "pools/agent-debug/debug/deps";
    let stat: Stats;
    try { stat = lstatSync(current); }
    catch (error) {
      if ((error as NodeJS.ErrnoException).code === "ENOENT") {
        if (!child) return true; // Absent scan root, not a missing enumerated child.
        if (listed?.isFile() || compilerDirectory) return false;
      }
      throw error;
    }
    if (expected && (!stat.isDirectory() || stat.dev !== expected.dev || stat.ino !== expected.ino)) throw new Error("inventory_probe_identity");
    if (!child) result.present = true;
    device ??= Number(stat.dev);
    if (Number(stat.dev) !== device) throw new Error("inventory_unexpected_mount");
    if (compilerDirectory && !stat.isDirectory()) throw new Error("inventory_directory_changed");
    if (stat.isSymbolicLink()) {
      account(stat);
      result.linksNotFollowed++;
      if (protectOutputs && protectedOutputRoot(child)) throw new Error("inventory_output_symlink");
      return true;
    }
    if (protectOutputs && !child && !stat.isDirectory()) throw new Error("inventory_output_not_directory");
    if (!stat.isDirectory()) { account(stat); return true; }

    const { entries, logicalBytes, allocatedBytes, linksNotFollowed } = result, mark = added.length, dirMark = directories.length;
    const sameDirectory = (): Stats => {
      const now = lstatSync(current);
      if (!now.isDirectory() || now.dev !== stat.dev || now.ino !== stat.ino) throw new Error("inventory_directory_changed");
      return now;
    };
    let vanished: string | undefined, confirmAbsentDirectory = false;
    for (;;) {
      directories.push([current, stat.dev, stat.ino, stat.ctimeMs, stat.mtimeMs, stat.nlink, stat.size]);
      account(stat);
      // Descend the known compiler route first. Probe only a verified real parent,
      // never a compound speculative path. Remember every name the probe exposed:
      // a fresh enumeration must actually encounter those entries, not infer them.
      const preferred = protectOutputs ? compilerRoute.get(child) : undefined;
      const witnessed = preferred && !vanished ? new Map<string, Dirent>() : undefined;
      let first: Stats | undefined;
      if (preferred && witnessed) {
        const probe = opendirSync(current);
        try {
          let row: Dirent | null;
          while ((row = probe.readSync()) !== null) {
            charge();
            if (witnessed.has(row.name)) throw new Error("inventory_probe_duplicate");
            witnessed.set(row.name, row);
            if (row.name !== preferred) continue;
            if (!row.isDirectory()) throw new Error(row.isSymbolicLink() ? "inventory_output_symlink" : "inventory_output_not_directory");
            const next = join(current, preferred);
            charge(); sameDirectory();
            first = lstatSync(next);
            if (!first.isDirectory()) throw new Error(first.isSymbolicLink() ? "inventory_output_symlink" : "inventory_output_not_directory");
            if (first.dev !== device) throw new Error("inventory_unexpected_mount");
            if (!visit(next, child ? `${child}/${preferred}` : preferred, depth + 1, row, false, first)) throw new Error("inventory_probe_missing");
            break;
          }
        } finally { probe.closeSync(); }
        sameDirectory();
      }
      const directory = opendirSync(current);
      let entry: Dirent | null;
      try {
        while ((entry = directory.readSync()) !== null) {
          const probed = witnessed?.get(entry.name);
          if (probed) {
            if (probed.isDirectory() !== entry.isDirectory() || probed.isFile() !== entry.isFile() || probed.isSymbolicLink() !== entry.isSymbolicLink()) throw new Error("inventory_probe_type_changed");
            witnessed!.delete(entry.name);
          }
          if (first && entry.name === preferred) {
            if (!probed) throw new Error("inventory_probe_duplicate");
            charge();
            const now = lstatSync(join(current, entry.name));
            if (!now.isDirectory() || now.dev !== first.dev || now.ino !== first.ino) throw new Error("inventory_probe_identity");
            continue;
          }
          // We have no inode witness for that name. Do not accept a replacement (even a link).
          if (entry.name === vanished) throw new Error("inventory_unstable_directory:reappeared");
          if (!visit(join(current, entry.name), child ? `${child}/${entry.name}` : entry.name, depth + 1, entry, compilerDeps && entry.isDirectory())) break;
        }
        if (witnessed?.size) throw new Error("inventory_probe_missing");
        if (!entry && vanished !== undefined) {
          if (confirmAbsentDirectory) {
            charge();
            try { lstatSync(join(current, vanished)); throw new Error("inventory_unstable_directory:reappeared"); }
            catch (error) { if ((error as NodeJS.ErrnoException).code !== "ENOENT") throw error; }
          }
          // Both the replayed subtree and already-enumerated destinations must
          // still have their witnessed memberships. This rereads metadata, not
          // directory listings, and consumes the original scan's work/deadline.
          verifyReplayFrontier();
        }
        try { sameDirectory(); } catch (error) {
          // Only a first, empty enumeration reached EOF without unresolved child work.
          if (compilerDirectory && !entry && !vanished && result.entries === entries + 1 && (error as NodeJS.ErrnoException).code === "ENOENT") return false;
          throw error;
        }
        checkTime();
        if (!entry) return true; // Only an enumeration that reached EOF can commit.
        if (vanished !== undefined) throw new Error("inventory_unstable_directory:repeated_loss");
        vanished = entry.name;
        confirmAbsentDirectory = compilerDeps && entry.isDirectory();
      } finally { directory.closeSync(); }
      priorAllocatedBytes = Math.max(priorAllocatedBytes, result.allocatedBytes);
      Object.assign(result, { entries, logicalBytes, allocatedBytes, linksNotFollowed });
      while (added.length > mark) seen.delete(added.pop()!);
      directories.length = dirMark;
      charge(); // Rolled-back visits and this reopen still cost work against the original deadline.
      stat = sameDirectory(); // Revalidate after rollback and re-account the directory's own blocks.
    }
  };
  try { visit(path, "", 0); }
  catch (error) { result.complete = false; result.errors.push(String(error)); }
  if (priorAllocatedBytes > result.allocatedBytes) result.priorAllocatedBytes = priorAllocatedBytes;
  return result;
}

function targetAgentAllocation(root: string): AllocationObservation {
  try {
    const stat = lstatSync(root);
    if (stat.isSymbolicLink() || !stat.isDirectory()) throw new Error("inventory_repository_not_directory");
    let storage: string;
    try { storage = managedStorageRoot(root); }
    catch (error) { throw new Error(`inventory_output_symlink:${String(error)}`); }
    const path = storage === root ? join(root, "target-agent") : storage;
    const linked = existsSync(join(root, "target-agent")) && lstatSync(join(root, "target-agent")).isSymbolicLink();
    const expectedDevice = linked ? Number(lstatSync(storage).dev) : Number(stat.dev);
    const observation = scanAllocation(path, [], new Set(), expectedDevice, true);
    if (managedStorageRoot(root) !== storage) throw new Error("inventory_storage_route_changed");
    return observation;
  } catch (error) {
    return { present: false, complete: false, logicalBytes: 0, allocatedBytes: 0, entries: 0, linksNotFollowed: 0, errors: [String(error)] };
  }
}
function positiveLimit(name: string, fallback: number, minimum = 0, maximum = Number.MAX_SAFE_INTEGER): number {
  const value = process.env[name];
  if (value === undefined) return fallback;
  const parsed = Number(value);
  if (!/^\d+$/.test(value) || !Number.isSafeInteger(parsed) || parsed < minimum || parsed > maximum) throw new Error(`invalid_${name}`);
  return parsed;
}
function byteLimit(name: string, fallback: number): number {
  const bytes = positiveLimit(name, fallback) * 1024 ** 3;
  if (!Number.isSafeInteger(bytes)) throw new Error(`invalid_${name}`);
  return bytes;
}
function targetAgentBudget(root: string): number | null {
  if (process.env.SCRIPT_KIT_AGENT_TARGET_BUDGET_GB !== undefined) return byteLimit("SCRIPT_KIT_AGENT_TARGET_BUDGET_GB", 40);
  const path = join(resolve(root), "target-agent");
  let stat: Stats | undefined;
  try { stat = lstatSync(path); }
  catch (error) { if ((error as NodeJS.ErrnoException).code !== "ENOENT") throw error; }
  if (stat?.isSymbolicLink()) {
    const route = trustedManagedRootLink(path);
    // Only an identity-pinned root on another volume is uncapped.
    if (route.destinationDevice !== route.linkDevice) return null;
  }
  return 40 * 1024 ** 3;
}

export function buildLimits(root: string) {
  return {
    targetAgentBudgetBytes: targetAgentBudget(root),
    minimumFreeBytes: byteLimit("SCRIPT_KIT_AGENT_MIN_FREE_GB", 25),
    freeSampleTimeoutMs: positiveLimit("SCRIPT_KIT_AGENT_FREE_SAMPLE_TIMEOUT_MS", 2_000, 1, 60_000),
    compilerWorkers: 2, nativeWorkersWithinEachCompiler: 1, testWorkers: 2,
    logBytesPerTask: 4 * 1024 ** 2,
    budgetScope: "all target-agent entries; dev targets, external caches and evidence outside target-agent reported separately",
    hardQuota: false as const, automaticEviction: false as const,
    deletion: "only exact finalized managed task/artifact candidates; never automatic cache eviction",
  };
}
export function buildStorage(root: string, external = true, budgetOnly = false) {
  // Budget authority must never share diagnostic inode state: a link in target/ cannot hide budgeted bytes.
  const targetAgent = targetAgentAllocation(root);
  let storageRoot = root;
  try { storageRoot = managedStorageRoot(root); } catch { /* targetAgent carries the refusal. */ }
  const seen = new Set<string>(), poolSeen = new Set<string>();
  const categories = [
    ["dev-target", "target", ["goose-sidecar"], false],
    ["agent-pools", "target-agent/pools", [], true],
    ["legacy-exclusive-targets", "target-agent/agents", [], true],
    ["shared-caches", "target-agent/shared", [], false],
    ["immutable-artifacts", "target-agent/artifacts", [], false],
    ["runtime", "target-agent/runtime", [], false],
    ["evidence-and-tasks", ".test-output", [], false],
    ["legacy-evidence", ".artifacts", [], false],
    ["hidden-screenshots", ".test-screenshots", [], false],
    ["screenshots", "test-screenshots", [], false],
    ["leases", "target-agent/.locks", [], false],
    ["other-agent-storage", "target-agent", ["pools", "agents", "shared", "artifacts", "runtime", ".locks"], false],
  ] as const;
  const observations = categories.filter(([, child]) => !budgetOnly || child.startsWith("target-agent")).map(([category, child, exclude, poolBudgeted]) => ({ category, path: child, poolBudgeted,
    deletionScope: ["immutable-artifacts", "evidence-and-tasks"].includes(category) ? "finalized-managed-records-only" : "none",
    ...(!targetAgent.complete && child.startsWith("target-agent")
      ? { present: targetAgent.present, complete: false, logicalBytes: 0, allocatedBytes: 0, entries: 0, linksNotFollowed: 0, errors: [...targetAgent.errors] }
      : allocation(child.startsWith("target-agent") && storageRoot !== root ? join(storageRoot, child.slice("target-agent".length)) : join(root, child), exclude, poolBudgeted ? poolSeen : seen)) }));
  const cargoHome = process.env.CARGO_HOME || join(homedir(), ".cargo");
  const kitHome = process.env.SK_PATH || join(homedir(), ".scriptkit");
  const candidates = [["cargo-registry", join(cargoHome, "registry")], ["cargo-git", join(cargoHome, "git")],
    ["toolchains", process.env.RUSTUP_HOME || join(homedir(), ".rustup")], ["goose-sidecar", join(root, "target/goose-sidecar")],
    ["models-candidate", join(kitHome, "models")], ["tools-candidate", join(kitHome, "bin")]];
  if (process.env.SCCACHE_DIR) candidates.push(["configured-sccache", resolve(process.env.SCCACHE_DIR)]);
  const externalObservations = external ? candidates.map(([category, path]) => ({ category, path, deletionScope: "never", ...allocation(path!) })) : [];
  let volume: { complete: boolean; availableBytes: number | null; reason?: string };
  try {
    const availableBytes = managedAvailableBytes(root);
    if (!Number.isSafeInteger(availableBytes) || availableBytes < 0) throw new Error("inventory_invalid_available");
    volume = { complete: true, availableBytes };
  }
  catch (error) { volume = { complete: false, availableBytes: null, reason: String(error) }; }
  return { categories: observations, external: externalObservations, volume,
    targetAgentAllocatedBytes: targetAgent.allocatedBytes, targetAgentComplete: targetAgent.complete, targetAgentErrors: targetAgent.errors,
    agentPoolAllocatedBytes: observations.filter(value => value.poolBudgeted).reduce((sum, value) => sum + value.allocatedBytes, 0),
    uniquePhysicalBytes: null, reclaimablePhysicalBytes: null, hardQuota: false as const, automaticEviction: false as const,
    measurement: "allocated blocks; independent target-agent budget and pool subtotal inode sets; only a pinned root link is followed; APFS clone sharing unknown; diagnostic and external totals not additive" };
}

function managedAvailableBytes(root: string): number {
  const storage = managedStorageRoot(root);
  const bytes = managedVolumePaths(root, storage).map(path => { const value = statfsSync(path); return value.bavail * value.bsize; });
  if (bytes.some(value => !Number.isSafeInteger(value) || value < 0)) throw new Error("inventory_invalid_available");
  if (managedStorageRoot(root) !== storage) throw new Error("inventory_storage_route_changed");
  return Math.min(...bytes);
}
function managedVolumePaths(root: string, storage: string): string[] {
  // Ordinary managed directories share the repository volume by contract.
  return storage !== root && lstatSync(join(root, "target-agent")).isSymbolicLink() ? [root, storage] : [root];
}
function executable(path: string | null): boolean {
  if (!path) return false;
  try { accessSync(path, constants.X_OK); return true; } catch { return false; }
}
export function buildDependencies(root: string) {
  const kitHome = process.env.SK_PATH || join(homedir(), ".scriptkit");
  const explicit = process.env.SCRIPT_KIT_GOOSE_BINARY;
  const candidates = explicit ? [explicit.replace(/^~(?=\/|$)/, homedir())] : [join(root, "target/goose-sidecar/goose")];
  const lock = existsSync(join(root, "Cargo.lock")) ? Bun.TOML.parse(readFileSync(join(root, "Cargo.lock"), "utf8")) as { package?: Array<{ name: string; version: string }> } : {};
  return {
    tools: ["bun", "python3", "git", "rustup", "cargo-watch", "sccache", "clang", "cmake", "xcrun", "cargo-bundle"].map(name => {
      const path = Bun.which(name); return { name, path, executablePresent: executable(path), health: "not-executed", version: null };
    }),
    goose: { candidates: candidates.map(path => ({ path, executablePresent: executable(path) })), health: "not-probed", owner: "scripts/prepare-goose-sidecar.sh" },
    models: [join(kitHome, "models/whisper-medium-Q5_K_M.gguf"), join(kitHome, "models/parakeet-tdt-0.6b-v3-Q8_0.gguf")].map(path => ({ path, present: existsSync(path), validity: "not-loaded" })),
    lockedLibraries: (lock.package ?? []).filter(value => ["transcribe-cpp", "transcribe-cpp-sys", "llama-cpp-sys-2"].includes(value.name)).map(({ name, version }) => ({ name, version })),
    metal: { owner: "vendor/gpui_macos/build.rs", requiredTools: ["xcrun metal", "xcrun metallib"], nativeBuild: "not-executed" },
    nativeRuntimeProof: "not-evaluated", installsTools: false, fetchesModels: false, startsSidecars: false,
  };
}
export interface BuildResourceObservation {
  phase: string;
  complete: boolean;
  targetAgentAllocatedBytes: number;
  availableBytes: number | null;
  reserveBytes: number;
  targetAgentBudgetBytes: number | null;
  minimumFreeBytes: number;
  withinLimits: boolean;
  failureCodes: string[];
  errors?: string[];
  scope: "target-agent";
  measurement: "allocated-blocks";
  hardQuota: false;
  automaticEviction: false;
}

export interface BuildResourceSummary {
  sampleCount: number;
  maximumSampledAllocatedBytes: number | null;
  minimumSampledAvailableBytes: number | null;
  complete: boolean;
  trigger: BuildResourceObservation | null;
  transientFailureCount?: number;
  workerClosed?: boolean;
  workerThreadId?: number;
  callbackFailed?: boolean;
  initialObservationIncluded?: boolean;
}

export interface BuildResourceReport {
  scope: "target-agent";
  measurement: "allocated-blocks";
  hardQuota: false;
  automaticEviction: false;
  checks: BuildResourceObservation[];
  monitoring: BuildResourceSummary | null;
  refusal: BuildResourceObservation | null;
}

export class BuildResourceError extends Error {
  readonly exitCode = 75;
  constructor(readonly code: string, readonly observation: BuildResourceObservation) {
    super(`${code}: ${observation.phase}`);
    this.name = "BuildResourceError";
  }
}

interface ResourcePolicy {
  targetAgentBudgetBytes: number | null;
  minimumFreeBytes: number;
  conflict: boolean;
}
function resourcePolicy(root: string): ResourcePolicy {
  try { return { ...buildLimits(root), conflict: process.env.SCRIPT_KIT_AGENT_ALLOW_LOW_DISK === "1" }; }
  catch { return { targetAgentBudgetBytes: 0, minimumFreeBytes: 0, conflict: true }; }
}

function resourceObservation(policy: ResourcePolicy, allocated: (Pick<AllocationObservation, "complete" | "allocatedBytes"> & Partial<Pick<AllocationObservation, "errors" | "priorAllocatedBytes">>) | null, available: number | null, phase: string, reserveBytes = 0): BuildResourceObservation {
  const validReserve = Number.isSafeInteger(reserveBytes) && reserveBytes >= 0;
  const validAvailable = available !== null && Number.isSafeInteger(available) && available >= 0;
  const failureCodes: string[] = [];
  if (policy.conflict || !validReserve) failureCodes.push("resource_policy_conflict");
  const reserve = validReserve ? reserveBytes : 0;
  const observedBytes = Math.max(allocated?.allocatedBytes ?? 0, allocated?.priorAllocatedBytes ?? 0);
  if (policy.targetAgentBudgetBytes !== null && allocated && observedBytes > policy.targetAgentBudgetBytes - reserve) failureCodes.push("resource_budget_exceeded");
  if (validAvailable && available - reserve < policy.minimumFreeBytes) failureCodes.push("resource_free_space_below_floor");
  const complete = allocated?.complete === true && validAvailable;
  if (!complete) failureCodes.push("resource_observation_incomplete");
  return { phase, complete, targetAgentAllocatedBytes: failureCodes.includes("resource_budget_exceeded") ? observedBytes : allocated?.allocatedBytes ?? 0,
    availableBytes: validAvailable ? available : null, reserveBytes: reserve,
    targetAgentBudgetBytes: policy.targetAgentBudgetBytes, minimumFreeBytes: policy.minimumFreeBytes,
    withinLimits: failureCodes.length === 0, failureCodes, errors: allocated?.errors,
    scope: "target-agent", measurement: "allocated-blocks", hardQuota: false, automaticEviction: false };
}

export function requireBuildAdmission(root: string, options: { phase?: string; reserveBytes?: number } = {}): BuildResourceObservation {
  const policy = resourcePolicy(root), allocated = targetAgentAllocation(root);
  let available: number | null = null;
  try { available = managedAvailableBytes(root); } catch { /* Incomplete observations refuse below. */ }
  const observation = resourceObservation(policy, allocated, available, options.phase ?? "preflight", options.reserveBytes ?? 0);
  if (!observation.withinLimits) throw new BuildResourceError(observation.failureCodes[0]!, observation);
  return observation;
}

const ALLOCATION_WORKER = "build-resource-allocation";
// The request owns this worker and its only protocol. No process-wide watcher or persisted sample log.
if (!workerThreads.isMainThread && workerThreads.workerData?.role === ALLOCATION_WORKER) {
  workerThreads.parentPort!.on("message", (id: number) => {
    workerThreads.parentPort!.postMessage({ id, allocation: targetAgentAllocation(workerThreads.workerData.root) });
  });
}

export function startBuildResourceGuard(root: string, onRefusal: (error: BuildResourceError) => void, initialObservation?: BuildResourceObservation): { stop(): Promise<BuildResourceSummary> } {
  const freeSampleTimeoutMs = positiveLimit("SCRIPT_KIT_AGENT_FREE_SAMPLE_TIMEOUT_MS", 2_000, 1, 60_000);
  const policy = resourcePolicy(root);
  const validInitial = initialObservation != null && initialObservation.complete === true && initialObservation.withinLimits === true
    && initialObservation.reserveBytes === 0 && initialObservation.targetAgentBudgetBytes === policy.targetAgentBudgetBytes
    && initialObservation.minimumFreeBytes === policy.minimumFreeBytes && initialObservation.scope === "target-agent"
    && initialObservation.measurement === "allocated-blocks" && initialObservation.hardQuota === false && initialObservation.automaticEviction === false
    && Array.isArray(initialObservation.failureCodes) && initialObservation.failureCodes.length === 0
    && Number.isSafeInteger(initialObservation.targetAgentAllocatedBytes) && initialObservation.targetAgentAllocatedBytes >= 0
    && resourceObservation(policy, { complete: true, allocatedBytes: initialObservation.targetAgentAllocatedBytes }, initialObservation.availableBytes, initialObservation.phase).withinLimits;
  if (initialObservation !== undefined && !validInitial) policy.conflict = true;
  // This is the caller's exact lease-owned preflight, not a synthetic successful worker sample.
  const initial = validInitial ? { complete: true, allocatedBytes: initialObservation!.targetAgentAllocatedBytes } : null;
  const summary: BuildResourceSummary = { sampleCount: initial ? 1 : 0, maximumSampledAllocatedBytes: initial?.allocatedBytes ?? null,
    minimumSampledAvailableBytes: initial ? initialObservation!.availableBytes : null,
    complete: true, trigger: null, transientFailureCount: 0, workerClosed: true, callbackFailed: false, initialObservationIncluded: initial !== null };
  const timers = new Set<NodeJS.Timeout>();
  let stopped = false, stopPromise: Promise<BuildResourceSummary> | undefined;
  let worker: workerThreads.Worker | undefined, workerExited = false;
  let allocated: AllocationObservation | null = null, available: number | null = initial ? initialObservation!.availableBytes : null;
  let availableError: string | undefined;
  let scanId = 0, pendingScan = 0, scanStarted = 0, scanFailures = 0, freeFailures = 0;
  let scanDeadline: NodeJS.Timeout | undefined, pendingFree = false;
  let finalPair: { started: number; nextScanId: number; allocation: boolean; free: boolean; done: PromiseWithResolvers<BuildResourceSummary> } | undefined;
  // Waiting for an older, still-live query consumes the closing pair's time, not a second deadline.
  const remaining = (ms: number) => finalPair ? Math.max(0, ms - (performance.now() - finalPair.started)) : ms;

  const cancel = (timer: NodeJS.Timeout | undefined): void => {
    if (timer !== undefined) { clearTimeout(timer); timers.delete(timer); }
  };
  const later = (fn: () => void, ms: number): NodeJS.Timeout => {
    const timer = setTimeout(() => { timers.delete(timer); if (!stopped) fn(); }, ms);
    timers.add(timer); return timer;
  };
  const close = (): Promise<BuildResourceSummary> => {
    if (stopPromise) return stopPromise;
    stopped = true;
    for (const timer of timers) clearTimeout(timer);
    timers.clear();
    if (!(allocated ?? initial)?.complete || available === null) summary.complete = false;
    const owned = worker;
    stopPromise = (async () => {
      if (owned && !workerExited) {
        const timedOut = Promise.withResolvers<false>();
        const deadline = setTimeout(() => timedOut.resolve(false), 1_000);
        try {
          summary.workerClosed = await Promise.race([
            Promise.resolve(owned.terminate()).then(() => true, () => false),
            timedOut.promise,
          ]);
        } catch { summary.workerClosed = false; }
        finally { clearTimeout(deadline); }
      }
      if (owned) {
        owned.removeAllListeners();
        // A failed termination is reported, never represented as observed closure.
        if (!summary.workerClosed) { owned.on("error", () => {}); owned.unref(); }
      }
      if (!summary.workerClosed) summary.complete = false;
      return summary;
    })();
    return stopPromise;
  };
  const refuse = (observation: BuildResourceObservation): void => {
    if (stopped || summary.trigger) return;
    summary.complete = false; summary.trigger = observation;
    // Stop scheduling before the owner's close callback, which may itself call stop().
    const closing = close();
    finalPair?.done.resolve(closing);
    try { onRefusal(new BuildResourceError(observation.failureCodes[0]!, observation)); }
    catch { summary.callbackFailed = true; }
    void closing;
  };
  const check = (forceIncomplete = false): void => {
    const currentPolicy = resourcePolicy(root);
    currentPolicy.conflict ||= policy.conflict || currentPolicy.targetAgentBudgetBytes !== policy.targetAgentBudgetBytes
      || currentPolicy.minimumFreeBytes !== policy.minimumFreeBytes;
    const observation = resourceObservation(currentPolicy, allocated ?? initial, available, "in-build");
    if (availableError) observation.errors = [...(observation.errors ?? []), availableError];
    // While the initial two independent samples are pending, known breaches still refuse immediately.
    if (forceIncomplete || observation.failureCodes.some(code => code !== "resource_observation_incomplete")) refuse(observation);
    else if (finalPair?.allocation && finalPair.free && observation.withinLimits) finalPair.done.resolve(close());
  };
  // Match the leading OS code, never a filename. Allocation ENOENT has already had its
  // only safe retry in the exact directory; a whole-tree retry could hide a replacement.
  const transient = (errors: readonly string[], free = false): boolean => errors.length > 0 && errors.every(error =>
    /^(?:Error: )?(?:ESTALE|EAGAIN|EBUSY)\b/.test(error) || (free && /^(?:Error: )?ENOENT\b/.test(error)));
  const failedAllocation = (errors: string[]): void => {
    allocated = { present: false, complete: false, logicalBytes: 0, allocatedBytes: 0, entries: 0, linksNotFollowed: 0, errors };
    summary.sampleCount++;
    check(true);
  };
  const scan = (): void => {
    if (stopped || pendingScan || finalPair?.allocation || !worker) return;
    pendingScan = ++scanId; scanStarted = performance.now();
    scanDeadline = later(() => { pendingScan = 0; failedAllocation(["inventory_time_limit"]); }, remaining(ALLOCATION_TIMEOUT_MS + 1_000));
    try { worker.postMessage(pendingScan); }
    catch (error) { cancel(scanDeadline); pendingScan = 0; failedAllocation([String(error)]); }
  };
  const sampleFree = (): void => {
    if (stopped || pendingFree || finalPair?.free) return;
    pendingFree = true;
    const requestedFor = finalPair, started = performance.now();
    let settled = false;
    const deadline = later(() => {
      settled = true; available = null; availableError = "inventory_free_space_time_limit"; summary.sampleCount++; check(true);
    }, remaining(freeSampleTimeoutMs));
    const failed = (error: unknown): void => {
      if (stopped || settled) return;
      settled = true; cancel(deadline); pendingFree = false; available = null; availableError = String(error); summary.sampleCount++;
      if (!finalPair && transient([String(error)], true) && freeFailures++ === 0) {
        summary.transientFailureCount = (summary.transientFailureCount ?? 0) + 1; later(sampleFree, 250);
      } else check(true);
    };
    try {
      const storage = managedStorageRoot(root);
      void Promise.all(managedVolumePaths(root, storage).map(path => fsPromises.statfs(path))).then(values => {
        if (stopped || settled) return;
        const amounts = values.map(value => value.bavail * value.bsize);
        const bytes = Math.min(...amounts);
        if (amounts.some(value => !Number.isSafeInteger(value) || value < 0)) { failed(new Error("inventory_invalid_available")); return; }
        try { if (managedStorageRoot(root) !== storage) throw new Error("inventory_storage_route_changed"); }
        catch (error) { failed(error); return; }
        settled = true; cancel(deadline); pendingFree = false; freeFailures = 0; available = bytes; availableError = undefined; summary.sampleCount++;
        summary.minimumSampledAvailableBytes = Math.min(summary.minimumSampledAvailableBytes ?? bytes, bytes);
        if (requestedFor) requestedFor.free = true;
        check();
        if (finalPair) sampleFree(); // Replace a result that was already in flight at stop.
        else if (!stopped) later(sampleFree, Math.max(0, 1_000 - (performance.now() - started)));
      }, failed);
    } catch (error) { failed(error); }
  };

  if (policy.conflict) {
    queueMicrotask(() => { if (!stopped) check(true); });
  } else {
    try {
      worker = new workerThreads.Worker(new URL(import.meta.url), { workerData: { role: ALLOCATION_WORKER, root: resolve(root) } });
      summary.workerClosed = false;
      const captureThreadId = (): void => {
        const threadId = worker!.threadId;
        if (Number.isSafeInteger(threadId) && threadId > 0) summary.workerThreadId = threadId;
      };
      captureThreadId();
      worker.once("online", captureThreadId);
      worker.on("message", (message: { id?: number; allocation?: AllocationObservation }) => {
        if (stopped) return;
        if (!pendingScan || message?.id !== pendingScan) { failedAllocation(["inventory_worker_protocol"]); return; }
        cancel(scanDeadline); pendingScan = 0;
        const sample = message.allocation;
        if (!sample || typeof sample.complete !== "boolean" || !Array.isArray(sample.errors)
          || sample.complete !== (sample.errors.length === 0) || !sample.errors.every(error => typeof error === "string")
          || !Number.isSafeInteger(sample.allocatedBytes) || sample.allocatedBytes < 0
          || (sample.priorAllocatedBytes !== undefined && (!Number.isSafeInteger(sample.priorAllocatedBytes) || sample.priorAllocatedBytes < 0))) {
          failedAllocation(["inventory_worker_protocol"]); return;
        }
        allocated = sample; summary.sampleCount++;
        summary.maximumSampledAllocatedBytes = Math.max(summary.maximumSampledAllocatedBytes ?? sample.allocatedBytes, sample.allocatedBytes, sample.priorAllocatedBytes ?? 0);
        if (finalPair && sample.complete && message.id >= finalPair.nextScanId) finalPair.allocation = true;
        check();
        if (stopped) return;
        if (!finalPair && !sample.complete && transient(sample.errors) && scanFailures++ === 0) {
          summary.transientFailureCount = (summary.transientFailureCount ?? 0) + 1; later(scan, 250); return;
        }
        if (!sample.complete) { check(true); return; }
        scanFailures = 0;
        if (finalPair) scan(); // An older in-flight result cannot settle the final pair.
        else if (!stopped) later(scan, Math.max(0, 10_000 - (performance.now() - scanStarted)));
      });
      worker.on("error", error => { if (!stopped) failedAllocation([String(error)]); });
      worker.on("exit", () => {
        workerExited = true; summary.workerClosed = true;
        if (!stopped) failedAllocation(["inventory_worker_exited"]);
      });
      scan(); sampleFree();
    } catch (error) {
      queueMicrotask(() => { if (!stopped) failedAllocation([String(error)]); });
    }
  }
  const stop = (): Promise<BuildResourceSummary> => {
    if (finalPair) return finalPair.done.promise;
    if (stopped || (!scanFailures && !freeFailures)) return close();
    // Only a still-pending transient gets a closing sample. Both halves must start
    // afresh and pass normal checks/deadlines; neither preflight nor a partial scan counts.
    finalPair = { started: performance.now(), nextScanId: scanId + 1, allocation: false, free: false, done: Promise.withResolvers<BuildResourceSummary>() };
    scan(); sampleFree();
    return finalPair.done.promise;
  };
  return { stop };
}
if (workerThreads.isMainThread && import.meta.main && process.argv[2] === "admission") {
  try { requireBuildAdmission(resolve(process.argv[3]!)); } catch (error) { console.error(String(error)); process.exitCode = 75; }
}
