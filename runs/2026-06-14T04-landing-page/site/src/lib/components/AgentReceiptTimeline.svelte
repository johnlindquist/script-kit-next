<script lang="ts">
	import { receiptSteps } from '$lib/data/agent-receipts';

	let open = $state('context');
</script>

<div class="receipt" aria-label="Illustrative Agent Chat receipt">
	<div class="receipt-label">Illustrative Agent Chat receipt</div>
	{#each receiptSteps as step}
		<button type="button" class="receipt-step" class:active={open === step.id} onclick={() => (open = step.id)}>
			<span class={`receipt-dot ${step.status}`}></span>
			<span class="receipt-copy">
				<strong>{step.label}</strong>
				<code>{step.target}</code>
				{#if open === step.id}<small>{step.detail}</small>{/if}
			</span>
		</button>
	{/each}
</div>
