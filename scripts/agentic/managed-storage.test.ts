import { afterEach, expect, test } from "bun:test";
import { chmodSync, mkdirSync, mkdtempSync, readFileSync, realpathSync, renameSync, rmSync, symlinkSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { assertManagedPathComponents, managedStorageRoot, registerManagedStorageRoute } from "./managed-storage.ts";

const roots: string[] = [];
afterEach(() => { for (const root of roots.splice(0)) rmSync(root, { recursive: true, force: true }); });
function fixture() {
  const base = realpathSync(mkdtempSync(join(tmpdir(), "script-kit-storage-route-")));
  roots.push(base);
  const root = join(base, "repository"), destination = join(base, "storage"), link = join(root, "target-agent");
  mkdirSync(root); mkdirSync(destination); symlinkSync(destination, link);
  return { root, destination, link, policy: join(root, ".agent-cargo-storage.json") };
}
test("an unregistered root link stays unavailable", () => {
  const item = fixture();
  expect(() => managedStorageRoot(item.root)).toThrow();
});
test("replacement link and destination identities require renewed explicit trust", () => {
  const item = fixture();
  registerManagedStorageRoute(item.root, item.destination);
  expect(managedStorageRoot(item.root)).toBe(item.destination);
  renameSync(item.link, join(item.root, "old-link"));
  symlinkSync(item.destination, item.link);
  expect(() => managedStorageRoot(item.root)).toThrow("managed_storage_route_changed:linkInode");
  rmSync(item.link); renameSync(join(item.root, "old-link"), item.link);
  renameSync(item.destination, `${item.destination}-old`); mkdirSync(item.destination);
  expect(() => managedStorageRoot(item.root)).toThrow("managed_storage_route_changed:destinationInode");
});
test("only the pinned root component is trusted", () => {
  const item = fixture();
  registerManagedStorageRoute(item.root, item.destination);
  mkdirSync(join(item.destination, "pools"));
  symlinkSync(item.root, join(item.destination, "pools/agent-debug"));
  expect(() => assertManagedPathComponents(join(item.link, "pools/agent-debug/unknown"))).toThrow("protected cache ownership cannot follow a symlink");
});
test("unsafe authority and copying authority to another repository are rejected", () => {
  const item = fixture();
  registerManagedStorageRoute(item.root, item.destination);
  chmodSync(item.policy, 0o666);
  expect(() => managedStorageRoot(item.root)).toThrow("managed_storage_policy_ownership");
  chmodSync(item.policy, 0o600);
  const other = join(item.root, "other"); mkdirSync(other);
  symlinkSync(item.destination, join(other, "target-agent"));
  writeFileSync(join(other, ".agent-cargo-storage.json"), readFileSync(item.policy), { mode: 0o600 });
  expect(() => managedStorageRoot(other)).toThrow("managed_storage_policy_invalid");
  expect(() => registerManagedStorageRoute(item.root, item.destination)).toThrow();
});
