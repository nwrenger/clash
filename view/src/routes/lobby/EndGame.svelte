<script lang="ts">
	import api from '$lib/api';
	import { Modal } from '@skeletonlabs/skeleton-svelte';
	import type { Connection } from './+page.svelte';
	import { Flag, LandPlot, X } from 'lucide-svelte';

	interface Props {
		connection: Connection;
	}

	let { connection }: Props = $props();

	let open = $state(false);

	function modalClose() {
		// reset
		open = false;
	}

	function end_game() {
		api.send_ws(connection.ws!, { type: 'EndGame' });
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
		<button class="btn-icon preset-filled-error-500" title="End Current Game">
			<Flag size={18} />
		</button>
	{/snippet}
	{#snippet content()}
		<header class="flex justify-between">
			<h4 class="h4">Confirmation</h4>
		</header>
		<article class="space-y-4">
			<p class="opacity-80">
				Do you really want to end the current game? If so you will proceed to the game over screen!
			</p>
		</article>
		<footer class="flex justify-end gap-4">
			<button type="button" class="btn preset-tonal" onclick={modalClose}>Cancel</button>
			<button type="button" class="btn preset-filled-error-500" onclick={end_game}>
				<Flag size={20} />
				End Game
			</button>
		</footer>
	{/snippet}
</Modal>
