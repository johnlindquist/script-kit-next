/**
 * Script Kit Config SDK Module
 * 
 * Provides a programmatic interface for AI agents to access Script Kit configuration.
 * Wraps the config-cli.ts CLI tool and exposes a typed API.
 * 
 * Usage:
 *   import { config } from './kit-sdk-config';
 *   
 *   // Read config values
 *   const all = await config.get();                    // Get all config
 *   const fontSize = await config.get('editorFontSize'); // Get specific key
 *   const hotkey = await config.get('hotkey.key');     // Get nested key
 *   
 *   // Set config values
 *   await config.set('editorFontSize', 16);
 *   await config.set('hotkey.key', 'KeyK');
 *   
 *   // List all options with metadata
 *   const options = await config.list();
 *   
 *   // Validate current config
 *   const result = await config.validate();
 */

import { spawn } from 'node:child_process';
import { realpathSync } from 'node:fs';
import { homedir, devNull } from 'node:os';
import { configPath, directSettingsBun, DisplacedSettingsWrite, type SettingsWriteErrorCode, StaleConfigRevision } from './config-document.ts';
export { DisplacedSettingsWrite, StaleConfigRevision };
import * as path from 'node:path';
import { fileURLToPath } from 'node:url';

// =============================================================================
// Types — re-exported from the canonical SDK source
// =============================================================================

import type { Config } from "./kit-sdk";

export type { Config };

// Re-export validation types from config-schema
export type { ValidationMessage, ValidateConfigChangeResult, ConfigChange } from './config-schema';

export interface ConfigOption {
  key: string;
  type: string;
  current: unknown;
  default: unknown;
  isCustom: boolean;
  description: string;
  example?: string;
  impact: string;
  manualOnly: boolean;
  disableOnly: boolean;
}

export interface GetAllResult {
  revision: string;
  path: string;
  config: Config;
  exists?: boolean;
  message?: string;
}

export interface GetKeyResult {
  path: string;
  exists: boolean;
  impact: string;
  manualOnly: boolean;
  disableOnly: boolean;
  revision: string;
  key: string;
  value: unknown;
  isDefault: boolean;
  default: unknown;
}

export interface SetResult {
  revision: string;
  key: string;
  value: unknown;
  message: string;
}

export interface ListResult {
  revision: string;
  path: string;
  exists: boolean;
  options: ConfigOption[];
}

export interface ValidateResult {
  valid: boolean;
  exists?: boolean;
  message?: string;
  errors?: string[];
  warnings?: string[];
}

interface CLIResponse<T> {
  success: boolean;
  errorCode?: SettingsWriteErrorCode;
  data?: T;
  error?: string;
  // For validate command
  valid?: boolean;
  errors?: string[];
  warnings?: string[];
}

// =============================================================================
// CLI Invocation
// =============================================================================

// Determine the path to config-cli.ts relative to this file
const __dirname = path.dirname(fileURLToPath(import.meta.url));
const CLI_PATH = path.join(__dirname, 'config-cli.ts');

// Bun already identified itself by running this script. A Node caller can name
// its Bun explicitly; an invalid pin must not silently switch to another runtime.
function settingsBun(): string {
  if (process.versions.bun) return realpathSync.native(process.execPath);
  if (process.env.BUN_PATH !== undefined) {
    const pinned = directSettingsBun(process.env.BUN_PATH);
    if (pinned) return pinned;
    throw new Error('BUN_PATH must name an absolute executable Bun binary, not a shim');
  }
  const exe = process.platform === 'win32' ? 'bun.exe' : 'bun';
  const installed = [path.join(homedir(), '.bun/bin', exe)];
  if (process.platform !== 'win32') installed.push('/opt/homebrew/bin/bun', '/usr/local/bin/bun');
  for (const candidate of installed) {
    const direct = directSettingsBun(candidate);
    if (direct) return direct;
  }
  const cwd = process.cwd();
  for (const bin of (process.env.PATH ?? '').split(path.delimiter)) {
    if (!path.isAbsolute(bin)) continue;
    // Do not normalize away a link followed by .. before the selector examines it.
    const direct = directSettingsBun(`${bin}${path.sep}${exe}`, cwd);
    if (direct) return direct;
  }
  throw new Error('No direct Bun executable found; set BUN_PATH to the absolute Bun binary');
}

