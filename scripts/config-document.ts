// A deliberately small, non-executing language: an optional SDK type import,
// then a default-exported literal object. Unsupported TS remains user-owned.
// Offsets are UTF-16 source offsets, never offsets into reformatted text.
import { createHash, randomUUID } from 'node:crypto';
import * as fs from 'node:fs';
import * as path from 'node:path';
import * as os from 'node:os';

function checkedSettingsPath(value: string, kind: 'workspace' | 'config'): string {
  // Check the supplied spelling before join() can erase a link followed by '..'.
  if (!path.isAbsolute(value) || value.split(process.platform === 'win32' ? /[\\/]/ : '/').includes('..'))
    throw new Error(`Settings require an absolute ${kind} path without '..'`);
  return value;
}

export function configPath() {
  const home = os.homedir();
  // Match setup::get_kit_path for ordinary home/env expansions. Never use
  // the process CWD as an implicit settings home or helper directory.
  let file = process.env.SCRIPT_KIT_CONFIG_PATH;
  if (file === undefined) {
    const root = process.env.SK_PATH ?? path.join(home, '.scriptkit');
    let expanded = root.replace(
      /\$\{([A-Za-z_][A-Za-z0-9_]*)\}|\$([A-Za-z_][A-Za-z0-9_]*)/g,
      (_, braced, bare) => {
        const replacement = process.env[braced ?? bare];
        if (replacement === undefined)
          throw new Error('Unknown variable in SK_PATH; use an absolute workspace path');
        return replacement;
      },
    );
    if (expanded.includes('$') || (expanded.startsWith('~') && !root.startsWith('~')))
      throw new Error('Unsupported expansion in SK_PATH; use an absolute workspace path');
    expanded = expanded.replace(/^~(?=\/|$)/, () => home);
    file = path.join(checkedSettingsPath(expanded, 'workspace'), 'config.ts');
  }
  return checkedSettingsPath(file, 'config');
}

/** Select a concrete executable, never a project/manager shim. Return the resolved
 * pathname that the SDK must spawn, not the possibly redirected name we checked.
 * For inherited PATH candidates, exclude the invoking directory and its contents. */
export function directSettingsBun(file: string, excludeCwd?: string): string | undefined {
  if (!path.isAbsolute(file)) return;
  const shim = (directory: string) => {
    const parts = directory.toLowerCase().split(path.sep);
    return parts.includes('shims') || parts.some((p, i) => p === '.bin' && parts[i - 1] === 'node_modules');
  };
  try {
    const cwd = excludeCwd === undefined ? undefined : fs.realpathSync.native(excludeCwd);
    const inside = (p: string) => {
      if (cwd === undefined) return false;
      const rel = path.relative(cwd, p);
      return !rel || (cwd !== path.parse(cwd).root && rel !== '..' && !rel.startsWith('..' + path.sep) && !path.isAbsolute(rel));
    };
    // realpath alone loses the middle of bin/bun -> shims/bun -> mise. Check directory
    // links too, and apply '..' to the directory reached, not to the unexpanded link.
    let hops = 0;
    const packageDirs = new Set<string>(); // node_modules itself may link to a package store
    const walk = (base: string, input: string, directory: boolean): string | undefined => {
      const root = path.parse(input).root;
      if (root && !path.isAbsolute(input)) return;
      let here = root || base;
      const parts = input.slice(root.length).split(path.sep === '/' ? '/' : /[\\/]/).filter(Boolean);
      for (let i = 0; i < parts.length; i++) {
        const part = parts[i];
        if (part === '.') continue;
        if (part === '..') { here = path.dirname(here); continue; }
        const next = path.join(here, part);
        const dir = directory || i + 1 < parts.length;
        const name = part.toLowerCase();
        if (inside(next) || (dir && (shim(next) || (name === '.bin' && packageDirs.has(here))))) return;
        const st = fs.lstatSync(next);
        if (st.isSymbolicLink()) {
          if (++hops > 40) return;
          const dest = walk(here, fs.readlinkSync(next), dir);
          if (dest === undefined) return;
          here = dest;
        } else {
          if (dir && !st.isDirectory()) return;
          here = next;
        }
        if (dir && name === 'node_modules') packageDirs.add(here);
      }
      return here;
    };
    if (inside(path.dirname(file))) return;
    const walked = walk('', file, false);
    if (walked === undefined) return;
    // Canonicalize the path we examined. Bun may collapse a caller's literal link/..
    // before realpath sees it; that would name a different executable.
    const resolved = fs.realpathSync.native(walked);
    if (inside(path.dirname(resolved)) || shim(path.dirname(resolved))) return;
    fs.accessSync(resolved, fs.constants.X_OK);
    return fs.statSync(resolved).isFile() ? resolved : undefined;
  } catch {
    return;
  }
}

