import { closeSync, constants, fstatSync, linkSync, lstatSync, mkdtempSync, openSync, readdirSync, readSync, realpathSync, rmSync, chmodSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { fileSha256, unsignedMachOBytesPayloadSha256 } from "./macho-payload.ts";

// Release transport for one ready-to-sign engine. File names and modes from
// the upload do not decide what gets installed in the signing workspace.
const MAX_ENGINE = 256 * 1024 ** 2;
function need(ok: unknown, code: string): asserts ok { if (!ok) throw Error(code); }
function direct(path: string) {
  need(!path.split("/").includes("..") && realpathSync.native(path) === resolve(path), "sidecar_indirect_path");
  return path;
}
function snapshot(path: string) {
  // Reject special files before reading. Bound allocation even if a file grows.
  const fd = openSync(direct(path), constants.O_RDONLY | constants.O_NOFOLLOW | constants.O_NONBLOCK);
  try {
    const s = fstatSync(fd);
    need(s.isFile() && s.nlink === 1 && !(s.mode & 0o7000) && s.size >= 32 && s.size <= MAX_ENGINE, "sidecar_unsafe_input");
    const bytes = Buffer.allocUnsafe(s.size);
    for (let n = 0; n < bytes.length;) {
      const count = readSync(fd, bytes, n, bytes.length - n, n);
      need(count > 0, "sidecar_changed_input"); n += count;
    }
    const end = fstatSync(fd);
    need(end.size === s.size && end.nlink === 1 && end.ctimeMs === s.ctimeMs, "sidecar_changed_input");
    need(bytes.readUInt32LE(0) === 0xfeedfacf && bytes.readUInt32LE(4) === 0x0100000c && bytes.readUInt32LE(12) === 2, "sidecar_not_arm64_executable");
    return bytes;
  } finally { closeSync(fd); }
}
const digest = (s: string) => need(s.length === 64 && /^[a-f0-9]{64}$/.test(s), "sidecar_invalid_expected_digest");
export function engineIdentity(file: string) {
  const bytes = snapshot(file);
  return { sha256: fileSha256(bytes), payloadSha256: unsignedMachOBytesPayloadSha256(bytes) };
}
export function receiveEngine(inbox: string, app: string, raw: string, payload: string) {
  digest(raw); digest(payload);
  const names = readdirSync(direct(inbox));
  need(names.length === 1 && names[0] === "goose", "sidecar_unexpected_members");
  const bytes = snapshot(join(inbox, "goose"));
  need(fileSha256(bytes) === raw && unsignedMachOBytesPayloadSha256(bytes) === payload, "sidecar_producer_mismatch");
  const macos = direct(join(direct(app), "Contents/MacOS"));
  need(lstatSync(macos).isDirectory(), "sidecar_bundle_directory_missing");
  const tmp = mkdtempSync(join(macos, ".release-engine-"));
  try {
    const stage = join(tmp, "goose");
    writeFileSync(stage, bytes, { flag: "wx", mode: 0o600 });
    chmodSync(stage, 0o755);
    // Publish only the verified bytes. An existing file or dangling link is
    // not permission to overwrite another component in the unsigned bundle.
    linkSync(stage, join(macos, "goose"));
  } finally { rmSync(tmp, { recursive: true, force: true }); }
}
export function verifySignedEngine(file: string, payload: string) {
  digest(payload);
  need(unsignedMachOBytesPayloadSha256(snapshot(file)) === payload, "sidecar_signed_payload_mismatch");
}
if (import.meta.main) {
  const [op, ...args] = process.argv.slice(2);
  try {
    if (op === "identify" && args.length === 1) {
      const id = engineIdentity(args[0]!);
      // Both values come from the unprivileged producer, never from the download.
      console.log("sha256=" + id.sha256 + "\npayload_sha256=" + id.payloadSha256);
    } else if (op === "receive" && args.length === 4) {
      receiveEngine(args[0]!, args[1]!, args[2]!, args[3]!);
    } else if (op === "verify-signed" && args.length === 2) {
      verifySignedEngine(args[0]!, args[1]!);
    } else throw Error("usage: release-sidecar identify <engine> | receive <inbox> <bundle> <producer-sha256> <producer-payload> | verify-signed <engine> <producer-payload>");
  } catch (error) { console.error(error instanceof Error ? error.message : "sidecar_verification_failed"); process.exitCode = 1; }
}