/**
 * Execute the config CLI with the given arguments
 */
async function runCLI<T>(args: string[]): Promise<T> {
  return new Promise((resolve, reject) => {
    const bunPath = settingsBun();
    const proc = spawn(bunPath, [`--config=${devNull}`, '--no-install', '--no-env-file', CLI_PATH, ...args], {
      cwd: __dirname,
      stdio: ['ignore', 'pipe', 'pipe'],
      env: { HOME: homedir(), SCRIPT_KIT_CONFIG_PATH: configPath(), SCRIPT_KIT_NONINTERACTIVE: process.env.SCRIPT_KIT_NONINTERACTIVE, CI: process.env.CI }
    });

    let stdout = '';

    proc.stdout.on('data', (data) => {
      stdout += data.toString();
    });

    proc.stderr.resume(); // drain diagnostics without redisclosing raw config text

    proc.on('close', (code) => {
      let result: CLIResponse<T>;
      try {
        result = JSON.parse(stdout);
      } catch {
        reject(new Error(code === 0 ? 'Settings CLI returned invalid JSON' : 'Settings CLI failed without a readable result'));
        return;
      }
      if (code === 0 && result?.success === true && Object.hasOwn(result, 'data')) {
        resolve(result.data as T);
      } else if (code === 1 && result?.success === false) {
        if ((args[0] === 'validate' || args[0] === 'validate-change') && result.valid === false) {
          resolve(result as unknown as T);
        } else {
          if (result.errorCode === 'stale_revision') reject(new StaleConfigRevision());
          else if (result.errorCode === 'settings_write_displaced') reject(new DisplacedSettingsWrite());
          else reject(new Error(result.error || 'Settings CLI refused the request'));
        }
      } else {
        reject(new Error('Settings CLI exited without a matching result'));
      }
    });

    proc.on('error', (err) => {
      reject(new Error(`Failed to spawn CLI: ${err.message}`));
    });
  });
}

// =============================================================================
// Config API
// =============================================================================

/**
 * Get all config values or a specific key
 */
async function get(): Promise<GetAllResult>;
async function get(key: string): Promise<GetKeyResult>;
async function get(key?: string): Promise<GetAllResult | GetKeyResult> {
  if (key) {
    return runCLI<GetKeyResult>(['get', key]);
  }
  return runCLI<GetAllResult>(['get']);
}

/**
 * Set a config value
 */
async function set(key: string, value: unknown, expectedRevision?: string): Promise<SetResult> {
  const json = JSON.stringify(value);
  if (json === undefined) throw new Error('Expected a JSON config value');
  return runCLI<SetResult>(['set-json', key, json, ...(expectedRevision === undefined ? [] : ['--if-revision', expectedRevision])]);
}

/**
 * List all available config options with current values and metadata
 */
async function list(): Promise<ListResult> {
  return runCLI<ListResult>(['list']);
}

/**
 * Validate the current config file
 */
async function validate(): Promise<ValidateResult> {
  try {
    return await runCLI<ValidateResult>(['validate']);
  } catch (err) {
    // Structured validation results have already been checked against the exit status.
    return { valid: false, errors: [err instanceof Error ? err.message : String(err)] };
  }
}

/**
 * Validate a proposed config change before writing.
 * Resolves with a structured result for both valid AND invalid changes —
 * never throws on normal validation failure.
 */
export async function validateChange(change: { key: string; value: unknown }): Promise<
  import('./config-schema').ValidateConfigChangeResult
> {
  const {valid, normalizedValue, errors, warnings} = await runCLI<import('./config-schema').ValidateConfigChangeResult>(['validate-change', JSON.stringify(change)]);
  return {valid, normalizedValue, errors, warnings};
}

