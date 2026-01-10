/**
 * Script Kit Conductor Extension - Type Definitions
 *
 * Types for integrating Script Kit with the Conductor app
 * (https://conductor.build)
 */

/**
 * Conductor environment variables available when running inside Conductor
 */
export interface ConductorEnv {
  /** Path to the repository root */
  rootPath: string;
  /** Path to the current workspace */
  workspacePath: string;
  /** First port in a range of 10 allocated ports */
  port: number;
  /** Unique workspace identifier (city name) */
  workspaceName: string;
  /** Path to Conductor's bin directory */
  binDir?: string;
}

/**
 * Conductor.json scripts configuration
 */
export interface ConductorScripts {
  /** Runs when workspace is created */
  setup?: string;
  /** Runs when clicking "Run" button */
  run?: string;
  /** Runs when workspace is archived */
  archive?: string;
}

/**
 * Run script options
 */
export interface ConductorRunOptions {
  /**
   * When 'nonconcurrent', terminates previous run before starting new one.
   * Default is concurrent (allows multiple runs).
   */
  mode?: 'nonconcurrent';
}

/**
 * Full conductor.json configuration
 */
export interface ConductorConfig {
  scripts?: ConductorScripts;
  run?: ConductorRunOptions;
}

/**
 * Hook types for conductor.local/hooks
 */
export type ConductorHookStage = 'setup' | 'run' | 'archive';
export type ConductorHookTiming = 'pre' | 'post';
export type ConductorHookType = `${ConductorHookTiming}-${ConductorHookStage}`;

/**
 * Hook script metadata
 */
export interface ConductorHook {
  /** Name of the hook script (e.g., "01-install-deps.sh") */
  name: string;
  /** Full path to the hook script */
  path: string;
  /** Hook type (e.g., "pre-setup", "post-run") */
  type: ConductorHookType;
  /** Whether the script is executable */
  executable: boolean;
}

/**
 * Workspace information
 */
export interface ConductorWorkspace {
  /** Workspace name (city name) */
  name: string;
  /** Full path to workspace directory */
  path: string;
  /** Repository name */
  repo: string;
  /** Allocated port */
  port: number;
  /** Range of available ports (10 consecutive) */
  portRange: [number, number];
}

/**
 * Options for launching Conductor
 */
export interface LaunchOptions {
  /** Repository URL or local path to open */
  repo?: string;
  /** Whether to wait for Conductor to open */
  wait?: boolean;
}
