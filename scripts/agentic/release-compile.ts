#!/usr/bin/env bun
// Package the already-built application and prove the signer received the same input.
// Only seal emits a new receipt. Verification is read-only, including on retries.
import { lstatSync, realpathSync, writeFileSync } from "node:fs";
import { relative, resolve } from "node:path";
import { artifactHash, canonicalArtifactPath, hashArtifactFile, receiveReleaseCompile, unsignedMachOPayloadSha256, verifyImmutableArtifact, type ArtifactReference } from "./build-artifact";
import type { BuildResourceReport } from "../devtools/lib/build-ops-inventory";
import { canonicalJson, managedTaskRecordPath, readManagedTask, readOwnedJson } from "./artifact-lifecycle";
import { githubReleaseRun, verifyReleaseCompile, type ReleaseCompileReceipt, type ReleaseRun, type VerifiedReleaseCompile } from "./release-compile-contract";

export function checkReleaseZip(input: VerifiedReleaseCompile, zip: string): string {
  if (hashArtifactFile(zip) !== input.receipt.zipSha256) throw Error("release_compile_zip_mismatch");
  return input.receipt.zipSha256;
}

export function checkReleaseExecutable(input: VerifiedReleaseCompile, exe: string): string {
  const expected = input.receipt.manifest, stat = lstatSync(exe);
  if (!stat.isFile() || stat.nlink !== 1 || !(stat.mode & 0o111) || stat.size !== expected.sizeBytes || hashArtifactFile(exe) !== expected.binarySha256 ||
      unsignedMachOPayloadSha256(exe) !== input.receipt.payloadSha256) throw Error("release_compile_executable_mismatch");
  return expected.binarySha256;
}

/** A signing operation can change signature bytes, not the program that goes to Apple. */
export function checkSignedPayload(input: VerifiedReleaseCompile, executable: string): string {
  const actual = unsignedMachOPayloadSha256(executable);
  if (actual !== input.receipt.payloadSha256) throw Error("release_compile_signed_payload_mismatch");
  return actual;
}

export function sealReleaseCompile(root: string, ref: ArtifactReference, bundleExe: string, zip: string, producer: ReleaseRun): VerifiedReleaseCompile {
  const verified = verifyImmutableArtifact(root, ref, { kind: "application", packageName: "script-kit-gpui", targetName: "script-kit-gpui", profile: "release", sourcePolicy: "clean-exact-head" });
  const m = verified.manifest, build = m.publication.buildTask;
  const task = readManagedTask(managedTaskRecordPath(root, build), build), resources = task.result.resources as BuildResourceReport | undefined;
  if (task.state !== "closed" || task.result.status !== "succeeded" || task.result.exitCode !== 0 || !task.cleanup.closed || !task.cleanup.referencesFinalized ||
    !task.cleanup.processExited || !task.cleanup.processGroupExited || !task.cleanup.streamsDrained || task.cleanup.survivors.length || task.cleanup.failureCodes.length ||
    resources?.refusal !== null || resources.monitoring?.complete !== true || resources.monitoring?.workerClosed !== true ||
    !Number.isSafeInteger(resources.monitoring.sampleCount) || resources.monitoring.sampleCount < 1) throw Error("release_compile_incomplete_build");
  const receipt: ReleaseCompileReceipt = { schemaVersion: 1, producer, reference: ref, manifest: m,
    finalized: { identity: task.identity, closed: true, status: "succeeded", monitoredSamples: resources.monitoring.sampleCount },
    zipSha256: hashArtifactFile(zip), payloadSha256: unsignedMachOPayloadSha256(verified.executablePath) };
  const bytes = canonicalJson(receipt) + "\n", input = verifyReleaseCompile(bytes, artifactHash(bytes), producer);
  checkReleaseExecutable(input, bundleExe);
  return input;
}

if (import.meta.main) {
  const [verb, ...args] = process.argv.slice(2);
  if (!verb || verb === "help" || verb === "--help") {
    console.log("release-compile seal <reference.json> <packaged-executable> <unsigned.zip> <new-receipt.json> | verify-zip <receipt> <producer-output-sha256> <unsigned.zip> | verify-executable <receipt> <producer-output-sha256> <packaged-executable> | verify-signed-payload <receipt> <producer-output-sha256> <signed-executable>");
  } else {
    const root = realpathSync(process.cwd()), path = (arg: string) => canonicalArtifactPath(root, relative(root, resolve(arg)));
    if (verb === "seal" && args.length === 4) {
      const run = githubReleaseRun("build-macos-app");
      const verified = sealReleaseCompile(root, readOwnedJson(path(args[0]!)) as unknown as ArtifactReference, path(args[1]!), path(args[2]!), run);
      writeFileSync(path(args[3]!), verified.bytes, { flag: "wx", mode: 0o400 });
      console.log(verified.sha256);
    } else if ((verb === "verify-zip" || verb === "verify-executable" || verb === "verify-signed-payload") && args.length === 3) {
      const input = receiveReleaseCompile(root, path(args[0]!), args[1]!, githubReleaseRun("sign-notarize-macos"));
      if (verb === "verify-zip") console.log(checkReleaseZip(input, path(args[2]!)));
      else if (verb === "verify-executable") console.log(checkReleaseExecutable(input, path(args[2]!)));
      else console.log(checkSignedPayload(input, path(args[2]!)));
    } else throw Error("invalid_release_compile_arguments");
  }
}
