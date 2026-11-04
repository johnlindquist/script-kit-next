import { expect, test } from "bun:test";
import { linkSync, lstatSync, mkdirSync, mkdtempSync, readdirSync, readFileSync, realpathSync, renameSync, rmSync, symlinkSync, unlinkSync, writeFileSync } from "node:fs";
import { spawnSync } from "node:child_process";
import { engineIdentity, receiveEngine, verifySignedEngine } from "./release-sidecar";
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
  const header=Buffer.alloc(48);header.writeUInt32LE(0xfeedfacf);header.writeUInt32LE(0x0100000c,4);header.writeUInt32LE(2,12);header.writeUInt32LE(1,16);header.writeUInt32LE(16,20);
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

test("only the producer's ready-to-sign engine reaches the bundle; signing may change its signature, not its program", () => {
  const root = realpathSync(mkdtempSync(join(tmpdir(), "kit-engine-transfer-")));
  try {
    const inbox = join(root, "received"), app = join(root, "Script Kit.app"), macos = join(app, "Contents/MacOS");
    mkdirSync(inbox); mkdirSync(macos, {recursive:true});
    const file = join(inbox, "goose"), dest = join(macos, "goose");
    const original = mach("engine release", "linker"), other = mach("other program!", "linker");
    writeFileSync(file, original, {mode:0o644});
    const {sha256, payloadSha256} = engineIdentity(file);
    const receive = (raw=sha256, payload=payloadSha256) => receiveEngine(inbox, app, raw, payload);
    writeFileSync(file, other); expect(() => receive()).toThrow("sidecar_producer_mismatch");
    writeFileSync(file, original); expect(() => receive(sha256,hash("different"))).toThrow("sidecar_producer_mismatch");
    expect(() => receive("")).toThrow("sidecar_invalid_expected_digest");
    expect(readdirSync(macos)).toEqual([]);
    receive(); expect(readFileSync(dest)).toEqual(original); expect(lstatSync(dest).mode & 0o777).toBe(0o755);
    writeFileSync(dest, mach("engine release", "new signature fixture")); verifySignedEngine(dest,payloadSha256);
    expect(() => receive()).toThrow(); expect(readFileSync(dest)).toEqual(mach("engine release", "new signature fixture"));
    writeFileSync(dest, mach("other program!", "new signature fixture"));
    expect(() => verifySignedEngine(dest,payloadSha256)).toThrow("sidecar_signed_payload_mismatch");
  } finally { rmSync(root,{recursive:true,force:true}); }
});

test("an unrelated upload member, indirect path or occupied destination never becomes the signing input", () => {
  const root = realpathSync(mkdtempSync(join(tmpdir(), "kit-engine-shapes-")));
  try {
    const app=join(root,"bundle"), macos=join(app,"Contents/MacOS"), inbox=join(root,"in"), goose=join(inbox,"goose");
    mkdirSync(macos,{recursive:true}); mkdirSync(inbox);
    const bytes=mach("payload","signature"); writeFileSync(goose,bytes);
    const {sha256,payloadSha256}=engineIdentity(goose);
    const receive=(dir=inbox,bundle=app)=>receiveEngine(dir,bundle,sha256,payloadSha256);
    writeFileSync(join(inbox,"extra"),"unexpected"); expect(()=>receive()).toThrow("sidecar_unexpected_members"); unlinkSync(join(inbox,"extra"));
    const original=join(root,"original"); renameSync(goose,original); symlinkSync(original,goose);
    expect(()=>receive()).toThrow("sidecar_indirect_path"); unlinkSync(goose); linkSync(original,goose);
    expect(()=>receive()).toThrow("sidecar_unsafe_input"); unlinkSync(goose); writeFileSync(goose,bytes);
    symlinkSync(inbox,join(root,"alias")); expect(()=>receive(join(root,"alias"))).toThrow("sidecar_indirect_path");
    expect(()=>receive(inbox, app+"/../bundle")).toThrow("sidecar_indirect_path");
    const wrongCpu=Buffer.from(bytes); wrongCpu.writeUInt32LE(0x01000007,4); writeFileSync(goose,wrongCpu);
    expect(()=>engineIdentity(goose)).toThrow("sidecar_not_arm64_executable"); writeFileSync(goose,bytes);
    expect(readdirSync(macos)).toEqual([]);
    symlinkSync(original,join(macos,"goose")); expect(()=>receive()).toThrow();
    expect(lstatSync(join(macos,"goose")).isSymbolicLink()).toBe(true); expect(readFileSync(original)).toEqual(bytes);
    expect(readdirSync(macos)).toEqual(["goose"]);
  } finally { rmSync(root,{recursive:true,force:true}); }
});

