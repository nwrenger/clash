<script lang="ts">
	import api from '$lib/api';
	import { Modal } from '@skeletonlabs/skeleton-svelte';

	interface Props {
		ws: WebSocket | undefined;
	}

	let { ws }: Props = $props();

	let deckcode = $state('');
	let openState = $state(false);

	function modalClose() {
		// reset
		deckcode = '';
		openState = false;
	}

	function add() {
		api.send_ws(ws!, { type: 'AddDeck', data: { deckcode } });
		modalClose();
	}
</script>

<Modal
	open={openState}
	onOpenChange={(e) => (openState = e.open)}
	triggerBase="sm:w-auto w-full"
	contentBase="card bg-surface-100-900 p-4 space-y-4 shadow-xl max-w-screen-sm"
	backdropClasses="backdrop-blur-sm"
>
	{#snippet trigger()}
		<button class="btn preset-filled-primary-500 flex w-full sm:w-auto" title="Add Deck">
			Add Deck
		</button>
	{/snippet}
	{#snippet content()}
		<header class="flex justify-between">
			<h4 class="h4">Adding a Deck</h4>
		</header>
		<article class="space-y-4">
			<p class="opacity-80">
				To add a custom deck, please visit the <a
					href="https://cast.clrtd.com/"
					target="_blank"
					class="anchor">clrtd website</a
				>, copy its deck code, and paste it here.
			</p>
			<label class="label">
				<span class="label-text">Deckcode</span>
				<input class="input" bind:value={deckcode} placeholder="Input deckcode..." />
			</label>
		</article>
		<footer class="flex justify-end gap-4">
			<button type="button" class="btn preset-tonal" onclick={modalClose}>Cancel</button>
			<button type="button" class="btn preset-filled" onclick={add} disabled={!deckcode}>Add</button
			>
		</footer>
	{/snippet}
</Modal>
