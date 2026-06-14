export type LauncherItem = {
	id: string;
	name: string;
	description: string;
	kind: 'script' | 'builtin' | 'agent' | 'memory';
	shortcut?: string;
};

export const launcherItems: LauncherItem[] = [
	{
		id: 'script-release-notes',
		name: 'Draft Release Notes',
		description: 'Read changed files, collect commits, and open an editor prompt.',
		kind: 'script',
		shortcut: 'cmd r'
	},
	{
		id: 'agent-ui-receipt',
		name: 'Agent Chat: UI Receipt',
		description: 'Attach desktop context, choose a semantic target, and verify the result.',
		kind: 'agent',
		shortcut: 'cmd enter'
	},
	{
		id: 'builtin-window-switcher',
		name: 'Window Switcher',
		description: 'Jump between open app windows without leaving the command surface.',
		kind: 'builtin',
		shortcut: 'cmd tab'
	},
	{
		id: 'script-package-audit',
		name: 'Package Audit Prompt',
		description: 'Use bun packages inside a focused TypeScript prompt workflow.',
		kind: 'script'
	},
	{
		id: 'memory-day-page',
		name: 'Open Day Page',
		description: 'Work from local markdown memory for today.',
		kind: 'memory',
		shortcut: 'tap'
	},
	{
		id: 'builtin-terminal-prompt',
		name: 'Quick Terminal Prompt',
		description: 'Run a terminal flow from a native prompt shell.',
		kind: 'builtin'
	}
];
