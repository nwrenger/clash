<script lang="ts">
	import api from '$lib/api';
	import Card from '$lib/components/Card.svelte';
	import type { Connection, Round } from './+page.svelte';

	interface Props {
		connection: Connection;
		round: Round;
		selectable: boolean;
	}

	let { connection, round, selectable }: Props = $props();

	let innerHeight = $state(0);
	let smol = $derived(innerHeight < 800);
	// svelte-ignore state_referenced_locally
	// Change selectedIndex only locally, allow overwrites by the server
	let selectedIndex = $state(round.result?.winning_card_index);
	$effect(() => {
		selectedIndex = round.result?.winning_card_index;
	});

	function selectCards(index: number) {
		if (selectable && selectedIndex == undefined) {
			selectedIndex = index;
			api.send_ws(connection.ws!, { type: 'CzarPick', data: { index } });
		}
	}
</script>

<svelte:window bind:innerHeight />

{#if smol}
	<div class="w-full overflow-x-auto px-4 py-6">
		<div class="flex w-full items-end space-x-12">
			<div class="pt-6">
				{@render BlackCard()}
			</div>

			{@render RevealedCards('pt-6')}
		</div>
	</div>
{:else}
	<div class="flex w-full items-center justify-center">
		{@render BlackCard()}
	</div>

	<div class="flex h-full w-full items-center space-x-8 overflow-x-auto px-4 pt-9 pb-2">
		{@render RevealedCards()}
	</div>
{/if}

{#snippet BlackCard()}
	{#if round.black_card}
		<Card
			card={round.black_card}
			card_classes="bg-surface-950 shadow-xl hover:-translate-y-2 hover:scale-110 hover:-rotate-2"
			text_classes="text-surface-50"
		/>
	{/if}
{/snippet}

{#snippet RevealedCards(classes?: string)}
	{#each round.revealed_cards as pair, i}
		<div class="group flex shrink-0 cursor-pointer items-center {classes}">
			{#each pair as card}
				<Card
					{card}
					onclick={() => selectCards(i)}
					card_classes="bg-surface-50 first:ml-0 ml-[-18px] group-hover:-rotate-2
              				{i === selectedIndex
						? 'z-20 -translate-y-4 scale-120'
						: 'group-hover:-translate-y-2 group-hover:scale-110'}"
					text_classes="text-surface-950"
				/>
			{/each}
		</div>
	{/each}
{/snippet}
