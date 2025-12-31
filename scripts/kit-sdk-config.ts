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
import * as path from 'node:path';
import { fileURLToPath } from 'node:url';

// =============================================================================
// Types
// =============================================================================

type KeyModifier = "meta" | "ctrl" | "alt" | "shift";
type KeyCode =
  | "KeyA" | "KeyB" | "KeyC" | "KeyD" | "KeyE" | "KeyF" | "KeyG"
  | "KeyH" | "KeyI" | "KeyJ" | "KeyK" | "KeyL" | "KeyM" | "KeyN"
  | "KeyO" | "KeyP" | "KeyQ" | "KeyR" | "KeyS" | "KeyT" | "KeyU"
  | "KeyV" | "KeyW" | "KeyX" | "KeyY" | "KeyZ"
  | "Digit0" | "Digit1" | "Digit2" | "Digit3" | "Digit4"
  | "Digit5" | "Digit6" | "Digit7" | "Digit8" | "Digit9"
  | "Space" | "Enter" | "Semicolon"
  | "F1" | "F2" | "F3" | "F4" | "F5" | "F6"
  | "F7" | "F8" | "F9" | "F10" | "F11" | "F12";

interface HotkeyConfig {
  modifiers: KeyModifier[];
  key: KeyCode;
}

interface ContentPadding {
  top?: number;
  left?: number;
  right?: number;
}

interface BuiltInConfig {
  clipboardHistory?: boolean;
  appLauncher?: boolean;
  windowSwitcher?: boolean;
}

interface ProcessLimits {
  maxMemoryMb?: number;
  maxRuntimeSeconds?: number;
  healthCheckIntervalMs?: number;
}

export interface Config {
  hotkey: HotkeyConfig;
  bun_path?: string;
  editor?: string;
  padding?: ContentPadding;
  editorFontSize?: number;
  terminalFontSize?: number;
  uiScale?: number;
  builtIns?: BuiltInConfig;
  clipboardHistoryMaxTextLength?: number;
  processLimits?: ProcessLimits;
}

export interface ConfigOption {
  key: string;
  type: string;
  current: unknown;
  default: unknown;
  isCustom: boolean;
  description: string;
  example?: string;
}

export interface GetAllResult {
  path: string;
  config: Config;
  exists?: boolean;
  message?: string;
}

export interface GetKeyResult {
  key: string;
  value: unknown;
  isDefault: boolean;
  default: unknown;
}

export interface SetResult {
  key: string;
  value: unknown;
  message: string;
}

export interface ListResult {
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

/**
 * Execute the config CLI with the given arguments
 */
async function runCLI<T>(args: string[]): Promise<T> {
  return new Promise((resolve, reject) => {
    const bunPath = process.env.BUN_PATH || 'bun';
    const proc = spawn(bunPath, [CLI_PATH, ...args], {
      stdio: ['ignore', 'pipe', 'pipe'],
      env: { ...process.env }
    });

    let stdout = '';
    let stderr = '';

    proc.stdout.on('data', (data) => {
      stdout += data.toString();
    });

    proc.stderr.on('data', (data) => {
      stderr += data.toString();
    });

    proc.on('close', (code) => {
      // Parse the JSON output
      try {
        const result = JSON.parse(stdout) as CLIResponse<T>;
        
        if (result.success) {
          resolve(result.data as T);
        } else {
          // Check for validate-style response
          if ('valid' in result) {
            resolve(result as unknown as T);
          }
          reject(new Error(result.error || 'Unknown CLI error'));
        }
      } catch (parseError) {
        // If we can't parse JSON, report the raw output
        if (code !== 0) {
          reject(new Error(`CLI failed (exit ${code}): ${stderr || stdout}`));
        } else {
          reject(new Error(`Failed to parse CLI output: ${stdout}`));
        }
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
async function set(key: string, value: unknown): Promise<SetResult> {
  // Convert value to string for CLI
  const valueStr = typeof value === 'string' ? value : JSON.stringify(value);
  return runCLI<SetResult>(['set', key, valueStr]);
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
    // validate command returns structured error, need to handle specially
    const message = err instanceof Error ? err.message : String(err);
    // Try to extract the structured response from stderr/error
    try {
      // Check if error contains JSON
      const jsonMatch = message.match(/\{[\s\S]*\}/);
      if (jsonMatch) {
        return JSON.parse(jsonMatch[0]) as ValidateResult;
      }
    } catch {
      // Ignore parse errors
    }
    return {
      valid: false,
      errors: [message]
    };
  }
}

/**
 * Reset a config value to default (or all values if no key specified)
 */
async function reset(key?: string): Promise<{ key?: string; value?: unknown; message: string; config?: Config }> {
  if (key) {
    return runCLI(['reset', key]);
  }
  return runCLI(['reset']);
}

// =============================================================================
// Export
// =============================================================================

export const config = {
  get,
  set,
  list,
  validate,
  reset
};

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
