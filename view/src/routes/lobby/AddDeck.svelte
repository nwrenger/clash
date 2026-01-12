<script lang="ts">
	import api from '$lib/api';
	import { Dialog, Portal } from '@skeletonlabs/skeleton-svelte';
	import type { Connection } from './+page.svelte';
	import { ExternalLink, Plus, X } from 'lucide-svelte';
	import { flyin, opacity } from '$lib/animations';

	interface Props {
		connection: Connection;
		disabled: boolean;
	}

	let { connection, disabled }: Props = $props();

	let deckcode = $state('');
	let open = $state(false);

	function cancle() {
		open = false;
		// reset
		deckcode = '';
	}

	function add() {
		api.send_ws(connection.ws!, { type: 'AddDeck', data: { deckcode: deckcode.trim() } });
		cancle();
	}
</script>

<Dialog {open} onOpenChange={(e) => (open = e.open)}>
	<Dialog.Trigger class="btn preset-filled-primary-500 w-full" title="Add a Deck" {disabled}>
		<Plus size={20} />
		Add
	</Dialog.Trigger>
	<Portal>
		<Dialog.Backdrop class="bg-surface-50-950/50 fixed inset-0 z-50 backdrop-blur-sm {opacity}" />
		<Dialog.Positioner class="fixed inset-0 z-50 flex items-center justify-center p-4">
			<Dialog.Content
				class="card bg-surface-100-900 max-w-screen-sm space-y-4 p-4 shadow-xl {flyin}"
			>
				<header class="flex items-center justify-between">
					<Dialog.Title class="text-2xl font-bold">Adding a Deck</Dialog.Title>
					<Dialog.CloseTrigger class="btn-icon hover:preset-tonal">
						<X class="size-4" />
					</Dialog.CloseTrigger>
				</header>
				<Dialog.Description>
					<p>
						To add a custom deck, please visit the
						<a
							class="anchor"
							href="https://cast.clrtd.com/"
							target="_blank"
							rel="noreferrer"
							title="Open clrtd website"
						>
							clrtd website
						</a>
						its deck code, and paste it here.
					</p>
				</Dialog.Description>
				<label class="label">
					<span class="label-text flex items-center justify-between">
						<span>Deckcode</span>
						<span>
							press <kbd class="kbd text-xs">↵</kbd> to add
						</span>
					</span>
					<input
						class="input"
						bind:value={deckcode}
						placeholder="Input deckcode..."
						onkeydown={(e) => e.key === 'Enter' && deckcode.trim() && add()}
					/>
				</label>
				<footer class="flex justify-end gap-2">
					<Dialog.CloseTrigger class="btn preset-tonal">Cancel</Dialog.CloseTrigger>
					<button type="button" class="btn preset-filled" onclick={add} disabled={!deckcode.trim()}>
						<Plus size={20} />
						Add
					</button>
				</footer>
			</Dialog.Content>
		</Dialog.Positioner>
	</Portal>
</Dialog>
