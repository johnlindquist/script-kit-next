# Script Kit Conductor Extension

Integrate Script Kit with [Conductor](https://conductor.build) for parallel AI coding agent workflows.

## Installation

This extension is bundled with Script Kit at `~/.scriptkit/kit/conductor`.

## Quick Start

```ts
import {
  isInsideConductor,
  getConductorEnv,
  getPort,
  launch,
} from '@scriptkit/kit/conductor';

// Check if running inside a Conductor workspace
if (isInsideConductor()) {
  const env = getConductorEnv();
  console.log(`Workspace: ${env.workspaceName}`);
  console.log(`Available port: ${env.port}`);
}

// Launch Conductor from outside
await launch();
```

## API Reference

### Environment Detection

#### `isInsideConductor(): boolean`

Check if the script is running inside a Conductor workspace.

```ts
if (isInsideConductor()) {
  // We're in Conductor!
}
```

#### `getConductorEnv(): ConductorEnv | null`

Get Conductor environment variables.

```ts
const env = getConductorEnv();
if (env) {
  console.log(env.workspaceName);  // e.g., "salvador"
  console.log(env.port);           // e.g., 55100
  console.log(env.rootPath);       // e.g., "/Users/you/project"
  console.log(env.workspacePath);  // e.g., "/Users/you/conductor/workspaces/project/salvador"
}
```

#### `getWorkspace(): ConductorWorkspace | null`

Get detailed workspace information.

```ts
const workspace = getWorkspace();
if (workspace) {
  console.log(workspace.name);      // "salvador"
  console.log(workspace.repo);      // "project"
  console.log(workspace.portRange); // [55100, 55109]
}
```

#### `getPort(offset?: number): number | null`

Get an available port from the Conductor port range (0-9 offset).

```ts
const webPort = getPort(0);    // 55100 - web server
const dbPort = getPort(1);     // 55101 - database
const cachePort = getPort(2);  // 55102 - cache
```

#### `requireConductor(): ConductorEnv`

Require running inside Conductor (throws helpful error if not).

```ts
const env = requireConductor();
// Safe to use env.port, env.workspaceName, etc.
```

#### `listWorkspaces(repo?: string): Promise<string[]>`

List all workspace directories.

```ts
// All workspaces
const all = await listWorkspaces();

// Workspaces for specific repo
const projectWorkspaces = await listWorkspaces('my-project');
```

### Config Management

#### `readConfig(dir?: string): Promise<ConductorConfig | null>`

Read `conductor.json` configuration.

```ts
const config = await readConfig();
if (config?.scripts?.setup) {
  console.log(`Setup: ${config.scripts.setup}`);
}
```

#### `writeConfig(config, dir?): Promise<void>`

Write `conductor.json` configuration.

```ts
await writeConfig({
  scripts: {
    setup: 'npm install',
    run: 'npm run dev',
  },
});
```

#### `updateConfig(updates, dir?): Promise<ConductorConfig>`

Merge updates into existing config.

```ts
await updateConfig({
  scripts: { run: 'npm run dev -- --port $CONDUCTOR_PORT' },
});
```

#### `setScript(type, command, dir?): Promise<void>`

Set a specific script.

```ts
await setScript('setup', 'bun install');
await setScript('run', 'bun run dev --port $CONDUCTOR_PORT');
await setScript('archive', 'docker compose down');
```

#### `initConfig(options?, dir?): Promise<ConductorConfig>`

Initialize a new `conductor.json`.

```ts
await initConfig({
  scripts: {
    setup: 'npm install',
    run: 'npm run dev',
  },
});
```

### Launch Utilities

#### `launch(options?): Promise<void>`

Launch Conductor app.

```ts
// Just open Conductor
await launch();

// Open with specific repo (if supported)
await launch({ repo: 'https://github.com/user/repo' });
```

#### `isInstalled(): Promise<boolean>`

Check if Conductor is installed.

```ts
if (!await isInstalled()) {
  console.log('Install from https://conductor.build');
}
```

#### `getVersion(): Promise<string | null>`

Get installed Conductor version.

```ts
const version = await getVersion();
console.log(`Conductor ${version}`); // e.g., "0.29.2"
```

#### `isRunning(): Promise<boolean>`

Check if Conductor is currently running.

#### `activate(): Promise<boolean>`

Bring Conductor to front.

#### `quit(): Promise<boolean>`

Quit Conductor app.

#### `openDocs(section?): Promise<void>`

Open Conductor documentation.

```ts
await openDocs();                  // Main docs
await openDocs('core/scripts');    // Scripts section
```

### Hooks Management

Manage `conductor.local/hooks/` scripts.

#### `listHooks(hookType, dir?): Promise<ConductorHook[]>`

List hooks of a specific type.

```ts
const preSetup = await listHooks('pre-setup');
for (const hook of preSetup) {
  console.log(`${hook.name} (${hook.executable ? 'executable' : 'bash'})`);
}
```

#### `createHook(hookType, name, content, options?): Promise<ConductorHook>`

Create a new hook script.

```ts
await createHook('pre-setup', '01-check-env.sh', `#!/bin/bash
echo "Checking environment..."
`);
```

#### `createBashHook(hookType, name, commands, options?): Promise<ConductorHook>`

Create a bash hook from commands.

```ts
await createBashHook('pre-setup', '01-deps.sh', [
  'echo "Installing deps..."',
  'npm install',
]);
```

#### `deleteHook(hookType, name, dir?): Promise<boolean>`

Delete a hook script.

#### `readHook(hookType, name, dir?): Promise<string | null>`

Read hook content.

#### `ensureHooksDir(dir?): Promise<void>`

Create all hook directories.

## Hook Types

- `pre-setup` - Before workspace setup
- `post-setup` - After workspace setup
- `pre-run` - Before run script
- `post-run` - After run script
- `pre-archive` - Before archive
- `post-archive` - After archive

## Environment Variables

When running inside Conductor, these env vars are available:

| Variable | Description |
|----------|-------------|
| `CONDUCTOR_WORKSPACE_NAME` | Unique workspace identifier (city name) |
| `CONDUCTOR_WORKSPACE_PATH` | Full path to workspace directory |
| `CONDUCTOR_ROOT_PATH` | Repository root path |
| `CONDUCTOR_PORT` | First of 10 allocated ports |
| `CONDUCTOR_BIN_DIR` | Conductor binaries directory |

## Conductor Quick Reference

### conductor.json

```json
{
  "scripts": {
    "setup": "npm install",
    "run": "npm run dev -- --port $CONDUCTOR_PORT",
    "archive": "docker compose down"
  },
  "run": {
    "mode": "nonconcurrent"
  }
}
```

### Local Hooks Structure

```
conductor.local/
  hooks/
    pre-setup.d/
      01-check-deps.sh
    post-setup.d/
      01-seed-db.sh
    pre-run.d/
    post-run.d/
    pre-archive.d/
    post-archive.d/
```

## Resources

- [Conductor Website](https://conductor.build)
- [Conductor Documentation](https://docs.conductor.build)
- [Scripts Guide](https://docs.conductor.build/core/scripts)
