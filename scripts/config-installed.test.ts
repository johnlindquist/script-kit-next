import { describe, expect, it } from 'bun:test';
import {
  copyFileSync,
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  realpathSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { directSettingsBun } from './config-document.ts';

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
