// Name: SDK Test - Notes Surface
// Description: Verifies Notes remains an automation-target surface with no invented JS globals

import '../../scripts/kit-sdk';

interface TestResult {
  test: string;
  status: 'running' | 'pass' | 'fail' | 'skip';
  timestamp: string;
  result?: unknown;
  error?: string;
  duration_ms?: number;
}

function logTest(name: string, status: TestResult['status'], extra?: Partial<TestResult>) {
  console.log(JSON.stringify({
    test: name,
    status,
    timestamp: new Date().toISOString(),
    ...extra,
  }));
}

function debug(msg: string) {
  console.error(`[TEST] ${msg}`);
}

async function runTests() {
  {
    const test = 'notes-no-invented-globals';
    const start = Date.now();
    logTest(test, 'running');
    try {
      const forbidden = ['notesOpen', 'notesCreate', 'notesSearch'];
      const present = forbidden.filter((name) => typeof (globalThis as any)[name] === 'function');
      if (present.length > 0) {
        throw new Error(`unexpected globals: ${present.join(', ')}`);
      }
      debug('No Notes globals are exposed on globalThis');
      logTest(test, 'pass', { result: 'no invented Notes globals', duration_ms: Date.now() - start });
    } catch (err) {
      logTest(test, 'fail', { error: String(err), duration_ms: Date.now() - start });
    }
  }

  {
    const test = 'notes-skill-documents-automation-target';
    const start = Date.now();
    logTest(test, 'running');
    try {
      const skill = await Bun.file(new URL('../../kit-init/skills/notes/SKILL.md', import.meta.url)).text();
      const required = [
        '"type": "kind", "kind": "notes"',
        'panel:notes-window',
        'input:notes-editor',
        '"type": "getElements"',
        '"type": "waitFor"',
        '"type": "batch"',
      ];
      for (const needle of required) {
        if (!skill.includes(needle)) {
          throw new Error(`missing notes skill text: ${needle}`);
        }
      }
      debug('Notes skill documents automation target and semantic IDs');
      logTest(test, 'pass', { result: required, duration_ms: Date.now() - start });
    } catch (err) {
      logTest(test, 'fail', { error: String(err), duration_ms: Date.now() - start });
    }
  }

  {
    const test = 'sdk-source-has-no-notes-globals';
    const start = Date.now();
    logTest(test, 'running');
    try {
      const sdkSource = await Bun.file(new URL('../../scripts/kit-sdk.ts', import.meta.url)).text();
      for (const symbol of [
        'globalThis.notesOpen',
        'globalThis.notesCreate',
        'globalThis.notesSearch',
      ]) {
        if (sdkSource.includes(symbol)) {
          throw new Error(`unexpected Notes global in SDK source: ${symbol}`);
        }
      }
      debug('scripts/kit-sdk.ts contains no invented Notes globals');
      logTest(test, 'pass', { result: 'sdk-source-clean', duration_ms: Date.now() - start });
    } catch (err) {
      logTest(test, 'fail', { error: String(err), duration_ms: Date.now() - start });
    }
  }

  debug('Notes SDK tests complete');
  process.exit(0);
}

runTests();