test("the unprivileged build seals one binary; the signer verifies that producer before its keychain and Apple", () => {
  type Job={ steps:Array<{id?:string;run?:string;with?:Record<string,string>;env?:Record<string,string>}>;env:Record<string,string>;outputs:Record<string,string> };
  const {env,jobs}=Bun.YAML.parse(readFileSync(join(import.meta.dir,"../../.github/workflows/release.yml"),"utf8")) as {env?:Record<string,string>;jobs:Record<string,Job>};
  const build=jobs['build-macos-app']!,sign=jobs['sign-notarize-macos']!;const cmds=(job:Job)=>job.steps.map(s=>s.run??"");
  const source=cmds(build),sink=cmds(sign);
  expect(source.filter((s:string)=>s.includes("build-ops act app-build"))).toHaveLength(1);
  const flag="SCRIPT_KIT_AGENT_OWNED_COMPILER_SAMPLE", producer=build.steps.find(step=>step.run?.includes("build-ops act app-build"));
  expect(env?.[flag]).toBeUndefined();expect(Object.values(jobs).every(job=>job.env?.[flag]===undefined)).toBe(true);
  if (!producer) throw Error("missing unprivileged app build");
  expect(Object.values(jobs).flatMap(job=>job.steps??[]).filter(step=>step.env?.[flag]!==undefined)).toEqual([producer]);
  expect(producer?.env?.[flag]).toBe("1");
  expect(source.find((s:string)=>s.includes("CARGO_BUNDLE_SKIP_BUILD"))).toContain('install -m 0755 "$BIN" target/release/script-kit-gpui');
  expect(JSON.stringify(build)).not.toContain("secrets.");expect(sink.some((s:string)=>s.includes("build-ops act app-build") || s.includes("cargo bundle"))).toBe(false);
  const at=(part:string)=>{const i=sink.findIndex((s:string)=>s.includes(part));expect(i).toBeGreaterThanOrEqual(0);return i;};
  expect(at("release-compile.ts verify-executable")).toBeLessThan(at("security import"));expect(at("release-compile.ts verify-signed-payload")).toBeLessThan(at("notarytool submit"));
  expect(sign.env.EXPECTED_COMPILE_RECEIPT).toBe("${{ needs.build-macos-app.outputs.compile_receipt_sha }}");
  const engine=jobs["build-goose-sidecar"]!;
  expect(engine.outputs).toEqual({engine_id:"${{ steps.upload-engine.outputs.artifact-id }}",engine_sha256:"${{ steps.prepare-engine.outputs.sha256 }}",engine_payload:"${{ steps.prepare-engine.outputs.payload_sha256 }}"});
  expect(engine.steps.find(s=>s.id==="upload-engine")?.with).toMatchObject({path:"${{ runner.temp }}/release-engine/goose","if-no-files-found":"error"});
  expect(sign.env.EXPECTED_ENGINE_SHA256).toBe("${{ needs.build-goose-sidecar.outputs.engine_sha256 }}");
  expect(sign.env.EXPECTED_ENGINE_PAYLOAD).toBe("${{ needs.build-goose-sidecar.outputs.engine_payload }}");
  expect(sign.steps.map(s=>s.with?.['artifact-ids']).filter(Boolean)).toEqual(["${{ needs.build-macos-app.outputs.design_id }}","${{ needs.validate-release-gates.outputs.proof_id }}","${{ needs.build-macos-app.outputs.unsigned_id }}","${{ needs.build-goose-sidecar.outputs.engine_id }}"]);
  const inputs={DESIGN_ID:"1",PROOF_ID:"2",UNSIGNED_ID:"3",ENGINE_ID:"4",EXPECTED_COMPILE_RECEIPT:hash("receipt"),EXPECTED_ENGINE_SHA256:hash("ready"),EXPECTED_ENGINE_PAYLOAD:hash("program")};
  const admission=sink[at("Missing or invalid release producer outputs")]!;
  const admitted=(env:Record<string,string>)=>spawnSync("/bin/bash",["-e","-c",admission],{env}).status;
  expect(admitted(inputs)).toBe(0);
  for(const field of Object.keys(inputs)) expect(admitted({...inputs,[field]:""})).toBe(1);
  expect(at("release-sidecar.ts receive")).toBeLessThan(at("security create-keychain"));
  expect(at('SCRIPT_KIT_EXPECTED_GOOSE_SHA256="$EXPECTED_ENGINE_SHA256"')).toBeLessThan(at("security create-keychain"));
  expect(at("release-sidecar.ts verify-signed")).toBeLessThan(at("notarytool submit"));
  expect(sink.find(s=>s.includes('CHECK="$RUNNER_TEMP/installed-workspace-check.json"'))).toContain('verify-signed "$APP/Contents/MacOS/goose" "$EXPECTED_ENGINE_PAYLOAD"');
});
