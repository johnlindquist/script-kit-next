/**
 * Script Kit Conductor Extension - Test Suite
 *
 * Run with: bun run ~/.scriptkit/kit/conductor/test.ts
 */

import {
  // Environment
  isInsideConductor,
  getConductorEnv,
  getWorkspace,
  getPort,
  requireConductor,
  getConductorRoot,
  listWorkspaces,
  CONDUCTOR_ENV_VARS,

  // Config
  readConfig,
  writeConfig,
  updateConfig,
  setScript,
  getScript,
  hasConfig,
  initConfig,
  getConfigPath,

  // Launch
  isInstalled,
  getVersion,
  isRunning,
  CONDUCTOR_URL_SCHEME,
  CONDUCTOR_BUNDLE_ID,
  CONDUCTOR_APP_PATH,

  // Hooks
  listHooks,
  listAllHooks,
  createHook,
  createBashHook,
  deleteHook,
  readHook,
  ensureHooksDir,
  HOOK_TYPES,

  // Meta
  VERSION,
  conductor,
} from './index';

interface TestResult {
  name: string;
  passed: boolean;
  error?: string;
  duration: number;
}

const results: TestResult[] = [];

async function test(name: string, fn: () => Promise<void> | void): Promise<void> {
  const start = Date.now();
  try {
    await fn();
    results.push({ name, passed: true, duration: Date.now() - start });
    console.log(`  ✓ ${name}`);
  } catch (error: any) {
    results.push({
      name,
      passed: false,
      error: error.message,
      duration: Date.now() - start,
    });
    console.log(`  ✗ ${name}: ${error.message}`);
  }
}

function assert(condition: boolean, message: string): void {
  if (!condition) {
    throw new Error(message);
  }
}

function assertEqual<T>(actual: T, expected: T, message?: string): void {
  if (actual !== expected) {
    throw new Error(message || `Expected ${expected}, got ${actual}`);
  }
}

