<script lang="ts">
	import api from '$lib/api';
	import { Dialog, Portal } from '@skeletonlabs/skeleton-svelte';
	import type { Connection } from './+page.svelte';
	import { Flag, X } from 'lucide-svelte';
	import { flyin, opacity } from '$lib/animations';

	interface Props {
		connection: Connection;
	}

	let { connection }: Props = $props();

	function end_game() {
		api.send_ws(connection.ws!, { type: 'EndGame' });
	}
</script>

<Dialog>
	<Dialog.Trigger class="btn-icon preset-filled-error-500 w-full" title="End Current Game">
		<Flag size={18} />
	</Dialog.Trigger>
	<Portal>
		<Dialog.Backdrop class="bg-surface-50-950/50 fixed inset-0 z-50 backdrop-blur-sm {opacity}" />
		<Dialog.Positioner class="fixed inset-0 z-50 flex items-center justify-center p-4">
			<Dialog.Content
				class="card bg-surface-100-900 max-w-screen-sm space-y-4 p-4 shadow-xl {flyin}"
			>
				<header class="flex items-center justify-between">
					<Dialog.Title class="text-2xl font-bold">Confirmation</Dialog.Title>
					<Dialog.CloseTrigger class="btn-icon hover:preset-tonal">
						<X class="size-4" />
					</Dialog.CloseTrigger>
				</header>
				<Dialog.Description>
					Do you really want to end the current game? If so you will proceed to the game over
					screen!
				</Dialog.Description>
				<footer class="flex justify-end gap-2">
					<Dialog.CloseTrigger class="btn preset-tonal">Cancel</Dialog.CloseTrigger>
					<Dialog.CloseTrigger class="btn preset-filled-error-500" onclick={end_game}>
						<Flag size={20} />
						End Game
					</Dialog.CloseTrigger>
				</footer>
			</Dialog.Content>
		</Dialog.Positioner>
	</Portal>
</Dialog>
