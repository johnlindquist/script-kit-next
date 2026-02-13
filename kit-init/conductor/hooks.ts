/**
 * Script Kit Conductor Extension - Hooks Management
 *
 * Utilities for managing conductor.local hooks
 */

import type { ConductorHook, ConductorHookType, ConductorHookStage, ConductorHookTiming } from './types';
import { getConductorEnv } from './env';

/**
 * Local hooks directory name
 */
export const HOOKS_DIR = 'conductor.local/hooks';

/**
 * Valid hook types
 */
export const HOOK_TYPES: ConductorHookType[] = [
  'pre-setup',
  'post-setup',
  'pre-run',
  'post-run',
  'pre-archive',
  'post-archive',
];

/**
 * Get the hooks directory path for a given hook type
 *
 * @param hookType - The hook type
 * @param rootDir - Root directory (defaults to repo root or cwd)
 * @returns Path to the hook directory
 */
export function getHooksDir(hookType: ConductorHookType, rootDir?: string): string {
  const { join } = require('path');

  const base = rootDir || getConductorEnv()?.rootPath || process.cwd();
  return join(base, HOOKS_DIR, `${hookType}.d`);
}

/**
 * List all hooks of a specific type
 *
 * @param hookType - The hook type to list
 * @param rootDir - Root directory (defaults to repo root or cwd)
 * @returns Array of hook objects
 *
 * @example
 * ```ts
 * import { listHooks } from '@scriptkit/conductor';
 *
 * const setupHooks = await listHooks('pre-setup');
 * for (const hook of setupHooks) {
 *   console.log(`${hook.name}: ${hook.executable ? 'executable' : 'needs bash'}`);
 * }
 * ```
 */
export async function listHooks(
  hookType: ConductorHookType,
  rootDir?: string
): Promise<ConductorHook[]> {
  const { readdir, stat } = await import('fs/promises');
  const { join } = await import('path');

  const dir = getHooksDir(hookType, rootDir);

  try {
    const entries = await readdir(dir, { withFileTypes: true });
    const hooks: ConductorHook[] = [];

    for (const entry of entries) {
      if (entry.isFile()) {
        const fullPath = join(dir, entry.name);
        const stats = await stat(fullPath);
        const executable = (stats.mode & 0o111) !== 0;

        hooks.push({
          name: entry.name,
          path: fullPath,
          type: hookType,
          executable,
        });
      }
    }

    // Sort by name (lexical order, like Conductor does)
    return hooks.sort((a, b) => a.name.localeCompare(b.name));
  } catch (error: any) {
    if (error.code === 'ENOENT') {
      return [];
    }
    throw error;
  }
}

/**
 * List all hooks across all types
 *
 * @param rootDir - Root directory (defaults to repo root or cwd)
 * @returns Object mapping hook types to their hooks
 *
 * @example
 * ```ts
 * import { listAllHooks } from '@scriptkit/conductor';
 *
 * const allHooks = await listAllHooks();
 * console.log(`Pre-setup hooks: ${allHooks['pre-setup'].length}`);
 * ```
 */
export async function listAllHooks(
  rootDir?: string
): Promise<Record<ConductorHookType, ConductorHook[]>> {
  const result = {} as Record<ConductorHookType, ConductorHook[]>;

  for (const hookType of HOOK_TYPES) {
    result[hookType] = await listHooks(hookType, rootDir);
  }

  return result;
}

/**
 * Create a new hook script
 *
 * @param hookType - The hook type
 * @param name - Script name (should start with number for ordering, e.g., "01-install.sh")
 * @param content - Script content
 * @param options - Additional options
 * @returns The created hook object
 *
 * @example
 * ```ts
 * import { createHook } from '@scriptkit/conductor';
 *
 * await createHook('pre-setup', '01-check-env.sh', `#!/bin/bash
 * echo "Checking environment..."
 * if [ -z "$NODE_VERSION" ]; then
 *   echo "Warning: NODE_VERSION not set"
 * fi
 * `);
 * ```
 */
