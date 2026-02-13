/**
 * Script Kit Conductor Extension - Launch Utilities
 *
 * Utilities for launching Conductor and interacting with it
 */

import type { LaunchOptions } from './types';

/**
 * Conductor URL scheme
 */
export const CONDUCTOR_URL_SCHEME = 'conductor://';

/**
 * Bundle identifier for Conductor app
 */
export const CONDUCTOR_BUNDLE_ID = 'com.conductor.app';

/**
 * Path to Conductor app (standard location)
 */
export const CONDUCTOR_APP_PATH = '/Applications/Conductor.app';

/**
 * Check if Conductor is installed
 *
 * @returns true if Conductor.app is installed
 *
 * @example
 * ```ts
 * import { isInstalled } from '@scriptkit/conductor';
 *
 * if (!isInstalled()) {
 *   console.log('Please install Conductor from https://conductor.build');
 * }
 * ```
 */
export async function isInstalled(): Promise<boolean> {
  const { access } = await import('fs/promises');

  try {
    await access(CONDUCTOR_APP_PATH);
    return true;
  } catch {
    return false;
  }
}

/**
 * Get Conductor version if installed
 *
 * @returns Version string or null if not installed
 */
export async function getVersion(): Promise<string | null> {
  const { readFile } = await import('fs/promises');
  const { join } = await import('path');

  try {
    const plistPath = join(CONDUCTOR_APP_PATH, 'Contents/Info.plist');
    const content = await readFile(plistPath, 'utf-8');

    // Simple regex to extract version from plist
    const versionMatch = content.match(/<key>CFBundleShortVersionString<\/key>\s*<string>([^<]+)<\/string>/);
    return versionMatch ? versionMatch[1] : null;
  } catch {
    return null;
  }
}

/**
 * Launch Conductor app
 *
 * @param options - Launch options
 *
 * @example
 * ```ts
 * import { launch } from '@scriptkit/conductor';
 *
 * // Just open Conductor
 * await launch();
 *
 * // Open Conductor with a specific repo
 * await launch({ repo: 'https://github.com/user/repo' });
 * ```
 */
export async function launch(options: LaunchOptions = {}): Promise<void> {
  const { spawn } = await import('child_process');

  const installed = await isInstalled();
  if (!installed) {
    throw new Error(
      'Conductor is not installed. Download it from https://conductor.build'
    );
  }

  return new Promise((resolve, reject) => {
    // Use 'open' command to launch via URL scheme or directly
    let args: string[];

    if (options.repo) {
      // Try to use URL scheme with repo parameter
      // Note: The exact URL format is not publicly documented
      const url = `${CONDUCTOR_URL_SCHEME}open?repo=${encodeURIComponent(options.repo)}`;
      args = [url];
    } else {
      // Just open the app
      args = ['-a', 'Conductor'];
    }

    const child = spawn('open', args, {
      stdio: 'ignore',
      detached: !options.wait,
    });

    if (options.wait) {
      child.on('close', (code) => {
        if (code === 0) {
          resolve();
        } else {
          reject(new Error(`Conductor exited with code ${code}`));
        }
      });

      child.on('error', reject);
    } else {
      child.unref();
      // Give it a moment to start
      setTimeout(resolve, 500);
    }
  });
}

/**
 * Open Conductor's website
 *
 * @example
 * ```ts
 * import { openWebsite } from '@scriptkit/conductor';
 *
 * await openWebsite();
 * ```
 */
export async function openWebsite(): Promise<void> {
  const { spawn } = await import('child_process');

  return new Promise((resolve) => {
    const child = spawn('open', ['https://conductor.build'], {
      stdio: 'ignore',
      detached: true,
    });
    child.unref();
    setTimeout(resolve, 200);
  });
}

/**
 * Open Conductor's documentation
 *
 * @param section - Optional documentation section
 *
 * @example
 * ```ts
 * import { openDocs } from '@scriptkit/conductor';
 *
 * // Open main docs
 * await openDocs();
 *
 * // Open specific section
 * await openDocs('core/scripts');
 * ```
 */
export async function openDocs(section?: string): Promise<void> {
  const { spawn } = await import('child_process');

  const url = section
    ? `https://docs.conductor.build/${section}`
    : 'https://docs.conductor.build';

  return new Promise((resolve) => {
    const child = spawn('open', [url], {
      stdio: 'ignore',
      detached: true,
    });
    child.unref();
    setTimeout(resolve, 200);
  });
}

/**
 * Bring Conductor to front if it's running
 *
 * @returns true if Conductor was activated, false if not running
 */
export async function activate(): Promise<boolean> {
  const { exec } = await import('child_process');
  const { promisify } = await import('util');
  const execAsync = promisify(exec);

  try {
    // Check if Conductor is running
    const { stdout } = await execAsync(
      `osascript -e 'tell application "System Events" to get name of every process'`
    );

    if (!stdout.includes('Conductor')) {
      return false;
    }

    // Activate it
    await execAsync(
      `osascript -e 'tell application "Conductor" to activate'`
    );

    return true;
  } catch {
    return false;
  }
}

/**
 * Check if Conductor is currently running
 *
 * @returns true if Conductor process is running
 */
export async function isRunning(): Promise<boolean> {
  const { exec } = await import('child_process');
  const { promisify } = await import('util');
  const execAsync = promisify(exec);

  try {
    const { stdout } = await execAsync('pgrep -x Conductor || pgrep -f "Conductor.app"');
    return stdout.trim().length > 0;
  } catch {
    return false;
  }
}

/**
 * Quit Conductor app
 *
 * @returns true if Conductor was quit, false if not running
 */
export async function quit(): Promise<boolean> {
  const { exec } = await import('child_process');
  const { promisify } = await import('util');
  const execAsync = promisify(exec);

  try {
    const running = await isRunning();
    if (!running) {
      return false;
    }

    await execAsync(
      `osascript -e 'tell application "Conductor" to quit'`
    );

    return true;
  } catch {
    return false;
  }
}
