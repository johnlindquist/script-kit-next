// A deliberately small, non-executing language: an optional SDK type import,
// then a default-exported literal object. Unsupported TS remains user-owned.
// Offsets are UTF-16 source offsets, never offsets into reformatted text.
import { createHash, randomUUID } from 'node:crypto';
import * as fs from 'node:fs';
import * as path from 'node:path';
import * as os from 'node:os';

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
    if (!path.isAbsolute(expanded)) throw new Error('Settings require an absolute workspace path');
    file = path.join(expanded, 'config.ts');
  }
  if (!path.isAbsolute(file)) throw new Error('Settings require an absolute config path');
  return file;
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
    const dir = path.dirname(file);
    const resolvedDir = fs.realpathSync.native(dir);
    const resolved = fs.realpathSync.native(file);
    if ([dir, resolvedDir, path.dirname(resolved)].some(shim)) return;
    if (excludeCwd !== undefined) {
      const cwd = fs.realpathSync.native(excludeCwd);
      const inside = (p: string) => {
        const rel = path.relative(cwd, p);
        // The filesystem root as cwd does not make every installed program project-local.
        return !rel || (cwd !== path.parse(cwd).root && rel !== '..' && !rel.startsWith('..' + path.sep) && !path.isAbsolute(rel));
      };
      if ([dir, resolvedDir, resolved].some(inside)) return;
    }
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
      const list = [];
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
  return snapshot(file)?.source ?? null;
}

/** A revision comparison actually ran and rejected this write. A lock or I/O failure is not this error. */
export class StaleConfigRevision extends Error {
  readonly code = 'stale_revision';
  constructor() {
    super('Stale config revision; reread the config and review the change');
    this.name = 'StaleConfigRevision';
  }
}

/** Atomic publication and optimistic concurrency between Kit settings writers.
 * Noncooperating editors are checked again immediately before publication.
 * A stale lock after process death is a refusal, never broken on a timer. */
export function mutate(
  file: string,
  expected: string | undefined,
  change: (old: string | null) => string | null,
) {
  const dir = path.dirname(file);
  fs.mkdirSync(dir, { recursive: true, mode: 0o700 });
  const lock = `${file}.settings-lock`;
  try {
    fs.mkdirSync(lock, { mode: 0o700 });
  } catch (e) {
    if ((e as NodeJS.ErrnoException).code === 'EEXIST')
      throw new Error(`Settings writer busy: ${lock}; retry, or inspect the lock if a writer crashed`);
    throw e;
  }
  try {
    const old = snapshot(file),
      source = old?.source ?? null;
    if (expected !== undefined && expected !== revision(source))
      throw new StaleConfigRevision();
    // Parse even for reset-all: never erase a dynamic config we cannot inspect.
    if (source !== null) parse(source);
    const next = change(source);
    if (next === null || next === source) return revision(source);
    if (Buffer.byteLength(next) > MAX_BYTES) throw new Error('Config exceeds 2 MiB');
    parse(next);
    const tmp = path.join(lock, randomUUID());
    const fd = fs.openSync(tmp, 'wx', 0o600);
    try {
      try {
        fs.writeFileSync(fd, next, 'utf8');
        if (old) fs.fchmodSync(fd, old.mode);
        fs.fsyncSync(fd);
      } finally {
        fs.closeSync(fd);
      }
      const current = snapshot(file);
      if (
        current?.source !== old?.source ||
        current?.ino !== old?.ino ||
        current?.dev !== old?.dev ||
        current?.mode !== old?.mode
      )
        throw new Error('Stale config; changed during edit');
      if (old) fs.renameSync(tmp, file);
      else fs.linkSync(tmp, file); // exclusive create
    } finally {
      fs.rmSync(tmp, { force: true });
    }
    return revision(next);
  } finally {
    fs.rmdirSync(lock);
  }
}
