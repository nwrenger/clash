<script lang="ts">
	import api from '$lib/api';
	import Card from '$lib/components/Card.svelte';

	interface Props {
		ws: WebSocket | undefined;
		selectable?: boolean;
		black_card: api.BlackCard;
		revealed_cards: api.WhiteCard[][];
		selectedIndex?: number;
	}

	let innerHeight = $state(0);
	let smol = $derived(innerHeight < 800);

	let { ws, selectable, black_card, revealed_cards, selectedIndex = undefined }: Props = $props();

	function selectCards(index: number) {
		if (selectable && selectedIndex === undefined) {
			selectedIndex = index;
			api.send_ws(ws!, { type: 'CzarPick', data: { index } });
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

			<div class="pt-6">
				{@render RevealedCards()}
			</div>
		</div>
	</div>
{:else}
	<div class="flex w-full items-center justify-center">
		{@render BlackCard()}
	</div>

	<div class="flex h-full w-full items-center space-x-8 overflow-x-auto px-4 pt-9 pb-2">
		{#each revealed_cards as pair, i}
			{@render RevealedCards()}
		{/each}
	</div>
{/if}

{#snippet BlackCard()}
	<Card
		card={black_card}
		card_classes="bg-surface-950 shadow-xl hover:-translate-y-2 hover:scale-110 hover:-rotate-2"
		text_classes="text-surface-50"
	/>
{/snippet}

{#snippet RevealedCards()}
	{#each revealed_cards as pair, i}
		<div class="group flex flex-shrink-0 cursor-pointer items-center">
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