export async function createHook(
  hookType: ConductorHookType,
  name: string,
  content: string,
  options: { executable?: boolean; rootDir?: string } = {}
): Promise<ConductorHook> {
  const { writeFile, mkdir, chmod } = await import('fs/promises');
  const { join } = await import('path');

  const dir = getHooksDir(hookType, options.rootDir);
  await mkdir(dir, { recursive: true });

  const fullPath = join(dir, name);
  await writeFile(fullPath, content, 'utf-8');

  // Make executable by default if it's a shell script
  const makeExecutable = options.executable ?? name.endsWith('.sh');
  if (makeExecutable) {
    await chmod(fullPath, 0o755);
  }

  return {
    name,
    path: fullPath,
    type: hookType,
    executable: makeExecutable,
  };
}

/**
 * Delete a hook script
 *
 * @param hookType - The hook type
 * @param name - Script name to delete
 * @param rootDir - Root directory
 * @returns true if deleted, false if didn't exist
 */
export async function deleteHook(
  hookType: ConductorHookType,
  name: string,
  rootDir?: string
): Promise<boolean> {
  const { unlink } = await import('fs/promises');
  const { join } = await import('path');

  const dir = getHooksDir(hookType, rootDir);
  const fullPath = join(dir, name);

  try {
    await unlink(fullPath);
    return true;
  } catch (error: any) {
    if (error.code === 'ENOENT') {
      return false;
    }
    throw error;
  }
}

/**
 * Read a hook script's content
 *
 * @param hookType - The hook type
 * @param name - Script name to read
 * @param rootDir - Root directory
 * @returns Script content or null if not found
 */
export async function readHook(
  hookType: ConductorHookType,
  name: string,
  rootDir?: string
): Promise<string | null> {
  const { readFile } = await import('fs/promises');
  const { join } = await import('path');

  const dir = getHooksDir(hookType, rootDir);
  const fullPath = join(dir, name);

  try {
    return await readFile(fullPath, 'utf-8');
  } catch (error: any) {
    if (error.code === 'ENOENT') {
      return null;
    }
    throw error;
  }
}

/**
 * Update a hook script's content
 *
 * @param hookType - The hook type
 * @param name - Script name to update
 * @param content - New content
 * @param rootDir - Root directory
 * @returns true if updated, false if didn't exist
 */
export async function updateHook(
  hookType: ConductorHookType,
  name: string,
  content: string,
  rootDir?: string
): Promise<boolean> {
  const { writeFile, access } = await import('fs/promises');
  const { join } = await import('path');

  const dir = getHooksDir(hookType, rootDir);
  const fullPath = join(dir, name);

  try {
    await access(fullPath);
    await writeFile(fullPath, content, 'utf-8');
    return true;
  } catch (error: any) {
    if (error.code === 'ENOENT') {
      return false;
    }
    throw error;
  }
}

/**
 * Ensure the hooks directory structure exists
 *
 * @param rootDir - Root directory
 *
 * @example
 * ```ts
 * import { ensureHooksDir } from '@scriptkit/conductor';
 *
 * // Create all hook directories
 * await ensureHooksDir();
 * ```
 */
export async function ensureHooksDir(rootDir?: string): Promise<void> {
  const { mkdir } = await import('fs/promises');

  for (const hookType of HOOK_TYPES) {
    const dir = getHooksDir(hookType, rootDir);
    await mkdir(dir, { recursive: true });
  }
}

/**
 * Create a simple bash hook from a command
 *
 * @param hookType - The hook type
 * @param name - Script name
 * @param commands - Array of commands or single command
 * @param options - Additional options
 * @returns The created hook object
 *
 * @example
 * ```ts
 * import { createBashHook } from '@scriptkit/conductor';
 *
 * await createBashHook('pre-setup', '01-deps.sh', [
 *   'echo "Installing dependencies..."',
 *   'npm install',
 *   'echo "Done!"'
 * ]);
 * ```
 */
export async function createBashHook(
  hookType: ConductorHookType,
  name: string,
  commands: string | string[],
  options: { rootDir?: string } = {}
): Promise<ConductorHook> {
  const cmdArray = Array.isArray(commands) ? commands : [commands];
  const content = `#!/bin/bash
set -e

${cmdArray.join('\n')}
`;

  return createHook(hookType, name, content, {
    ...options,
    executable: true,
  });
}
