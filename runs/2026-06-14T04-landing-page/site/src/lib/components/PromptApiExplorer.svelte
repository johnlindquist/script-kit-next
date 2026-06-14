<script lang="ts">
	import { promptApis, type PromptApi } from '$lib/data/apis';

	const groups: Array<PromptApi['group'] | 'all'> = ['all', 'input', 'ui', 'system', 'agent', 'media'];
	let group = $state<PromptApi['group'] | 'all'>('all');
	let selectedName = $state('arg');

	const visibleApis = $derived(group === 'all' ? promptApis : promptApis.filter((api) => api.group === group));
	const selectedApi = $derived(promptApis.find((api) => api.name === selectedName) ?? visibleApis[0] ?? promptApis[0]);
</script>

<div class="api-explorer">
	<div class="segmented" aria-label="Filter prompt APIs">
		{#each groups as candidate}
			<button type="button" class:active={group === candidate} onclick={() => (group = candidate)}>
				{candidate}
			</button>
		{/each}
	</div>

	<div class="api-grid">
		<div class="api-list" aria-label="Prompt APIs">
			{#each visibleApis as api (api.name)}
				<button type="button" class:active={api.name === selectedApi.name} onclick={() => (selectedName = api.name)}>
					<strong>{api.name}()</strong>
					<span>{api.oneLiner}</span>
				</button>
			{/each}
		</div>
		<div class="code-panel">
			<div class="code-title">
				<span>{selectedApi.group}</span>
				<strong>{selectedApi.name}()</strong>
			</div>
			<pre><code>{selectedApi.sample}</code></pre>
		</div>
	</div>
</div>
