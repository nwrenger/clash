<script lang="ts">
	import api from '$lib/api';
	import Card from '$lib/components/Card.svelte';

	interface Props {
		ws: WebSocket | undefined;
		selectedIndexes: number[];
		black_card: api.BlackCard;
		selectable?: boolean;
		cards: api.WhiteCard[];
		disabled?: boolean;
	}

	let {
		ws,
		selectedIndexes = $bindable(),
		black_card,
		selectable,
		cards,
		disabled
	}: Props = $props();

	function hasIndex(index: number) {
		return selectedIndexes.find((p) => p === index) != undefined;
	}

	function selectCard(index: number) {
		if (selectable) {
			if (selectedIndexes.length < black_card.fields && !hasIndex(index)) {
				selectedIndexes.push(index);
			}
			if (selectedIndexes.length == black_card.fields) {
				api.send_ws(ws!, { type: 'SubmitOwnCards', data: { indexes: selectedIndexes } });
				selectable = false;
			}
		}
	}
</script>

<!-- Container fixed at bottom with soft gradient overlay -->
<div
	class="fixed bottom-0 left-0 z-50 w-full bg-gradient-to-t from-black/60 via-transparent to-transparent py-6"
>
	<div class="flex h-full w-full items-center justify-center">
		<div class="flex items-center space-x-4 overflow-x-auto px-4 pt-9 pb-2 perspective-distant">
			{#each cards as card, index}
				<Card
					{card}
					card_classes="{hasIndex(index)
						? 'z-20 -translate-y-4 scale-120'
						: 'hover:-translate-y-2 hover:scale-110'}
						bg-surface-50 ml-[-18px] first:ml-0 hover:-rotate-2
						{disabled ? 'brightness-60 cursor-default' : ''}"
					text_classes="text-surface-950"
					onclick={() => selectCard(index)}
				/>
			{/each}

			<div>
				<span class="me-[0.1px] h-48"></span>
			</div>
		</div>
	</div>
</div>