// The writer validates the typed value inside the same operation as the edit.
const setValidated = set;

/**
 * Reset a config value to default (or all values if no key specified)
 */
async function reset(key?: string, expectedRevision?: string): Promise<{ key?: string; value?: unknown; message: string; config?: Config; revision: string }> {
  return runCLI(['reset', ...(key === undefined ? [] : [key]), ...(expectedRevision === undefined ? [] : ['--if-revision',expectedRevision])]);
}

// =============================================================================
// Export
// =============================================================================

export const config = {
  get,
  set,
  setValidated,
  list,
  validate,
  validateChange,
  reset
};

// Convenience alias for Tab AI consumers
export const configValidation = { validateChange };

// Default export for convenience
export default config;

// =============================================================================
// Self-Test (run with: bun scripts/kit-sdk-config.ts)
// =============================================================================

if (import.meta.main) {
  console.log('Testing kit-sdk-config module...\n');

  async function runTests() {
    const results: { name: string; passed: boolean; error?: string }[] = [];

    // Test 1: config.get() - get all config
    try {
      console.log('Test 1: config.get() - get all config');
      const allConfig = await config.get();
      console.log('  Result:', JSON.stringify(allConfig, null, 2).slice(0, 200) + '...');
      results.push({ name: 'config.get()', passed: !!allConfig.config });
    } catch (err) {
      console.log('  Error:', err);
      results.push({ name: 'config.get()', passed: false, error: String(err) });
    }

    // Test 2: config.get('hotkey.key') - get specific key
    try {
      console.log('\nTest 2: config.get("hotkey.key") - get specific key');
      const keyResult = await config.get('hotkey.key');
      console.log('  Result:', JSON.stringify(keyResult, null, 2));
      results.push({ name: 'config.get(key)', passed: keyResult.key === 'hotkey.key' });
    } catch (err) {
      console.log('  Error:', err);
      results.push({ name: 'config.get(key)', passed: false, error: String(err) });
    }

    // Test 3: config.list() - list all options
    try {
      console.log('\nTest 3: config.list() - list all options');
      const listResult = await config.list();
      console.log('  Options count:', listResult.options.length);
      console.log('  First option:', JSON.stringify(listResult.options[0], null, 2));
      results.push({ name: 'config.list()', passed: listResult.options.length > 0 });
    } catch (err) {
      console.log('  Error:', err);
      results.push({ name: 'config.list()', passed: false, error: String(err) });
    }

    // Test 4: config.validate() - validate config
    try {
      console.log('\nTest 4: config.validate() - validate config');
      const validateResult = await config.validate();
      console.log('  Result:', JSON.stringify(validateResult, null, 2));
      results.push({ name: 'config.validate()', passed: 'valid' in validateResult });
    } catch (err) {
      console.log('  Error:', err);
      results.push({ name: 'config.validate()', passed: false, error: String(err) });
    }

    // Test 5: config.get('editorFontSize') - get numeric value
    try {
      console.log('\nTest 5: config.get("editorFontSize") - get numeric value');
      const fontResult = await config.get('editorFontSize');
      console.log('  Result:', JSON.stringify(fontResult, null, 2));
      results.push({ name: 'config.get(numeric)', passed: typeof fontResult.value === 'number' || fontResult.isDefault });
    } catch (err) {
      console.log('  Error:', err);
      results.push({ name: 'config.get(numeric)', passed: false, error: String(err) });
    }

    // Summary
    console.log('\n' + '='.repeat(50));
    console.log('Test Summary:');
    console.log('='.repeat(50));
    for (const result of results) {
      const status = result.passed ? '✓ PASS' : '✗ FAIL';
      console.log(`  ${status}: ${result.name}${result.error ? ` (${result.error})` : ''}`);
    }
    
    const passCount = results.filter(r => r.passed).length;
    console.log(`\nTotal: ${passCount}/${results.length} passed`);
    
    if (passCount < results.length) {
      process.exit(1);
    }
  }

  runTests().catch(err => {
    console.error('Test runner failed:', err);
    process.exit(1);
  });
}
