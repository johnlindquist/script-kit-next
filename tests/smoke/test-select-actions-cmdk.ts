import '../../scripts/kit-sdk';

export const metadata = {
  name: 'Test Select Cmd+K Actions',
  description: 'Verify Cmd+K opens actions panel for select prompt',
};

console.error('[SMOKE] test-select-actions-cmdk starting...');

await setActions([
  {
    name: 'Action One',
    shortcut: 'cmd+1',
    onAction: () => {
      console.error('[SMOKE] Action One triggered');
      process.exit(0);
    },
  },
]);

const result = await select('Select items (Cmd+K for actions):', [
  'One',
  'Two',
  'Three',
]);

console.error('[SMOKE] Select result:', JSON.stringify(result));
console.error('[SMOKE] test-select-actions-cmdk completed');

process.exit(0);