async function runTests() {
  console.log('═══════════════════════════════════════════');
  console.log(' Script Kit Conductor Extension Test Suite');
  console.log(`              Version ${VERSION}`);
  console.log('═══════════════════════════════════════════\n');

  // === Environment Tests ===
  console.log('Environment Detection:');

  await test('isInsideConductor returns boolean', () => {
    const result = isInsideConductor();
    assert(typeof result === 'boolean', 'Should return boolean');
  });

  await test('getConductorEnv returns env or null', () => {
    const env = getConductorEnv();
    if (isInsideConductor()) {
      assert(env !== null, 'Should have env when inside Conductor');
      assert(typeof env!.workspaceName === 'string', 'Should have workspace name');
      assert(typeof env!.port === 'number', 'Should have port');
    }
  });

  await test('getWorkspace extracts repo name', () => {
    const workspace = getWorkspace();
    if (workspace) {
      assert(workspace.name.length > 0, 'Should have name');
      assert(workspace.repo.length > 0, 'Should have repo');
      assertEqual(workspace.portRange[1] - workspace.portRange[0], 9, 'Port range should be 10');
    }
  });

  await test('getPort returns valid ports', () => {
    if (isInsideConductor()) {
      const port0 = getPort(0);
      const port5 = getPort(5);
      const port10 = getPort(10);

      assert(port0 !== null, 'Port 0 should be valid');
      assert(port5 !== null, 'Port 5 should be valid');
      assert(port10 === null, 'Port 10 should be out of range');
      assertEqual(port5! - port0!, 5, 'Ports should be sequential');
    }
  });

  await test('getConductorRoot finds conductor directory', () => {
    const root = getConductorRoot();
    if (isInsideConductor()) {
      assert(root !== null, 'Should find root');
      assert(root!.endsWith('/conductor'), 'Should end with conductor');
    }
  });

  await test('listWorkspaces returns array', async () => {
    const workspaces = await listWorkspaces();
    assert(Array.isArray(workspaces), 'Should return array');
  });

  // === Config Tests ===
  console.log('\nConfig Management:');

  const { mkdir, rm } = await import('fs/promises');
  const testDir = '/tmp/conductor-extension-test';
  await rm(testDir, { recursive: true, force: true });
  await mkdir(testDir, { recursive: true });

  await test('getConfigPath returns correct path', () => {
    const path = getConfigPath(testDir);
    assert(path.endsWith('conductor.json'), 'Should be conductor.json');
  });

  await test('hasConfig returns false for missing config', async () => {
    const has = await hasConfig(testDir);
    assertEqual(has, false);
  });

  await test('initConfig creates config file', async () => {
    const config = await initConfig({ scripts: { setup: 'npm install' } }, testDir);
    assertEqual(config.scripts?.setup, 'npm install');
  });

  await test('hasConfig returns true after init', async () => {
    const has = await hasConfig(testDir);
    assertEqual(has, true);
  });

  await test('readConfig reads existing config', async () => {
    const config = await readConfig(testDir);
    assertEqual(config?.scripts?.setup, 'npm install');
  });

  await test('updateConfig merges configs', async () => {
    const updated = await updateConfig({ scripts: { run: 'npm start' } }, testDir);
    assertEqual(updated.scripts?.setup, 'npm install');
    assertEqual(updated.scripts?.run, 'npm start');
  });

  await test('setScript/getScript work correctly', async () => {
    await setScript('archive', 'echo bye', testDir);
    const script = await getScript('archive', testDir);
    assertEqual(script, 'echo bye');
  });

  // === Launch Tests ===
  console.log('\nLaunch Utilities:');

  await test('constants are defined', () => {
    assertEqual(CONDUCTOR_URL_SCHEME, 'conductor://');
    assertEqual(CONDUCTOR_BUNDLE_ID, 'com.conductor.app');
    assert(CONDUCTOR_APP_PATH.includes('Conductor.app'), 'App path should include Conductor.app');
  });

  await test('isInstalled returns boolean', async () => {
    const result = await isInstalled();
    assert(typeof result === 'boolean', 'Should return boolean');
  });

  await test('getVersion returns version or null', async () => {
    const version = await getVersion();
    if (await isInstalled()) {
      assert(version !== null, 'Should have version when installed');
      assert(/^\d+\.\d+\.\d+/.test(version!), 'Should be semver format');
    }
  });

  await test('isRunning returns boolean', async () => {
    const result = await isRunning();
    assert(typeof result === 'boolean', 'Should return boolean');
  });

  // === Hooks Tests ===
  console.log('\nHooks Management:');

  await test('HOOK_TYPES has all types', () => {
    assertEqual(HOOK_TYPES.length, 6);
    assert(HOOK_TYPES.includes('pre-setup'), 'Should have pre-setup');
    assert(HOOK_TYPES.includes('post-archive'), 'Should have post-archive');
  });

  await test('ensureHooksDir creates directories', async () => {
    await ensureHooksDir(testDir);
    const { access } = await import('fs/promises');
    const { join } = await import('path');
    await access(join(testDir, 'conductor.local/hooks/pre-setup.d'));
  });

  await test('createHook creates executable script', async () => {
    const hook = await createHook('pre-setup', 'test.sh', '#!/bin/bash\necho test', { rootDir: testDir });
    assertEqual(hook.name, 'test.sh');
    assertEqual(hook.executable, true);
  });

  await test('listHooks finds created hooks', async () => {
    const hooks = await listHooks('pre-setup', testDir);
    assert(hooks.length >= 1, 'Should find at least one hook');
    assert(hooks.some(h => h.name === 'test.sh'), 'Should find test.sh');
  });

  await test('readHook returns content', async () => {
    const content = await readHook('pre-setup', 'test.sh', testDir);
    assert(content !== null, 'Should have content');
    assert(content!.includes('echo test'), 'Should contain echo test');
  });

  await test('createBashHook creates proper script', async () => {
    const hook = await createBashHook('post-setup', 'install.sh', ['npm install', 'npm run build'], { rootDir: testDir });
    const content = await readHook('post-setup', 'install.sh', testDir);
    assert(content!.includes('set -e'), 'Should have set -e');
    assert(content!.includes('npm install'), 'Should have npm install');
  });

  await test('deleteHook removes script', async () => {
    const deleted = await deleteHook('pre-setup', 'test.sh', testDir);
    assertEqual(deleted, true);
    const hooks = await listHooks('pre-setup', testDir);
    assert(!hooks.some(h => h.name === 'test.sh'), 'Should not find deleted hook');
  });

  await test('listAllHooks returns all types', async () => {
    const all = await listAllHooks(testDir);
    assertEqual(Object.keys(all).length, 6);
  });

  // === Conductor Helper Tests ===
  console.log('\nConductor Helper:');

  await test('conductor.version is set', () => {
    assertEqual(conductor.version, VERSION);
  });

  await test('conductor.isAvailable matches isInsideConductor', () => {
    assertEqual(conductor.isAvailable, isInsideConductor());
  });

  await test('conductor.env() returns same as getConductorEnv()', () => {
    const env1 = conductor.env();
    const env2 = getConductorEnv();
    assertEqual(env1?.workspaceName, env2?.workspaceName);
  });

  // Cleanup
  await rm(testDir, { recursive: true, force: true });

  // === Summary ===
  console.log('\n═══════════════════════════════════════════');
  const passed = results.filter(r => r.passed).length;
  const failed = results.filter(r => !r.passed).length;
  const totalTime = results.reduce((sum, r) => sum + r.duration, 0);

  console.log(`Results: ${passed} passed, ${failed} failed`);
  console.log(`Total time: ${totalTime}ms`);

  if (failed > 0) {
    console.log('\nFailed tests:');
    for (const r of results.filter(r => !r.passed)) {
      console.log(`  ✗ ${r.name}: ${r.error}`);
    }
    console.log('═══════════════════════════════════════════');
    process.exit(1);
  } else {
    console.log('═══════════════════════════════════════════');
    console.log('All tests passed!');
    process.exit(0);
  }
}

runTests().catch(err => {
  console.error('Test suite error:', err);
  process.exit(1);
});
