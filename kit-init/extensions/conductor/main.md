---
name: Conductor
description: Integration with Conductor for parallel AI coding agents
author: Script Kit
icon: terminal
---

# Conductor

Integration tools for [Conductor](https://conductor.build) - run parallel Claude Code agents.

---

## Open Conductor

<!--
description: Launch Conductor app
shortcut: cmd shift c
-->

```open
conductor://
```

---

## Open Conductor Docs

<!--
description: Open Conductor documentation
-->

```open
https://docs.conductor.build
```

---

## Open Conductor Website

<!--
description: Open Conductor website
-->

```open
https://conductor.build
```

---

## Show Workspace Info

<!--
description: Display current Conductor workspace information
-->

```ts
import { isInsideConductor, getConductorEnv, getWorkspace } from '../../conductor';

if (!isInsideConductor()) {
  await div(`
    <div class="p-8 text-center">
      <h2 class="text-2xl font-bold text-yellow-400 mb-4">Not in Conductor</h2>
      <p class="text-gray-400">This script is not running inside a Conductor workspace.</p>
      <p class="text-gray-500 mt-2">Open Conductor and run a script from within a workspace.</p>
    </div>
  `);
  exit();
}

const env = getConductorEnv();
const workspace = getWorkspace();

await div(`
  <div class="p-6">
    <h2 class="text-2xl font-bold text-green-400 mb-6">Conductor Workspace</h2>

    <div class="space-y-4">
      <div class="bg-gray-800 p-4 rounded-lg">
        <p class="text-gray-400 text-sm">Workspace Name</p>
        <p class="text-white text-xl font-mono">${workspace?.name || 'Unknown'}</p>
      </div>

      <div class="bg-gray-800 p-4 rounded-lg">
        <p class="text-gray-400 text-sm">Repository</p>
        <p class="text-white text-xl font-mono">${workspace?.repo || 'Unknown'}</p>
      </div>

      <div class="bg-gray-800 p-4 rounded-lg">
        <p class="text-gray-400 text-sm">Base Port</p>
        <p class="text-white text-xl font-mono">${env?.port || 'N/A'}</p>
      </div>

      <div class="bg-gray-800 p-4 rounded-lg">
        <p class="text-gray-400 text-sm">Port Range</p>
        <p class="text-white text-xl font-mono">${workspace?.portRange?.[0] || 'N/A'} - ${workspace?.portRange?.[1] || 'N/A'}</p>
      </div>

      <div class="bg-gray-800 p-4 rounded-lg">
        <p class="text-gray-400 text-sm">Root Path</p>
        <p class="text-white text-sm font-mono break-all">${env?.rootPath || 'N/A'}</p>
      </div>

      <div class="bg-gray-800 p-4 rounded-lg">
        <p class="text-gray-400 text-sm">Workspace Path</p>
        <p class="text-white text-sm font-mono break-all">${env?.workspacePath || 'N/A'}</p>
      </div>
    </div>
  </div>
`);
```

---

## Get Available Port

<!--
description: Get an available port from Conductor's allocated range
-->

```ts
import { isInsideConductor, getPort } from '../../conductor';

if (!isInsideConductor()) {
  await div(`
    <div class="p-8 text-center">
      <h2 class="text-2xl font-bold text-yellow-400 mb-4">Not in Conductor</h2>
      <p class="text-gray-400">Run this from inside a Conductor workspace to get allocated ports.</p>
    </div>
  `);
  exit();
}

const offset = await arg({
  placeholder: 'Select port offset (0-9)',
  choices: Array.from({ length: 10 }, (_, i) => ({
    name: `Port offset ${i}`,
    value: i,
    description: `Port ${getPort(i)}`,
  })),
});

const port = getPort(Number(offset));

await clipboard.writeText(String(port));

await div(`
  <div class="p-8 text-center">
    <h2 class="text-2xl font-bold text-green-400 mb-4">Port ${port}</h2>
    <p class="text-gray-400">Copied to clipboard!</p>
    <p class="text-gray-500 mt-4 text-sm">Use this port for your development server.</p>
  </div>
`);
```

---

## View conductor.json Config

<!--
description: View the current conductor.json configuration
-->

```ts
import { readConfig, hasConfig, getConfigPath } from '../../conductor';

const configPath = getConfigPath();
const exists = await hasConfig();

if (!exists) {
  await div(`
    <div class="p-8 text-center">
      <h2 class="text-2xl font-bold text-yellow-400 mb-4">No conductor.json</h2>
      <p class="text-gray-400">No conductor.json found in ${configPath}</p>
      <p class="text-gray-500 mt-4">Create one to configure setup, run, and archive scripts.</p>
    </div>
  `);
  exit();
}

const config = await readConfig();

await div(`
  <div class="p-6">
    <h2 class="text-2xl font-bold text-blue-400 mb-4">conductor.json</h2>
    <pre class="bg-gray-800 p-4 rounded-lg overflow-auto text-sm font-mono text-green-300">${JSON.stringify(config, null, 2)}</pre>
    <p class="text-gray-500 mt-4 text-sm">${configPath}</p>
  </div>
`);
```

---

## Initialize conductor.json

<!--
description: Create a new conductor.json with common defaults
-->

```ts
import { initConfig, hasConfig, getConfigPath } from '../../conductor';

const configPath = getConfigPath();
const exists = await hasConfig();

if (exists) {
  const overwrite = await arg({
    placeholder: 'conductor.json already exists. Overwrite?',
    choices: [
      { name: 'No, keep existing', value: false },
      { name: 'Yes, overwrite', value: true },
    ],
  });

  if (!overwrite) {
    exit();
  }
}

const setupScript = await arg({
  placeholder: 'Setup script (runs when workspace is created)',
  hint: 'e.g., npm install, bun install',
});

const runScript = await arg({
  placeholder: 'Run script (runs when clicking Run button)',
  hint: 'e.g., npm run dev -- --port $CONDUCTOR_PORT',
});

const config = await initConfig({
  scripts: {
    setup: setupScript || undefined,
    run: runScript || undefined,
  },
});

await div(`
  <div class="p-6">
    <h2 class="text-2xl font-bold text-green-400 mb-4">Created conductor.json</h2>
    <pre class="bg-gray-800 p-4 rounded-lg overflow-auto text-sm font-mono text-green-300">${JSON.stringify(config, null, 2)}</pre>
    <p class="text-gray-500 mt-4 text-sm">${configPath}</p>
  </div>
`);
```

---

## List All Workspaces

<!--
description: List all Conductor workspaces on this machine
-->

```ts
import { listWorkspaces, getConductorRoot } from '../../conductor';

const root = getConductorRoot();
const workspaces = await listWorkspaces();

if (workspaces.length === 0) {
  await div(`
    <div class="p-8 text-center">
      <h2 class="text-2xl font-bold text-yellow-400 mb-4">No Workspaces Found</h2>
      <p class="text-gray-400">No Conductor workspaces found in ${root}/workspaces</p>
    </div>
  `);
  exit();
}

const selected = await arg({
  placeholder: 'Select a workspace to open',
  choices: workspaces.map(ws => {
    const parts = ws.split('/');
    const name = parts[parts.length - 1];
    const repo = parts[parts.length - 2];
    return {
      name: `${name}`,
      value: ws,
      description: repo,
    };
  }),
});

await open(selected);
```

---

## Check Conductor Status

<!--
description: Check if Conductor is installed and running
-->

```ts
import { isInstalled, isRunning, getVersion } from '../../conductor';

const installed = await isInstalled();
const running = await isRunning();
const version = installed ? await getVersion() : null;

await div(`
  <div class="p-6">
    <h2 class="text-2xl font-bold text-blue-400 mb-6">Conductor Status</h2>

    <div class="space-y-4">
      <div class="flex items-center gap-4 bg-gray-800 p-4 rounded-lg">
        <span class="${installed ? 'text-green-400' : 'text-red-400'} text-2xl">
          ${installed ? '✓' : '✗'}
        </span>
        <div>
          <p class="text-white font-medium">Installed</p>
          <p class="text-gray-400 text-sm">${installed ? `Version ${version}` : 'Not found at /Applications/Conductor.app'}</p>
        </div>
      </div>

      <div class="flex items-center gap-4 bg-gray-800 p-4 rounded-lg">
        <span class="${running ? 'text-green-400' : 'text-yellow-400'} text-2xl">
          ${running ? '✓' : '○'}
        </span>
        <div>
          <p class="text-white font-medium">Running</p>
          <p class="text-gray-400 text-sm">${running ? 'Conductor is running' : 'Conductor is not running'}</p>
        </div>
      </div>
    </div>

    ${!installed ? `
      <div class="mt-6 p-4 bg-blue-900/30 rounded-lg">
        <p class="text-blue-300">Download Conductor from <a href="https://conductor.build" class="underline">conductor.build</a></p>
      </div>
    ` : ''}
  </div>
`);
```

---

## List Hooks

<!--
description: View configured Conductor hooks
-->

```ts
import { listAllHooks, ensureHooksDir } from '../../conductor';

const allHooks = await listAllHooks();
const hookTypes = Object.keys(allHooks);
const totalHooks = Object.values(allHooks).reduce((sum, hooks) => sum + hooks.length, 0);

if (totalHooks === 0) {
  await div(`
    <div class="p-8 text-center">
      <h2 class="text-2xl font-bold text-yellow-400 mb-4">No Hooks Configured</h2>
      <p class="text-gray-400 mb-4">No hooks found in conductor.local/hooks/</p>
      <p class="text-gray-500 text-sm">
        Hook types: pre-setup, post-setup, pre-run, post-run, pre-archive, post-archive
      </p>
    </div>
  `);
  exit();
}

const hookList = hookTypes.flatMap(type =>
  allHooks[type].map(hook => `
    <div class="flex items-center gap-3 bg-gray-800 p-3 rounded-lg mb-2">
      <span class="${hook.executable ? 'text-green-400' : 'text-gray-500'}">
        ${hook.executable ? '✓' : '○'}
      </span>
      <div>
        <p class="text-white font-mono text-sm">${hook.name}</p>
        <p class="text-gray-500 text-xs">${type}</p>
      </div>
    </div>
  `)
).join('');

await div(`
  <div class="p-6">
    <h2 class="text-2xl font-bold text-purple-400 mb-6">Conductor Hooks (${totalHooks})</h2>
    ${hookList}
  </div>
`);
```

---

## Open Workspace in Editor

<!--
description: Open the current Conductor workspace in your editor
-->

```ts
import { isInsideConductor, getConductorEnv } from '../../conductor';

if (!isInsideConductor()) {
  await div(`
    <div class="p-8 text-center">
      <h2 class="text-2xl font-bold text-yellow-400 mb-4">Not in Conductor</h2>
      <p class="text-gray-400">Run this from inside a Conductor workspace.</p>
    </div>
  `);
  exit();
}

const env = getConductorEnv();
const editor = await arg({
  placeholder: 'Select editor',
  choices: [
    { name: 'VS Code', value: 'code' },
    { name: 'Cursor', value: 'cursor' },
    { name: 'Zed', value: 'zed' },
    { name: 'Sublime Text', value: 'subl' },
    { name: 'Finder', value: 'open' },
  ],
});

await exec(`${editor} "${env?.workspacePath}"`);
```

---

## Copy Environment Variable

<!--
description: Copy a Conductor environment variable to clipboard
-->

```ts
import { isInsideConductor, getConductorEnv, CONDUCTOR_ENV_VARS } from '../../conductor';

if (!isInsideConductor()) {
  await div(`
    <div class="p-8 text-center">
      <h2 class="text-2xl font-bold text-yellow-400 mb-4">Not in Conductor</h2>
      <p class="text-gray-400">Run this from inside a Conductor workspace.</p>
    </div>
  `);
  exit();
}

const env = getConductorEnv();

const varName = await arg({
  placeholder: 'Select environment variable to copy',
  choices: [
    { name: 'CONDUCTOR_WORKSPACE_NAME', value: env?.workspaceName, description: env?.workspaceName },
    { name: 'CONDUCTOR_PORT', value: String(env?.port), description: String(env?.port) },
    { name: 'CONDUCTOR_ROOT_PATH', value: env?.rootPath, description: env?.rootPath },
    { name: 'CONDUCTOR_WORKSPACE_PATH', value: env?.workspacePath, description: env?.workspacePath },
    { name: 'CONDUCTOR_BIN_DIR', value: env?.binDir || '', description: env?.binDir || 'Not set' },
  ],
});

await clipboard.writeText(varName);

await div(`
  <div class="p-8 text-center">
    <h2 class="text-2xl font-bold text-green-400 mb-4">Copied!</h2>
    <p class="text-gray-400 font-mono">${varName}</p>
  </div>
`);
```
