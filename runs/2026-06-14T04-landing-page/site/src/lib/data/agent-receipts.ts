export type ReceiptStep = {
	id: string;
	label: string;
	target: string;
	status: 'context' | 'proposed' | 'approved' | 'verified';
	detail: string;
};

export const receiptSteps: ReceiptStep[] = [
	{
		id: 'context',
		label: 'Context read',
		target: 'resource:desktop-context',
		status: 'context',
		detail: 'Agent Chat starts from explicit context: launcher state, selected text, files, clipboard, or visible UI state.'
	},
	{
		id: 'target',
		label: 'Semantic target selected',
		target: 'input:agent_chat-composer',
		status: 'proposed',
		detail: 'Actions address stable semantic IDs instead of brittle screen coordinates or timing guesses.'
	},
	{
		id: 'approval',
		label: 'Action proposed',
		target: 'transaction:submit-intent',
		status: 'approved',
		detail: 'The useful path is scoped and inspectable: what will be read, what will be changed, and why.'
	},
	{
		id: 'verified',
		label: 'Transaction verified',
		target: 'receipt:visible-state',
		status: 'verified',
		detail: 'The result is checked through the app state or UI surface before the workflow claims success.'
	}
];
