/**
 * Script Kit Conductor Extension - Environment Detection
 *
 * Utilities for detecting and working with Conductor's environment
 */

import type { ConductorEnv, ConductorWorkspace } from './types';

/**
 * Environment variable names used by Conductor
 */
export const CONDUCTOR_ENV_VARS = {
  ROOT_PATH: 'CONDUCTOR_ROOT_PATH',
  WORKSPACE_PATH: 'CONDUCTOR_WORKSPACE_PATH',
  PORT: 'CONDUCTOR_PORT',
  WORKSPACE_NAME: 'CONDUCTOR_WORKSPACE_NAME',
  BIN_DIR: 'CONDUCTOR_BIN_DIR',
} as const;

/**
 * Check if the current script is running inside Conductor
 *
 * @returns true if running inside a Conductor workspace
 *
 * @example
 * ```ts
 * import { isInsideConductor } from '@scriptkit/conductor';
 *
 * if (isInsideConductor()) {
 *   console.log('Running in Conductor!');
 * }
 * ```
 */
export function isInsideConductor(): boolean {
  return !!(
    process.env[CONDUCTOR_ENV_VARS.WORKSPACE_NAME] ||
    process.env[CONDUCTOR_ENV_VARS.WORKSPACE_PATH]
  );
}

/**
 * Get Conductor environment variables
 *
 * @returns ConductorEnv object if inside Conductor, null otherwise
 *
 * @example
 * ```ts
 * import { getConductorEnv } from '@scriptkit/conductor';
 *
 * const env = getConductorEnv();
 * if (env) {
 *   console.log(`Workspace: ${env.workspaceName}`);
 *   console.log(`Port: ${env.port}`);
 * }
 * ```
 */
export function getConductorEnv(): ConductorEnv | null {
  if (!isInsideConductor()) {
    return null;
  }

  const port = process.env[CONDUCTOR_ENV_VARS.PORT];

  return {
    rootPath: process.env[CONDUCTOR_ENV_VARS.ROOT_PATH] || '',
    workspacePath: process.env[CONDUCTOR_ENV_VARS.WORKSPACE_PATH] || process.cwd(),
    port: port ? parseInt(port, 10) : 0,
    workspaceName: process.env[CONDUCTOR_ENV_VARS.WORKSPACE_NAME] || '',
    binDir: process.env[CONDUCTOR_ENV_VARS.BIN_DIR],
  };
}

/**
 * Get the current Conductor workspace information
 *
 * @returns ConductorWorkspace object if inside Conductor, null otherwise
 *
 * @example
 * ```ts
 * import { getWorkspace } from '@scriptkit/conductor';
 *
 * const workspace = getWorkspace();
 * if (workspace) {
 *   console.log(`Working in ${workspace.name}`);
 *   console.log(`Ports ${workspace.portRange[0]}-${workspace.portRange[1]} available`);
 * }
 * ```
 */
export function getWorkspace(): ConductorWorkspace | null {
  const env = getConductorEnv();
  if (!env) {
    return null;
  }

  // Extract repo name from workspace path
  // Typical: ~/conductor/workspaces/<repo>/<workspace-name>
  const pathParts = env.workspacePath.split('/');
  const workspacesIdx = pathParts.indexOf('workspaces');
  const repo = workspacesIdx >= 0 && pathParts[workspacesIdx + 1]
    ? pathParts[workspacesIdx + 1]
    : pathParts[pathParts.length - 2] || 'unknown';

  return {
    name: env.workspaceName,
    path: env.workspacePath,
    repo,
    port: env.port,
    portRange: [env.port, env.port + 9],
  };
}

/**
 * Get an available port from the Conductor port range
 *
 * @param offset - Offset from the base port (0-9)
 * @returns The port number, or null if not in Conductor or offset out of range
 *
 * @example
 * ```ts
 * import { getPort } from '@scriptkit/conductor';
 *
 * const webPort = getPort(0);    // Base port for web server
 * const dbPort = getPort(1);     // +1 for database
 * const cachePort = getPort(2);  // +2 for cache
 * ```
 */
export function getPort(offset: number = 0): number | null {
  if (offset < 0 || offset > 9) {
    return null;
  }

  const env = getConductorEnv();
  if (!env || !env.port) {
    return null;
  }

  return env.port + offset;
}

/**
 * Require running inside Conductor, throwing if not
 *
 * @throws Error if not running inside Conductor
 * @returns ConductorEnv object
 *
 * @example
 * ```ts
 * import { requireConductor } from '@scriptkit/conductor';
 *
 * // This will throw if not in Conductor
 * const env = requireConductor();
 * console.log(`Safe to use port ${env.port}`);
 * ```
 */
export function requireConductor(): ConductorEnv {
  const env = getConductorEnv();
  if (!env) {
    throw new Error(
      'This script requires Conductor. Please run it inside a Conductor workspace.'
    );
  }
  return env;
}

/**
 * Get the Conductor root directory (where repos and workspaces live)
 *
 * @returns Path to ~/conductor or null if not found
 */
export function getConductorRoot(): string | null {
  const home = process.env.HOME || process.env.USERPROFILE;
  if (!home) {
    return null;
  }

  // Check if running inside Conductor first
  const env = getConductorEnv();
  if (env?.workspacePath) {
    // Extract from workspace path: ~/conductor/workspaces/...
    const parts = env.workspacePath.split('/');
    const conductorIdx = parts.indexOf('conductor');
    if (conductorIdx >= 0) {
      return parts.slice(0, conductorIdx + 1).join('/');
    }
  }

  // Default to ~/conductor
  return `${home}/conductor`;
}

/**
 * List all workspace directories for a given repo
 *
 * @param repo - Repository name to list workspaces for
 * @returns Array of workspace paths
 */
export async function listWorkspaces(repo?: string): Promise<string[]> {
  const { readdir } = await import('fs/promises');
  const { join } = await import('path');

  const conductorRoot = getConductorRoot();
  if (!conductorRoot) {
    return [];
  }

  const workspacesDir = join(conductorRoot, 'workspaces');

  try {
    if (repo) {
      const repoWorkspacesDir = join(workspacesDir, repo);
      const entries = await readdir(repoWorkspacesDir, { withFileTypes: true });
      return entries
        .filter(e => e.isDirectory())
        .map(e => join(repoWorkspacesDir, e.name));
    } else {
      // List all workspaces across all repos
      const repos = await readdir(workspacesDir, { withFileTypes: true });
      const allWorkspaces: string[] = [];

      for (const repoEntry of repos) {
        if (repoEntry.isDirectory()) {
          const repoWorkspacesDir = join(workspacesDir, repoEntry.name);
          try {
            const workspaceEntries = await readdir(repoWorkspacesDir, { withFileTypes: true });
            for (const ws of workspaceEntries) {
              if (ws.isDirectory()) {
                allWorkspaces.push(join(repoWorkspacesDir, ws.name));
              }
            }
          } catch {
            // Skip repos we can't read
          }
        }
      }

      return allWorkspaces;
    }
  } catch {
    return [];
  }
}
