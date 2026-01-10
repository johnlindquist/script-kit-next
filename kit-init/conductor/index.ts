/**
 * Script Kit Conductor Extension
 *
 * Integrate Script Kit with Conductor (https://conductor.build)
 * for parallel AI coding agent workflows.
 *
 * @module @scriptkit/conductor
 *
 * @example
 * ```ts
 * import {
 *   isInsideConductor,
 *   getConductorEnv,
 *   getPort,
 *   readConfig,
 *   launch,
 * } from '@scriptkit/kit/conductor';
 *
 * // Check if running in Conductor
 * if (isInsideConductor()) {
 *   const env = getConductorEnv();
 *   console.log(`Workspace: ${env.workspaceName}`);
 *   console.log(`Port: ${env.port}`);
 * }
 *
 * // Read conductor.json config
 * const config = await readConfig();
 * console.log(`Setup script: ${config?.scripts?.setup}`);
 *
 * // Launch Conductor
 * await launch();
 * ```
 */

// Types
export type {
  ConductorEnv,
  ConductorScripts,
  ConductorRunOptions,
  ConductorConfig,
  ConductorHookStage,
  ConductorHookTiming,
  ConductorHookType,
  ConductorHook,
  ConductorWorkspace,
  LaunchOptions,
} from './types';

// Environment detection
export {
  CONDUCTOR_ENV_VARS,
  isInsideConductor,
  getConductorEnv,
  getWorkspace,
  getPort,
  requireConductor,
  getConductorRoot,
  listWorkspaces,
} from './env';

// Config management
export {
  CONDUCTOR_CONFIG_FILENAME,
  getConfigPath,
  readConfig,
  writeConfig,
  updateConfig,
  setScript,
  getScript,
  hasConfig,
  initConfig,
  setRunMode,
} from './config';

// Launch utilities
export {
  CONDUCTOR_URL_SCHEME,
  CONDUCTOR_BUNDLE_ID,
  CONDUCTOR_APP_PATH,
  isInstalled,
  getVersion,
  launch,
  openWebsite,
  openDocs,
  activate,
  isRunning,
  quit,
} from './launch';

// Hooks management
export {
  HOOKS_DIR,
  HOOK_TYPES,
  getHooksDir,
  listHooks,
  listAllHooks,
  createHook,
  deleteHook,
  readHook,
  updateHook,
  ensureHooksDir,
  createBashHook,
} from './hooks';

/**
 * Conductor extension version
 */
export const VERSION = '1.0.0';

/**
 * Quick check for Conductor environment with helpful error
 *
 * @example
 * ```ts
 * import { conductor } from '@scriptkit/kit/conductor';
 *
 * // Returns environment info or throws helpful error
 * const env = conductor.require();
 *
 * // Check capabilities
 * console.log(conductor.isAvailable()); // true/false
 * console.log(conductor.version);       // '1.0.0'
 * ```
 */
export const conductor = {
  /** Check if running inside Conductor */
  get isAvailable() {
    const { isInsideConductor } = require('./env');
    return isInsideConductor();
  },

  /** Extension version */
  version: VERSION,

  /** Require Conductor environment (throws if not available) */
  require() {
    const { requireConductor } = require('./env');
    return requireConductor();
  },

  /** Get environment (returns null if not in Conductor) */
  env() {
    const { getConductorEnv } = require('./env');
    return getConductorEnv();
  },

  /** Get current workspace info */
  workspace() {
    const { getWorkspace } = require('./env');
    return getWorkspace();
  },
};

export default conductor;
