#!/usr/bin/env bun
/** One explicitly trusted target-agent root; nested links remain forbidden. */
import { constants, closeSync, fstatSync, fsyncSync, lstatSync, openSync, readFileSync, readlinkSync, realpathSync, writeFileSync } from "node:fs";
import { dirname, isAbsolute, join, resolve, sep } from "node:path";

const policyName = ".agent-cargo-storage.json";
interface StorageRoute {
  schemaVersion: 1;
  repositoryRoot: string;
  linkedRoot: "target-agent";
  destination: string;
  linkDevice: number;
  linkInode: number;
  destinationDevice: number;
  destinationInode: number;
  ownerUid: number;
}
function uid(): number {
  if (!process.getuid) throw new Error("managed_storage_owner_unavailable");
  return process.getuid();
}
function statIfPresent(path: string) {
  try { return lstatSync(path); }
  catch (error) { if ((error as NodeJS.ErrnoException).code === "ENOENT") return null; throw error; }
}
function ordinaryComponents(path: string): void {
  const absolute = resolve(path);
  let cursor = sep;
  for (const part of absolute.slice(sep.length).split(sep)) {
    cursor = join(cursor, part);
    const stat = statIfPresent(cursor);
    if (!stat) break;
    if (stat.isSymbolicLink()) throw new Error(`managed_storage_destination_symlink:${cursor}`);
  }
}
function routeFacts(repositoryRoot: string, destination: string): StorageRoute {
  const root = realpathSync(repositoryRoot), link = join(root, "target-agent");
  ordinaryComponents(root);
  if (!isAbsolute(destination) || resolve(destination) !== destination) throw new Error("managed_storage_noncanonical_destination");
  ordinaryComponents(destination);
  const target = lstatSync(destination), source = lstatSync(link);
  if (!source.isSymbolicLink() || readlinkSync(link) !== destination || realpathSync(link) !== destination) throw new Error("managed_storage_link_destination_mismatch");
  if (!target.isDirectory() || target.uid !== uid() || source.uid !== uid() || (target.mode & 0o022)) throw new Error("managed_storage_destination_ownership");
  if (destination === root || root.startsWith(destination + sep) || destination === join(root, "target") || destination.startsWith(join(root, "target") + sep)) throw new Error("managed_storage_destination_conflict");
  return { schemaVersion: 1, repositoryRoot: root, linkedRoot: "target-agent", destination,
    linkDevice: source.dev, linkInode: source.ino, destinationDevice: target.dev,
    destinationInode: target.ino, ownerUid: uid() };
}
export function trustedManagedRootLink(path: string): StorageRoute {
  const root = dirname(path);
  if (path !== join(root, "target-agent")) throw new Error(`protected cache ownership cannot follow a symlink: ${path}`);
  ordinaryComponents(root);
  const file = join(root, policyName);
  const fd = openSync(file, constants.O_RDONLY | constants.O_NOFOLLOW);
  let policy: StorageRoute;
  try {
    const stat = fstatSync(fd);
    if (!stat.isFile() || stat.nlink !== 1 || stat.uid !== uid() || (stat.mode & 0o022) || stat.size > 16_384) throw new Error("managed_storage_policy_ownership");
    policy = JSON.parse(readFileSync(fd, "utf8"));
  } finally { closeSync(fd); }
  if (policy.schemaVersion !== 1 || policy.repositoryRoot !== root || policy.linkedRoot !== "target-agent" || typeof policy.destination !== "string") throw new Error("managed_storage_policy_invalid");
  const actual = routeFacts(root, policy.destination);
  for (const key of Object.keys(actual) as Array<keyof StorageRoute>) if (policy[key] !== actual[key]) throw new Error(`managed_storage_route_changed:${key}`);
  return actual;
}
export function assertManagedPathComponents(path: string): void {
  let cursor = sep;
  for (const part of resolve(path).slice(sep.length).split(sep)) {
    cursor = join(cursor, part);
    const stat = statIfPresent(cursor);
    if (!stat) break;
    if (stat.isSymbolicLink()) trustedManagedRootLink(cursor);
  }
}
export function managedStorageRoot(repositoryRoot: string): string {
  const path = join(realpathSync(repositoryRoot), "target-agent");
  const stat = statIfPresent(path);
  if (stat?.isSymbolicLink()) return trustedManagedRootLink(path).destination;
  assertManagedPathComponents(path);
  if (stat && !stat.isDirectory()) throw new Error("managed_storage_root_not_directory");
  return stat ? path : realpathSync(repositoryRoot);
}
export function registerManagedStorageRoute(repositoryRoot: string, destination: string): StorageRoute {
  const route = routeFacts(repositoryRoot, destination);
  // Registration creates local authority only; never moves storage or replaces a policy.
  const file = join(route.repositoryRoot, policyName);
  const fd = openSync(file, constants.O_CREAT | constants.O_EXCL | constants.O_WRONLY | constants.O_NOFOLLOW, 0o600);
  try { writeFileSync(fd, `${JSON.stringify(route, null, 2)}\n`); fsyncSync(fd); } finally { closeSync(fd); }
  return trustedManagedRootLink(join(route.repositoryRoot, "target-agent"));
}
if (import.meta.main) {
  try {
    const [operation, rootOrPath, destination, ...extra] = process.argv.slice(2);
    if (!rootOrPath) throw new Error("usage: managed-storage.ts validate <absolute-path>... | inspect <repository> | trust <repository> <absolute-destination>");
    if (operation === "validate") {
      const paths = process.argv.slice(3);
      for (const path of paths) assertManagedPathComponents(path);
      console.log(JSON.stringify({ status: "ok", paths }));
    }
    else if (operation === "inspect" && !destination && !extra.length) console.log(JSON.stringify({ status: "ok", storageRoot: managedStorageRoot(rootOrPath) }));
    else if (operation === "trust" && destination && !extra.length) console.log(JSON.stringify({ status: "ok", route: registerManagedStorageRoute(rootOrPath, destination) }));
    else throw new Error("managed_storage_arguments_invalid");
  } catch (error) { console.error(`MANAGED_STORAGE refused: ${error instanceof Error ? error.message : "unknown"}`); process.exitCode = 64; }
}
