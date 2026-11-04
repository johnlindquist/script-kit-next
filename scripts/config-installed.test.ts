import { describe, expect, it } from 'bun:test';
import {
  chmodSync,
  copyFileSync,
  existsSync,
  lstatSync,
  mkdirSync,
  mkdtempSync,
  readdirSync,
  readFileSync,
  realpathSync,
  renameSync,
  rmdirSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { directSettingsBun, DisplacedSettingsWrite, editLiteral, mutate, revision } from './config-document.ts';

// No checkout-relative modules or global packages in the child. The Rust MCP
// fixture exercises the real embedded first-run installer; here we can also
// exercise contested writes and a malicious config without launching the app.
function fixture(source: string) {
  const root = mkdtempSync(join(tmpdir(), 'settings-installed-'));
  const sk = join(root, 'workspace');
  const sdk = join(sk, 'sdk');
  mkdirSync(sdk, { recursive: true });
  const config = join(sk, 'config.ts');
  for (const name of ['config-cli.ts', 'config-schema.ts', 'kit-sdk-config.ts', 'config-document.ts']) {
    const file = join(import.meta.dir, name);
    if (existsSync(file)) copyFileSync(file, join(sdk, name));
  }
  writeFileSync(config, source);
  const read = () => readFileSync(config, 'utf8');
  const run = async (...args: string[]) => {
    const proc = Bun.spawn([process.execPath, join(sdk, 'config-cli.ts'), ...args], {
      cwd: root,
      env: { HOME: root, SK_PATH: sk, PATH: '/usr/bin:/bin' },
      stdout: 'pipe',
      stderr: 'pipe',
    });
    const [code, out, err] = await Promise.all([
      proc.exited,
      new Response(proc.stdout).text(),
      new Response(proc.stderr).text(),
    ]);
    if (!out) throw new Error(err);
    return { code, ...JSON.parse(out) };
  };
  return { root, config, sdk, read, run, close: () => rmSync(root, { recursive: true, force: true }) };
}

const source = `import type { Config } from '@scriptkit/sdk';
export default {
  // editorFontSize: 8
  unrelated: {editorFontSize: 12, note: 'editorFontSize: 13, }'},
  editorFontSize: 16, // the real leaf
  dictation: { startInKit: false, },
} satisfies Config;\n`;

describe('installed settings helper', () => {
  it('does not spend a matching revision in a neighboring workspace across a link and parent segment', () => {
    const f = fixture(source);
    try {
      const other = join(f.root, 'other'), link = join(f.root, 'via');
      mkdirSync(join(other, 'anchor'), { recursive: true });
      mkdirSync(join(other, 'workspace'));
      writeFileSync(join(other, 'workspace/config.ts'), source); // same bytes and revision in both workspaces
      symlinkSync(join(other, 'anchor'), link, 'dir');
      const raw = link + '/../workspace';
      for (const key of ['SK_PATH', 'SCRIPT_KIT_CONFIG_PATH']) {
        const run = Bun.spawnSync([process.execPath, join(f.sdk, 'config-cli.ts'), 'set-json', 'editorFontSize', '27', '--if-revision', revision(source)], {
          cwd: f.root, env: { HOME: f.root, PATH: '/usr/bin:/bin', [key]: key === 'SK_PATH' ? raw : raw + '/config.ts' },
        });
        expect(run.exitCode).toBe(1);
        expect(JSON.parse(run.stdout.toString()).error).toContain("without '..'");
      }
      expect(f.read()).toBe(source);
      expect(readFileSync(join(other, 'workspace/config.ts'), 'utf8')).toBe(source);
    } finally { f.close(); }
  });
  it('binds a concrete settings runtime without following project or manager shims', () => {
    const root = realpathSync(mkdtempSync(join(tmpdir(), 'settings-bun-paths-')));
    try {
      const project = join(root, 'project');
      const pkg = join(project, 'node_modules/.bin');
      const nested = join(project, 'packages/sub');
      const trusted = join(root, 'project-tools/bin'); // common prefix, different directory
      const shims = join(root, 'manager/shims');
      const alias = join(root, 'alias');
      for (const dir of [pkg, nested, trusted, shims, alias]) mkdirSync(dir, { recursive: true });
      const good = join(trusted, 'bun');
      const decoy = join(project, 'bun');
      const runtime = realpathSync(process.execPath);
      symlinkSync(runtime, good);
      writeFileSync(decoy, 'fixture only', { mode: 0o700 });
      for (const dir of [pkg, shims]) symlinkSync(good, join(dir, 'bun'));
      symlinkSync(shims, join(root, 'shim-alias'), 'dir');
      symlinkSync(decoy, join(alias, 'bun'));
      symlinkSync(good, join(trusted, 'pinned'));
      expect(directSettingsBun(good, project)).toBe(runtime);
      expect(directSettingsBun(join(trusted, 'pinned'))).toBe(runtime);
      for (const name of [decoy, join(alias, 'bun'), join(shims, 'bun'), join(root, 'shim-alias/bun'), './bun'])
        expect(directSettingsBun(name, project)).toBeUndefined();
      expect(directSettingsBun(join(pkg, 'bun'), nested)).toBeUndefined();
      expect(directSettingsBun(join(shims, 'bun'))).toBeUndefined(); // even as BUN_PATH
    } finally {
      rmSync(root, { recursive: true, force: true });
    }
  });
  it('checks intermediate Bun links even when both ends appear to be direct installations', () => {
    const root = realpathSync(mkdtempSync(join(tmpdir(), 'settings-bun-links-')));
    try {
      const real = join(root, 'versions/v1/bin/bun');
      const bin = join(root, 'bin');
      const shim = join(root, 'manager/shims');
      const pkg = join(root, 'project/node_modules/.bin');
      for (const dir of [join(root, 'versions/v1/bin'), bin, shim, pkg]) mkdirSync(dir, { recursive: true });
      writeFileSync(real, 'fixture executable', { mode: 0o700 });
      symlinkSync('../../versions/v1/bin/bun', join(shim, 'bun'));
      symlinkSync('../manager/shims/bun', join(bin, 'via-mise'));
      symlinkSync('../../../versions/v1/bin', join(pkg, 'programs'), 'dir');
      symlinkSync('../project/node_modules/.bin/programs', join(bin, 'via-package'), 'dir');
      symlinkSync('../versions', join(root, 'manager/versions'), 'dir');
      symlinkSync('../manager/versions/v1/bin/bun', join(bin, 'direct'));
      const project = join(root, 'project');
      symlinkSync('../versions/v1/bin/bun', join(project, 'project-choice'));
      symlinkSync('../project/project-choice', join(bin, 'via-project'));
      for (const path of [join(bin, 'via-mise'), join(bin, 'via-package/bun')]) {
        expect(realpathSync(path)).toBe(real);
        expect(directSettingsBun(path)).toBeUndefined();
        expect(directSettingsBun(path, project)).toBeUndefined();
      }
      const store = join(root, 'store/modules');
      mkdirSync(join(store, '.bin'), {recursive: true});
      mkdirSync(join(root, 'linked'));
      symlinkSync('../store/modules', join(root, 'linked/node_modules'), 'dir');
      symlinkSync('../../../versions/v1/bin/bun', join(store, '.bin/bun'));
      symlinkSync('.bin', join(store, 'commands'), 'dir');
      symlinkSync('../linked/node_modules/commands', join(bin, 'via-store'), 'dir');
      expect(realpathSync(join(bin, 'via-store/bun'))).toBe(real);
      expect(directSettingsBun(join(bin, 'via-store/bun'))).toBeUndefined();
      expect(directSettingsBun(join(bin, 'via-project'), project)).toBeUndefined();
      expect(directSettingsBun(join(bin, 'via-project'))).toBe(real); // an explicit non-shim pin
      expect(directSettingsBun(join(bin, 'direct'), project)).toBe(real);
      symlinkSync('../versions/v1/bin', join(bin, 'shelf'), 'dir');
      expect(directSettingsBun(bin + '/shelf/../bin/bun', project)).toBe(real);
      expect(directSettingsBun(real, '/')).toBe(real);
    } finally { rmSync(root, {recursive: true, force: true}); }
  });
  it('targets SK_PATH and edits only the selected literal; stale proposals cannot clobber human edits', async () => {
    const f = fixture(source);
    try {
      const got = await f.run('get', 'editorFontSize');
      expect(got.data.value).toBe(16);
      expect(got.data.revision).toMatch(/^[a-f0-9]{64}$/);
      expect(
        (await f.run('set-json', 'editorFontSize', '18', '--if-revision', got.data.revision)).code,
      ).toBe(0);
      expect(f.read()).toBe(source.replace('16, //', '18, //'));
      const before = f.read() + '// saved by me\n';
      writeFileSync(f.config, before);
      const stale = await f.run('set-json', 'editorFontSize', '22', '--if-revision', got.data.revision);
      expect(stale).toMatchObject({ code: 1, success: false, errorCode: 'stale_revision' });
      expect(stale.error).toMatch(/stale/i);
      mkdirSync(f.config + '.settings-lock');
      const busy = await f.run('set-json', 'editorFontSize', '22', '--if-revision', got.data.revision);
      expect(busy.code).toBe(1);
      expect(busy.errorCode).toBeUndefined(); // The writer never reached the revision check.
      expect(f.read()).toBe(before);
    } finally {
      f.close();
    }
  });
  it.each(['directory', 'symlink'] as const)('refuses a displaced writer and preserves the replacement %s', async (replacement) => {
    const f = fixture(source);
    try {
      const lock = f.config + '.settings-lock', retired = join(f.root, 'retired');
      let replacementInode = 0;
      expect(() => mutate(f.config, revision(source), old => {
        renameSync(lock, retired);
        if (replacement === 'directory') mkdirSync(lock);
        else symlinkSync(retired, lock, 'dir'); // follows back to the same inode; still not our lock
        replacementInode = lstatSync(lock).ino;
        return replacement === 'directory' ? editLiteral(old!, ['editorFontSize'], 27) : old;
      })).toThrow(DisplacedSettingsWrite);
      expect(f.read()).toBe(source);
      expect(lstatSync(lock).ino).toBe(replacementInode);
      expect((await f.run('set-json', 'editorFontSize', '27')).error).toMatch(/writer busy/i);
      if (replacement === 'directory') rmdirSync(lock); else rmSync(lock);
      expect((await f.run('set-json', 'editorFontSize', '27', '--if-revision', revision(source))).code).toBe(0);
      expect(f.read()).toBe(source.replace('16, //', '27, //'));
    } finally { f.close(); }
  });
  it.each(['identity', 'bytes', 'mode'] as const)('does not publish or unlink a draft with changed %s', (change) => {
    const f = fixture(source);
    try {
      chmodSync(f.config, 0o600);
      const hook = join(f.root, 'alter-draft.mjs');
      // Only the child is instrumented. Let all IO and decoding happen normally;
      // change the real staged file during the helper's re-read of the original.
      writeFileSync(hook, `import fs from 'node:fs'; import path from 'node:path';
        const file = path.join(process.env.SK_PATH, 'config.ts'), lock = file + '.settings-lock';
        const original = fs.readFileSync(file, 'utf8'), decode = TextDecoder.prototype.decode;
        let changed = false;
        TextDecoder.prototype.decode = function(...args) {
          const text = decode.apply(this, args);
          if (!changed && text === original && fs.existsSync(lock)) {
            const names = fs.readdirSync(lock);
            if (names.length === 1) {
              changed = true;
              const draft = path.join(lock, names[0]);
              if (process.env.TEST_CHANGE === 'identity') {
                fs.renameSync(draft, file + '.first-draft'); fs.copyFileSync(file + '.first-draft', draft);
              } else if (process.env.TEST_CHANGE === 'bytes') {
                fs.writeFileSync(draft, original.replace('16, //', '63, //'));
              } else { fs.chmodSync(draft, 0o644); }
            }
          }
          return text;
        };`);
      const child = Bun.spawnSync([process.execPath, '--preload', hook, join(f.sdk, 'config-cli.ts'),
        'set-json', 'editorFontSize', '27', '--if-revision', revision(source)], {
        cwd: f.sdk, env: { HOME: f.root, SK_PATH: join(f.root, 'workspace'), PATH: '/usr/bin:/bin', TEST_CHANGE: change },
      });
      expect(child.exitCode).toBe(1);
      expect(JSON.parse(child.stdout.toString())).toMatchObject({ success: false, errorCode: 'settings_write_displaced' });
      expect(f.read()).toBe(source);
      expect(lstatSync(f.config).mode & 0o777).toBe(0o600);
      const lock = f.config + '.settings-lock', names = readdirSync(lock);
      expect(names).toHaveLength(1);
      const draft = join(lock, names[0]);
      expect(readFileSync(draft, 'utf8')).toBe(source.replace('16, //', change === 'bytes' ? '63, //' : '27, //'));
      expect(lstatSync(draft).mode & 0o777).toBe(change === 'mode' ? 0o644 : 0o600);
      if (change === 'identity') expect(lstatSync(draft).ino).not.toBe(lstatSync(f.config + '.first-draft').ino);
    } finally { f.close(); }
  });
  it('a no-op does not acknowledge an obsolete settings revision', () => {
    const f = fixture(source);
    try {
      const newer = source + '// saved separately\n';
      expect(() => mutate(f.config, revision(source), old => {
        writeFileSync(f.config, newer);
        return old;
      })).toThrow(/changed during edit/i);
      expect(f.read()).toBe(newer);
    } finally { f.close(); }
  });
  it('rejects wrong types, unknown keys and malformed paths before writing', async () => {
    const f = fixture(source);
    try {
      for (const [key, value] of [
        ['dictation.startInKit', 'yes'],
        ['editorFontSize', '18'],
        ['invented.setting', 17],
        ['__proto__.oops', true],
        ['editor', 17],
        ['powerSyntax.enabled', 1],
        ['dictation.startInKit', null],
      ] as const) {
        expect((await f.run('validate-change', JSON.stringify({ key, value }))).code).toBe(1);
        expect((await f.run('set-json', key, JSON.stringify(value))).code).toBe(1);
      }
      expect((await f.run('validate-change', '{"key":"dictation.startInKit","value":true}')).code).toBe(
        0,
      );
      expect(f.read()).toBe(source);
    } finally {
      f.close();
    }
  });
  it('scoped tools refuse executable/remote/permission changes; static validation still recognizes manual config', async () => {
    const original = `export default { hotkey:{modifiers:['meta'],key:'KeyK'}, editor: 'code', commands: { 'builtin/quit': { confirmationRequired: true } } }`;
    const f = fixture(original);
    try {
      expect((await f.run('validate')).code).toBe(0);
      for (const [key, value] of [
        ['bun_path', '/tmp/trap'],
        ['editor', 'trap'],
        ['claudeCode', { enabled: true }],
        ['mcp.enabled', true],
        ['promptTargets', {}],
        ['brainRemote.enabled', true],
        ['dictation.saveHistory', true],
        ['dictation.screenSpelling', 'accessibilityAndOcr'],
        ['commands.builtin/quit.confirmationRequired', false],
      ] as const) {
        expect((await f.run('validate-change', JSON.stringify({ key, value }))).errors[0].code).toBe(
          'manualOnly',
        );
        expect((await f.run('set-json', key, JSON.stringify(value))).code).toBe(1);
        expect((await f.run('reset', key)).code).toBe(1);
      }
      expect((await f.run('reset')).code).toBe(1);
      expect((await f.run('set-json', 'commands', '{}')).code).toBe(1);
      expect(f.read()).toBe(original);
      expect(
        (await f.run('list')).data.options.find((o: { key: string }) => o.key === 'bun_path'),
      ).toMatchObject({ impact: 'execution-policy', manualOnly: true });
    } finally {
      f.close();
    }
  });
  it('installed SDK preserves JSON types and revisions, even when the caller changes into a hostile Bun project', async () => {
    const f = fixture(source);
    try {
      writeFileSync(join(f.root, 'bunfig.toml'), 'preload = ["./trap.ts"]');
      writeFileSync(join(f.root, 'trap.ts'), 'throw Error("hostile project was loaded")');
      const prog = join(f.sdk, 'exercise.ts');
      writeFileSync(
        prog,
        `import {config} from './kit-sdk-config.ts';
        process.chdir(process.env.HOME!); process.env.NODE_OPTIONS = '--require ./trap.ts';
        const old = await config.get('editorFontSize');
        await config.setValidated('editorFontSize',18,old.revision);
        const type = await config.set('editorFontSize','20').then(()=>false,()=>true);
        const stale = await config.set('editorFontSize',20,old.revision).then(()=>null,e=>e.code);
        console.log(JSON.stringify({type,stale}));`,
      );
      const p = Bun.spawnSync([process.execPath, prog], {
        cwd: f.sdk,
        env: { HOME: f.root, SK_PATH: join(f.root, 'workspace'), PATH: '/usr/bin:/bin' },
      });
      expect(p.exitCode).toBe(0);
      expect(JSON.parse(p.stdout.toString())).toEqual({ type: true, stale: 'stale_revision' });
      expect(f.read()).toBe(source.replace('16, //', '18, //'));
    } finally {
      f.close();
    }
  });
  it('returns exact Unicode settings from split replies and refuses invalid UTF-8', () => {
    const dirs = ['/work/東京/🎙️', '/work/literal-�'];
    const original = `export default { claudeCode: { addDirs: ${JSON.stringify(dirs)} } };`;
    const f = fixture(original);
    try {
      const caller = join(f.sdk, 'read-dirs.ts');
      // In this disposable caller only, fragment the real helper's bytes at a
      // known boundary. No dependence on pipe buffer sizes or scheduler timing.
      writeFileSync(caller, `import {Readable} from 'node:stream';
        import {config} from './kit-sdk-config.ts';
        const emit = Readable.prototype.emit, marker = Buffer.from('🎙');
        let damage = false, injections = 0;
        Readable.prototype.emit = function(event, ...args) {
          const bytes = args[0];
          const at = event === 'data' && Buffer.isBuffer(bytes) ? bytes.indexOf(marker) : -1;
          if (at < 0) return emit.call(this, event, ...args);
          injections++;
          if (damage) { const bad = Buffer.from(bytes); bad[at] = 0xff; return emit.call(this, event, bad); }
          emit.call(this, event, bytes.subarray(0, at + 2));
          return emit.call(this, event, bytes.subarray(at + 2));
        };
        try {
          const {value, revision} = await config.get('claudeCode.addDirs');
          damage = true;
          const refused = await config.get('claudeCode.addDirs').then(() => false, e => e.message === 'Settings CLI returned invalid JSON');
          console.log(JSON.stringify({value, revision, refused, injections}));
        } finally { Readable.prototype.emit = emit; }`);
      const child = Bun.spawnSync([process.execPath, caller], {
        cwd: f.sdk, env: { HOME: f.root, SK_PATH: join(f.root, 'workspace'), PATH: '/usr/bin:/bin' },
      });
      expect(child.exitCode).toBe(0);
      expect(JSON.parse(child.stdout.toString())).toEqual({ value: dirs, revision: revision(original), refused: true, injections: 2 });
      expect(f.read()).toBe(original);
    } finally { f.close(); }
  });
  it('does not accept a settings success reply from a process that failed', () => {
    const f = fixture(source);
    try {
      writeFileSync(join(f.sdk, 'config-cli.ts'),
        `console.log(JSON.stringify(process.argv[2] === 'validate'
          ? { success:false, error:'unrelated failure {"valid":true}' }
          : { success:true, data:{value:99} })); process.exit(1);`);
      const caller = join(f.sdk, 'caller.ts');
      writeFileSync(caller, `import {config} from './kit-sdk-config.ts';
        const saved = await config.set('editorFontSize',20).then(()=> 'accepted',()=> 'refused');
        console.log(JSON.stringify({saved, valid:(await config.validate()).valid}));`);
      const child = Bun.spawnSync([process.execPath, caller], {
        cwd: f.sdk, env: { HOME: f.root, SK_PATH: join(f.root, 'workspace'), PATH: '/usr/bin:/bin' },
      });
      expect(child.exitCode).toBe(0);
      expect(JSON.parse(child.stdout.toString())).toEqual({ saved: 'refused', valid: false });
      expect(f.read()).toBe(source);
    } finally {
      f.close();
    }
  });
  it('preserves a displaced-write refusal through the SDK', () => {
    const f = fixture(source);
    try {
      writeFileSync(join(f.sdk, 'config-cli.ts'),
        "console.log(JSON.stringify({success:false,errorCode:'settings_write_displaced'}));process.exit(1)");
      const caller = join(f.sdk, 'caller.ts');
      writeFileSync(caller, `import {config, DisplacedSettingsWrite} from './kit-sdk-config.ts';
        await config.set('editorFontSize', 20).then(() => { throw Error('unexpected save') }, e => {
          if (!(e instanceof DisplacedSettingsWrite)) throw e;
          console.log(e.code);
        });`);
      const child = Bun.spawnSync([process.execPath, caller], {
        cwd: f.sdk, env: { HOME: f.root, SK_PATH: join(f.root, 'workspace'), PATH: '/usr/bin:/bin' },
      });
      expect(child.exitCode).toBe(0);
      expect(child.stdout.toString().trim()).toBe('settings_write_displaced');
      expect(f.read()).toBe(source);
    } finally { f.close(); }
  });
  it('refuses dynamic and ambiguous source without running or repairing it', async () => {
    const f = fixture(source);
    try {
      for (const src of [
        `await Bun.write('executed','yes'); export default { editorFontSize: 16 }`,
        `export default { ...{}, editorFontSize: 16 }`,
        `export default { editorFontSize: 12, editorFontSize: 16 }`,
        `export default { editorFontSize: 16 }; console.log('later')`,
      ]) {
        writeFileSync(f.config, src);
        for (const args of [['get'], ['validate'], ['set', 'editorFontSize', '18'], ['reset']])
          expect((await f.run(...args)).code).toBe(1);
        expect(f.read()).toBe(src);
        expect(existsSync(join(f.root, 'executed'))).toBe(false);
      }
    } finally {
      f.close();
    }
  });
  it('admits at most one writer from the same snapshot, refuses symlink targets', async () => {
    const f = fixture(source);
    try {
      const { revision } = (await f.run('get')).data;
      const replies = await Promise.all(
        ['17', '18', '19', '20'].map((v) =>
          f.run('set-json', 'editorFontSize', v, '--if-revision', revision),
        ),
      );
      expect(replies.filter((r) => r.code === 0)).toHaveLength(1);
      const other = join(f.root, 'other.ts');
      writeFileSync(other, source);
      rmSync(f.config);
      symlinkSync(other, f.config);
      expect((await f.run('set', 'editorFontSize', '20')).code).toBe(1);
      expect(readFileSync(other, 'utf8')).toBe(source);
    } finally {
      f.close();
    }
  });
});
