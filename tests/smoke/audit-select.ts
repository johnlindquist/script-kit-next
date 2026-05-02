import '../../scripts/kit-sdk';

export const metadata = {
  name: 'Visual Audit - Select Prompt',
  description: 'Displays select() prompt row chrome and footer behavior',
};

console.error('[AUDIT] Starting select visual audit...');

await select('Choose a deployment target', [
  {
    name: 'Production',
    value: 'prod',
    description: 'Deploy script • type: script • shortcut: cmd+p',
  },
  {
    name: 'Staging',
    value: 'staging',
    description: 'Preview release • type: script • shortcut: cmd+s',
  },
  {
    name: 'Local',
    value: 'local',
    description: 'Run local workflow • type: scriptlet',
  },
  {
    name: 'Diagnostics',
    value: 'diagnostics',
    description: 'Collect logs • type: agent',
  },
]);
