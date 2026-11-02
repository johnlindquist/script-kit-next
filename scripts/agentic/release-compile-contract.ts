// A portable, passive receipt, not a lease, task index, or runnable local artifact.
// The signer gets the expected envelope digest from needs.build-macos-app.outputs,
// independently of the downloaded files. A checksum supplied by the ZIP is not authority.
import { createHash } from "node:crypto";
import { constants, closeSync, fstatSync, lstatSync, openSync, readSync } from "node:fs";
import { canonicalJson } from "./artifact-lifecycle";
import type { ArtifactManifestV3, ArtifactReference } from "./build-artifact";
import type { TaskIdentity } from "./artifact-lifecycle";

const MAX_RECEIPT = 256 * 1024;
const hash = (bytes: string | Uint8Array) => createHash("sha256").update(bytes).digest("hex");
const sha = (v: unknown): v is string => typeof v === "string" && /^[0-9a-f]{64}$/.test(v);
const positive = (v: unknown): v is string => typeof v === "string" && /^[1-9]\d{0,15}$/.test(v) && Number.isSafeInteger(Number(v));
const semver = (v: unknown): v is string => typeof v === "string" && /^v(?:0|[1-9]\d*)\.(?:0|[1-9]\d*)\.(?:0|[1-9]\d*)(?:-[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?$/.test(v);
const check: (ok: unknown, reason: string) => asserts ok = (ok, reason) => { if (!ok) throw Error(`release_compile_${reason}`); };

export interface ReleaseRun { repository: string; repositoryId: string; tag: string; commit: string; runId: string; attempt: string }
export interface ReleaseCompileReceipt {
  schemaVersion: 1; producer: ReleaseRun; reference: ArtifactReference; manifest: ArtifactManifestV3;
  finalized: { identity: TaskIdentity; closed: true; status: "succeeded"; monitoredSamples: number };
  zipSha256: string; payloadSha256: string;
}
export interface VerifiedReleaseCompile { readonly receipt: ReleaseCompileReceipt; readonly bytes: string; readonly sha256: string }
const verified = new WeakSet<object>();
export const isVerifiedReleaseCompile = (x: object): x is VerifiedReleaseCompile => verified.has(x);
function freeze<T>(v: T): T {
  if (v && typeof v === "object" && !Object.isFrozen(v)) { for (const child of Object.values(v)) freeze(child); Object.freeze(v); }
  return v;
}
function validRun(r: ReleaseRun | undefined): boolean {
  return !!r && typeof r.repository === "string" && /^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/.test(r.repository) && positive(r.repositoryId) && semver(r.tag) && typeof r.commit === "string" && /^[0-9a-f]{40}$/.test(r.commit) && positive(r.runId) && positive(r.attempt);
}

export function githubReleaseRun(job: "build-macos-app" | "sign-notarize-macos", env: NodeJS.ProcessEnv = process.env): ReleaseRun {
  const r = { repository: env.GITHUB_REPOSITORY!, repositoryId: env.GITHUB_REPOSITORY_ID!, tag: env.GITHUB_REF_NAME!, commit: env.GITHUB_SHA!, runId: env.GITHUB_RUN_ID!, attempt: env.GITHUB_RUN_ATTEMPT! };
  check(validRun(r) && env.GITHUB_ACTIONS === "true" && env.GITHUB_EVENT_NAME === "push" && env.GITHUB_JOB === job &&
    env.GITHUB_REF === `refs/tags/${r.tag}` && env.GITHUB_WORKFLOW_REF === `${r.repository}/.github/workflows/release.yml@${env.GITHUB_REF}`, "wrong_workflow");
  return r;
}

/** Do not read a symlink, a hardlink, a special file, an unbounded file or changing bytes. */
export function readReleaseReceiptBytes(path: string): string {
  const fd = openSync(path, constants.O_RDONLY | constants.O_NOFOLLOW | constants.O_NONBLOCK);
  try {
    const before = fstatSync(fd);
    check(before.isFile() && before.nlink === 1 && before.size > 0 && before.size <= MAX_RECEIPT, "unsafe_receipt");
    const buf = Buffer.alloc(before.size + 1);
    let used = 0;
    while (used < buf.length) { const got = readSync(fd, buf, used, buf.length - used, null); if (!got) break; used += got; }
    check(used === before.size, "receipt_changed");
    const text = new TextDecoder("utf-8", { fatal: true }).decode(buf.subarray(0, used)), after = lstatSync(path);
    check(Buffer.byteLength(text) === before.size && after.dev === before.dev && after.ino === before.ino && after.nlink === 1 &&
      after.size === before.size && after.mtimeMs === before.mtimeMs && after.ctimeMs === before.ctimeMs, "receipt_changed");
    return text;
  } finally { closeSync(fd); }
}

/** Validate the original finalized reference and its exact transport identity. GitHub can
 * legitimately reuse a successful build from an earlier attempt of this same workflow run. */
export function verifyReleaseCompile(bytes: string, expectedSha256: string, consumer: ReleaseRun): VerifiedReleaseCompile {
  check(Buffer.byteLength(bytes) <= MAX_RECEIPT && sha(expectedSha256) && hash(bytes) === expectedSha256, "receipt_digest");
  const x = JSON.parse(bytes) as ReleaseCompileReceipt;
  check(x?.schemaVersion === 1 && validRun(consumer) && validRun(x.producer) && x.producer.repository === consumer.repository && x.producer.repositoryId === consumer.repositoryId &&
    x.producer.commit === consumer.commit && x.producer.tag === consumer.tag && x.producer.runId === consumer.runId &&
    Number(x.producer.attempt) <= Number(consumer.attempt), "wrong_run");
  const m = x.manifest, ref = x.reference, t = m?.publication?.buildTask, f = x.finalized;
  check(m?.schemaVersion === 3 && /^artifact-[0-9a-f-]{36}$/.test(m.artifactId) && ref?.manifestPath === `target-agent/artifacts/${m.artifactId}/manifest.json` &&
    sha(ref.manifestSha256) && hash(canonicalJson(m) + "\n") === ref.manifestSha256, "manifest_digest");
  check(!m.derivation && m.artifactKind === "application" && m.binaryPath === `target-agent/artifacts/${m.artifactId}/script-kit-gpui` &&
    m.target?.packageName === "script-kit-gpui" && typeof m.target.packageId === "string" && m.target.packageId.startsWith("path+file:///") &&
    m.target.packageId.endsWith(`#script-kit-gpui@${consumer.tag.slice(1)}`) && m.target.targetName === "script-kit-gpui" && m.target.sourcePath === "src/main.rs" &&
    m.target.targetKind?.includes("bin") && m.target.crateTypes?.includes("bin") && m.target.requestedProfile === "release" && m.target.cargoProfile?.test === false &&
    m.target.targetTriple === "aarch64-apple-darwin" && m.target.features?.includes("owned-ui-evaluation") && m.requiresExactGitHead === true &&
    m.source?.algorithm === "reviewed-worktree-content-v1" && m.source.gitHead === consumer.commit && m.source.compilerDirty === false && m.source.repositoryDirty === false &&
    m.source.changeBoundaryBeforeSha256 === m.source.changeBoundaryAfterSha256 && m.source.hermeticBuild === false &&
    Number.isSafeInteger(m.sizeBytes) && m.sizeBytes > 0 && m.sizeBytes <= 512 * 1024 ** 2 &&
    [x.zipSha256, x.payloadSha256, m.binarySha256, m.source.compilerInputSha256, m.source.compilerInputOwnerSha256, m.source.committedInputTreeSha256,
      m.source.changeBoundaryBeforeSha256, m.toolchain?.rustcSha256, m.toolchain?.cargoSha256].every(sha), "wrong_target");
  check(m.publication.owner === "scripts/agentic/agent-cargo.sh" && m.publication.pool === "agent-debug" && m.publication.exportedWhileLeaseHeld === true &&
    m.publication.immutable === true && m.publication.leaseGeneration && t?.kind === "build-job" && /^build-[0-9a-f-]{36}$/.test(t.id) && /^[0-9a-f-]{36}$/.test(t.generation) &&
    Number.isSafeInteger(t.revision) && t.revision > 0 && f?.closed === true && f.status === "succeeded" && Number.isSafeInteger(f.monitoredSamples) && f.monitoredSamples > 0 && f.identity?.kind === t.kind &&
    f.identity.id === t.id && f.identity.generation === t.generation && Number.isSafeInteger(f.identity.revision) && f.identity.revision > t.revision &&
    hash(canonicalJson(m.effectiveConfiguration)) === m.effectiveConfigurationSha256 &&
    hash(canonicalJson(m.effectiveConfiguration.requestedPolicy)) === m.requestedPolicySha256, "not_finalized");
  const result = freeze({ receipt: x, bytes, sha256: expectedSha256 }); verified.add(result); return result;
}
