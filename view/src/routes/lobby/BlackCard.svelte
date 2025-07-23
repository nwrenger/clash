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

	let { ws, selectable, black_card, revealed_cards, selectedIndex = undefined }: Props = $props();

	function selectCards(index: number) {
		if (selectable && selectedIndex === undefined) {
			selectedIndex = index;
			api.send_ws(ws!, { type: 'CzarPick', data: { index } });
		}
	}
</script>

<div class="flex w-full items-center justify-center">
	<div class="flex flex-col items-center space-y-6">
		<Card card={black_card} card_classes="bg-surface-950" text_classes="text-surface-50" />
	</div>
</div>

<!-- outer row: allow scrolling, pad inside, and big gaps between each pair -->
<div class="flex h-full w-full items-center space-x-8 overflow-x-auto px-4 pt-9 pb-2">
	{#each revealed_cards as pair, pairIndex}
		<!-- each pair is a non‑shrinking flex container with its own right‑margin -->
		<!-- svelte-ignore a11y_click_events_have_key_events -->
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div
			class="group flex flex-shrink-0 cursor-pointer items-center"
			onclick={() => selectCards(pairIndex)}
		>
			{#each pair as card}
				<Card
					{card}
					card_classes="{pairIndex === selectedIndex
						? 'z-20 -translate-y-4 scale-120 group-hover:-translate-y-4! group-hover:scale-120!'
						: ''}
					bg-surface-50 ml-[-3rem] first:ml-0
				group-hover:-translate-y-2 group-hover:scale-110 group-hover:-rotate-2"
					text_classes="text-surface-950"
				/>
			{/each}
		</div>
	{/each}
</div>
