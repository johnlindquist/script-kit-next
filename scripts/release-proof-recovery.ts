// Only the cold unit compile can defer. This classifies a failed measurement;
// it never repairs one, runs Cargo, or supplies release-gate evidence.
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { appendFileSync, lstatSync, readFileSync, readdirSync, realpathSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { readManagedTask, readOwnedJson } from "./agentic/artifact-lifecycle";
import { hashArtifactFile } from "./agentic/build-artifact";
import { readReleaseReceiptBytes } from "./agentic/release-compile-contract";

const diagnostic = "Error: inventory_unstable_replay_scope:membership:at=deps:replay=deps:confirmation-churn";
const incomplete = "resource_observation_incomplete";
const GiB = 1024 ** 3;
const maxHandoffBytes = 40 * GiB, minimumHostedFree = 25 * GiB;
const uuid = "[a-f0-9]{8}-[a-f0-9]{4}-4[a-f0-9]{3}-[89ab][a-f0-9]{3}-[a-f0-9]{12}";
const unitArgs = ["test", "--locked", "--lib", "--package", "sk-color", "alpha::alpha_byte_tests::"];
const pool = "target-agent/pools/agent-debug";
const marker = ".test-output/release-proof-refusal.json";
function markerBytes(): Buffer {
  assert.equal(realpathSync(marker), resolve(marker));
  return Buffer.from(readReleaseReceiptBytes(marker), "utf8");
}
function compileLog(temp: string): string {
  const path=join(realpathSync(temp),"release-unit.log"), st=lstatSync(path);
  assert(st.isFile() && st.nlink===1 && st.size>0 && st.size<=32*1024**2, "unsafe or unbounded compiler log");
  return readFileSync(path,"utf8");
}
const natural = (v: unknown): v is number => typeof v === "number" && Number.isSafeInteger(v) && v >= 0;
const positive = (v: unknown) => natural(v) && v > 0;
type ObjectJson = Record<string, any>;

type Origin = { sha: string; ref: string; run: string; attempt: string };
function event(): Origin {
  const env = process.env;
  assert(env.GITHUB_ACTIONS === "true" && env.GITHUB_EVENT_NAME === "push" && env.RUNNER_OS === "macOS");
  const { GITHUB_SHA: sha, GITHUB_REF: ref, GITHUB_RUN_ID: run, GITHUB_RUN_ATTEMPT: attempt } = env;
  assert(sha && /^[0-9a-f]{40}$/.test(sha) && ref && /^refs\/tags\/v\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$/.test(ref));
  assert(run && /^[1-9]\d*$/.test(run) && attempt && /^[1-9]\d*$/.test(attempt));
  assert.equal(execFileSync("git", ["rev-parse", "HEAD"], { encoding: "utf8" }).trim(), sha);
  assert.equal(execFileSync("git", ["status", "--porcelain", "--untracked-files=all"], { encoding: "utf8" }), "");
  return { sha, ref, run, attempt };
}

export function classifyColdRefusal(receipt: ObjectJson, record: ObjectJson, origin: Origin, job: string, cold: string, code: string, log: string) {
  assert(job === "validate-proof-contracts" && origin.attempt === "1" && cold === "true" && code === "75");
  const { status, exitCode, disposition, failureCode, artifacts, resources, cleanup, task } = receipt;
  assert(status === "failed" && exitCode === 75 && disposition === "BLOCKED_RESOURCE_BUDGET" && failureCode === incomplete);
  assert.deepEqual(artifacts, []);
  assert.equal(task.kind, "build-job");
  assert.match(task.id, new RegExp(`^build-${uuid}$`));
  assert.match(task.generation, new RegExp(`^${uuid}$`));
  assert(positive(task.revision));
  assert.deepEqual(record.identity, task);
  assert.equal(record.state, "closed");
  const { recordPath: _recordPath, task: _task, cleanup: _cleanup, ...reported } = receipt;
  assert.deepEqual(reported, { status, exitCode, disposition, failureCode, artifacts, resources });
  assert.deepEqual(record.result, reported);
  assert.deepEqual(record.cleanup, cleanup);
  assert(record.source.gitHead === origin.sha && record.source.repositoryDirty === false && record.source.compilerDirty === false);
  assert.deepEqual(record.effectiveConfiguration.requestedPolicy.args, unitArgs);
  // The guard calls child.close() on refusal. The real hosted refusal records
  // request_closed even with a proven-dead group; no other close code qualifies.
  const closedByGuard = cleanup.failureCodes?.length === 1 && cleanup.failureCodes[0] === "request_closed";
  assert.deepEqual(cleanup, { resourcesAcquired: true, processExited: true, processGroupExited: true, streamsDrained: true,
    logWriterClosed: true, ownedWindowsClosed: null, referencesFinalized: true, closed: true, survivors: [], failureCodes: closedByGuard ? ["request_closed"] : [] });
  // The guard's closing error can supersede Cargo's exit. Only an unfinished
  // compile is eligible, never a harness that started or a compiler error.
  assert(/^\s*Compiling \S/m.test(log) && !/^\s*(?:error(?:\[|:)|Running (?:unittests|tests)|running \d+ tests?\b|test result:|test .* \.\.\. FAILED)/m.test(log));

  assert(resources.scope === "target-agent" && resources.measurement === "allocated-blocks" && resources.hardQuota === false && resources.automaticEviction === false);
  const { checks, refusal, monitoring: m } = resources;
  assert.deepEqual(checks.map((r: ObjectJson) => r.phase), ["preflight", "pre-cargo", "post-cargo"]);
  assert.deepEqual(m.trigger, refusal);
  assert(m.complete === false && m.workerClosed === true && positive(m.workerThreadId) && positive(m.sampleCount)
    && m.initialObservationIncluded === true && m.callbackFailed === false && m.transientFailureCount === 0);
  // Hosted macOS uses the capped local pool. A null/unknown/zero cap (or a
  // changed policy), unobserved floor, or a prior peak breach cannot qualify.
  const { targetAgentBudgetBytes: budget, minimumFreeBytes: floor } = refusal;
  assert(positive(budget) && budget <= maxHandoffBytes && positive(floor) && floor >= minimumHostedFree);
  const checkBytes = (used: unknown, available: unknown) => assert(natural(used) && used <= budget && natural(available) && available >= floor);
  checkBytes(m.maximumSampledAllocatedBytes, m.minimumSampledAvailableBytes);
  for (const r of [...checks, refusal]) {
    assert(r.scope === "target-agent" && r.measurement === "allocated-blocks" && r.hardQuota === false && r.automaticEviction === false);
    assert(r.reserveBytes === 0 && r.targetAgentBudgetBytes === budget && r.minimumFreeBytes === floor);
    checkBytes(r.targetAgentAllocatedBytes, r.availableBytes);
    if (r !== refusal) {
      assert(r.complete === true && r.withinLimits === true);
      assert.deepEqual(r.failureCodes, []); assert.deepEqual(r.errors, []);
    }
  }
  assert(refusal.phase === "in-build" && refusal.withinLimits === false && refusal.complete === false);
  assert.deepEqual(refusal.failureCodes, [incomplete]); assert.deepEqual(refusal.errors, [diagnostic]);
  return { status, exitCode, disposition, failureCode, resources, cleanup, task, origin };
}

// The refusal path alone pays for hashing the compiled cache. Bind all paths,
// file contents and executable bits; no lease, task record or HOME travels.
export function compilerCache(root = resolve(pool, "debug")) {
  assert.equal(realpathSync(root), root);
  const device = lstatSync(root).dev, digest = createHash("sha256");
  let compiledCrates = 0, entries = 0, bytes = 0;
  const links = new Map<string, { count: number; expected: number }>(), started = performance.now();
  const scanned: { relative: string; identity: string; directory: boolean; executable: number }[] = [];
  const stamp = (s: ReturnType<typeof lstatSync>) => [s!.dev,s!.ino,s!.nlink,s!.mode,s!.size,s!.mtimeMs,s!.ctimeMs].join(":");
  const rootIdentity = stamp(lstatSync(root));
  const visit = (dir: string) => {
    for (const name of readdirSync(join(root, dir)).sort()) {
      const relative = join(dir, name), path = join(root, relative), stat = lstatSync(path);
      assert(stat.dev === device && (stat.isDirectory() || stat.isFile()), "unsafe compiler cache entry");
      bytes += stat.isFile() ? stat.size : 0;
      assert(++entries <= 250_000 && bytes <= maxHandoffBytes && performance.now() - started < 120_000, "compiler cache observation limit");
      if (stat.isFile() && stat.nlink > 1) {
        const key = stat.dev + ":" + stat.ino, entry = links.get(key) ?? { count: 0, expected: stat.nlink };
        assert(entry.expected === stat.nlink, "compiler cache hardlink changed"); entry.count++; links.set(key, entry);
      }
      scanned.push({ relative, identity: stamp(stat), directory: stat.isDirectory(), executable: stat.mode & 0o111 });
      if (stat.isDirectory()) visit(relative);
      else if (/^deps\/lib[A-Za-z0-9_]+-[a-f0-9]+\.rlib$/.test(relative) && stat.size > 0) compiledCrates++;
    }
  };
  visit(""); assert(positive(compiledCrates), "no compiled crate in the managed cache");
  assert([...links.values()].every(x => x.count === x.expected), "compiler cache links outside the handoff");
  // Establish link containment before reading any file contents. Reject replacement
  // or late membership changes, not just a different set after archive transfer.
  for (const e of scanned) {
    const path = join(root,e.relative);
    assert(performance.now() - started < 120_000 && stamp(lstatSync(path)) === e.identity, "compiler cache changed");
    digest.update(JSON.stringify([e.relative, e.executable, e.directory ? "directory" : hashArtifactFile(path)]) + "\n");
  }
  assert(stamp(lstatSync(root)) === rootIdentity && scanned.every(e => performance.now() - started < 120_000 && stamp(lstatSync(join(root,e.relative))) === e.identity), "compiler cache changed");
  return { sha256: digest.digest("hex"), compiledCrates };
}
export function handoffKey(origin: Origin, markerBytes: Buffer) {
  return `compiler-proof-v1-${origin.sha}-${origin.run}-${origin.attempt}-${createHash("sha256").update(markerBytes).digest("hex")}`;
}
export function exactCache(expected: string, primary: string, matched: string, hit: string, want: "miss" | "hit") {
  assert(expected && primary === expected && (want === "hit" ? matched === expected && hit === "true" : matched === "" && hit === ""), `not an exact cache ${want}`);
}
export function verifyRestored(origin: Origin, expected: string, bytes: Buffer, cache: ReturnType<typeof compilerCache>) {
  assert.equal(expected, handoffKey(origin, bytes));
  const saved = JSON.parse(bytes.toString("utf8"));
  assert.deepEqual(saved.origin, origin); assert.deepEqual(saved.cache, cache);
}

if (import.meta.main) {
  const origin = event(), env = process.env, op = process.argv[2];
  const output = (key: string, value: string) => appendFileSync(env.GITHUB_OUTPUT!, `${key}=${value}\n`);
  if (op === "start") {
    const absent = (path: string) => lstatSync(path, { throwIfNoEntry: false }) === undefined;
    assert(absent(marker));
    if (env.GITHUB_JOB === "recover-proof-contracts") assert(absent("target-agent"), "recovery requires a fresh runner");
    else {
      assert.equal(env.GITHUB_JOB, "validate-proof-contracts");
      assert(env.NORMAL_CACHE === "false" || (env.NORMAL_CACHE === "true" && env.NORMAL_MATCH === "true"), "ambiguous normal cache restore");
      output("cold", String(env.NORMAL_CACHE === "false" && absent("target-agent")));
    }
  } else if (op === "defer") {
    assert(lstatSync("target-agent/.locks/pool-agent-debug.lock", { throwIfNoEntry: false }) === undefined, "pool lease still present");
    const result = readOwnedJson(resolve(".test-output/release-unit-result.json"));
    assert.match(result.task.id, new RegExp(`^build-${uuid}$`));
    const path = resolve(".test-output/managed-tasks", result.task.id, "task.json");
    assert(path === result.recordPath && path === realpathSync(path));
    const failed = classifyColdRefusal(result, readManagedTask(path, result.task), origin, env.GITHUB_JOB!, env.COLD!, process.argv[3]!, compileLog(env.RUNNER_TEMP!));
    writeFileSync(marker, JSON.stringify({ ...failed, cache: compilerCache() }) + "\n", { flag: "wx" });
    output("cache-key", handoffKey(origin, markerBytes()));
    console.log("::warning::Primary compiler measurement FAILED (75); no proof evidence. Requesting one exact-cache recovery.");
  } else if (op === "miss" || op === "hit" || op === "restored") {
    exactCache(env.EXPECTED_KEY!, env.PRIMARY_KEY!, env.MATCHED_KEY!, env.HIT!, op === "miss" ? "miss" : "hit");
    if (op === "restored") {
      assert.equal(env.GITHUB_JOB, "recover-proof-contracts");
      verifyRestored(origin, env.EXPECTED_KEY!, markerBytes(), compilerCache());
    }
  } else throw Error("unknown recovery operation");
}