interface Member {
  start: number;
  node: Node;
  comma?: number;
}
interface Node {
  start: number;
  end: number;
  value: unknown;
  members?: Map<string, Member>;
}

function parse(source: string) {
  let i = 0;
  const comments: number[] = [];
  const fail = (reason: string): never => {
    throw new Error(`Unsupported config.ts at offset ${i}: ${reason}; edit this file manually`);
  };
  const take = (re: RegExp) => {
    re.lastIndex = i;
    const m = re.exec(source);
    if (m) i = re.lastIndex;
    return m?.[0];
  };
  function trivia() {
    while (true) {
      if (take(/\s+/y)) continue;
      if (source.startsWith('//', i)) {
        comments.push(i);
        take(/\/\/[^\r\n\u2028\u2029]*/y);
        continue;
      }
      if (source.startsWith('/*', i)) {
        comments.push(i);
        const end = source.indexOf('*/', i + 2);
        if (end < 0) fail('unterminated comment');
        i = end + 2;
        continue;
      }
      return;
    }
  }
  function eat(s: string) {
    trivia();
    if (!source.startsWith(s, i)) return false;
    i += s.length;
    return true;
  }
  function need(s: string) {
    if (!eat(s)) fail(`expected ${s}`);
  }
  function word() {
    trivia();
    return take(/[A-Za-z_$][A-Za-z0-9_$]*/y);
  }
  function str() {
    const q = source[i++];
    let v = '';
    while (i < source.length) {
      const c = source[i++];
      if (c === q) return v;
      if (c === '\r' || c === '\n') fail('newline in string');
      if (c !== '\\') {
        v += c;
        continue;
      }
      const e = source[i++];
      const esc: Record<string, string> = {
        n: '\n',
        r: '\r',
        t: '\t',
        b: '\b',
        f: '\f',
        '\\': '\\',
        '/': '/',
        '"': '"',
        "'": "'",
      };
      if (Object.hasOwn(esc, e)) v += esc[e];
      else if (e === 'u') {
        const hex = take(/[\da-fA-F]{4}/y);
        if (!hex) fail('invalid Unicode escape');
        v += String.fromCharCode(Number.parseInt(hex!, 16));
      } else fail('unsupported string escape');
    }
    return fail('unterminated string');
  }
  function literal(depth: number): Node {
    if (depth > 80) fail('object nesting limit');
    trivia();
    const start = i;
    let value: unknown;
    let members: Map<string, Member> | undefined;
    if (eat('{')) {
      const record = Object.create(null);
      value = record;
      members = new Map();
      while (!eat('}')) {
        const keyStart = i;
        const key = source[i] === '"' || source[i] === "'" ? str() : word();
        if (key === undefined)
          return fail('expected a literal property name (no spread or computed keys)');
        if (key === '__proto__' || members.has(key)) fail('ambiguous or reserved property');
        need(':');
        const node = literal(depth + 1);
        record[key] = node.value;
        const member: Member = { start: keyStart, node };
        members.set(key, member);
        if (eat(',')) member.comma = i - 1;
        else {
          need('}');
          break;
        }
      }
    } else if (eat('[')) {
      const list: unknown[] = [];
      value = list;
      while (!eat(']')) {
        list.push(literal(depth + 1).value);
        if (!eat(',')) {
          need(']');
          break;
        }
      }
    } else if (source[i] === '"' || source[i] === "'") value = str();
    else {
      const number = take(/-?(?:0|[1-9]\d*)(?:\.\d+)?(?:[eE][+-]?\d+)?/y);
      if (number !== undefined) {
        value = Number(number);
        if (!Number.isFinite(value)) fail('non-finite number');
      } else
        switch (word()) {
          case 'true':
            value = true;
            break;
          case 'false':
            value = false;
            break;
          case 'null':
            value = null;
            break;
          case 'undefined':
            value = undefined;
            break;
          default:
            return fail('expected a data literal, not executable TypeScript');
        }
    }
    return { start, end: i, value, members };
  }
  let head = word();
  if (head === 'import') {
    if (word() !== 'type') fail('only the SDK type import is supported');
    need('{');
    if (word() !== 'Config') fail('expected Config type');
    need('}');
    if (word() !== 'from') fail('expected SDK type import');
    trivia();
    if (!['"', "'"].includes(source[i]) || str() !== '@scriptkit/sdk') fail('expected @scriptkit/sdk');
    const importEnd = i;
    if (!eat(';') && !/[\n\r\u2028\u2029]/.test(source.slice(importEnd, i)))
      fail('expected semicolon or newline after type import');
    head = word();
  }
  if (head !== 'export' || word() !== 'default') fail('expected export default { ... }');
  const root = literal(0);
  if (!root.members) fail('expected an object');
  const annotation = word();
  if (annotation !== undefined && (!['satisfies', 'as'].includes(annotation) || word() !== 'Config'))
    fail('unsupported type annotation');
  eat(';');
  trivia();
  if (i !== source.length) fail('unexpected expression or statement');
  return { root, comments };
}

