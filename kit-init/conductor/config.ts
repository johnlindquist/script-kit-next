/**
 * Script Kit Conductor Extension - Config Management
 *
 * Utilities for reading and writing conductor.json configuration
 */

import type { ConductorConfig, ConductorScripts, ConductorRunOptions } from './types';
import { getConductorEnv, getConductorRoot } from './env';

/**
 * Default conductor.json filename
 */
export const CONDUCTOR_CONFIG_FILENAME = 'conductor.json';

/**
 * Get the path to conductor.json for a given directory
 *
 * @param dir - Directory to look in (defaults to repo root or cwd)
 * @returns Path to conductor.json
 */
export function getConfigPath(dir?: string): string {
  const { join } = require('path');

  if (dir) {
    return join(dir, CONDUCTOR_CONFIG_FILENAME);
  }

  // Try to use Conductor root path first
  const env = getConductorEnv();
  if (env?.rootPath) {
    return join(env.rootPath, CONDUCTOR_CONFIG_FILENAME);
  }

  // Fall back to current working directory
  return join(process.cwd(), CONDUCTOR_CONFIG_FILENAME);
}

/**
 * Read conductor.json configuration
 *
 * @param dir - Directory to read from (defaults to repo root or cwd)
 * @returns ConductorConfig object or null if file doesn't exist
 *
 * @example
 * ```ts
 * import { readConfig } from '@scriptkit/conductor';
 *
 * const config = await readConfig();
 * if (config?.scripts?.setup) {
 *   console.log(`Setup script: ${config.scripts.setup}`);
 * }
 * ```
 */
export async function readConfig(dir?: string): Promise<ConductorConfig | null> {
  const { readFile } = await import('fs/promises');
  const configPath = getConfigPath(dir);

  try {
    const content = await readFile(configPath, 'utf-8');
    return JSON.parse(content) as ConductorConfig;
  } catch (error: any) {
    if (error.code === 'ENOENT') {
      return null;
    }
    throw new Error(`Failed to read conductor.json: ${error.message}`);
  }
}

/**
 * Write conductor.json configuration
 *
 * @param config - Configuration to write
 * @param dir - Directory to write to (defaults to repo root or cwd)
 *
 * @example
 * ```ts
 * import { writeConfig } from '@scriptkit/conductor';
 *
 * await writeConfig({
 *   scripts: {
 *     setup: 'npm install',
 *     run: 'npm run dev'
 *   }
 * });
 * ```
 */
export async function writeConfig(config: ConductorConfig, dir?: string): Promise<void> {
  const { writeFile } = await import('fs/promises');
  const configPath = getConfigPath(dir);

  const content = JSON.stringify(config, null, 2) + '\n';
  await writeFile(configPath, content, 'utf-8');
}

/**
 * Update conductor.json configuration (merge with existing)
 *
 * @param updates - Partial configuration to merge
 * @param dir - Directory containing conductor.json
 * @returns The updated configuration
 *
 * @example
 * ```ts
 * import { updateConfig } from '@scriptkit/conductor';
 *
 * // Add a run script without overwriting other settings
 * const updated = await updateConfig({
 *   scripts: { run: 'npm run dev' }
 * });
 * ```
 */
export async function updateConfig(
  updates: Partial<ConductorConfig>,
  dir?: string
): Promise<ConductorConfig> {
  const existing = await readConfig(dir) || {};

  const merged: ConductorConfig = {
    ...existing,
    ...updates,
  };

  // Deep merge scripts
  if (updates.scripts && existing.scripts) {
    merged.scripts = {
      ...existing.scripts,
      ...updates.scripts,
    };
  }

  // Deep merge run options
  if (updates.run && existing.run) {
    merged.run = {
      ...existing.run,
      ...updates.run,
    };
  }

  await writeConfig(merged, dir);
  return merged;
}

/**
 * Set a specific script in conductor.json
 *
 * @param scriptType - Type of script ('setup', 'run', or 'archive')
 * @param command - Command to run
 * @param dir - Directory containing conductor.json
 *
 * @example
 * ```ts
 * import { setScript } from '@scriptkit/conductor';
 *
 * await setScript('setup', 'bun install && bun run build');
 * await setScript('run', 'bun run dev --port $CONDUCTOR_PORT');
 * ```
 */
export async function setScript(
  scriptType: keyof ConductorScripts,
  command: string,
  dir?: string
): Promise<void> {
  await updateConfig({
    scripts: { [scriptType]: command }
  }, dir);
}

/**
 * Get a specific script from conductor.json
 *
 * @param scriptType - Type of script ('setup', 'run', or 'archive')
 * @param dir - Directory containing conductor.json
 * @returns The script command or null if not set
 */
export async function getScript(
  scriptType: keyof ConductorScripts,
  dir?: string
): Promise<string | null> {
  const config = await readConfig(dir);
  return config?.scripts?.[scriptType] ?? null;
}

/**
 * Check if conductor.json exists
 *
 * @param dir - Directory to check
 * @returns true if conductor.json exists
 */
export async function hasConfig(dir?: string): Promise<boolean> {
  const { access } = await import('fs/promises');
  const configPath = getConfigPath(dir);

  try {
    await access(configPath);
    return true;
  } catch {
    return false;
  }
}

/**
 * Initialize a new conductor.json with default configuration
 *
 * @param options - Initial configuration options
 * @param dir - Directory to create conductor.json in
 * @returns The created configuration
 *
 * @example
 * ```ts
 * import { initConfig } from '@scriptkit/conductor';
 *
 * // Create a basic conductor.json
 * await initConfig({
 *   scripts: {
 *     setup: 'npm install',
 *     run: 'npm run dev'
 *   }
 * });
 * ```
 */
export async function initConfig(
  options: Partial<ConductorConfig> = {},
  dir?: string
): Promise<ConductorConfig> {
  const config: ConductorConfig = {
    scripts: {
      setup: options.scripts?.setup,
      run: options.scripts?.run,
      archive: options.scripts?.archive,
    },
    ...options,
  };

  // Clean up undefined values
  if (config.scripts) {
    Object.keys(config.scripts).forEach(key => {
      if (config.scripts![key as keyof ConductorScripts] === undefined) {
        delete config.scripts![key as keyof ConductorScripts];
      }
    });
    if (Object.keys(config.scripts).length === 0) {
      delete config.scripts;
    }
  }

  await writeConfig(config, dir);
  return config;
}

/**
 * Set the run mode (concurrent or nonconcurrent)
 *
 * @param mode - 'nonconcurrent' to kill previous runs, or undefined for concurrent
 * @param dir - Directory containing conductor.json
 *
 * @example
 * ```ts
 * import { setRunMode } from '@scriptkit/conductor';
 *
 * // Kill previous run script before starting new one
 * await setRunMode('nonconcurrent');
 * ```
 */
export async function setRunMode(
  mode: 'nonconcurrent' | undefined,
  dir?: string
): Promise<void> {
  if (mode) {
    await updateConfig({ run: { mode } }, dir);
  } else {
    const config = await readConfig(dir);
    if (config?.run) {
      delete config.run.mode;
      if (Object.keys(config.run).length === 0) {
        delete config.run;
      }
      await writeConfig(config, dir);
    }
  }
}
