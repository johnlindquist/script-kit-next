// Name: Previous-app window lookup replies
// A real, non-autosubmitting SDK over pipes. The peer never contacts a window system.
import { spawn } from 'node:child_process';
import { mkdtempSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { createInterface } from 'node:readline';
import { fileURLToPath } from 'node:url';

const test = 'frontmost-window-distinguishes-empty-unavailable-and-unverified';
const log = (status: string, error?: string) => console.log(JSON.stringify({ test, status, error, timestamp: new Date().toISOString() }));
const home = mkdtempSync(join(tmpdir(), 'kit-window-replies-'));
const target = { windowId: 95031, title: 'Fixture', appName: 'Window fixture' };
const replies = [
  { window: target }, {}, { error: 'No frontmost window found' },
  { error: 'PRIVATE_HOST_DETAIL' }, { window: target, error: 'No frontmost window found' },
  { window: { title: 'No handle', appName: 'Fixture' } }, { type: 'displayListResult', displays: [] },
];
const expected = ['acted', 'empty', 'empty', 'ERR_WINDOW_LOOKUP_UNAVAILABLE', ...Array(3).fill('ERR_WINDOW_LOOKUP_UNVERIFIED')];
const script = `for (let i=0;i<${replies.length};i++) {
  try {
    const w = await getFrontmostWindow(); if (w) await tileWindow(w.windowId, 'left');
    console.log(JSON.stringify({result:w ? 'acted' : 'empty'}));
  } catch (e) { console.log(JSON.stringify({result:e.code ?? 'unexpected', message:e.message})); }
}`;
log('running');
try {
  const child = spawn(process.execPath, [process.platform === 'win32' ? '--config=NUL' : '--config=/dev/null',
    '--no-install', '--no-env-file', '--no-macros', '--preload', fileURLToPath(new URL('../../scripts/kit-sdk.ts', import.meta.url)), '-e', script], {
    cwd: home, env: { HOME: home, SK_PATH: home, SCRIPT_KIT_NONINTERACTIVE: '1', BUN_RUNTIME_TRANSPILER_CACHE_PATH: '0' },
    stdio: ['pipe', 'pipe', 'ignore'], timeout: 4000, killSignal: 'SIGKILL',
  });
  const closed = new Promise<number | null>((resolve, reject) => { child.once('error', reject); child.once('close', resolve); });
  let requests = 0, actions = 0;
  const outcomes: string[] = [];
  try {
    for await (const line of createInterface({ input: child.stdout })) {
      const msg = JSON.parse(line);
      if (msg.type === 'frontmostWindow' && requests < replies.length) {
        child.stdin.write(JSON.stringify({ type: 'frontmostWindowResult', requestId: msg.requestId, ...replies[requests++] }) + '\n');
      } else if (msg.type === 'windowAction') {
        if (++actions !== 1 || requests !== 1 || msg.windowId !== target.windowId || msg.action !== 'tile' || msg.tilePosition !== 'left') {
          throw Error('An unverified target received a window action');
        }
        child.stdin.write(JSON.stringify({ type: 'windowActionResult', requestId: msg.requestId, success: true }) + '\n');
      } else if (typeof msg.result === 'string' && !line.includes('PRIVATE_HOST_DETAIL')) outcomes.push(msg.result);
      else throw Error('Unexpected window lookup message');
    }
    if (await closed !== 0 || requests !== replies.length || actions !== 1 || JSON.stringify(outcomes) !== JSON.stringify(expected)) {
      throw Error('Window lookup did not distinguish the reported outcomes');
    }
  } finally {
    child.stdin.end(); if (child.exitCode === null && child.signalCode === null) child.kill('SIGKILL'); await closed;
  }
  log('pass');
} catch (e) {
  log('fail', e instanceof Error ? e.message : 'Window lookup transport failed'); process.exitCode = 1;
} finally { rmSync(home, { recursive: true, force: true }); }
