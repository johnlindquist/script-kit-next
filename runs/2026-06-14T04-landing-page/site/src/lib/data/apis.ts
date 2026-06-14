export type PromptApi = {
	name: string;
	group: 'input' | 'ui' | 'system' | 'agent' | 'media';
	oneLiner: string;
	sample: string;
};

export const promptApis: PromptApi[] = [
	{
		name: 'arg',
		group: 'input',
		oneLiner: 'Ask for one value with optional choices.',
		sample: "const repo = await arg('Pick a repo', repos);"
	},
	{
		name: 'fields',
		group: 'input',
		oneLiner: 'Collect structured values from several fields.',
		sample: "const [title, tag] = await fields(['Title', 'Tag']);"
	},
	{
		name: 'form',
		group: 'input',
		oneLiner: 'Build a richer form prompt for workflow inputs.',
		sample: "const issue = await form({ title: 'New issue', fields });"
	},
	{
		name: 'div',
		group: 'ui',
		oneLiner: 'Render custom HTML-style prompt UI.',
		sample: "await div(`<h1>${summary}</h1><pre>${diff}</pre>`);"
	},
	{
		name: 'editor',
		group: 'ui',
		oneLiner: 'Open a focused text editor prompt.',
		sample: "const note = await editor(seed, 'markdown');"
	},
	{
		name: 'term',
		group: 'system',
		oneLiner: 'Run an interactive terminal command.',
		sample: "await term('bun test --watch');"
	},
	{
		name: 'drop',
		group: 'system',
		oneLiner: 'Accept files by drag and drop.',
		sample: 'const files = await drop();'
	},
	{
		name: 'hotkey',
		group: 'system',
		oneLiner: 'Capture a keyboard shortcut.',
		sample: "const shortcut = await hotkey('Press the trigger');"
	},
	{
		name: 'path',
		group: 'system',
		oneLiner: 'Pick a file or folder from the local filesystem.',
		sample: "const file = await path({ startPath: '~/Documents' });"
	},
	{
		name: 'chat',
		group: 'agent',
		oneLiner: 'Route intent into Agent Chat with context.',
		sample: "await chat('Summarize this selection with receipts');"
	},
	{
		name: 'mic',
		group: 'media',
		oneLiner: 'Capture microphone input for voice workflows.',
		sample: 'const audio = await mic();'
	},
	{
		name: 'webcam',
		group: 'media',
		oneLiner: 'Capture image input for visual workflows.',
		sample: 'const frame = await webcam();'
	}
];
