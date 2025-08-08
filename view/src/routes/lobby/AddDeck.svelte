<script lang="ts">
	import api from '$lib/api';
	import { Modal } from '@skeletonlabs/skeleton-svelte';
	import type { Connection } from './+page.svelte';

	interface Props {
		connection: Connection;
		disabled: boolean;
	}

	let { connection, disabled }: Props = $props();

	let deckcode = $state('');
	let open = $state(false);

	function modalClose() {
		// reset
		deckcode = '';
		open = false;
	}

	function add() {
		api.send_ws(connection.ws!, { type: 'AddDeck', data: { deckcode } });
		modalClose();
	}
</script>

<Modal
	{open}
	onOpenChange={(e) => (open = e.open)}
	triggerBase="w-full"
	contentBase="card bg-surface-100-900 p-4 space-y-4 shadow-xl max-w-screen-sm"
	backdropClasses="backdrop-blur-sm"
>
	{#snippet trigger()}
		<button class="btn preset-filled-primary-500 w-full" title="Add a Deck" {disabled}>
			Add
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
