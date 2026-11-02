import { expect, test } from "bun:test";
import { mkdtempSync, readFileSync, rmSync, symlinkSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { canonicalJson } from "./artifact-lifecycle";
import { artifactHash as hash, unsignedMachOPayloadSha256, type ArtifactManifestV3 } from "./build-artifact";
import { githubReleaseRun, readReleaseReceiptBytes, verifyReleaseCompile, type ReleaseCompileReceipt } from "./release-compile-contract";
import { checkReleaseExecutable, checkReleaseZip, checkSignedPayload } from "./release-compile";

const run = { repository:"johnlindquist/script-kit-next", repositoryId:"456", tag:"v0.0.0", commit:"a".repeat(40), runId:"123", attempt:"1" };
const id = `artifact-${crypto.randomUUID()}`, stamp = hash("fixture");
const upstream = { kind:"build-job" as const, id:`build-${crypto.randomUUID()}`, generation:crypto.randomUUID(), revision:4 };
const config = { requestedPolicy:{ args:["build","--locked","--profile","release"] }, compatibility:{} };
const manifest: ArtifactManifestV3 = {
  schemaVersion:3, artifactId:id, artifactKind:"application", binaryPath:`target-agent/artifacts/${id}/script-kit-gpui`, binarySha256:stamp, sizeBytes:100,
  target:{packageId:"path+file:///fixture/builder#script-kit-gpui@0.0.0",packageName:"script-kit-gpui",targetName:"script-kit-gpui",targetKind:["bin"],crateTypes:["bin"],sourcePath:"src/main.rs",features:["owned-ui-evaluation"],cargoProfile:{test:false},requestedProfile:"release",targetTriple:"aarch64-apple-darwin"},
  source:{algorithm:"reviewed-worktree-content-v1",compilerInputSha256:stamp,compilerInputOwnerSha256:stamp,committedInputTreeSha256:stamp,gitHead:run.commit,repositoryDirty:false,compilerDirty:false,inputPaths:["src"],changeBoundaryBeforeSha256:stamp,changeBoundaryAfterSha256:stamp,hermeticBuild:false},
  toolchain:{channel:"1.98.0",rustcVerboseVersion:"fixture",cargoVersion:"fixture",rustcSha256:stamp,cargoSha256:stamp},
  requestedPolicySha256:hash(canonicalJson(config.requestedPolicy)), effectiveConfiguration:config, effectiveConfigurationSha256:hash(canonicalJson(config)), requiresExactGitHead:true,
  publication:{owner:"scripts/agentic/agent-cargo.sh",pool:"agent-debug",leaseGeneration:crypto.randomUUID(),buildTask:upstream,immutable:true,exportedWhileLeaseHeld:true},
};
function fixture(m = structuredClone(manifest)): ReleaseCompileReceipt {
  return {schemaVersion:1,producer:{...run},reference:{manifestPath:`target-agent/artifacts/${id}/manifest.json`,manifestSha256:hash(canonicalJson(m)+"\n")},manifest:m,
    finalized:{identity:{...upstream,revision:7},status:"succeeded",closed:true,monitoredSamples:6},zipSha256:hash("zip fixture"),payloadSha256:stamp};
}
const bind = (r: ReleaseCompileReceipt, receiver=run) => { const bytes=canonicalJson(r)+"\n";return verifyReleaseCompile(bytes,hash(bytes),receiver); };

test("same run may promote a successful earlier attempt; not a future attempt, another tag/run, PR or guessed checksum", () => {
  const r=fixture(),bytes=canonicalJson(r)+"\n";
  expect(bind(r,{...run,attempt:"2"}).receipt.finalized.monitoredSamples).toBe(6);
  expect(()=>verifyReleaseCompile(bytes+" ",hash(bytes),run)).toThrow("digest");
  for(const patch of [{attempt:"0"},{runId:"124"},{tag:"v0.0.1"},{commit:"b".repeat(40)},{repository:"another/repo"},{repositoryId:"457"}]) expect(()=>bind(r,{...run,...patch})).toThrow();
  expect(()=>bind({...r,producer:{...run,attempt:"2"}})).toThrow("wrong_run");
  const env={GITHUB_ACTIONS:"true",GITHUB_REPOSITORY:run.repository,GITHUB_REPOSITORY_ID:run.repositoryId,GITHUB_EVENT_NAME:"push",GITHUB_SHA:run.commit,GITHUB_REF_NAME:run.tag,GITHUB_REF:`refs/tags/${run.tag}`,GITHUB_JOB:"build-macos-app",GITHUB_WORKFLOW_REF:`${run.repository}/.github/workflows/release.yml@refs/tags/${run.tag}`,GITHUB_RUN_ID:run.runId,GITHUB_RUN_ATTEMPT:run.attempt};
  expect(githubReleaseRun("build-macos-app",env)).toEqual(run);
  const tag="v0.0.0-rc.2", ref="refs/tags/"+tag;
  expect(githubReleaseRun("build-macos-app",{...env,GITHUB_REF_NAME:tag,GITHUB_REF:ref,GITHUB_WORKFLOW_REF:run.repository+"/.github/workflows/release.yml@"+ref}).tag).toBe(tag);
  for(const change of [{GITHUB_ACTIONS:"false"},{GITHUB_EVENT_NAME:"pull_request_target"},{GITHUB_JOB:"sign-notarize-macos"},{GITHUB_WORKFLOW_REF:"other"},{GITHUB_REF:"refs/heads/main"}]) expect(()=>githubReleaseRun("build-macos-app",{...env,...change})).toThrow("wrong_workflow");
});

test("recomputed transport digest cannot qualify a stale manifest, dirty source, wrong target, or unfinalized task", () => {
  const changed=fixture();changed.reference={...changed.reference,manifestSha256:hash("other manifest")};expect(()=>bind(changed)).toThrow("manifest_digest");
  for(const edit of [(m:any)=>m.target.features=[],(m:any)=>m.target.requestedProfile="dev",(m:any)=>m.target.targetTriple="x86_64-apple-darwin",(m:any)=>m.target.packageId+="-other",(m:any)=>m.requiresExactGitHead=false,(m:any)=>m.source.compilerDirty=true,(m:any)=>m.source.repositoryDirty=true,(m:any)=>m.source.gitHead="b".repeat(40),(m:any)=>m.source.changeBoundaryAfterSha256=hash("changed"),(m:any)=>m.publication.exportedWhileLeaseHeld=false,(m:any)=>m.derivation={input:"external"}]) {
    const m=structuredClone(manifest);edit(m);expect(()=>bind(fixture(m))).toThrow();
  }
  for(const edit of [(r:any)=>r.finalized.status="failed",(r:any)=>r.finalized.closed=false,(r:any)=>r.finalized.monitoredSamples=0,(r:any)=>r.finalized.identity.generation=crypto.randomUUID(),(r:any)=>r.finalized.identity.revision=upstream.revision]) { const r=fixture();edit(r);expect(()=>bind(r)).toThrow("not_finalized"); }
});

// Minimal synthetic Mach-O layout for testing the signature-only boundary, not an Apple ticket or a runnable application.
function mach(program:string,signature:string) {
  const header=Buffer.alloc(48);header.writeUInt32LE(0xfeedfacf);header.writeUInt32LE(0x0100000c,4);header.writeUInt32LE(1,16);header.writeUInt32LE(16,20);
  header.writeUInt32LE(0x1d,32);header.writeUInt32LE(16,36);header.writeUInt32LE(48+Buffer.byteLength(program),40);header.writeUInt32LE(Buffer.byteLength(signature),44);
  return Buffer.concat([header,Buffer.from(program),Buffer.from(signature)]);
}
test("raw unsigned bytes, ZIP and post-sign program each have to match their own identity", () => {
  const root=mkdtempSync(join(tmpdir(),"release-handoff-"));
  try {
    const exe=join(root,"script-kit-gpui"),zip=join(root,"unsigned.zip"),a=mach("original","adhoc");writeFileSync(exe,a,{mode:0o755});writeFileSync(zip,"zip fixture");
    const r=fixture({...manifest,binarySha256:hash(a),sizeBytes:a.length});r.payloadSha256=unsignedMachOPayloadSha256(exe);const proof=bind(r);
    expect(checkReleaseExecutable(proof,exe)).toBe(hash(a));expect(checkReleaseZip(proof,zip)).toBe(hash("zip fixture"));
    writeFileSync(exe,mach("original","signed (fixture)"));expect(checkSignedPayload(proof,exe)).toBe(r.payloadSha256);expect(()=>checkReleaseExecutable(proof,exe)).toThrow("executable_mismatch");
    writeFileSync(exe,mach("tampered","signed (fixture)"));expect(()=>checkSignedPayload(proof,exe)).toThrow("signed_payload_mismatch");
    writeFileSync(zip,"substituted");expect(()=>checkReleaseZip(proof,zip)).toThrow("zip_mismatch");
    const receipt=join(root,"receipt.json");writeFileSync(receipt,proof.bytes);expect(readReleaseReceiptBytes(receipt)).toBe(proof.bytes);
    symlinkSync(receipt,join(root,"alias"));expect(()=>readReleaseReceiptBytes(join(root,"alias"))).toThrow();writeFileSync(receipt,Buffer.alloc(256*1024+1));expect(()=>readReleaseReceiptBytes(receipt)).toThrow("unsafe_receipt");
  } finally { rmSync(root,{recursive:true,force:true}); }
});

test("the unprivileged build seals one binary; the signer verifies that producer before its keychain and Apple", () => {
  type Job={ steps:Array<{run?:string;with?:Record<string,string>}>;env:Record<string,string> };
  const {jobs}=Bun.YAML.parse(readFileSync(join(import.meta.dir,"../../.github/workflows/release.yml"),"utf8")) as {jobs:Record<string,Job>};
  const build=jobs['build-macos-app']!,sign=jobs['sign-notarize-macos']!;const cmds=(job:Job)=>job.steps.map(s=>s.run??"");
  const source=cmds(build),sink=cmds(sign);
  expect(source.filter((s:string)=>s.includes("build-ops act app-build"))).toHaveLength(1);
  expect(source.find((s:string)=>s.includes("CARGO_BUNDLE_SKIP_BUILD"))).toContain('install -m 0755 "$BIN" target/release/script-kit-gpui');
  expect(JSON.stringify(build)).not.toContain("secrets.");expect(sink.some((s:string)=>s.includes("build-ops act app-build") || s.includes("cargo bundle"))).toBe(false);
  const at=(part:string)=>{const i=sink.findIndex((s:string)=>s.includes(part));expect(i).toBeGreaterThanOrEqual(0);return i;};
  expect(at("release-compile.ts verify-executable")).toBeLessThan(at("security import"));expect(at("release-compile.ts verify-signed-payload")).toBeLessThan(at("notarytool submit"));
  expect(sign.env.EXPECTED_COMPILE_RECEIPT).toBe("${{ needs.build-macos-app.outputs.compile_receipt_sha }}");expect(sign.steps.map(s=>s.with?.['artifact-ids']).filter(Boolean)).toEqual(["${{ needs.validate-release-gates.outputs.proof_id }}","${{ needs.build-macos-app.outputs.unsigned_id }}"]);
});
