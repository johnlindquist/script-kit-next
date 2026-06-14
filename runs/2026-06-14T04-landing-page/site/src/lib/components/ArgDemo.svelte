<script lang="ts">
	type DemoSnippet = {
		label: string;
		code: string;
		renderedState: 'list' | 'form' | 'terminal';
	};

	const snippets: DemoSnippet[] = [
		{
			label: 'arg()',
			renderedState: 'list',
			code: `import "@scriptkit/sdk";

const repo = await arg("Open a repo", [
  "script-kit-gpui",
  "docs-site",
  "lesson-plans"
]);

await term(\`cd ~/dev/\${repo} && bun test\`);`
		},
		{
			label: 'fields()',
			renderedState: 'form',
			code: `const [title, channel] = await fields([
  { name: "title", label: "Release title" },
  { name: "channel", label: "Audience" }
]);

await editor(\`# \${title}\\n\\nShip notes for \${channel}\`, "markdown");`
		},
		{
			label: 'term()',
			renderedState: 'terminal',
			code: `const command = await arg("Command", [
  "bun test",
  "bun run build",
  "git status --short"
]);

await term(command);`
		}
	];

	let selected = $state(0);
	const current = $derived(snippets[selected]);
</script>

<div class="demo">
	<div class="segmented" aria-label="Prompt demo selector">
		{#each snippets as snippet, index}
			<button type="button" class:active={selected === index} onclick={() => (selected = index)}>
				{snippet.label}
			</button>
		{/each}
	</div>
	<div class="demo-grid">
		<pre class="code-window"><code>{current.code}</code></pre>
		<div class="prompt-window" data-state={current.renderedState}>
			<div class="prompt-title">{current.label} prompt</div>
			{#if current.renderedState === 'list'}
				<div class="prompt-input">Open a repo</div>
				<div class="mock-row selected">script-kit-gpui <kbd>enter</kbd></div>
				<div class="mock-row">docs-site</div>
				<div class="mock-row">lesson-plans</div>
			{:else if current.renderedState === 'form'}
				<div class="mock-field">Release title <span>Script Kit GPUI alpha</span></div>
				<div class="mock-field">Audience <span>automation-heavy Mac developers</span></div>
				<button type="button">Open editor</button>
			{:else}
				<div class="terminal-line">$ bun run build</div>
				<div class="terminal-line muted">vite build</div>
				<div class="terminal-line success">static output ready</div>
			{/if}
		</div>
	</div>
</div>
