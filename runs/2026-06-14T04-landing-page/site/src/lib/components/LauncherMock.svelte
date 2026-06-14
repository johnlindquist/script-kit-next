<script lang="ts">
	import { launcherItems } from '$lib/data/launcher-items';

	let query = $state('');
	let selected = $state(0);
	let receipt = $state('Ready');

	const filtered = $derived(
		launcherItems.filter((item) => {
			const haystack = `${item.name} ${item.description} ${item.kind}`.toLowerCase();
			return haystack.includes(query.toLowerCase());
		})
	);

	function clampSelection() {
		if (selected >= filtered.length) selected = Math.max(0, filtered.length - 1);
	}

	function runItem(index = selected) {
		const item = filtered[index];
		if (!item) return;
		selected = index;
		receipt = `Ran ${item.name} via ${item.id}`;
	}

	function onKeydown(event: KeyboardEvent) {
		if (event.key === 'ArrowDown') {
			event.preventDefault();
			selected = Math.min(selected + 1, filtered.length - 1);
		}
		if (event.key === 'ArrowUp') {
			event.preventDefault();
			selected = Math.max(selected - 1, 0);
		}
		if (event.key === 'Enter') {
			event.preventDefault();
			runItem();
		}
	}
</script>

<div class="workbench" aria-label="Example workbench UI">
	<div class="workbench-topline">
		<span>Example workbench UI</span>
		<span class="status-pill">{receipt}</span>
	</div>
	<label class="search-label" for="launcher-search">Command</label>
	<input
		id="launcher-search"
		value={query}
		oninput={(event) => {
			query = event.currentTarget.value;
			selected = 0;
			clampSelection();
		}}
		onkeydown={onKeydown}
		placeholder="Search scripts, built-ins, agents, memory..."
	/>
	<div class="launcher-list" role="listbox" aria-label="Script Kit GPUI command list">
		{#each filtered as item, index (item.id)}
			<button
				class:active={index === selected}
				type="button"
				role="option"
				aria-selected={index === selected}
				onclick={() => runItem(index)}
			>
				<span class="kind">{item.kind}</span>
				<span class="row-main">
					<strong>{item.name}</strong>
					<small>{item.description}</small>
				</span>
				{#if item.shortcut}<kbd>{item.shortcut}</kbd>{/if}
			</button>
		{/each}
	</div>
</div>