export function values(source: string): Record<string, unknown> {
  return parse(source).root.value as Record<string, unknown>;
}

/** Set/remove exactly one path. Object/array replacements never silently discard comments. */
export function editLiteral(source: string, parts: string[], value: unknown): string {
  const { root, comments } = parse(source);
  let parent = root;
  for (let k = 0; k < parts.length; k++) {
    const key = parts[k];
    if (key === '__proto__' || !key) throw new Error('Invalid config key');
    if (!parent.members) throw new Error(`Expected a literal object at ${parts.slice(0, k).join('.')}`);
    const entry = parent.members.get(key);
    if (entry && k + 1 < parts.length) {
      parent = entry.node;
      continue;
    }
    if (entry) {
      const start = value === undefined ? entry.start : entry.node.start;
      const end = entry.node.end;
      if (comments.some((offset) => offset >= start && offset < end))
        throw new Error('Change would discard comments inside this value; edit it manually');
      const tail = source.slice(end);
      if (value !== undefined) return source.slice(0, start) + JSON.stringify(value) + tail;
      // Remove the following comma separately so any intervening comment survives.
      const after =
        entry.comma === undefined
          ? tail
          : source.slice(end, entry.comma) + source.slice(entry.comma + 1);
      return source.slice(0, start) + after;
    }
    if (value === undefined) return source;
    const indent = '  '.repeat(k + 1);
    let nested = JSON.stringify(value);
    for (let j = parts.length - 1; j >= k; j--) {
      const name = /^[a-zA-Z_$][\w$]*$/.test(parts[j]) ? parts[j] : JSON.stringify(parts[j]);
      nested = `${name}: ${nested}`;
      if (j > k) nested = `{\n${'  '.repeat(j + 1)}${nested}\n${'  '.repeat(j)}}`;
    }
    const close = parent.end - 1;
    let before = source.slice(0, close);
    const last = [...parent.members.values()].at(-1);
    if (last && last.comma === undefined)
      before = before.slice(0, last.node.end) + ',' + before.slice(last.node.end);
    return `${before}\n${indent}${nested}\n${'  '.repeat(k)}${source.slice(close)}`;
  }
  throw new Error('Missing config path');
}

const MAX_BYTES = 2 * 1024 * 1024;
export function revision(source: string | null) {
  return source === null ? 'missing' : createHash('sha256').update(source).digest('hex');
}
function snapshot(file: string) {
  let fd: number;
  try {
    fd = fs.openSync(file, fs.constants.O_RDONLY | fs.constants.O_NOFOLLOW | fs.constants.O_NONBLOCK);
  } catch (e) {
    if ((e as NodeJS.ErrnoException).code === 'ENOENT') return null;
    throw e;
  }
  try {
    const stat = fs.fstatSync(fd);
    if (!stat.isFile() || stat.nlink !== 1 || stat.size > MAX_BYTES)
      throw new Error('Config must be a single regular file under 2 MiB');
    const source = new TextDecoder('utf-8', { fatal: true, ignoreBOM: true }).decode(
      fs.readFileSync(fd),
    );
    return { source, mode: stat.mode & 0o777, dev: stat.dev, ino: stat.ino };
  } finally {
    fs.closeSync(fd);
  }
}
export function readSource(file: string) {
  return snapshot(checkedSettingsPath(file, 'config'))?.source ?? null;
}

/** A revision comparison actually ran and rejected this write. A lock or I/O failure is not this error. */
export class StaleConfigRevision extends Error {
  readonly code = 'stale_revision';
  constructor() {
    super('Stale config revision; reread the config and review the change');
    this.name = 'StaleConfigRevision';
  }
}

