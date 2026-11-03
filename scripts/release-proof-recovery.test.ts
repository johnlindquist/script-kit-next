import { expect, test } from "bun:test";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { cpSync, linkSync, mkdirSync, mkdtempSync, readFileSync, realpathSync, rmSync, symlinkSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { classifyColdRefusal, compilerCache, exactCache, handoffKey, verifyRestored } from "./release-proof-recovery";

const origin = { ref: "refs/tags/v0.1.24", sha: "f".repeat(40), run: "123", attempt: "1" };
const primary = "validate-proof-contracts", recovery = "recover-proof-contracts";
const compiling = "   Compiling example v0.1.0\n";
function fixture() {
  const common = { scope: "target-agent", measurement: "allocated-blocks", hardQuota: false, automaticEviction: false };
  const obs = { ...common, complete: true, withinLimits: true, targetAgentAllocatedBytes: 10, targetAgentBudgetBytes: 40 * 1024 ** 3,
    availableBytes: 60 * 1024 ** 3, minimumFreeBytes: 25 * 1024 ** 3, reserveBytes: 0, failureCodes: [], errors: [] };
  const failureCode = "resource_observation_incomplete";
  const refusal = { ...obs, phase: "in-build", complete: false, withinLimits: false, failureCodes: [failureCode],
    errors: ["Error: inventory_unstable_replay_scope:membership:at=deps:replay=deps:confirmation-churn"] };
  const resources = { ...common, refusal, checks: ["preflight", "pre-cargo", "post-cargo"].map(phase => ({ ...obs, phase })), monitoring: {
    trigger: refusal, complete: false, workerClosed: true, workerThreadId: 1, initialObservationIncluded: true,
    sampleCount: 288, callbackFailed: false, transientFailureCount: 0, maximumSampledAllocatedBytes: 20, minimumSampledAvailableBytes: 51 * 1024 ** 3 } };
  const result = { status: "failed", exitCode: 75, disposition: "BLOCKED_RESOURCE_BUDGET", failureCode, resources, artifacts: [] };
  const cleanup = { closed: true, resourcesAcquired: true, processExited: true, processGroupExited: true, streamsDrained: true,
    logWriterClosed: true, ownedWindowsClosed: null, referencesFinalized: true, survivors: [], failureCodes: [] };
  const identity = { id: `build-${crypto.randomUUID()}`, generation: crypto.randomUUID(), revision: 5, kind: "build-job" };
  const record = { identity, result, cleanup, state: "closed", effectiveConfiguration: { requestedPolicy: { args: ["test", "--locked", "--lib", "--package", "sk-color", "alpha::alpha_byte_tests::"] } },
    source: { gitHead: origin.sha, repositoryDirty: false, compilerDirty: false } };
  const receipt = { ...result, task: identity, cleanup };
  return { receipt, record, resources, classify: (job = primary, cold = "true", status = "75", log = compiling) => classifyColdRefusal(receipt, record, origin, job, cold, status, log) };
}

test("only a closed cold compiler refusal is deferred; the saved measurement still failed", () => {
  const f = fixture();
  expect(f.classify()).toMatchObject({ status: "failed", exitCode: 75, resources: { monitoring: { complete: false } }, origin });
  // Actual hosted abort: the guard requested shutdown and proved it completed.
  Object.assign(f.receipt.cleanup,{failureCodes:["request_closed"]}); expect(f.classify().status).toBe("failed");
  expect(()=>classifyColdRefusal(f.receipt,f.record,{...origin,attempt:"2"},primary,"true","75",compiling)).toThrow();
  for (const args of [[recovery], [primary, "false"], [primary, ""], [primary, "true", "1"], [primary, "true", "75", ""],
    [primary, "true", "75", compiling + "error[E1234]: compiler error"], [primary, "true", "75", compiling + "running 6 tests"],
    [primary, "true", "75", compiling + "test result: FAILED"]]) expect(() => f.classify(...args)).toThrow();
});

type F = ReturnType<typeof fixture>;
const ineligible: Record<string, (f: F) => void> = {
  "different refusal": f => f.resources.refusal.errors[0] += "-other",
  "extra error": f => f.resources.refusal.errors.push("Error: EIO"),
  "extra failure": f => f.resources.refusal.failureCodes.push("resource_budget_exceeded"),
  "wrong wrapper disposition": f => { f.receipt.disposition = f.record.result.disposition = "UNKNOWN"; },
  "unknown status": f => { f.receipt.status = f.record.result.status = "unknown"; },
  "worker unclosed": f => { f.resources.monitoring.workerClosed = false; },
  "worker unknown": f => Reflect.deleteProperty(f.resources.monitoring, "workerClosed"),
  "sample absent": f => { f.resources.monitoring.sampleCount = 0; },
  "past allocation breach": f => { f.resources.monitoring.maximumSampledAllocatedBytes = f.resources.refusal.targetAgentBudgetBytes + 1; },
  "unexpectedly loosened policy": f => { for (const x of [...f.resources.checks, f.resources.refusal]) x.minimumFreeBytes = 1; },
  "past space breach": f => { f.resources.monitoring.minimumSampledAvailableBytes = 24; },
  "inconsistent latest bytes": f => { f.resources.refusal.availableBytes = 0; },
  "bad post check": f => { f.resources.checks[2].withinLimits = false; },
  "missing preflight": f => { f.resources.checks.shift(); },
  "wrong source": f => { f.record.source.gitHead = "0".repeat(40); },
  "dirty source": f => { f.record.source.repositoryDirty = true; },
  "another task": f => { f.receipt.task = { ...f.record.identity, generation: crypto.randomUUID() }; },
  "open owner": f => { f.record.state = "protected"; },
  "extra close failure": f => Object.assign(f.receipt.cleanup,{failureCodes:["request_closed","worker_unclosed"]}),
  "open process group": f => { f.receipt.cleanup.processGroupExited = false; },
  "open stream": f => { f.receipt.cleanup.streamsDrained = false; },
  "survivor": f => Object.assign(f.receipt.cleanup, { survivors: [{ kind: "resource-monitor" }] }),
};
for (const [name, poison] of Object.entries(ineligible)) test(`never defer: ${name}`, () => {
  const f = fixture(); poison(f); expect(() => f.classify()).toThrow();
});

function scratch(fn: (root: string) => void) {
  const root = realpathSync(mkdtempSync(join(tmpdir(), "proof-cache-test-")));
  try { fn(root); } finally { rmSync(root, { recursive: true, force: true }); }
}
test("cache lookup and restore must be exact, unambiguous and from this release/run/attempt", () => scratch(root => {
  mkdirSync(join(root, "deps"));
  expect(() => compilerCache(root)).toThrow("no compiled crate");
  const path = join(root, "deps/liba-1abc.rlib");
  writeFileSync(path, ""); expect(() => compilerCache(root)).toThrow("no compiled crate");
  writeFileSync(path, "warm"); const cache = compilerCache(root);
  const bytes = Buffer.from(JSON.stringify({ origin, cache })), key = handoffKey(origin, bytes);
  exactCache(key, key, "", "", "miss"); exactCache(key, key, key, "true", "hit"); verifyRestored(origin, key, bytes, cache);
  for (const [matched, hit] of [["", ""], ["other", "true"], [key, "false"], ["", "false"]]) expect(() => exactCache(key, key, matched, hit, "hit")).toThrow();
  for (const [primaryKey, matched, hit] of [["", "", ""], [key, key, "true"], [key, "prefix", "false"], [key, "", "false"]]) expect(() => exactCache(key, primaryKey, matched, hit, "miss")).toThrow();
  for (const change of [{ sha: "0".repeat(40) }, { ref: "refs/tags/v0.1.25" }, { run: "124" }, { attempt: "2" }]) expect(() => verifyRestored({ ...origin, ...change }, key, bytes, cache)).toThrow();
  expect(() => verifyRestored(origin, key, Buffer.from(bytes.toString() + "\n"), cache)).toThrow();
  writeFileSync(join(root, "other"), "stale"); expect(() => verifyRestored(origin, key, bytes, compilerCache(root))).toThrow();
  const sub=join(root,"separate-scope"); mkdirSync(sub); mkdirSync(join(sub,"deps"));
  linkSync(path,join(sub,"deps/libalias-0001.rlib")); expect(()=>compilerCache(sub)).toThrow("links outside");
  symlinkSync(path, join(root, "alias")); expect(() => compilerCache(root)).toThrow("unsafe compiler cache entry");
}));

test("two isolated fixture runners transfer only the exact failed measurement and compiled cache", () => scratch(root => {
  const a = join(root, "primary"), b = join(root, "recovery"), script = join(import.meta.dir, "release-proof-recovery.ts");
  mkdirSync(a); mkdirSync(b);
  writeFileSync(join(root, "git"), `#!/bin/sh\ncase "$1" in rev-parse) echo ${origin.sha};; status) exit 0;; *) exit 2;; esac`, { mode: 0o700 });
  const invoke = (cwd: string, args: string[], env: Record<string, string> = {}) => spawnSync(process.execPath, [script, ...args], { cwd, env: {
    PATH: `${root}:${process.env.PATH}`, GITHUB_ACTIONS: "true", RUNNER_OS: "macOS", GITHUB_EVENT_NAME: "push", GITHUB_REF: origin.ref,
    GITHUB_SHA: origin.sha, GITHUB_RUN_ID: origin.run, GITHUB_RUN_ATTEMPT: origin.attempt, GITHUB_JOB: cwd === a ? primary : recovery,
    GITHUB_OUTPUT: join(root, "out"), RUNNER_TEMP: root, ...env }, encoding: "utf8" });
  expect(invoke(a, ["start"], { NORMAL_CACHE: "true", NORMAL_MATCH: "false" }).status).not.toBe(0);
  expect(invoke(a, ["start"], { NORMAL_CACHE: "false" }).status).toBe(0);
  expect(readFileSync(join(root, "out"), "utf8")).toBe("cold=true\n");
  const { record, receipt } = fixture(), dir = join(a, ".test-output/managed-tasks", receipt.task.id);
  const put = (file: string, v: unknown) => writeFileSync(file, JSON.stringify(v));
  mkdirSync(dir, { recursive: true }); put(join(dir, "task.json"), { ...record, ownedProcesses: [], artifactReferences: [] });
  put(join(dir, ".artifact-lifecycle-owner.json"), { owner: "script-kit-gpui-probe", schemaVersion: 1, token: "fixture", runId: receipt.task.id, canonicalRoot: dir });
  put(join(a, ".test-output/release-unit-result.json"), { ...receipt, recordPath: join(dir, "task.json") });
  writeFileSync(join(root, "release-unit.log"), compiling);
  const debug = "target-agent/pools/agent-debug/debug", marker = ".test-output/release-proof-refusal.json";
  mkdirSync(join(a, debug, "deps"), { recursive: true }); writeFileSync(join(a, debug, "deps/libcrate-1abc.rlib"), "compiled fixture");
  const defer = invoke(a, ["defer", "75"], { COLD: "true" }); expect(defer.status, defer.stderr).toBe(0);
  const key = readFileSync(join(root, "out"), "utf8").match(/^cache-key=(.+)$/m)![1];
  expect(invoke(a, ["defer", "75"], { COLD: "true", GITHUB_RUN_ATTEMPT: "2" }).status).not.toBe(0);
  expect(invoke(b, ["start"]).status).toBe(0);
  const exact = { EXPECTED_KEY: key, PRIMARY_KEY: key, MATCHED_KEY: key, HIT: "true" };
  expect(invoke(b, ["restored"], exact).status).not.toBe(0);
  mkdirSync(join(b, ".test-output")); cpSync(join(a, marker), join(b, marker)); cpSync(join(a, debug), join(b, debug), { recursive: true });
  expect(invoke(b, ["restored"], { ...exact, GITHUB_RUN_ATTEMPT: "2" }).status).not.toBe(0);
  expect(invoke(b, ["restored"], { ...exact, GITHUB_SHA: "a".repeat(40) }).status).not.toBe(0);
  const restored = invoke(b, ["restored"], exact); expect(restored.status, restored.stderr).toBe(0);
}));

type ProofWorkflow = { jobs: Record<string, { steps: Array<{ name: string; id?: string; run: string; with: Record<string,string> }> }> };
const yaml = (file: string) => Bun.YAML.parse(readFileSync(join(import.meta.dir, `../.github/workflows/${file}.yml`), "utf8")) as ProofWorkflow;
test("the publisher requires the signer's artifact ID and workspace digest before download", () => {
  const step = yaml("release").jobs.release.steps.find(s => s.id === "require-publication-inputs")!;
  const check = (id: string, sha: string) => spawnSync("bash", ["-ec", step.run], {
    env: { RELEASE_ID: id, WORKSPACE_SHA: sha }, encoding: "utf8",
  }).status;
  const digest = "a".repeat(64);
  expect(check("42", digest)).toBe(0);
  for (const [id, sha] of [["", digest], ["0", digest], ["42,43", digest], ["42", ""], ["42", "not-a-digest"]]) {
    expect(check(id, sha)).not.toBe(0);
  }
});

test("the final report check rejects missing, misleading, and substituted downloads", () => scratch(root => {
  const dir = join(root, "downloaded assets"), recomputed = join(root, "independent scorecard.json");
  mkdirSync(dir);
  const sha = "a".repeat(40), zip = "b".repeat(64), sdk = "c".repeat(64);
  const json = (f: string, v: unknown) => writeFileSync(f, JSON.stringify(v) + "\n");
  const manifest = join(dir, "release-manifest.json"), card = join(dir, "release-scorecard.json");
  const report = join(dir, "installed-workspace-check.json");
  const result = { sourceSha: sha, archiveSha256: zip, journeys: [{ measuresPaint: false }] };
  const workspace = { schemaVersion: 1, version: "0.1.29", packaged: true, freshWorkspace: true,
    sdkSha256: sdk, starterScriptsIndexed: 12, userEditsPreserved: true, settingsTransaction: "verified",
    scratchRemoved: true, sourceSha: sha, zipSha256: zip, settingsProof: { key: "editorFontSize", beforeValue: 16,
      value: 27, beforeSha256: "d".repeat(64), writtenSha256: "e".repeat(64), afterRejectionSha256: "e".repeat(64), rejection: "stale_revision" } };
  json(manifest, { version: "0.1.29", source_sha: sha, artifacts: [{ sha256: zip }], sdk: { sha256: sdk } });
  json(card, result); json(recomputed, result); json(report, workspace);
  const digest = () => createHash("sha256").update(readFileSync(report)).digest("hex");
  const signer = digest();
  const run = (expected = signer, independent = recomputed) => spawnSync("bash", [
    join(import.meta.dir, "verify-release-reports.sh"), dir, independent, expected,
  ], { encoding: "utf8" });
  expect(run().status).toBe(0);
  expect(run(signer, card).status).not.toBe(0);
  expect(run("").status).not.toBe(0);
  json(card, { ...result, journeys: [{ measuresPaint: true }] });
  expect(run().stderr).toContain("scorecard differs");
  rmSync(card); expect(run().status).not.toBe(0); json(card, result);
  rmSync(report); expect(run().status).not.toBe(0);
  // Even a separately valid account of these settings is not the signing job's original bytes.
  json(report, { ...workspace, settingsProof: { ...workspace.settingsProof, beforeSha256: "f".repeat(64) } });
  expect(run().stderr).toContain("differs from the signing job");
  for (const field of ["sourceSha", "zipSha256", "sdkSha256"] as const) {
    json(report, { ...workspace, [field]: "0".repeat(workspace[field].length) });
    expect(run(digest()).stderr).toContain("different source, SDK, or archive");
  }
  json(report, { ...workspace, settingsProof: null }); expect(run(digest()).stderr).toContain("invalid packaged settings");
  writeFileSync(report, JSON.stringify(workspace) + "\n" + JSON.stringify(workspace));
  expect(run(digest()).status).not.toBe(0);
  const indirect = join(root, "other.json"); json(indirect, workspace); rmSync(report); symlinkSync(indirect, report);
  expect(run().status).not.toBe(0);
}));

const steps = yaml("release-compiler-proof").jobs[primary].steps;
test("both runners validate the fetched tag and immutable HEAD before using repo code", () => scratch(root => {
  expect(steps[0].with.ref).toBe("${{ github.ref }}");
  expect(steps[1].name).toBe("Pin the fetched tag to the release event");
  writeFileSync(join(root, "git"), '#!/bin/sh\ncase "$3" in HEAD) echo "$HEAD";; "$GITHUB_REF^{commit}") echo "$TAG";; *) exit 64;; esac', { mode: 0o700 });
  const pin = (head: string, tag: string) => spawnSync("/bin/bash", ["-ec", steps[1].run], { cwd: root,
    env: { PATH: root, GITHUB_REF: origin.ref, GITHUB_SHA: origin.sha, HEAD: head, TAG: tag } }).status;
  expect(pin(origin.sha, origin.sha)).toBe(0);
  expect(pin("other commit", origin.sha)).not.toBe(0); expect(pin(origin.sha, "moved tag")).not.toBe(0);
}));

test("real compiler-step shell preserves exits, requires both measurements and never defers twice", () => scratch(root => {
  mkdirSync(join(root, "scripts/agentic"), { recursive: true });
  writeFileSync(join(root, "scripts/agentic/agent-cargo.sh"), `if [[ "$*" == *--doc* ]]; then rc=$DOC; count=3; else rc=$UNIT; count=$COUNT; fi
    printf '{"status":"succeeded","exitCode":0,"passedTests":%s,"failedTests":0,"cleanup":{"closed":true,"survivors":[]},"resources":{"monitoring":{"complete":true,"workerClosed":true},"refusal":null}}' "$count" > "$SCRIPT_KIT_AGENT_RESULT_PATH"
    exit "$rc"`);
  writeFileSync(join(root, "bun"), '#!/bin/sh\ntest "$*" = "scripts/release-proof-recovery.ts defer 75" || exit 97\necho checked >> "$GITHUB_OUTPUT"\nexit "$CLASSIFIER"', { mode: 0o700 });
  const drive = (job: string, unit: string, doc: string, classifier: string, count = "6") => {
    const output = join(root, "out"); writeFileSync(output, "");
    const result = spawnSync("bash", ["-c", steps.find(s => s.id === "compiler")!.run], { cwd: root,
      env: { PATH: `${root}:${process.env.PATH}`, UNIT: unit, DOC: doc, COUNT: count, CLASSIFIER: classifier, GITHUB_JOB: job, RUNNER_TEMP: root, GITHUB_OUTPUT: output } });
    return [result.status, readFileSync(output, "utf8").trim()];
  };
  expect(drive(primary, "0", "0", "99")).toEqual([0, "state=passed"]);
  expect(drive(recovery, "0", "0", "99")).toEqual([0, "state=passed"]);
  expect(drive(primary, "0", "0", "99", "0")[0]).not.toBe(0);
  expect(drive(primary, "75", "0", "0")).toEqual([0, "checked"]);
  expect(drive(primary, "75", "0", "65")).toEqual([65, "checked"]);
  expect(drive(recovery, "75", "0", "0")).toEqual([75, ""]);
  expect(drive(primary, "101", "0", "0")).toEqual([101, ""]);
  expect(drive(primary, "0", "75", "0")).toEqual([75, ""]);
}));

test("the real release aggregate admits only an uploaded complete proof from the selected path", () => {
  const aggregate = yaml("release").jobs["validate-release-gates"].steps[0];
  const run = (proof: string, first: string, key: string, recovered: string, design = "73", build = "success") => spawnSync("bash", ["-ec", aggregate.run.replace(/\$\{\{ needs\['([^']+)'\]\.result \}\}/g, (_: string, job: string) => job === primary ? proof : job === "build-macos-app" ? build : "success")],
    { env: { PRIMARY_EVIDENCE: first, RECOVERY_KEY: key, RECOVERED_EVIDENCE: recovered, DESIGN_ID: design } }).status;
  expect(run("success", "101", "", "", "", "success")).not.toBe(0);
  expect(run("success", "101", "", "", "73", "failure")).not.toBe(0);
  expect(run("success", "101", "", "")).toBe(0); expect(run("success", "", "scoped", "102")).toBe(0);
  for (const [proof, first, key, recovered] of [["skipped", "", "scoped", ""], ["cancelled", "", "scoped", "102"], ["failure", "101", "", ""],
    ["success", "", "scoped", ""], ["success", "101", "scoped", ""], ["success", "101", "scoped", "102"], ["success", "", "", ""], ["success", "", "", "102"]]) expect(run(proof, first, key, recovered)).not.toBe(0);
});
