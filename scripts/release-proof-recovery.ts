// Only the cold unit compile can defer. This classifies a failed measurement;
// it never repairs one, runs Cargo, or supplies release-gate evidence.
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { appendFileSync, lstatSync, readFileSync, readdirSync, realpathSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { readManagedTask, readOwnedJson } from "./agentic/artifact-lifecycle";
import { hashArtifactFile } from "./agentic/build-artifact";

const diagnostic = "Error: inventory_unstable_replay_scope:membership:at=deps:replay=deps:confirmation-churn";
const incomplete = "resource_observation_incomplete";
const uuid = "[a-f0-9]{8}-[a-f0-9]{4}-4[a-f0-9]{3}-[89ab][a-f0-9]{3}-[a-f0-9]{12}";
const unitArgs = ["test", "--locked", "--lib", "theme::alpha::alpha_byte_tests::"];
const pool = "target-agent/pools/agent-debug";
const marker = ".test-output/release-proof-refusal.json";
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
  assert(job === "validate-proof-contracts" && cold === "true" && code === "75");
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
  assert.deepEqual(cleanup, { resourcesAcquired: true, processExited: true, processGroupExited: true, streamsDrained: true,
    logWriterClosed: true, ownedWindowsClosed: null, referencesFinalized: true, closed: true, survivors: [], failureCodes: [] });
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
  assert(positive(budget) && positive(floor));
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
  let compiledCrates = 0;
  const visit = (dir: string) => {
    for (const name of readdirSync(join(root, dir)).sort()) {
      const relative = join(dir, name), path = join(root, relative), stat = lstatSync(path);
      assert(stat.dev === device && (stat.isDirectory() || stat.isFile()), "unsafe compiler cache entry");
      digest.update(JSON.stringify([relative, stat.mode & 0o111, stat.isFile() ? hashArtifactFile(path) : "directory"]) + "\n");
      if (stat.isDirectory()) visit(relative);
      else if (/^deps\/lib[A-Za-z0-9_]+-[a-f0-9]+\.rlib$/.test(relative) && stat.size > 0) compiledCrates++;
    }
  };
  visit(""); assert(positive(compiledCrates), "no compiled crate in the managed cache");
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
    const failed = classifyColdRefusal(result, readManagedTask(path, result.task), origin, env.GITHUB_JOB!, env.COLD!, process.argv[3]!, readFileSync(join(env.RUNNER_TEMP!, "release-unit.log"), "utf8"));
    writeFileSync(marker, JSON.stringify({ ...failed, cache: compilerCache() }) + "\n", { flag: "wx" });
    output("cache-key", handoffKey(origin, readFileSync(marker)));
    console.log("::warning::Primary compiler measurement FAILED (75); no proof evidence. Requesting one exact-cache recovery.");
  } else if (op === "miss" || op === "hit" || op === "restored") {
    exactCache(env.EXPECTED_KEY!, env.PRIMARY_KEY!, env.MATCHED_KEY!, env.HIT!, op === "miss" ? "miss" : "hit");
    if (op === "restored") {
      assert.equal(env.GITHUB_JOB, "recover-proof-contracts");
      assert.equal(realpathSync(marker), resolve(marker));
      verifyRestored(origin, env.EXPECTED_KEY!, readFileSync(marker), compilerCache());
    }
  } else throw Error("unknown recovery operation");
}