/** Ownership or staged bytes changed before publication. This is not a revision conflict. */
export class DisplacedSettingsWrite extends Error {
  readonly code = 'settings_write_displaced';
  constructor() {
    super('Settings write stopped: its lock or staged file changed. Wait for the active writer, then reread the config.');
    this.name = 'DisplacedSettingsWrite';
  }
}
export type SettingsWriteErrorCode = StaleConfigRevision['code'] | DisplacedSettingsWrite['code'];

// The open descriptor pins its inode, even if the name is removed and reused.
function namesDescriptor(name: string, fd: number): boolean {
  const entry = fs.lstatSync(name, { bigint: true, throwIfNoEntry: false });
  if (!entry) return false;
  const held = fs.fstatSync(fd, { bigint: true });
  return entry.dev === held.dev && entry.ino === held.ino;
}

function stagedMatches(fd: number, expected: Buffer, mode: number): boolean {
  const stat = fs.fstatSync(fd);
  if (stat.nlink !== 1 || stat.size !== expected.length || (stat.mode & 0o7777) !== mode) return false;
  const actual = Buffer.alloc(expected.length); // already bounded by MAX_BYTES
  for (let pos = 0; pos < actual.length;) {
    const n = fs.readSync(fd, actual, pos, actual.length - pos, pos);
    if (!n) return false;
    pos += n;
  }
  return actual.equals(expected);
}

/** Atomic publication and optimistic concurrency between Kit settings writers.
 * Noncooperating editors are checked again immediately before publication.
 * A stale lock after process death is a refusal, never broken on a timer. */
export function mutate(
  file: string,
  expected: string | undefined,
  change: (old: string | null) => string | null,
) {
  const dir = path.dirname(checkedSettingsPath(file, 'config'));
  fs.mkdirSync(dir, { recursive: true, mode: 0o700 });
  const lock = `${file}.settings-lock`;
  try {
    fs.mkdirSync(lock, { mode: 0o700 });
  } catch (e) {
    if ((e as NodeJS.ErrnoException).code === 'EEXIST')
      throw new Error(`Settings writer busy: ${lock}; retry, or inspect the lock if a writer crashed`);
    throw e;
  }
  // Do not clean up by name if opening the lock failed: its owner is unknown.
  const lockFd = fs.openSync(lock, fs.constants.O_RDONLY | fs.constants.O_DIRECTORY | fs.constants.O_NOFOLLOW);
  const checkLock = () => {
    if (!namesDescriptor(lock, lockFd)) throw new DisplacedSettingsWrite();
  };
  let disturbedStage = false;
  try {
    const old = snapshot(file), source = old?.source ?? null;
    const checkSource = () => {
      const current = snapshot(file);
      checkLock();
      if (current?.source !== old?.source || current?.ino !== old?.ino ||
          current?.dev !== old?.dev || current?.mode !== old?.mode)
        throw new Error('Stale config; changed during edit');
    };
    checkLock();
    if (expected !== undefined && expected !== revision(source))
      throw new StaleConfigRevision();
    // Parse even for reset-all: never erase a dynamic config we cannot inspect.
    if (source !== null) parse(source);
    const next = change(source);
    if (next === null || next === source) {
      checkSource();
      return revision(source);
    }
    if (Buffer.byteLength(next) > MAX_BYTES) throw new Error('Config exceeds 2 MiB');
    parse(next);
    checkLock();
    const bytes = Buffer.from(next, 'utf8');
    const tmp = path.join(lock, randomUUID());
    const fd = fs.openSync(tmp, 'wx+', 0o600);
    try {
      const mode = old?.mode ?? (fs.fstatSync(fd).mode & 0o777);
      fs.writeFileSync(fd, bytes);
      if (old) fs.fchmodSync(fd, old.mode);
      fs.fsyncSync(fd);
      checkSource();
      if (!stagedMatches(fd, bytes, mode) || !namesDescriptor(tmp, fd)) {
        disturbedStage = true;
        throw new DisplacedSettingsWrite();
      }
      checkLock();
      if (old) fs.renameSync(tmp, file);
      else fs.linkSync(tmp, file); // exclusive create
    } finally {
      try {
        if (!disturbedStage && namesDescriptor(lock, lockFd) && namesDescriptor(tmp, fd))
          fs.unlinkSync(tmp);
      } finally { fs.closeSync(fd); }
    }
    return revision(next);
  } finally {
    try {
      // Never remove a new owner's lock, or discard an unrecognized draft left inside ours.
      if (namesDescriptor(lock, lockFd)) fs.rmdirSync(lock);
    } catch (err) {
      if (!disturbedStage || (err as NodeJS.ErrnoException).code !== 'ENOTEMPTY') throw err;
    } finally { fs.closeSync(lockFd); }
  }
}
